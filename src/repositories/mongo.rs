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

macro_rules! inline {
    ($n:expr) => {
        doc! {
            "createIndexes": $n,
            "indexes": [{
                "name": "unique_id",
                "key": {
                    "id": 1
                },
                "unique": true
            }],
        }
    };
}
impl MongoUserRepository {
    pub async fn new_with(db: Database) -> ::anyhow::Result<Self> {
        for name in vec!["user#main", "user#posted", "user#bookmark"].drain(..) {
            db.run_command(inline!(name), None)
                .await
                .map_err(::anyhow::Error::new)?;
        }

        let main_coll = db.collection("user#main");
        let posted_coll = db.collection("user#posted");
        let bookmark_coll = db.collection("user#bookmark");

        Ok(Self {
            main_coll,
            posted_coll,
            bookmark_coll,
        })
    }
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
            size: posted.len() as i64,
            set: posted,
        };
        let bookmark_model = MongoUserBookmarkModel {
            id: id.to_string(),
            size: bookmark.len() as i64,
            set: bookmark,
        };

        let main_res = self
            .main_coll
            .insert_one(main_model, None)
            .await
            .unique_check()?;
        let posted_res = self
            .posted_coll
            .insert_one(posted_model, None)
            .await
            .unique_check()?;
        let bookmark_res = self
            .bookmark_coll
            .insert_one(bookmark_model, None)
            .await
            .unique_check()?;

        Ok(main_res && posted_res && bookmark_res)
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
            i => unreachable!("expected 0..=1 value, found {} values!", i),
        }
    }

    async fn find(&self, id: u64) -> Result<User> {
        let MongoUserModel {
            id: id_str,
            admin,
            sub_admin,
        } = self
            .main_coll
            .find_one(doc! { "id": id.to_string() }, None)
            .await
            .cvt()?
            .opt_cvt()?;
        assert_eq!(id_str, id.to_string(), "not matched id!");

        let MongoUserPostedModel { set: posted, .. } = self
            .posted_coll
            .find_one(doc! { "id": id.to_string() }, None)
            .await
            .cvt()?
            .opt_cvt()?;

        let MongoUserBookmarkModel { set: bookmark, .. } = self
            .bookmark_coll
            .find_one(doc! { "id": id.to_string() }, None)
            .await
            .cvt()?
            .opt_cvt()?;

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

trait DetectUniqueErr {
    fn unique_check(self) -> Result<bool>;
}
impl<T> DetectUniqueErr for ::mongodb::error::Result<T> {
    fn unique_check(self) -> Result<bool> {
        match match match self {
            Ok(_) => return Ok(true),
            Err(e) => (*e.kind.clone(), e),
        } {
            (
                ::mongodb::error::ErrorKind::Write(::mongodb::error::WriteFailure::WriteError(e)),
                src,
            ) => (e.code, src),
            (_, src) => return Err(RepositoryError::Internal(anyhow!(src))),
        } {
            (11000, _) => Ok(false),
            (_, src) => Err(RepositoryError::Internal(anyhow!(src))),
        }
    }
}

trait OptToErr<T> {
    fn opt_cvt(self) -> Result<T>;
}
impl<T> OptToErr<T> for Option<T> {
    fn opt_cvt(self) -> Result<T> {
        match self {
            Some(o) => Ok(o),
            None => Err(RepositoryError::NotFound),
        }
    }
}
