use std::collections::HashSet;

use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub admin: bool,
    pub sub_admin: bool,
    pub posted: HashSet<u64>,
    pub bookmark: HashSet<u64>,
}

#[derive(Debug, Clone)]
pub struct Content {
    pub id: Uuid,
    pub author: Author,
    pub posted: u64,
    pub content: String,
    pub liked: HashSet<u64>,
    pub bookmarked: u32,
    pub pinned: HashSet<u64>,
}

pub enum Author {
    User(u64),
    Virtual(String),
}
