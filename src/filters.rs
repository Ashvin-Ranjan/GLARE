use crate::handlers;
use warp::{get, http::Response, path, Filter, Rejection, Reply};

pub fn routes() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    base_page()
        .or(favicon())
        .or(api_data())
        .or(api_reload())
        .or(card_template())
}

fn api_data() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path!("api" / "data")
        .and(get())
        .and_then(handlers::api_data_handler)
}

fn api_reload() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path!("api" / "reload")
        .and(get())
        .and_then(handlers::api_reload_handler)
}

fn card_template() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path!(String).and(get()).map(|_| {
        Response::builder()
            .header("Content-Type", "text/html; charset=utf-8")
            .body(include_str!("./site/card-template.html"))
    })
}

fn favicon() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path!("favicon.ico").and(get()).map(|| {
        Response::builder()
            .header("Content-Type", "image/x-icon")
            .body(&include_bytes!("./site/favicon.ico")[..])
    })
}

fn base_page() -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    path::end().and(get()).map(|| {
        Response::builder()
            .header("Content-Type", "text/html; charset=utf-8")
            .body(include_str!("./site/template.html"))
    })
}
