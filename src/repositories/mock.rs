use async_trait::async_trait;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::{ContentRepository, RepositoryError, Result, UserMutation, UserRepository};
use crate::entities::{Content, User};

pub struct InMemoryRepository<T>(Mutex<Vec<T>>);

#[async_trait]
impl UserRepository for InMemoryRepository<User> {
    async fn insert(&self, item: User) -> Result<bool> {
        self.0.lock().await.push(item);

        Ok(unimplemented!())
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

    async fn finds(&self, query: super::UserQuery) -> Result<Vec<User>> {
        unimplemented!()
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

    async fn is_posted(&self, id: u64, content_id: Uuid) -> Result<bool> {
        let item = self.find(id).await?;

        match item.posted.iter().filter(|v| *v == content_id).count() {
            0 => Ok(false),
            1 => Ok(true),
            i => Err(RepositoryError::NoUnique { matched: i }),
        }
    }

    async fn insert_posted(&self, id: u64, content_id: Uuid) -> Result<bool> {
        unimplemented!()
    }

    async fn delete_posted(&self, id: u64, content_id: Uuid) -> Result<bool> {
        unimplemented!()
    }

    async fn is_bookmarked(&self, id: u64, content_id: Uuid) -> Result<bool> {
        let item = self.find(id).await?;

        match item.bookmark.iter().filter(|v| *v == content_id).count() {
            0 => Ok(false),
            1 => Ok(true),
            i => Err(RepositoryError::NoUnique { matched: i }),
        }
    }

    async fn insert_bookmarked(&self, id: u64, content_id: Uuid) -> Result<bool> {
        unimplemented!()
    }

    async fn delete_bookmarked(&self, id: u64, content_id: Uuid) -> Result<bool> {
        unimplemented!()
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

#[async_trait]
impl ContentRepository for InMemoryRepository<Content> {
    async fn insert(&self, item: Content) -> Result<bool> {
        unimplemented!()
    }

    async fn is_exists(&self, id: Uuid) -> Result<bool> {
        unimplemented!()
    }

    async fn find(&self, id: Uuid) -> Result<Content> {
        unimplemented!()
    }

    async fn finds(&self, query: super::ContentQuery) -> Result<Vec<Content>> {
        unimplemented!()
    }

    async fn update(&self, mutation: super::ContentMutation) -> Result<Content> {
        unimplemented!()
    }

    async fn is_liked(&self, id: Uuid, user_id: u64) -> Result<bool> {
        unimplemented!()
    }

    async fn insert_liked(&self, id: Uuid, user_id: u64) -> Result<bool> {
        unimplemented!()
    }

    async fn delete_liked(&self, id: Uuid, user_id: u64) -> Result<bool> {
        unimplemented!()
    }

    async fn is_pinned(&self, id: Uuid, user_id: u64) -> Result<bool> {
        unimplemented!()
    }

    async fn insert_pinned(&self, id: Uuid, user_id: u64) -> Result<bool> {
        unimplemented!()
    }

    async fn delete_pinned(&self, id: Uuid, user_id: u64) -> Result<bool> {
        unimplemented!()
    }

    async fn delete(&self, id: Uuid) -> Result<Content> {
        unimplemented!()
    }
}
