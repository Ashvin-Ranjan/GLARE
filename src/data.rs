use crate::utils;
use colored::Colorize;
use reqwest::header::HeaderMap;
use serde_json::Value;
use std::collections::HashMap;

mod types;
//           let index = 0.0
//           for (possibleTeam in teams) {
//             if let <object team> = possibleTeam {
//               if let <yes <string name>> = team["slug"] {
//                 let members = []
//                 let teamMembersData = request.get("https://api.github.com/orgs/getoutreach/teams/" + name + "/members?per_page=100", authHeader)!
//                 if (teamMembersData.code == 200) {
//                   if let <array teamMembers> = teamMembersData.return {
//                     for (possibleMember in teamMembers) {
//                       if let <object member> = possibleMember {
//                         if let <yes <string login>> = member["login"] {
//                           var members = members |> append(login)
//                         }
//                       }
//                     }
//                   }
//                 }
//                 var teamMembersRaw = teamMembersRaw |> append((name, members))
//               }
//             }
//             printWithEnd("\r", "[" + utils.GREEN + utils.multString(round(index), "#") + utils.multString(100 - round(index), " ") + utils.RESET + "] " + utils.YELLOW + intInBase10(round(index)) + "%" + utils.RESET)
//             var index = index + percentMultiplier
//           }

async fn get_teams_data(
    org: String,
    headers: HeaderMap,
) -> anyhow::Result<HashMap<String, Vec<String>>> {
    println!("Aggregating team data");
    let data = utils::fetch(
        format!("https://api.github.com/orgs/{}/teams?per_page=100", org),
        &headers,
    )
    .await?;

    let mut teams = HashMap::new();

    match data {
        Value::Array(v) => {
            let mut index: f32 = 0.0;
            for i in &v {
                if let Value::String(team_name) = &i["slug"] {
                    let team_member_data = utils::fetch(
                        format!(
                            "https://api.github.com/orgs/{}/teams/{}/members?per_page=100",
                            org, team_name
                        ),
                        &headers,
                    )
                    .await?;
                    let mut member_logins: Vec<String> = Vec::new();
                    if let Value::Array(members) = team_member_data {
                        for member in members {
                            if let Value::String(login) = &member["login"] {
                                member_logins.push(login.clone());
                            }
                        }
                        teams.insert(team_name.to_owned(), member_logins);
                    }
                }
                println!(
                    "[{}{}] {}%",
                    "#".repeat(index.round() as usize).green(),
                    " ".repeat(100 - (index.round() as usize)),
                    (index.round() as usize).to_string().yellow()
                );
                index += 100.0 / (v.len() as f32);
            }
            println!("");
        }
        _ => return Err(anyhow!("Team data is in invalid format")),
    }

    return Ok(teams);
}

pub async fn aggregate_data(
    author: String,
    repo: String,
    teams: bool,
    possible_token: Option<String>,
) -> anyhow::Result<types::RequestData> {
    // Setting up requests
    let mut headers = HeaderMap::new();
    if let Some(token) = possible_token {
        headers.insert("Authorization", format!("token {}", token).parse().unwrap());
    }

    let mut out: types::RequestData = types::RequestData {
        is_teams: teams,
        owner: author.to_owned(),
        repo: repo,
        data: vec![],
        pulls_open: 0,
        pulls_merged: 0,
        pulls_draft: 0,
        pulls_closed: 0,
        diffs_add: 0,
        diffs_removals: 0,
    };

    let mut team_members: HashMap<String, Vec<String>> = HashMap::new();

    if teams {
        team_members = get_teams_data(author.to_owned(), headers).await?;
    }

    return Ok(out);
}

// let pub aggregateData = [config:utils.configData] -> cmd[()] {
//     var currentlyUpdating = true
//     let possibleToken = FileIO.read("config/.token")!
//     let authHeader = mapFrom([])

