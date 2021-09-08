use std::collections::HashSet;
use std::ops::Bound;

use anyhow::anyhow;
use async_trait::async_trait;
use mongodb::bson::{doc, Document};
use mongodb::options::{Acknowledgment, ReadConcern, TransactionOptions, WriteConcern};
use mongodb::{bson, Client, ClientSession, Collection, Database};
use serde::{Deserialize, Serialize};
use serenity::futures::TryStreamExt;

use super::{
    AuthorQuery, ContentContentMutation, ContentMutation, ContentQuery, ContentRepository,
    PostedQuery, RepositoryError, Result, UserMutation, UserQuery, UserRepository,
};
use crate::entities::{Author, Content, ContentId, User, UserId};
use crate::utils::{self, LetChain};

mod type_convert;

pub(crate) struct MongoUserRepository {
    client: Client,
    coll: Collection<MongoUserModel>,
}

impl MongoUserRepository {
    pub(crate) async fn new_with(client: Client, db: Database) -> ::anyhow::Result<Self> {
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

        let coll = db.collection("user");

        Ok(Self { client, coll })
    }
}

pub(crate) struct MongoContentRepository {
    client: Client,
    coll: Collection<MongoContentModel>,
}

impl MongoContentRepository {
    pub(crate) async fn new_with(client: Client, db: Database) -> ::anyhow::Result<Self> {
        db.run_command(
            doc! {
                "createIndexes": "content",
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

        let coll = db.collection("content");

        Ok(Self { client, coll })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MongoUserModel {
    id: String,
    admin: bool,
    sub_admin: bool,
    posted: HashSet<ContentId>,
    posted_size: i64,
    bookmark: HashSet<ContentId>,
    bookmark_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MongoContentModel {
    id: ContentId,
    author: MongoContentAuthorModel,
    posted: MongoContentPostedModel,
    content: String,
    liked: HashSet<String>,
    liked_size: i64,
    pinned: HashSet<String>,
    pinned_size: i64,
    created: String,
    edited: Vec<String>,
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
struct MongoContentPostedModel {
    id: String,
    name: String,
    nick: Option<String>,
}

macro_rules! exec_transaction {
    ($f:expr $( , $a:expr )*) => {
        async {
            loop {
                let r = $f($( $a, )*).await;
                if let Err(ref e) = r {
                    if e.contains_label(::mongodb::error::TRANSIENT_TRANSACTION_ERROR) {
                        continue;
                    }

                    break r;
                }
            }
        }
    };
}

#[async_trait]
impl UserRepository for MongoUserRepository {
    async fn insert(&self, user: User) -> Result<bool> {
        let model: MongoUserModel = user.into();

        let res = self
            .coll
            .insert_one(model, None)
            .await
            .let_(try_unique_check)?;

        Ok(res)
    }

    async fn is_exists(&self, id: UserId) -> Result<bool> {
        let res = self
            .coll
            .count_documents(doc! { "id": id }, None)
            .await
            .let_(convert_repo_err)?
            .let_(to_bool);

        Ok(res)
    }

    async fn find(&self, id: UserId) -> Result<User> {
        let user: User = self
            .coll
            .find_one(doc! { "id": id }, None)
            .await
            .let_(convert_repo_err)?
            .let_(convert_404_or)?
            .into();
        assert_eq!(user.id, id, "not matched id!");

        Ok(user)
    }

    async fn finds(&self, query: UserQuery) -> Result<Vec<User>> {
        let query_doc: Document = query.into();

        let res = self
            .coll
            .find(query_doc, None)
            .await
            .let_(convert_repo_err)?
            .try_collect::<Vec<_>>()
            .await
            .let_(convert_repo_err)?
            .drain(..)
            .map(|m| m.into())
            .collect();

        Ok(res)
    }

    async fn update(&self, id: UserId, mutation: UserMutation) -> Result<User> {
        let mutation_doc: Document = mutation.into();

        async fn transaction(
            this: &MongoUserRepository,
            id: UserId,
            mutation: Document,
        ) -> ::mongodb::error::Result<Option<User>> {
            let mut session = make_session(&this.client).await?;

            match this
                .coll
                .update_one_with_session(
                    doc! { "id": id },
                    doc! { "$set": mutation },
                    None,
                    &mut session,
                )
                .await?
                .matched_count
                .let_(to_bool)
            {
                false => return Ok(None),
                true => (),
            };

            let user: User = this
                .coll
                .find_one_with_session(doc! { "id": id }, None, &mut session)
                .await?
                .unwrap()
                .into();
            assert_eq!(user.id, id, "not matched id!");

            process_transaction(&mut session).await.map(|_| Some(user))
        }

        let res = exec_transaction!(transaction, self, id, mutation_doc.clone()).await;
        Ok(res.let_(convert_repo_err)?.let_(convert_404_or)?)
    }

    async fn is_posted(&self, id: UserId, content_id: ContentId) -> Result<bool> {
        is_contains("posted", &self.coll, id.to_string(), content_id.to_string()).await
    }

    async fn insert_posted(&self, id: UserId, content_id: ContentId) -> Result<bool> {
        modify_set(
            "posted",
            &self.coll,
            &self.client,
            id.to_string(),
            content_id.to_string(),
            ModifyOpTy::Push,
        )
        .await
    }

    async fn delete_posted(&self, id: UserId, content_id: ContentId) -> Result<bool> {
        modify_set(
            "posted",
            &self.coll,
            &self.client,
            id.to_string(),
            content_id.to_string(),
            ModifyOpTy::Pull,
        )
        .await
    }

    async fn is_bookmark(&self, id: UserId, content_id: ContentId) -> Result<bool> {
        is_contains(
            "bookmark",
            &self.coll,
            id.to_string(),
            content_id.to_string(),
        )
        .await
    }

    async fn insert_bookmark(&self, id: UserId, content_id: ContentId) -> Result<bool> {
        modify_set(
            "bookmark",
            &self.coll,
            &self.client,
            id.to_string(),
            content_id.to_string(),
            ModifyOpTy::Push,
        )
        .await
    }

    async fn delete_bookmark(&self, id: UserId, content_id: ContentId) -> Result<bool> {
        modify_set(
            "bookmark",
            &self.coll,
            &self.client,
            id.to_string(),
            content_id.to_string(),
            ModifyOpTy::Pull,
        )
        .await
    }

    async fn delete(&self, id: UserId) -> Result<User> {
        async fn transaction(
            this: &MongoUserRepository,
            id: UserId,
        ) -> ::mongodb::error::Result<Option<User>> {
            let mut session = make_session(&this.client).await?;

            let user: User = match this
                .coll
                .find_one_with_session(doc! { "id": id }, None, &mut session)
                .await?
                .map(|m| m.into())
            {
                Some(u) => u,
                None => return Ok(None),
            };
            assert_eq!(user.id, id, "not matched id!");

            match this
                .coll
                .delete_one_with_session(doc! { "id": id }, None, &mut session)
                .await?
                .deleted_count
                .let_(to_bool) // checking "is `0 | 1`" (= "unique")
            {
                false => unreachable!("couldn't delete value"),
                true => (),
            };

            process_transaction(&mut session).await.map(|_| Some(user))
        }

        let res = exec_transaction!(transaction, self, id).await;
        Ok(res.let_(convert_repo_err)?.let_(convert_404_or)?)
    }
}

#[async_trait]
impl ContentRepository for MongoContentRepository {
    async fn insert(&self, content: Content) -> Result<bool> {
        let model: MongoContentModel = content.into();

        let res = self
            .coll
            .insert_one(model, None)
            .await
            .let_(try_unique_check)?;

        Ok(res)
    }

    async fn is_exists(&self, id: ContentId) -> Result<bool> {
        let res = self
            .coll
            .count_documents(doc! { "id": id }, None)
            .await
            .let_(convert_repo_err)?
            .let_(to_bool);

        Ok(res)
    }

    async fn find(&self, id: ContentId) -> Result<Content> {
        let content: Content = self
            .coll
            .find_one(doc! { "id": id }, None)
            .await
            .let_(convert_repo_err)?
            .let_(convert_404_or)?
            .into();
        assert_eq!(content.id, id, "not matched id!");

        Ok(content)
    }

    async fn finds(
        &self,
        ContentQuery {
            author,
            posted,
            content,
            liked,
            liked_num,
            pinned,
            pinned_num,
        }: ContentQuery,
    ) -> Result<Vec<Content>> {
        let query_doc = {
            let mut doc = doc! {};

            if let Some(mut set) = liked {
                if !set.is_empty() {
                    doc.insert("liked", doc! { "$in": set.drain().collect::<Vec<_>>() });
                }
            }

            if let Some((g, l)) = liked_num {
                let mut num_q = doc! {};

                match g {
                    Bound::Unbounded => (),
                    Bound::Included(n) => num_q.insert("$gte", n).let_(::core::mem::drop),
                    Bound::Excluded(n) => num_q.insert("$gt", n).let_(::core::mem::drop),
                }

                match l {
                    Bound::Unbounded => (),
                    Bound::Included(n) => num_q.insert("$lte", n).let_(::core::mem::drop),
                    Bound::Excluded(n) => num_q.insert("$lt", n).let_(::core::mem::drop),
                }

                if !num_q.is_empty() {
                    doc.insert("liked_size", num_q);
                }
            }

            if let Some(mut set) = pinned {
                if !set.is_empty() {
                    doc.insert("pinned", doc! { "$in": set.drain().collect::<Vec<_>>() });
                }
            }

            if let Some((g, l)) = pinned_num {
                let mut num_q = doc! {};

                match g {
                    Bound::Unbounded => (),
                    Bound::Included(n) => num_q.insert("$gte", n).let_(::core::mem::drop),
                    Bound::Excluded(n) => num_q.insert("$gt", n).let_(::core::mem::drop),
                }

                match l {
                    Bound::Unbounded => (),
                    Bound::Included(n) => num_q.insert("$lte", n).let_(::core::mem::drop),
                    Bound::Excluded(n) => num_q.insert("$lt", n).let_(::core::mem::drop),
                }

                if !num_q.is_empty() {
                    doc.insert("pinned_size", num_q);
                }
            }

            doc
        };

        let mut tmp_res = self
            .coll
            .find(query_doc, None)
            .await
            .let_(convert_repo_err)?
            .try_collect::<Vec<_>>()
            .await
            .let_(convert_repo_err)?
            .drain(..)
            .map::<Content, _>(|m| m.into())
            .collect::<Vec<_>>();

        let res = tmp_res
            .drain(..)
            .filter(|c| match &author {
                Some(AuthorQuery::UserId(id_q)) => match &c.author {
                    Author::User { id, .. } => id_q == id,
                    _ => false,
                },
                Some(AuthorQuery::UserName(name_q)) => match &c.author {
                    Author::User { name, .. } => name_q.is_match(name.as_str()),
                    _ => false,
                },
                Some(AuthorQuery::UserNick(nick_q)) => match &c.author {
                    Author::User { nick, .. } =>
                        nick.as_ref().map_or(false, |s| nick_q.is_match(s.as_str())),
                    _ => false,
                },
                Some(AuthorQuery::Virtual(name_q)) => match &c.author {
                    Author::Virtual(name) => name_q.is_match(name.as_str()),
                    _ => false,
                },
                Some(AuthorQuery::Any(any_q)) => match &c.author {
                    Author::User { name, nick, .. } =>
                        any_q.is_match(name.as_str())
                            || nick.as_ref().map_or(false, |s| any_q.is_match(s.as_str())),
                    Author::Virtual(name) => any_q.is_match(name.as_str()),
                },
                None => true,
            })
            .filter(|c| match &posted {
                Some(PostedQuery::UserId(id_q)) => &c.posted.id == id_q,
                Some(PostedQuery::UserName(name_q)) => name_q.is_match(c.posted.name.as_str()),
                Some(PostedQuery::UserNick(nick_q)) => c
                    .posted
                    .nick
                    .as_ref()
                    .map_or(false, |s| nick_q.is_match(s.as_str())),
                Some(PostedQuery::Any(any_q)) =>
                    any_q.is_match(c.posted.name.as_str())
                        || c.posted
                            .nick
                            .as_ref()
                            .map_or(false, |s| any_q.is_match(s.as_str())),
                None => true,
            })
            .filter(|c| match &content {
                Some(content_q) => content_q.is_match(c.content.as_str()),
                None => true,
            })
            .collect();

        Ok(res)
    }

    async fn update(&self, id: ContentId, mutation: ContentMutation) -> Result<Content> {
        async fn transaction(
            this: &MongoContentRepository,
            id: ContentId,
            ContentMutation {
                author,
                content,
                edited,
            }: ContentMutation,
        ) -> ::mongodb::error::Result<Option<Content>> {
            let mut session = make_session(&this.client).await?;

            let mut target_content: Content = match this
                .coll
                .find_one_with_session(doc! { "id": id }, None, &mut session)
                .await?
            {
                Some(c) => c.into(),
                None => return Ok(None),
            };

            if let Some(a) = author {
                target_content.author = a;
            }

            if let Some(c) = content {
                match c {
                    ContentContentMutation::Sed { capture, replace } =>
                        target_content.content = capture
                            .replace(target_content.content.as_str(), replace)
                            .to_string(),
                    ContentContentMutation::Complete(s) => target_content.content = s,
                }
            }

            let target_model: MongoContentModel = target_content.into();
            let edited_str = utils::date_to_string(edited);
            this.coll
                .update_one_with_session(
                    doc! { "id": id },
                    doc! {
                        "$set": bson::to_document(&target_model).unwrap(),
                        "$push": { "edited": edited_str }
                    },
                    None,
                    &mut session,
                )
                .await?;

            let new_content = this
                .coll
                .find_one_with_session(doc! { "id": id }, None, &mut session)
                .await?
                .unwrap()
                .into();

            process_transaction(&mut session)
                .await
                .map(|_| Some(new_content))
        }

        let res = exec_transaction!(transaction, self, id, mutation.clone()).await;
        Ok(res.let_(convert_repo_err)?.let_(convert_404_or)?)
    }

    async fn is_liked(&self, id: ContentId, user_id: UserId) -> Result<bool> {
        is_contains("liked", &self.coll, id.to_string(), user_id.to_string()).await
    }

    async fn insert_liked(&self, id: ContentId, user_id: UserId) -> Result<bool> {
        modify_set(
            "liked",
            &self.coll,
            &self.client,
            id.to_string(),
            user_id.to_string(),
            ModifyOpTy::Push,
        )
        .await
    }

    async fn delete_liked(&self, id: ContentId, user_id: UserId) -> Result<bool> {
        modify_set(
            "liked",
            &self.coll,
            &self.client,
            id.to_string(),
            user_id.to_string(),
            ModifyOpTy::Pull,
        )
        .await
    }

    async fn is_pinned(&self, id: ContentId, user_id: UserId) -> Result<bool> {
        is_contains("pinned", &self.coll, id.to_string(), user_id.to_string()).await
    }

    async fn insert_pinned(&self, id: ContentId, user_id: UserId) -> Result<bool> {
        modify_set(
            "pinned",
            &self.coll,
            &self.client,
            id.to_string(),
            user_id.to_string(),
            ModifyOpTy::Push,
        )
        .await
    }

    async fn delete_pinned(&self, id: ContentId, user_id: UserId) -> Result<bool> {
        modify_set(
            "pinned",
            &self.coll,
            &self.client,
            id.to_string(),
            user_id.to_string(),
            ModifyOpTy::Pull,
        )
        .await
    }

    async fn delete(&self, id: ContentId) -> Result<Content> {
        async fn transaction(
            this: &MongoContentRepository,
            id: ContentId,
        ) -> ::mongodb::error::Result<Option<Content>> {
            let mut session = make_session(&this.client).await?;

            let content: Content = match this
                .coll
                .find_one_with_session(doc! { "id": id }, None, &mut session)
                .await?
                .map(|m| m.into())
            {
                Some(c) => c,
                None => return Ok(None),
            };
            assert_eq!(content.id, id, "not matched id!");

            match this
                .coll
                .delete_one_with_session(doc! { "id": id }, None, &mut session)
                .await?
                .deleted_count
                .let_(to_bool)
            {
                false => unreachable!("couldn't delete value"),
                true => (),
            }

            process_transaction(&mut session)
                .await
                .map(|_| Some(content))
        }

        let res = exec_transaction!(transaction, self, id).await;
        Ok(res.let_(convert_repo_err)?.let_(convert_404_or)?)
    }
}

fn convert_repo_err<T, E>(result: ::core::result::Result<T, E>) -> Result<T>
where E: Sync + Send + ::std::error::Error + 'static {
    result.map_err(|e| RepositoryError::Internal(anyhow!(e)))
}

fn try_unique_check<T>(result: ::core::result::Result<T, ::mongodb::error::Error>) -> Result<bool> {
    match match match result {
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

fn convert_404_or<T>(option: Option<T>) -> Result<T> {
    match option {
        Some(t) => Ok(t),
        None => Err(RepositoryError::NotFound),
    }
}

fn to_bool<N>(number: N) -> bool
where N: ::core::convert::TryInto<i8> + ::core::fmt::Debug + Clone {
    match match ::core::convert::TryInto::<i8>::try_into(number.clone()) {
        Ok(n) => n,
        Err(_) => unreachable!("expected 0 or 1, found: {:?}", number),
    } {
        0 => false,
        1 => true,
        n => unreachable!("expected 0 or 1, found: {}", n),
    }
}

#[inline]
async fn make_session(c: &Client) -> ::mongodb::error::Result<ClientSession> {
    let mut s = c.start_session(None).await?;

    let ta_opt = TransactionOptions::builder()
        .read_concern(ReadConcern::snapshot())
        .write_concern(WriteConcern::builder().w(Acknowledgment::Majority).build())
        .build();
    s.start_transaction(ta_opt).await?;

    Ok(s)
}

#[inline]
async fn process_transaction(s: &mut ClientSession) -> ::mongodb::error::Result<()> {
    loop {
        let r = s.commit_transaction().await;
        if let Err(ref e) = r {
            if e.contains_label(::mongodb::error::UNKNOWN_TRANSACTION_COMMIT_RESULT) {
                continue;
            }
        }

        break r;
    }
}

#[inline]
async fn is_contains<T>(
    name: impl AsRef<str>,
    coll: &Collection<T>,
    id: impl Into<::mongodb::bson::Bson>,
    target: impl Into<::mongodb::bson::Bson>,
) -> Result<bool> {
    let res = coll
        .count_documents(
            doc! {
                "id": id.into(),
                name.as_ref(): { "$in": [target.into()] }
            },
            None,
        )
        .await
        .let_(convert_repo_err)?
        .let_(to_bool);

    Ok(res)
}

#[derive(Clone, Copy)]
enum ModifyOpTy {
    Push,
    Pull,
}
#[inline]
async fn modify_set<T>(
    name: impl AsRef<str>,
    coll: &Collection<T>,
    client: &Client,
    id: impl Into<::mongodb::bson::Bson>,
    target: impl Into<::mongodb::bson::Bson>,
    ty: ModifyOpTy,
) -> Result<bool> {
    async fn transaction<T>(
        name: &str,
        coll: &Collection<T>,
        client: &Client,
        id: &::mongodb::bson::Bson,
        target: &::mongodb::bson::Bson,
        ty: ModifyOpTy,
    ) -> ::mongodb::error::Result<Option<bool>> {
        let mut session = make_session(client).await?;

        let operation = match ty {
            ModifyOpTy::Push => "$addToSet",
            ModifyOpTy::Pull => "$pull",
        };
        let res = coll
            .update_one_with_session(
                doc! { "id": id },
                doc! { operation: { name: target } },
                None,
                &mut session,
            )
            .await?;

        if !res.matched_count.let_(to_bool) {
            return Ok(None);
        };
        if !res.modified_count.let_(to_bool) {
            return Ok(Some(false));
        }

        let inc_name = &format!("{}_size", name);
        let inc_value = match ty {
            ModifyOpTy::Push => 1,
            ModifyOpTy::Pull => -1,
        };
        let res = coll
            .update_one_with_session(
                doc! { "id": id },
                doc! { "$inc": { inc_name: inc_value } },
                None,
                &mut session,
            )
            .await?;

        if !res.matched_count.let_(to_bool) {
            unreachable!("not found value");
        }
        if !res.modified_count.let_(to_bool) {
            let op = match ty {
                ModifyOpTy::Push => "inc",
                ModifyOpTy::Pull => "dec",
            };
            unreachable!("cannot {} {} field", op, inc_name);
        }

        process_transaction(&mut session).await.map(|_| Some(true))
    }

    let id_bson = id.into();
    let target_bson = target.into();

    let res = exec_transaction!(
        transaction,
        name.as_ref(),
        coll,
        client,
        &id_bson,
        &target_bson,
        ty
    )
    .await;
    Ok(res.let_(convert_repo_err)?.let_(convert_404_or)?)
}
