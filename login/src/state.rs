use dashmap::DashMap;

pub struct State {
    pub sessions: DashMap<usize, Session>,
}

impl State {
    pub fn new() -> Self {
        Self {
            sessions: DashMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct Session {
    pub id: usize,
    pub account_id: i32,
    pub pin: String,
    pub pic: String,
    pub login_attempts: i32,
}

impl Session {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            account_id: -1,
            pin: "".to_string(),
            pic: "".to_string(),
            login_attempts: 1,
        }
    }
}
