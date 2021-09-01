use std::ops::RangeBounds;

use async_trait::async_trait;
use tokio::sync::Mutex;
use uuid::Uuid;

use super::{
    ContentMutation, ContentQuery, ContentRepository, RepositoryError, Result, UserMutation,
    UserQuery, UserRepository,
};
use crate::entities::{Content, User};

pub struct InMemoryRepository<T>(Mutex<Vec<T>>);

impl<T> InMemoryRepository<T> {
    pub fn new() -> Self { Self(Mutex::new(vec![])) }
}
impl<T> Default for InMemoryRepository<T> {
    fn default() -> Self { Self::new() }
}

#[inline]
fn find_mut<T, P>(v: &mut Vec<T>, preficate: P) -> Result<&mut T>
where P: FnMut(&&mut T) -> bool {
    let mut res = v.iter_mut().filter(preficate).collect::<Vec<_>>();

    match res.len() {
        0 => Err(RepositoryError::NotFound),
        1 => Ok(res.remove(0)),
        i => Err(RepositoryError::NoUnique { matched: i as u32 }),
    }
}

#[inline]
fn find_ref<T, P>(v: &Vec<T>, preficate: P) -> Result<&T>
where P: FnMut(&&T) -> bool {
    let mut res = v.iter().filter(preficate).collect::<Vec<_>>();

    match res.len() {
        0 => Err(RepositoryError::NotFound),
        1 => Ok(res.remove(0)),
        i => Err(RepositoryError::NoUnique { matched: i as u32 }),
    }
}

#[async_trait]
impl UserRepository for InMemoryRepository<User> {
    async fn insert(&self, item: User) -> Result<bool> {
        let mut guard = self.0.lock().await;

        match find_ref(&guard, |v| v.id == item.id) {
            Ok(_) => return Ok(false),
            Err(RepositoryError::NotFound) => (),
            Err(e) => return Err(e),
        }

        guard.push(item);
        Ok(true)
    }

    async fn is_exists(&self, id: u64) -> Result<bool> {
        let guard = self.0.lock().await;

        match find_ref(&guard, |v| v.id == id) {
            Ok(_) => Ok(true),
            Err(RepositoryError::NotFound) => Ok(false),
            Err(e) => Err(e),
        }
    }

    async fn find(&self, id: u64) -> Result<User> {
        let guard = self.0.lock().await;

        Ok(find_ref(&guard, |v| v.id == id)?.clone())
    }

    async fn finds(
        &self,
        UserQuery {
            posted,
            posted_num,
            bookmark,
            bookmark_num,
        }: UserQuery,
    ) -> Result<Vec<User>> {
        Ok(self
            .0
            .lock()
            .await
            .iter()
            .filter(|u| {
                posted
                    .as_ref()
                    .map(|s| s.is_subset(&u.posted))
                    .unwrap_or(true)
            })
            .filter(|u| {
                posted_num
                    .as_ref()
                    .map(|b| b.contains(&(u.posted.len() as u32)))
                    .unwrap_or(true)
            })
            .filter(|u| {
                bookmark
                    .as_ref()
                    .map(|s| s.is_subset(&u.bookmark))
                    .unwrap_or(true)
            })
            .filter(|u| {
                bookmark_num
                    .as_ref()
                    .map(|b| b.contains(&(u.bookmark.len() as u32)))
                    .unwrap_or(true)
            })
            .cloned()
            .collect())
    }

    async fn update(&self, id: u64, mutation: UserMutation) -> Result<User> {
        let mut guard = self.0.lock().await;
        let mut res = guard.iter_mut().filter(|v| v.id == id).collect::<Vec<_>>();
        let item = match res.len() {
            0 => return Err(RepositoryError::NotFound),
            1 => res.remove(0),
            i => return Err(RepositoryError::NoUnique { matched: i as u32 }),
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
        let guard = self.0.lock().await;
        let item = find_ref(&guard, |u| u.id == id)?;

        match item.posted.iter().filter(|v| **v == content_id).count() {
            0 => Ok(false),
            1 => Ok(true),
            i => Err(RepositoryError::NoUnique { matched: i as u32 }),
        }
    }

    async fn insert_posted(&self, id: u64, content_id: Uuid) -> Result<bool> {
        let mut guard = self.0.lock().await;
        let mut item = find_mut(&mut guard, |u| u.id == id)?;

        Ok(item.posted.insert(content_id))
    }

    async fn delete_posted(&self, id: u64, content_id: Uuid) -> Result<bool> { unimplemented!() }

    async fn is_bookmarked(&self, id: u64, content_id: Uuid) -> Result<bool> {
        let item = self.find(id).await?;

        match item.bookmark.iter().filter(|v| **v == content_id).count() {
            0 => Ok(false),
            1 => Ok(true),
            i => Err(RepositoryError::NoUnique { matched: i as u32 }),
        }
    }

    async fn insert_bookmarked(&self, id: u64, content_id: Uuid) -> Result<bool> {
        unimplemented!()
    }

    async fn delete_bookmarked(&self, id: u64, content_id: Uuid) -> Result<bool> {
        unimplemented!()
    }

    async fn delete(&self, id: u64) -> Result<User> {
        let mut guard = self.0.lock().await;
        let mut res = guard
            .iter()
            .enumerate()
            .filter(|(_, v)| v.id == id)
            .map(|(i, _)| i)
            .collect::<Vec<_>>();

        let index = match res.len() {
            0 => return Err(RepositoryError::NotFound),
            1 => res.remove(0),
            i => return Err(RepositoryError::NoUnique { matched: i as u32 }),
        };

        Ok(guard.remove(index))
    }
}

#[async_trait]
impl ContentRepository for InMemoryRepository<Content> {
    async fn insert(&self, item: Content) -> Result<bool> { unimplemented!() }

    async fn is_exists(&self, id: Uuid) -> Result<bool> { unimplemented!() }

    async fn find(&self, id: Uuid) -> Result<Content> { unimplemented!() }

    async fn finds(&self, query: ContentQuery) -> Result<Vec<Content>> { unimplemented!() }

    async fn update(&self, id: Uuid, mutation: ContentMutation) -> Result<Content> {
        unimplemented!()
    }

    async fn is_liked(&self, id: Uuid, user_id: u64) -> Result<bool> { unimplemented!() }

    async fn insert_liked(&self, id: Uuid, user_id: u64) -> Result<bool> { unimplemented!() }

    async fn delete_liked(&self, id: Uuid, user_id: u64) -> Result<bool> { unimplemented!() }

    async fn is_pinned(&self, id: Uuid, user_id: u64) -> Result<bool> { unimplemented!() }

    async fn insert_pinned(&self, id: Uuid, user_id: u64) -> Result<bool> { unimplemented!() }

    async fn delete_pinned(&self, id: Uuid, user_id: u64) -> Result<bool> { unimplemented!() }

    async fn delete(&self, id: Uuid) -> Result<Content> { unimplemented!() }
}
