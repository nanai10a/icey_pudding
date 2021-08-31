use std::collections::HashSet;
use std::fmt::Display;

use uuid::Uuid;

// TODO: replace to this structures
pub struct UserId(u64);
pub struct ContentId(Uuid);

#[derive(Debug, Clone)]
pub struct User {
    pub id: u64,
    pub admin: bool,
    pub sub_admin: bool,
    pub posted: HashSet<Uuid>,
    pub bookmark: HashSet<Uuid>,
}

// TODO: rename to `Post`
#[derive(Debug, Clone)]
pub struct Content {
    pub id: Uuid,
    pub author: Author,
    pub posted: u64,
    pub content: String,
    pub liked: HashSet<u64>,
    pub pinned: HashSet<u64>,
}

#[derive(Debug, Clone)]
pub enum Author {
    User {
        id: u64,
        name: String,
        nick: Option<String>,
    },
    Virtual(String),
}

#[derive(Debug, Clone)]
pub enum PartialAuthor {
    User(u64),
    Virtual(String),
}

impl Display for Author {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Author::User { id, name, nick } => {
                let mut nick_fmt = &String::new();
                if let Some(s) = nick {
                    nick_fmt = s;
                }

                write!(f, "{} ({} | {})", name, nick_fmt, id)
            },
            Author::Virtual(name) => write!(f, "{}", name),
        }
    }
}
