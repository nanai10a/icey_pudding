use async_trait::async_trait;
use mongodb::Collection;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::entities::{Author, Content, User};

type StdResult<T, E> = ::std::result::Result<T, E>;
type Result<T> = ::std::result::Result<T, RepositoryError>;

pub trait Same {
    fn is_same(&self, other: &Self) -> bool;
}

#[async_trait]
pub trait UserRepository: Send + Sync + Clone + Same {
    async fn insert(&self, item: User) -> Result<()>;
    async fn is_exists(&self, id: u64) -> Result<bool>;
    async fn find(&self, id: u64) -> Result<User>;
    async fn is_posted(&self, id: u64, content_id: Uuid) -> Result<bool>;
    async fn is_bookmarked(&self, id: u64, content_id: Uuid) -> Result<bool>;
    async fn update(&self, id: u64, mutation: UserMutation) -> Result<User>;
    async fn delete(&self, id: u64) -> Result<User>;
}

#[async_trait]
pub trait ContentRepository: Send + Sync + Clone + Same {
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

#[async_trait]
impl UserRepository for InMemoryRepository<User> {
    async fn insert(&self, item: User) -> Result<()> {
        self.0.lock().await.push(item);

        Ok(())
    }

    async fn is_exists(&self, id: u64) -> Result<bool> {
        let guard = self.0.lock().await;

        match guard.iter().filter(|v| *v.id == id).count() {
            0 => Ok(false),
            1 => Ok(true),
            i => Err(RepositoryError::NoUnique { matched: i }),
        }
    }

    async fn find(&self, id: u64) -> Result<User> {
        let guard = self.0.lock().await;
        let res = guard.iter().filter(|v| *v.id == id).collect::<Vec<_>>();

        match res.len() {
            0 => Err(RepositoryError::NotFound),
            1 => Ok(res.remove(0).clone()),
            i => Err(RepositoryError::NoUnique { matched: i }),
        }
    }

    async fn is_posted(&self, id: u64, content_id: Uuid) -> Result<bool> {
        let item = self.find(id).await?;

        match item.posted.iter().filter(|v| *v == content_id).count() {
            0 => Ok(false),
            1 => Ok(true),
            i => Err(RepositoryError::NoUnique { matched: i }),
        }
    }

    async fn is_bookmarked(&self, id: u64, content_id: Uuid) -> Result<bool> {
        let item = self.find(id).await?;

        match item.bookmark.iter().filter(|v| *v == content_id).count() {
            0 => Ok(false),
            1 => Ok(true),
            i => Err(RepositoryError::NoUnique { matched: i }),
        }
    }

    async fn update(&self, id: u64, mutation: UserMutation) -> Result<User> {
        let guard = self.0.lock().await;
        let res = guard.iter_mut().filter(|v| *v.id == id).collect::<Vec<_>>();
        let item = match res.len() {
            0 => return Err(RepositoryError::NotFound),
            1 => res.remove(0),
            i => return Err(RepositoryError::NoUnique { matched: i }),
        };

        let UserMutation { admin, sub_admin } = mutation;
        if let Some(val) = admin {
            item.admin = val;
        }
        if let Some(val) = sub_admin {
            item.sub_admin = val;
        }

        Ok(item.clone())
    }

    async fn delete(&self, id: u64) -> Result<User> {
        let guard = self.0.lock().await;
        let res = guard
            .iter()
            .enumerate()
            .filter(|(_, v)| *v.id == id)
            .map(|(i, _)| i)
            .collect::<Vec<_>>();

        let index = match res.len() {
            0 => return Err(RepositoryError::NotFound),
            1 => res.remove(0),
            i => return Err(RepositoryError::NoUnique { matched: i }),
        };

        Ok(guard.remove(index))
    }
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

pub struct InMemoryRepository<T>(Mutex<Vec<T>>);



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


