use colored::Colorize;
use reqwest::{header::HeaderMap, Client};
use serde_json;

lazy_static! {
    pub static ref CLIENT: Client = Client::new();
}

pub async fn fetch(url: String, headers: &HeaderMap) -> anyhow::Result<serde_json::Value> {
    let mut request = CLIENT.get(&url);

    for (i, k) in headers.iter() {
        request = request.header(i, k);
    }

    let request_response = request.send().await?;

    let code = request_response.error_for_status_ref();

    match &code {
        Ok(_) => (),
        Err(x) => {
            return Err(anyhow!(format!(
                "Unable to make fetch request to `{}`. (Status Code: {})",
                url.blue(),
                match x.status() {
                    Some(code) => code.as_u16().to_string(),
                    None => "Unknown".to_owned(),
                }
                .blue()
            )));
        }
    }

    let v = serde_json::from_str(&request_response.text().await?)?;

    return Ok(v);
}

pub async fn fetch_string(url: String, headers: &HeaderMap) -> anyhow::Result<String> {
    let mut request = CLIENT.get(&url);

    for (i, k) in headers.iter() {
        request = request.header(i, k);
    }

    let request_response = request.send().await?;

    let code = request_response.error_for_status_ref();

    match &code {
        Ok(_) => (),
        Err(x) => {
            return Err(anyhow!(format!(
                "Unable to make fetch request to `{}`. (Status Code: {})",
                url.blue(),
                match x.status() {
                    Some(code) => code.as_u16().to_string(),
                    None => "Unknown".to_owned(),
                }
                .blue()
            )));
        }
    }

    return Ok(request_response.text().await?);
}
