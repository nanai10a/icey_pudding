use std::collections::HashSet;

#[derive(Debug, Clone, Copy)]
pub(crate) struct UserId(u64);

#[derive(Debug, Clone)]
pub(crate) struct User {
    pub(crate) id: UserId,
    pub(crate) admin: bool,
    pub(crate) sub_admin: bool,
    pub(crate) posted: HashSet<PostId>,
    pub(crate) bookmark: HashSet<PostId>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct PostId(::uuid::Uuid);

// TODO: add `created` and `edited`
#[derive(Debug, Clone)]
pub(crate) struct Post {
    pub(crate) id: PostId,
    pub(crate) author: Author,
    pub(crate) from: PostFrom,
    pub(crate) content: String,
    pub(crate) liked: HashSet<UserId>,
    pub(crate) pinned: HashSet<UserId>,
}

#[derive(Debug, Clone)]
pub(crate) struct PostFrom {
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

impl ::core::fmt::Display for PostId {
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

impl ::core::fmt::Display for PostFrom {
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
