#[macro_use]
mod macros;

use crate::utils::{
    err::pretty_error,
    http::{fetch, fetch_string},
};
use colored::Colorize;
use reqwest::header::HeaderMap;
use serde_json::{Map, Number, Value};
use std::{collections::HashMap, fs, io::Write};

mod types;

fn request_data_to_json(v: types::RequestData) -> serde_json::Value {
    return Value::Object(Map::from_iter(vec![
        (
            "info".to_owned(),
            Value::Object(Map::from_iter(vec![
                ("is_teams".to_owned(), Value::Bool(v.is_teams)),
                ("owner".to_owned(), Value::String(v.owner)),
                ("repo_name ".to_owned(), Value::String(v.repo)),
            ])),
        ),
        (
            "data".to_owned(),
            Value::Object(Map::from_iter(v.data.iter().map(
                |(x, y)| -> (String, Value) {
                    return (
                        x.to_owned(),
                        Value::Object(Map::from_iter(vec![
                            (
                                "times_requested".to_owned(),
                                Value::Number(Number::from(y.times_requested)),
                            ),
                            (
                                "times_responded".to_owned(),
                                Value::Number(Number::from(y.times_responded)),
                            ),
                        ])),
                    );
                },
            ))),
        ),
        (
            "overall".to_owned(),
            Value::Object(Map::from_iter(vec![
                (
                    "pull_states".to_owned(),
                    Value::Object(Map::from_iter(vec![
                        ("open".to_owned(), Value::Number(Number::from(v.pulls_open))),
                        (
                            "merged".to_owned(),
                            Value::Number(Number::from(v.pulls_merged)),
                        ),
                        (
                            "closed".to_owned(),
                            Value::Number(Number::from(v.pulls_closed)),
                        ),
                        (
                            "draft".to_owned(),
                            Value::Number(Number::from(v.pulls_draft)),
                        ),
                    ])),
                ),
                (
                    "diffs".to_owned(),
                    Value::Object(Map::from_iter(vec![
                        (
                            "additions".to_owned(),
                            Value::Number(Number::from(v.diffs_add)),
                        ),
                        (
                            "removals".to_owned(),
                            Value::Number(Number::from(v.diffs_removals)),
                        ),
                    ])),
                ),
            ])),
        ),
    ]));
}

async fn get_teams_data(
    org: String,
    headers: &HeaderMap,
) -> anyhow::Result<HashMap<String, Vec<String>>> {
    println!("{}: Aggregating Team data", "Info".yellow());
    let data = fetch(
        format!("https://api.github.com/orgs/{}/teams?per_page=100", org),
        &headers,
    )
    .await?;

    let mut teams: HashMap<String, Vec<String>> = HashMap::new();

    match data {
        Value::Array(v) => {
            let mut index: f32 = 0.0;
            for i in &v {
                if let Value::String(team_name) = &i["slug"] {
                    let team_member_data = fetch(
                        format!(
                            "https://api.github.com/orgs/{}/teams/{}/members?per_page=100",
                            org, team_name
                        ),
                        &headers,
                    )
                    .await?;
                    if let Value::Array(members) = team_member_data {
                        for member in members {
                            if let Value::String(login) = &member["login"] {
                                let new_teams = match teams.get(login) {
                                    Some(x) => {
                                        let mut new_list = x.clone();
                                        new_list.push(team_name.to_owned());
                                        new_list
                                    }
                                    None => vec![team_name.to_owned()],
                                };
                                teams.insert(login.to_owned(), new_teams);
                            }
                        }
                    }
                }
                print!(
                    "{}: [{}{}] {}%\r",
                    "Info".yellow(),
                    "#".repeat(index.round() as usize).green(),
                    " ".repeat(100 - (index.round() as usize)),
                    (index.round() as usize).to_string().yellow()
                );
                std::io::stdout().flush()?;
                index += 100.0 / (v.len() as f32);
            }
            println!(
                "{}: [{}] {}%",
                "Info".yellow(),
                "#".repeat(100).green(),
                "100".yellow()
            );
            println!("{}: Aggregated Team data", "Info".yellow())
        }
        _ => return Err(anyhow!("Team data is in invalid format")),
    }

    return Ok(teams);
}

