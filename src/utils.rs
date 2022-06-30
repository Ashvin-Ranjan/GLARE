use colored::Colorize;
use reqwest::{header::HeaderMap, Client};
use serde_json;
use std::{fmt, process::exit};

lazy_static! {
    pub static ref CLIENT: Client = Client::new();
}

pub async fn fetch(url: String, headers: &HeaderMap) -> anyhow::Result<serde_json::Value> {
    let mut request = CLIENT.get(&url);

    for (i, k) in headers.iter() {
        request = request.header(i, k);
    }

    let pull_response = request.send().await?;

    let code = pull_response.error_for_status_ref();

    match &code {
        Ok(_) => (),
        Err(_) => {
            return Err(anyhow!(format!(
                "Unable to make fetch request to `{}`.",
                url.blue()
            )))
        }
    }

    let v = serde_json::from_str(&pull_response.text().await?)?;

    return Ok(v);
}

pub trait FormatUnpack<T, E> {
    fn fup(self) -> T;
}

impl<T, E: fmt::Display> FormatUnpack<T, E> for Result<T, E> {
    #[cfg(not(feature = "panic_immediate_abort"))]
    fn fup(self) -> T {
        match self {
            Ok(x) => x,
            Err(y) => {
                println!("{}: {}", "Error".red(), y);
                exit(1)
            }
        }
    }
}
