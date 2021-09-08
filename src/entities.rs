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
pub(crate) struct UserId(pub u64);

#[derive(Debug, Clone)]
pub struct User {
    pub(crate) id: UserId,
    pub(crate) admin: bool,
    pub(crate) sub_admin: bool,
    pub(crate) posted: HashSet<ContentId>,
    pub(crate) bookmark: HashSet<ContentId>,
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
pub(crate) struct ContentId(pub ::uuid::Uuid);

#[derive(Debug, Clone)]
pub struct Content {
    pub(crate) id: ContentId,
    pub(crate) author: Author,
    pub(crate) posted: Posted,
    pub(crate) content: String,
    pub(crate) liked: HashSet<UserId>,
    pub(crate) pinned: HashSet<UserId>,
    // FIXME: replace type alias and provide helper fn
    pub(crate) created: ::chrono::DateTime<::chrono::Utc>,
    pub(crate) edited: Vec<::chrono::DateTime<::chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub(crate) struct Posted {
    pub(crate) id: UserId,
    pub(crate) name: String,
    pub(crate) nick: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) enum Author {
    User {
        id: UserId,
        name: String,
        nick: Option<String>,
    },
    Virtual(String),
}

#[derive(Debug, Clone)]
pub(crate) enum PartialAuthor {
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
            self.nick.as_ref().unwrap_or(&"".to_string()),
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