pub async fn aggregate_data(
    author: String,
    repo: String,
    teams: bool,
    no_diff: bool,
    export: bool,
    possible_token: Option<String>,
) -> anyhow::Result<Value> {
    // Setting up requests
    let mut headers = HeaderMap::new();
    headers.insert("User-Agent", "GLARE".parse().unwrap());
    if let Some(token) = possible_token {
        headers.insert("Authorization", format!("token {}", token).parse().unwrap());
    }

    let mut out: types::RequestData = types::RequestData {
        is_teams: teams,
        owner: author.to_owned(),
        repo: repo.to_owned(),
        data: HashMap::new(),
        pulls_open: 0,
        pulls_merged: 0,
        pulls_draft: 0,
        pulls_closed: 0,
        diffs_add: 0,
        diffs_removals: 0,
    };

    let mut team_members: HashMap<String, Vec<String>> = HashMap::new();

    if teams {
        team_members = get_teams_data(author.to_owned(), &headers).await?;
    }

    let pull_data = fetch(
        format!(
            "https://api.github.com/repos/{}/{}/pulls?state=all&per_page=100",
            author.to_owned(),
            repo.to_owned()
        ),
        &headers,
    )
    .await?;

    println!("{}: Aggregating PR data", "Info".yellow());

    let pulls = match_json!(pull_data, Value::Array);
    for (i, pull) in pulls.iter().enumerate() {
        let draft = match_json!(pull["draft"], Value::Bool);
        if !draft {
            let user_login = match_json!(
                &match_json!(&pull["user"], Value::Object)["login"],
                Value::String
            );
            if &pull["merged_at"] == &Value::Null {
                if &pull["closed_at"] == &Value::Null {
                    out.pulls_open += 1;
                    let requested = match_json!(
                        &pull[if teams {
                            "requested_teams"
                        } else {
                            "requested_reviewers"
                        }],
                        Value::Array
                    );
                    for request in requested {
                        let name = match_json!(
                            &request[if teams { "slug" } else { "login" }],
                            Value::String
                        );
                        let new_data = match out.data.get(name) {
                            Some(x) => types::ReviewData {
                                times_requested: x.times_requested + 1,
                                times_responded: 0,
                            },
                            None => types::ReviewData {
                                times_requested: 1,
                                times_responded: 0,
                            },
                        };
                        out.data.insert(name.to_owned(), new_data);
                    }
                } else {
                    out.pulls_closed += 1;
                }
                let number = match_json!(&pull["number"], Value::Number);
                let reviews = match_json!(
                    fetch(
                        format!(
                            "https://api.github.com/repos/{}/{}/pulls/{}/reviews",
                            author, repo, number
                        ),
                        &headers
                    )
                    .await?,
                    Value::Array
                );
                let mut reviewers: Vec<String> = Vec::new();
                for review in reviews {
                    let login = match_json!(
                        &match_json!(&review["user"], Value::Object)["login"],
                        Value::String
                    );
                    if login != user_login {
                        if teams {
                            if let Some(teams) = team_members.get(login) {
                                let mut teams_clone = teams.clone();
                                reviewers.append(&mut teams_clone);
                            }
                        } else {
                            reviewers.push(login.to_owned());
                        }
                    }
                }
                reviewers.sort();
                reviewers.dedup();
                for reviewer in reviewers {
                    let new_data = match out.data.get(&reviewer) {
                        Some(x) => types::ReviewData {
                            times_requested: x.times_requested + 1,
                            times_responded: x.times_responded + 1,
                        },
                        None => types::ReviewData {
                            times_requested: 1,
                            times_responded: 1,
                        },
                    };
                    out.data.insert(reviewer.to_owned(), new_data);
                }
            } else {
                out.pulls_merged += 1;
            }
            if !no_diff {
                let diff_url = match_json!(&pull["url"], Value::String);
                let mut diff_header = HeaderMap::new();
                for (i, k) in headers.iter() {
                    diff_header.insert(i, k.clone());
                }
                diff_header.insert("Accept", "application/vnd.github.v3.diff".parse()?);
                let diff = fetch_string(diff_url.to_owned(), &diff_header).await?;

                for line in diff.split("\n") {
                    if line.len() >= 6 && (&line[0..6] == "--- a/" || &line[0..6] == "+++ b/") {
                        continue;
                    }

                    if line.chars().next() == Some('+') {
                        out.diffs_add += 1;
                    } else if line.chars().next() == Some('-') {
                        out.diffs_removals += 1;
                    }
                }
            }
        } else {
            out.pulls_draft += 1;
        }
        print!(
            "{}: [{}{}] {}%\r",
            "Info".yellow(),
            "#".repeat(i + 1).green(),
            " ".repeat(99 - i),
            i.to_string().yellow()
        );
        std::io::stdout().flush()?;
    }
    println!(
        "{}: [{}] {}%",
        "Info".yellow(),
        "#".repeat(100).green(),
        "100".yellow()
    );
    println!("{}: Aggregated PR data", "Info".yellow());

    let json_out = request_data_to_json(out);

    if export {
        fs::write("/tmp/foo", serde_json::to_string(&json_out)?).expect("Unable to write file");
    }

    return Ok(json_out);
}
