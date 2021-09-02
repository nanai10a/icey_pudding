use std::collections::HashSet;

use anyhow::anyhow;
use async_trait::async_trait;
use mongodb::bson::doc;
use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};
use serenity::futures::TryStreamExt;
use uuid::Uuid;

use super::{
    ContentMutation, ContentQuery, ContentRepository, RepositoryError, Result, StdResult,
    UserMutation, UserQuery, UserRepository,
};
use crate::entities::{Content, User};

pub struct MongoUserRepository {
    main_coll: Collection<MongoUserModel>,
    posted_coll: Collection<MongoUserPostedModel>,
    bookmark_coll: Collection<MongoUserBookmarkModel>,
}

pub struct MongoContentRepository {
    main_coll: Collection<MongoContentModel>,
    liked_coll: Collection<MongoContentLikedModel>,
    pinned_coll: Collection<MongoContentPinnedModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MongoUserModel {
    id: String,
    admin: bool,
    sub_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MongoUserPostedModel {
    id: String,
    set: HashSet<Uuid>,
    size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MongoUserBookmarkModel {
    id: String,
    set: HashSet<Uuid>,
    size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MongoContentModel {
    id: Uuid,
    author: MongoContentAuthorModel,
    posted: String,
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum MongoContentAuthorModel {
    User {
        id: String,
        name: String,
        nick: Option<String>,
    },
    Virtual(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MongoContentLikedModel {
    id: Uuid,
    set: HashSet<String>,
    size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MongoContentPinnedModel {
    id: Uuid,
    set: HashSet<String>,
    size: i64,
}

#[async_trait]
impl UserRepository for MongoUserRepository {
    async fn insert(
        &self,
        User {
            id,
            admin,
            sub_admin,
            posted,
            bookmark,
        }: User,
    ) -> Result<bool> {
        let main_model = MongoUserModel {
            id: id.to_string(),
            admin,
            sub_admin,
        };
        let posted_model = MongoUserPostedModel {
            id: id.to_string(),
            set: posted,
            size: posted.len() as i64,
        };
        let bookmark_model = MongoUserBookmarkModel {
            id: id.to_string(),
            set: bookmark,
            size: bookmark.len() as i64,
        };

        self.main_coll.insert_one(main_model, None).await.cvt()?;
        self.posted_coll
            .insert_one(posted_model, None)
            .await
            .cvt()?;
        self.bookmark_coll
            .insert_one(bookmark_model, None)
            .await
            .cvt()?;

        Ok(true)
    }

    async fn is_exists(&self, id: u64) -> Result<bool> {
        match self
            .main_coll
            .count_documents(doc! { "id": id.to_string() }, None)
            .await
            .cvt()?
        {
            0 => Ok(false),
            1 => Ok(true),
            i => Err(RepositoryError::NoUnique { matched: i as u32 }),
        }
    }

    async fn find(&self, id: u64) -> Result<User> {
        let mut res = self
            .main_coll
            .find(doc! { "id": id.to_string() }, None)
            .await
            .cvt()?
            .try_collect::<Vec<_>>()
            .await
            .cvt()?;
        let MongoUserModel {
            id: id_str,
            admin,
            sub_admin,
        } = match res.len() {
            0 => return Err(RepositoryError::NotFound),
            1 => res.remove(0),
            i => return Err(RepositoryError::NoUnique { matched: i as u32 }),
        };
        assert_eq!(id_str, id.to_string(), "not matched id!");

        let posted = self
            .posted_coll(id)
            .find(doc! {}, None)
            .await
            .cvt()?
            .try_collect::<HashSet<_>>()
            .await
            .cvt()?;

        let bookmark = self
            .bookmarked_coll(id)
            .find(doc! {}, None)
            .await
            .cvt()?
            .try_collect::<HashSet<_>>()
            .await
            .cvt()?;

        Ok(User {
            id,
            admin,
            sub_admin,
            posted,
            bookmark,
        })
    }

    async fn finds(&self, query: UserQuery) -> Result<Vec<User>> { unimplemented!() }

    async fn update(&self, id: u64, mutation: UserMutation) -> Result<User> { unimplemented!() }

    async fn is_posted(&self, id: u64, content_id: Uuid) -> Result<bool> { unimplemented!() }

    async fn insert_posted(&self, id: u64, content_id: Uuid) -> Result<bool> { unimplemented!() }

    async fn delete_posted(&self, id: u64, content_id: Uuid) -> Result<bool> { unimplemented!() }

    async fn is_bookmarked(&self, id: u64, content_id: Uuid) -> Result<bool> { unimplemented!() }

    async fn insert_bookmarked(&self, id: u64, content_id: Uuid) -> Result<bool> {
        unimplemented!()
    }

    async fn delete_bookmarked(&self, id: u64, content_id: Uuid) -> Result<bool> {
        unimplemented!()
    }

    async fn delete(&self, id: u64) -> Result<User> { unimplemented!() }
}

#[async_trait]
impl ContentRepository for MongoContentRepository {
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

trait Convert<T> {
    fn cvt(self) -> T;
}
impl<T, E: Sync + Send + ::std::error::Error + 'static> Convert<Result<T>> for StdResult<T, E> {
    fn cvt(self) -> Result<T> { self.map_err(|e| RepositoryError::Internal(anyhow!(e))) }
}

trait Dispose {
    fn dispose(self);
}
impl<T> Dispose for T {
    fn dispose(self) {}
}
