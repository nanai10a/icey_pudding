use async_trait::async_trait;
use regex::Regex;
use uuid::Uuid;

use crate::entities::{Author, Content, User};

pub mod mock;
pub mod mongo;

type StdResult<T, E> = ::std::result::Result<T, E>;
type Result<T> = ::std::result::Result<T, RepositoryError>;

#[async_trait]
pub trait UserRepository {
    async fn insert(&self, item: User) -> Result<()>;
    async fn is_exists(&self, id: u64) -> Result<bool>;
    async fn find(&self, id: u64) -> Result<User>;
    async fn is_posted(&self, id: u64, content_id: Uuid) -> Result<bool>;
    async fn is_bookmarked(&self, id: u64, content_id: Uuid) -> Result<bool>;
    async fn update(&self, id: u64, mutation: UserMutation) -> Result<User>;
    async fn delete(&self, id: u64) -> Result<User>;
}

#[async_trait]
pub trait ContentRepository {
    async fn insert(&self, item: Content) -> Result<()>;
    async fn is_exists(&self, id: Uuid) -> Result<bool>;
    async fn find(&self, id: Uuid) -> Result<Content>;
    async fn find_author(&self, regex: Regex) -> Result<Vec<Content>>;
    async fn find_content(&self, regex: Regex) -> Result<Vec<Content>>;
    async fn is_liked(&self, user_id: u64) -> Result<bool>;
    async fn is_pinned(&self, user_id: u64) -> Result<bool>;
    async fn update(&self, mutation: ContentMutation) -> Result<Content>;
    async fn delete(&self, id: Uuid) -> Result<Content>;
}

#[derive(Debug, Clone)]
pub enum RepositoryError {
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

pub fn try_remove_target_from_vec<T>(
    vec: &mut Vec<T>,
    is_target: impl Fn(&T) -> bool,
) -> ::std::result::Result<T, usize> {
    let mut indexes: Vec<_> = vec
        .iter()
        .enumerate()
        .filter_map(|(i, v)| match is_target(v) {
            true => Some(i),
            false => None,
        })
        .collect();

    match indexes.len() {
        1 => Ok(vec.remove(indexes.remove(0))),
        _ => Err(indexes.len()),
    }
}

#[derive(Debug, Clone, Default)]
pub struct UserMutation {
    pub admin: Option<bool>,
    pub sub_admin: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct ContentMutation {
    pub author: Option<Author>,
    pub content: Option<String>,
}
