struct ReviewData {}

pub struct RequestData {
    pub is_teams: bool,
    pub owner: String,
    pub repo: String,

    pub data: Vec<RequestData>,

    pub pulls_open: u8,
    pub pulls_merged: u8,
    pub pulls_draft: u8,
    pub pulls_closed: u8,

    pub diffs_add: u128,
    pub diffs_removals: u128,
}
