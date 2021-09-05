use std::collections::HashSet;
use std::ops::Bound;

use anyhow::anyhow;
use async_trait::async_trait;
use mongodb::bson::doc;
use mongodb::{Client, Collection, Database};
use serde::{Deserialize, Serialize};
use serenity::futures::TryStreamExt;
use uuid::Uuid;

use super::{
    ContentMutation, ContentQuery, ContentRepository, RepositoryError, Result, StdResult,
    UserMutation, UserQuery, UserRepository,
};
use crate::entities::{Author, Content, Posted, User};

// FIXME: mustn't use multiple documents. (no manually separate)
pub struct MongoUserRepository {
    coll: Collection<MongoUserModel>,
}

impl MongoUserRepository {
    pub async fn new_with(db: Database) -> ::anyhow::Result<Self> {
        db.run_command(
            doc! {
                "createIndexes": "user",
                "indexes": [{
                    "name": "unique_id",
                    "key": {
                        "id": 1
                    },
                    "unique": true
                }],
            },
            None,
        )
        .await
        .map_err(::anyhow::Error::new)?;

        let coll = db.collection("user#main");

        Ok(Self { coll })
    }
}

pub struct MongoContentRepository {
    coll: Collection<MongoContentModel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MongoUserModel {
    id: String,
    admin: bool,
    sub_admin: bool,
    posted: HashSet<Uuid>,
    posted_size: i64,
    bookmark: HashSet<Uuid>,
    bookmark_size: i64,
}
impl Into<User> for MongoUserModel {
    fn into(self) -> User {
        let MongoUserModel {
            id,
            admin,
            sub_admin,
            posted,
            posted_size: _,
            bookmark,
            bookmark_size: _,
        } = self;

        User {
            id: id.parse().unwrap(),
            admin,
            sub_admin,
            posted,
            bookmark,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MongoContentModel {
    id: Uuid,
    author: MongoContentAuthorModel,
    posted: MongoContentPostedModel,
    content: String,
    liked: HashSet<String>,
    liked_size: i64,
    pinned: HashSet<String>,
    pinned_size: i64,
}
impl Into<Content> for MongoContentModel {
    fn into(self) -> Content {
        let MongoContentModel {
            id,
            author,
            posted,
            content,
            liked,
            liked_size: _,
            pinned,
            pinned_size: _,
        } = self;

        Content {
            id,
            author: author.into(),
            posted: posted.into(),
            content,
            liked: liked.drain().map(|s| s.parse().unwrap()).collect(),
            pinned: pinned.drain().map(|s| s.parse().unwrap()).collect(),
        }
    }
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
impl Into<Author> for MongoContentAuthorModel {
    fn into(self) -> Author {
        match self {
            MongoContentAuthorModel::User { id, name, nick } => Author::User {
                id: id.parse().unwrap(),
                name,
                nick,
            },
            MongoContentAuthorModel::Virtual(s) => Author::Virtual(s),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MongoContentPostedModel {
    id: String,
    name: String,
    nick: Option<String>,
}
impl Into<Posted> for MongoContentPostedModel {
    fn into(self) -> Posted {
        let MongoContentPostedModel { id, name, nick } = self;

        Posted {
            id: id.parse().unwrap(),
            name,
            nick,
        }
    }
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
        let model = MongoUserModel {
            id: id.to_string(),
            admin,
            sub_admin,
            posted_size: posted.len() as i64,
            posted,
            bookmark_size: bookmark.len() as i64,
            bookmark,
        };

        // FIXME: transaction begin ---
        let res = self.coll.insert_one(model, None).await.unique_check()?;
        // --- end

        // FIXME: if partially failed to insert doc, other logics will fall.
        // because "if can fetch data from `main_coll`, then must able to fetch
        // [sub_coll]" (logics)

        Ok(res)
    }

    async fn is_exists(&self, id: u64) -> Result<bool> {
        // FIXME: transaction begin ---
        let res = self
            .coll
            .count_documents(doc! { "id": id.to_string() }, None)
            .await
            .cvt()?
            .into_bool();
        // --- end

        Ok(res)
    }

    async fn find(&self, id: u64) -> Result<User> {
        // FIXME: transaction begin ---
        let MongoUserModel {
            id: id_str,
            admin,
            sub_admin,
            posted,
            bookmark,
            ..
        } = self
            .coll
            .find_one(doc! { "id": id.to_string() }, None)
            .await
            .cvt()?
            .opt_cvt()?;
        assert_eq!(id_str, id.to_string(), "not matched id!"); // FIXME: checking only this?

        // --- end

        Ok(User {
            id,
            admin,
            sub_admin,
            posted,
            bookmark,
        })
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
        // FIXME: do transaction
        let mut query = doc! {};

        if let Some(mut set_raw) = posted {
            if !set_raw.is_empty() {
                let set = set_raw.drain().map(|i| i.to_string()).collect::<Vec<_>>();
                query.insert("posted", doc! { "$in": set });
            }
        }
        if let Some((g, l)) = posted_num {
            let mut posted_num_q = doc! {};
            match g {
                Bound::Unbounded => (),
                Bound::Included(n) => posted_num_q.insert("$gte", n).dispose(),
                Bound::Excluded(n) => posted_num_q.insert("$gt", n).dispose(),
            }
            match l {
                Bound::Unbounded => (),
                Bound::Included(n) => posted_num_q.insert("$lte", n).dispose(),
                Bound::Excluded(n) => posted_num_q.insert("$lt", n).dispose(),
            }
            if !posted_num_q.is_empty() {
                query.insert("posted_size", posted_num_q);
            }
        }
        if let Some(mut set_raw) = bookmark {
            if !set_raw.is_empty() {
                let set = set_raw.drain().map(|i| i.to_string()).collect::<Vec<_>>();
                query.insert("bookmark", doc! { "$in": set });
            }
        }
        if let Some((g, l)) = bookmark_num {
            let mut bookmark_num_q = doc! {};
            match g {
                Bound::Unbounded => (),
                Bound::Included(n) => bookmark_num_q.insert("$gte", n).dispose(),
                Bound::Excluded(n) => bookmark_num_q.insert("$gt", n).dispose(),
            }
            match l {
                Bound::Unbounded => (),
                Bound::Included(n) => bookmark_num_q.insert("$lte", n).dispose(),
                Bound::Excluded(n) => bookmark_num_q.insert("$lt", n).dispose(),
            }
            if !bookmark_num_q.is_empty() {
                query.insert("bookmark_size", bookmark_num_q);
            }
        }

        let res = self
            .coll
            .find(query, None)
            .await
            .cvt()?
            .try_collect::<Vec<_>>()
            .await
            .cvt()?
            .drain(..)
            .map(|m| m.into())
            .collect();

        Ok(res)
    }

    async fn update(
        &self,
        id: u64,
        UserMutation { admin, sub_admin }: UserMutation,
    ) -> Result<User> {
        let mut mutation = doc! {};
        if let Some(val) = admin {
            mutation.insert("admin", val);
        }
        if let Some(val) = sub_admin {
            mutation.insert("sub_admin", val);
        }

        // FIXME: transaction begin ---
        self.coll
            .update_one(
                doc! { "id": id.to_string() },
                doc! { "$set": mutation },
                None,
            )
            .await
            .cvt()?
            .matched_count
            .into_bool()
            .expect_true()?;

        self.find(id).await
        // --- end
    }

    async fn is_posted(&self, id: u64, content_id: Uuid) -> Result<bool> {
        let res = self
            .coll
            .count_documents(
                doc! { "id": id.to_string(), "posted": { "$in": [content_id.to_string()] } },
                None,
            )
            .await
            .cvt()?
            .into_bool();

        Ok(res)
    }

    async fn insert_posted(&self, id: u64, content_id: Uuid) -> Result<bool> {
        let res = self
            .posted_coll
            .update_one(
                doc! { "id": id.to_string() },
                doc! { "$addToSet": { "set": content_id.to_string() } },
                None,
            )
            .await
            .cvt()?;

        res.matched_count.into_bool().expect_true()?;
        Ok(res.modified_count.into_bool())
    }

    async fn delete_posted(&self, id: u64, content_id: Uuid) -> Result<bool> {
        let res = self
            .posted_coll
            .update_one(
                doc! { "id": id.to_string() },
                doc! { "$pull": { "set": content_id.to_string() } },
                None,
            )
            .await
            .cvt()?;

        res.matched_count.into_bool().expect_true()?;
        Ok(res.modified_count.into_bool())
    }

    async fn is_bookmarked(&self, id: u64, content_id: Uuid) -> Result<bool> {
        let res = self
            .coll
            .count_documents(
                doc! { "id": id.to_string(), "bookmark": { "$in": [content_id.to_string()] } },
                None,
            )
            .await
            .cvt()?
            .into_bool();

        Ok(res)
    }

    async fn insert_bookmarked(&self, id: u64, content_id: Uuid) -> Result<bool> {
        let res = self
            .bookmark_coll
            .update_one(
                doc! { "id": id.to_string() },
                doc! { "$addToSet": { "set": content_id.to_string() } },
                None,
            )
            .await
            .cvt()?;

        res.matched_count.into_bool().expect_true()?;
        Ok(res.modified_count.into_bool())
    }

    async fn delete_bookmarked(&self, id: u64, content_id: Uuid) -> Result<bool> {
        let res = self
            .bookmark_coll
            .update_one(
                doc! { "id": id.to_string() },
                doc! { "$pull": { "set": content_id.to_string() } },
                None,
            )
            .await
            .cvt()?;

        res.matched_count.into_bool().expect_true()?;
        Ok(res.modified_count.into_bool())
    }

    async fn delete(&self, id: u64) -> Result<User> {
        // FIXME: transaction begin ---
        let user = self.find(id).await?;

        let main_res = self
            .main_coll
            .delete_one(doc! { "id": id.to_string() }, None)
            .await
            .cvt()?
            .deleted_count
            .into_bool();
        let posted_res = self
            .posted_coll
            .delete_one(doc! { "id": id.to_string() }, None)
            .await
            .cvt()?
            .deleted_count
            .into_bool();
        let bookmark_res = self
            .bookmark_coll
            .delete_one(doc! { "id": id.to_string() }, None)
            .await
            .cvt()?
            .deleted_count
            .into_bool();
        // --- end

        // `::into_bool` is checking "is `0 | 1`" (= "unique")

        match (main_res, posted_res, bookmark_res) {
            (true, true, true) => Ok(user),
            (false, false, false) => Err(RepositoryError::NotFound),
            _ => unreachable!(
                "delete was partially failed: [main: {}] [posted: {}] [bookmark: {}]",
                main_res, posted_res, bookmark_res
            ),
        }
    }
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

trait NumToBool {
    fn into_bool(self) -> bool;
}
impl<N: ::core::convert::TryInto<i8> + ::core::fmt::Debug + Copy> NumToBool for N {
    fn into_bool(self) -> bool {
        match match ::core::convert::TryInto::<i8>::try_into(self) {
            Ok(n) => n,
            Err(_) => unreachable!("expected 0 or 1, found: {:?}", self),
        } {
            0 => false,
            1 => true,
            n => unreachable!("expected 0 or 1, found: {}", n),
        }
    }
}

trait BoolToErr {
    fn expect_true(self) -> Result<()>;
}
impl BoolToErr for bool {
    fn expect_true(self) -> Result<()> {
        match self {
            true => Ok(()),
            false => Err(RepositoryError::NotFound),
        }
    }
}
