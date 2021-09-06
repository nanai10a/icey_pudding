use std::collections::HashSet;
use std::fmt::Display;

use uuid::Uuid;

// TODO: replace to this structures
#[allow(dead_code)]
pub(crate) struct UserId(u64);
#[allow(dead_code)]
pub(crate) struct ContentId(Uuid);

#[derive(Debug, Clone)]
pub struct User {
    pub(crate) id: u64,
    pub(crate) admin: bool,
    pub(crate) sub_admin: bool,
    pub(crate) posted: HashSet<Uuid>,
    pub(crate) bookmark: HashSet<Uuid>,
}

// TODO: rename to `Post`
// TODO: add `created` and `edited`
#[derive(Debug, Clone)]
pub struct Content {
    pub(crate) id: Uuid,
    pub(crate) author: Author,
    pub(crate) posted: Posted,
    pub(crate) content: String,
    pub(crate) liked: HashSet<u64>,
    pub(crate) pinned: HashSet<u64>,
}

#[derive(Debug, Clone)]
pub(crate) struct Posted {
    pub(crate) id: u64,
    pub(crate) name: String,
    pub(crate) nick: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) enum Author {
    User {
        id: u64,
        name: String,
        nick: Option<String>,
    },
    Virtual(String),
}

#[derive(Debug, Clone)]
pub(crate) enum PartialAuthor {
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

impl Display for Posted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({} | {})",
            self.name,
            self.nick.as_ref().unwrap_or(&"".to_string()),
            self.id
        )
    }
}
