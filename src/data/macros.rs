macro_rules! match_json {
    ($a:expr, $variant:path) => {
        if let $variant(x) = $a {
            x
        } else {
            pretty_error("JSON data was in an unexpected format!")
        }
    };
}
