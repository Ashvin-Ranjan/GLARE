use std::collections::HashMap;

#[derive(Copy, Clone)]
pub struct ReviewData {
    pub times_requested: u8,
    pub times_responded: u8,
}

pub struct RequestData {
    pub is_teams: bool,
    pub owner: String,
    pub repo: String,

    pub data: HashMap<String, ReviewData>,

    pub pulls_open: u8,
    pub pulls_merged: u8,
    pub pulls_draft: u8,
    pub pulls_closed: u8,

    pub diffs_add: u64,
    pub diffs_removals: u64,
}
