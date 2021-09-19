use std::collections::HashSet;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    ::serde::Serialize,
    ::serde::Deserialize,
)]
pub struct UserId(pub u64);

#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub admin: bool,
    pub sub_admin: bool,
    pub bookmark: HashSet<ContentId>,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    ::serde::Serialize,
    ::serde::Deserialize,
)]
pub struct ContentId(pub ::uuid::Uuid);

#[derive(Debug, Clone)]
pub struct Content {
    pub id: ContentId,
    pub author: Author,
    pub posted: Posted,
    pub content: String,
    pub liked: HashSet<UserId>,
    pub pinned: HashSet<UserId>,
    pub created: Date,
    pub edited: Vec<Date>,
}

#[derive(Debug, Clone)]
pub struct Posted {
    pub id: UserId,
    pub name: String,
    pub nick: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Author {
    User {
        id: UserId,
        name: String,
        nick: Option<String>,
    },
    Virtual(String),
}

pub type Date = ::chrono::DateTime<::chrono::Utc>;

#[derive(Debug, Clone)]
pub enum PartialAuthor {
    User(UserId),
    Virtual(String),
}

impl ::core::fmt::Display for UserId {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ::core::fmt::Display for ContentId {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ::core::fmt::Display for Author {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
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

impl ::core::fmt::Display for Posted {
    fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
        write!(
            f,
            "{} ({} | {})",
            self.name,
            self.nick.as_ref().unwrap_or(&"None".to_string()),
            self.id
        )
    }
}

impl From<u64> for UserId {
    fn from(n: u64) -> Self { Self(n) }
}

impl From<::uuid::Uuid> for ContentId {
    fn from(i: ::uuid::Uuid) -> Self { Self(i) }
}
