pub fn check_new_version() -> Option<String> {
    let version = env!("CARGO_PKG_VERSION");
    let index = crates_index::Index::new_cargo_default().unwrap();
    let glare_crate = match index.crate_("glare") {
        Some(v) => v,
        None => return None,
    };
    if version != glare_crate.highest_stable_version().unwrap().version() {
        return Some(String::from(
            glare_crate.highest_stable_version().unwrap().version(),
        ));
    }
    return None;
}
