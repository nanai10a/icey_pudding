use std::collections::HashSet;

use async_trait::async_trait;

use crate::entities::{Content, ContentId, User, UserId};
use crate::usecases::content::{ContentMutation, ContentQuery};
use crate::usecases::user::{UserMutation, UserQuery};

mod mock;
mod mongo;

pub use mock::InMemoryRepository;
pub use mongo::{MongoContentRepository, MongoUserRepository};

type Result<T> = ::std::result::Result<T, RepositoryError>;

#[async_trait]
pub trait UserRepository {
    async fn insert(&self, item: User) -> Result<bool>;
    async fn is_exists(&self, id: UserId) -> Result<bool>;

    async fn find(&self, id: UserId) -> Result<User>;
    async fn finds(&self, query: UserQuery) -> Result<Vec<User>>;

    async fn update(&self, id: UserId, mutation: UserMutation) -> Result<User>;

    async fn get_bookmark(&self, id: UserId) -> Result<HashSet<ContentId>>;
    async fn is_bookmark(&self, id: UserId, content_id: ContentId) -> Result<bool>;
    async fn insert_bookmark(&self, id: UserId, content_id: ContentId) -> Result<bool>;
    async fn delete_bookmark(&self, id: UserId, content_id: ContentId) -> Result<bool>;

    async fn delete(&self, id: UserId) -> Result<User>;
}

#[async_trait]
pub trait ContentRepository {
    async fn insert(&self, item: Content) -> Result<bool>;
    async fn is_exists(&self, id: ContentId) -> Result<bool>;

    async fn find(&self, id: ContentId) -> Result<Content>;
    async fn finds(&self, query: ContentQuery) -> Result<Vec<Content>>;

    async fn update(&self, id: ContentId, mutation: ContentMutation) -> Result<Content>;

    async fn get_liked(&self, id: ContentId) -> Result<HashSet<UserId>>;
    async fn is_liked(&self, id: ContentId, user_id: UserId) -> Result<bool>;
    async fn insert_liked(&self, id: ContentId, user_id: UserId) -> Result<bool>;
    async fn delete_liked(&self, id: ContentId, user_id: UserId) -> Result<bool>;

    async fn get_pinned(&self, id: ContentId) -> Result<HashSet<UserId>>;
    async fn is_pinned(&self, id: ContentId, user_id: UserId) -> Result<bool>;
    async fn insert_pinned(&self, id: ContentId, user_id: UserId) -> Result<bool>;
    async fn delete_pinned(&self, id: ContentId, user_id: UserId) -> Result<bool>;

    async fn delete(&self, id: ContentId) -> Result<Content>;
}

#[derive(Debug)]
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