//     if let <yes token> = possibleToken {
//       var authHeader = mapFrom([("Authorization", "token " + token)])
//     }
//     let reviewEntries = mapFrom([("unknown", { timesRequested: 0; timesResponded: 0; averageResponseTime: 0; })])
//     let pullResponse = request.get("https://api.github.com/repos/" + config.repoAuthor + "/" + config.repoName + "/pulls?state=all&per_page=100", authHeader)!
//     if pullResponse.code /= 200 {
//       print("Incorrect Authorization or Repo Does not Exist")
//       var currentlyUpdating = false
//       return ()
//     }
//     let numOpen = 0
//     let numMerged = 0
//     let numDraft = 0
//     let numClosed = 0
//     let overallAdditions = 0
//     let overallRemovals = 0
//     let teamMembersRaw = []
//     if config.checkTeams {
//       print("Aggregating team data")
//       let teamResponseData = request.get("https://api.github.com/orgs/getoutreach/teams?per_page=100", authHeader)!
//       if teamResponseData.code == 200 {
//         if let <array teams> = teamResponseData.return {
//           let percentMultiplier = 100.0/toFloat(len(teams))
//           let index = 0.0
//           for (possibleTeam in teams) {
//             if let <object team> = possibleTeam {
//               if let <yes <string name>> = team["slug"] {
//                 let members = []
//                 let teamMembersData = request.get("https://api.github.com/orgs/getoutreach/teams/" + name + "/members?per_page=100", authHeader)!
//                 if (teamMembersData.code == 200) {
//                   if let <array teamMembers> = teamMembersData.return {
//                     for (possibleMember in teamMembers) {
//                       if let <object member> = possibleMember {
//                         if let <yes <string login>> = member["login"] {
//                           var members = members |> append(login)
//                         }
//                       }
//                     }
//                   }
//                 }
//                 var teamMembersRaw = teamMembersRaw |> append((name, members))
//               }
//             }
//             printWithEnd("\r", "[" + utils.GREEN + utils.multString(round(index), "#") + utils.multString(100 - round(index), " ") + utils.RESET + "] " + utils.YELLOW + intInBase10(round(index)) + "%" + utils.RESET)
//             var index = index + percentMultiplier
//           }
//         }
//       print("[" + utils.GREEN + utils.multString(100, "#") + utils.RESET + "] " + utils.YELLOW + "100%" + utils.RESET)
//       }
//     }
//     let teamMembers = mapFrom(teamMembersRaw)
//     print("Aggregating PR data")
//     let index = 0
//     if let <array pulls> = pullResponse.return {
//       for (pull in pulls) {
//         if let <object pullData> = pull {
//           if let <yes <boolean draft>> = pullData["draft"] {
//             if not draft {
//               let creatorUser = ""
//               if let <yes <object creator>> = pullData["user"] {
//                 if let <yes <string login>> = creator["login"] {
//                   var creatorUser = login
//                 }
//               }
//               if pullData["merged_at"] == yes(json.null) && pullData["closed_at"] == yes(json.null) {
//                 if config.checkTeams {
//                   if let <yes <array requestedTeams>> = pullData["requested_teams"] {
//                     for (possibleTeam in requestedTeams) {
//                       if let <object team> = possibleTeam {
//                         let slug = "unknown"
//                         if let <yes <string sl>> = team["slug"] {
//                           var slug = sl
//                         }
//                         let ent = entries(reviewEntries)
//                         let reviewerData = reviewEntries[slug] |> default({ timesRequested: 0; timesResponded: 0; averageResponseTime: 0; })
//                         var reviewEntries = mapFrom([
//                           ..ent,
//                           (slug, {
//                             ..reviewerData
//                             timesRequested: reviewerData.timesRequested + 1
//                           }),
//                         ])
//                       }
//                     }
//                   }
//                 } else {
//                   if let <yes <array requestedReviewers>> = pullData["requested_reviewers"] {
//                     for (possibleReviewer in requestedReviewers) {
//                       if let <object reviewer> = possibleReviewer {
//                         let login = "unknown"
//                         if let <yes <string log>> = reviewer["login"] {
//                           var login = log
//                         }
//                         let ent = entries(reviewEntries)
//                         let reviewerData = reviewEntries[login] |> default({ timesRequested: 0; timesResponded: 0; averageResponseTime: 0; })
//                         var reviewEntries = mapFrom([
//                           ..ent,
//                           (login, {
//                             ..reviewerData
//                             timesRequested: reviewerData.timesRequested + 1
//                           }),
//                         ])
//                       }
//                     }
//                   }
//                 }
//               }
//               if let <yes <number pullNumber>> = pullData["number"] {
//                 let reviewData = request.get("https://api.github.com/repos/" + config.repoAuthor + "/" + config.repoName + "/pulls/" + intInBase10(round(pullNumber)) + "/reviews", authHeader)!
//                 if reviewData.code == 200 {
//                   if let <array reviews> = reviewData.return {
//                     let peopleReviewed = []
//                     for (possibleReview in reviews) {
//                       if let <object review> = possibleReview {
//                         if let <yes <object user>> = review["user"] {
//                           let login = "unknown"
//                           if let <yes <string log>> = user["login"] {
//                             var login = log
//                           }
//                           if (config.checkTeams) {
//                             let slugs = utils.getUserTeams(login, teamMembersRaw)
//                             for (slug in utils.removeDuplicates(slugs)) {
//                               var peopleReviewed = peopleReviewed |> append(slug)
//                             }
//                           } else {
//                             if login /= creatorUser {
//                               var peopleReviewed = peopleReviewed |> append(login)
//                             }
//                           }
//                         }
//                       }
//                     }
//                     for (slug in utils.removeDuplicates(peopleReviewed)) {
//                       let ent = entries(reviewEntries)
//                       let reviewerData = reviewEntries[slug] |> default({ timesRequested: 0; timesResponded: 0; averageResponseTime: 0; })
//                       var reviewEntries = mapFrom([
//                         ..ent,
//                         (slug, {
//                           ..reviewerData
//                           timesResponded: reviewerData.timesResponded + 1
//                           // It is assumed that the times they respond it is because they were requested
//                           timesRequested: reviewerData.timesRequested + 1
//                         }),
//                       ])
//                     }
//                   }
//                 }
//               }
//               if pullData["merged_at"] == yes(json.null) {
//                 if pullData["closed_at"] == yes(json.null) {
//                   var numOpen = numOpen + 1
//                 } else {
//                   var numClosed = numClosed + 1
//                 }
//               } else {
//                 var numMerged = numMerged + 1
//               }
//               if config.checkDiff {
//                 if let <yes <string diffUrl>> = pullData["url"] {
//                   let diffData = request.get(diffUrl, mapFrom([
//                     ..entries(authHeader),
//                     ("Accept", "application/vnd.github.v3.diff")
//                   ]))!
//                   if diffData.code == 200 {
//                     if let <string diff> = diffData.return {
//                       let a, r = utils.getChangesFromDiffFile(diff)
//                       var overallAdditions = overallAdditions + a
//                       var overallRemovals = overallRemovals + r
//                     }
//                   }
//                 }
//               }
//             } else {
//               var numDraft = numDraft + 1
//             }
//           }
//         }
//         printWithEnd("\r", "[" + utils.GREEN + utils.multString(index, "#") + utils.multString(100 - index, " ") + utils.RESET + "] " + utils.YELLOW + intInBase10(index) + "%" + utils.RESET)
//         var index = index + 1
//       }
//       print("[" + utils.GREEN + utils.multString(100, "#") + utils.RESET + "] " + utils.YELLOW + "100%" + utils.RESET)
//     }
//     var jsonData = json.object(mapFrom([
//       ("info", json.object(mapFrom([
//         ("isTeams", json.boolean(config.checkTeams)),
//         ("owner", json.string(config.repoAuthor)),
//         ("repoName", json.string(config.repoName)),
//       ]))),
//       ("data", utils.reviewDataToJson(reviewEntries)),
//       ("overall", json.object(mapFrom([
//         ("pullStates", json.object(mapFrom([
//           ("open", json.number(toFloat(numOpen))),
//           ("merged", json.number(toFloat(numMerged))),
//           ("draft", json.number(toFloat(numDraft))),
//           ("closed", json.number(toFloat(numClosed))),
//         ]))),
//         ("diffs", json.object(mapFrom([
//           ("additions", json.number(toFloat(overallAdditions))),
//           ("removals", json.number(toFloat(overallRemovals))),
//         ])))
//       ])))
//     ]))
//     if config.export {
//       let _ = FileIO.write("export.json", json.stringify(jsonData))!
//       print("Exporting json...")
//     }
//     var currentlyUpdating = false
//   }
