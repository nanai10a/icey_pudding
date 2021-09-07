use std::collections::HashSet;
use std::ops::Bound;

use async_trait::async_trait;
use regex::Regex;

use crate::entities::{Author, Content, ContentId, User, UserId};

pub(crate) mod mock;
pub(crate) mod mongo;

type StdResult<T, E> = ::std::result::Result<T, E>;
type Result<T> = ::std::result::Result<T, RepositoryError>;

#[async_trait]
pub(crate) trait UserRepository {
    async fn insert(&self, item: User) -> Result<bool>;
    async fn is_exists(&self, id: UserId) -> Result<bool>;

    async fn find(&self, id: UserId) -> Result<User>;
    async fn finds(&self, query: UserQuery) -> Result<Vec<User>>;

    async fn update(&self, id: UserId, mutation: UserMutation) -> Result<User>;

    async fn is_posted(&self, id: UserId, content_id: ContentId) -> Result<bool>;
    async fn insert_posted(&self, id: UserId, content_id: ContentId) -> Result<bool>;
    async fn delete_posted(&self, id: UserId, content_id: ContentId) -> Result<bool>;

    async fn is_bookmark(&self, id: UserId, content_id: ContentId) -> Result<bool>;
    async fn insert_bookmark(&self, id: UserId, content_id: ContentId) -> Result<bool>;
    async fn delete_bookmark(&self, id: UserId, content_id: ContentId) -> Result<bool>;

    async fn delete(&self, id: UserId) -> Result<User>;
}

#[async_trait]
pub(crate) trait ContentRepository {
    async fn insert(&self, item: Content) -> Result<bool>;
    async fn is_exists(&self, id: ContentId) -> Result<bool>;

    async fn find(&self, id: ContentId) -> Result<Content>;
    async fn finds(&self, query: ContentQuery) -> Result<Vec<Content>>;

    async fn update(&self, id: ContentId, mutation: ContentMutation) -> Result<Content>;

    async fn is_liked(&self, id: ContentId, user_id: UserId) -> Result<bool>;
    async fn insert_liked(&self, id: ContentId, user_id: UserId) -> Result<bool>;
    async fn delete_liked(&self, id: ContentId, user_id: UserId) -> Result<bool>;

    async fn is_pinned(&self, id: ContentId, user_id: UserId) -> Result<bool>;
    async fn insert_pinned(&self, id: ContentId, user_id: UserId) -> Result<bool>;
    async fn delete_pinned(&self, id: ContentId, user_id: UserId) -> Result<bool>;

    async fn delete(&self, id: ContentId) -> Result<Content>;
}

#[derive(Debug, Clone, Default)]
pub(crate) struct UserQuery {
    pub(crate) posted: Option<HashSet<ContentId>>,
    pub(crate) posted_num: Option<(Bound<u32>, Bound<u32>)>,
    pub(crate) bookmark: Option<HashSet<ContentId>>,
    pub(crate) bookmark_num: Option<(Bound<u32>, Bound<u32>)>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ContentQuery {
    pub(crate) author: Option<AuthorQuery>,
    pub(crate) posted: Option<PostedQuery>,
    pub(crate) content: Option<Regex>,
    pub(crate) liked: Option<HashSet<UserId>>,
    pub(crate) liked_num: Option<(Bound<u32>, Bound<u32>)>,
    pub(crate) pinned: Option<HashSet<UserId>>,
    pub(crate) pinned_num: Option<(Bound<u32>, Bound<u32>)>,
}

#[derive(Debug, Clone)]
pub(crate) enum PostedQuery {
    UserId(UserId),
    UserName(Regex),
    UserNick(Regex),
    Any(Regex),
}

#[derive(Debug, Clone)]
pub(crate) enum AuthorQuery {
    UserId(UserId),
    UserName(Regex),
    UserNick(Regex),
    Virtual(Regex),
    Any(Regex),
}

#[derive(Debug)]
pub(crate) enum RepositoryError {
    NotFound,
    NoUnique { matched: u32 },
    Internal(anyhow::Error),
}

impl ::std::fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            RepositoryError::NotFound => write!(f, "cannot find object."),
            RepositoryError::NoUnique { matched } => write!(
                f,
                "expected unique object, found non-unique objects (matched: {})",
                matched
            ),
            RepositoryError::Internal(e) => write!(f, "internal error: {}", e),
        }
    }
}
impl ::std::error::Error for RepositoryError {}

#[derive(Debug, Clone, Default)]
pub(crate) struct UserMutation {
    pub(crate) admin: Option<bool>,
    pub(crate) sub_admin: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ContentMutation {
    pub(crate) author: Option<Author>,
    pub(crate) content: Option<ContentContentMutation>,
}

#[derive(Debug, Clone)]
pub(crate) enum ContentContentMutation {
    Complete(String),
    Sed { capture: Regex, replace: String },
}
