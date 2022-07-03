use serde_json::Value;
use std::sync::Mutex;
use warp::{
    http::Response,
    reject::{custom, Reject, Rejection},
    Reply,
};

lazy_static! {
    pub static ref DATA: Mutex<Value> = Mutex::new(Value::Null);
    pub static ref RELOADING: bool = false;
}

#[derive(Debug)]
pub enum ApiFailure {
    DataLocked,
}

impl Reject for ApiFailure {}

pub async fn api_data_handler() -> Result<impl Reply, Rejection> {
    match DATA.lock() {
        Ok(d) => {
            return Ok(Response::builder()
                .header("Content-Type", "application/json; charset=utf-8")
                .body(match serde_json::to_string(&*d) {
                    Ok(v) => v,
                    Err(_) => return Err(custom(ApiFailure::DataLocked)),
                }))
        }
        Err(_) => return Err(custom(ApiFailure::DataLocked)),
    }
}

pub async fn api_reload_handler() -> Result<impl Reply, Rejection> {
    let lock = DATA.try_lock();
    if let Ok(_) = lock {
        // TODO: Reloading
        return Ok("Reloading...");
    } else {
        return Err(custom(ApiFailure::DataLocked));
    }
}
