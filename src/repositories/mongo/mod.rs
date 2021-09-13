use core::ops::Bound;
use std::collections::HashSet;

use async_trait::async_trait;
use mongodb::bson::{doc, Document};
use mongodb::{bson, Client, Collection, Database};
use serenity::futures::TryStreamExt;

use super::{ContentRepository, RepositoryError, Result, UserRepository};
use crate::entities::{Author, Content, ContentId, User, UserId};
use crate::usecases::content::{
    AuthorQuery, ContentContentMutation, ContentMutation, ContentQuery, PostedQuery,
};
use crate::usecases::user::{UserMutation, UserQuery};
use crate::utils::{self, LetChain};

mod converters;
mod helpers;
mod models;
mod type_convert;

use converters::*;
use helpers::*;
use models::*;

pub struct MongoUserRepository {
    client: Client,
    coll: Collection<MongoUserModel>,
}

impl MongoUserRepository {
    pub async fn new_with(client: Client, db: Database) -> ::anyhow::Result<Self> {
        initialize_coll("user", &db)
            .await
            .map_err(::anyhow::Error::new)?;

        let coll = db.collection("user");

        Ok(Self { client, coll })
    }
}

pub struct MongoContentRepository {
    client: Client,
    coll: Collection<MongoContentModel>,
}

impl MongoContentRepository {
    pub async fn new_with(client: Client, db: Database) -> ::anyhow::Result<Self> {
        initialize_coll("content", &db)
            .await
            .map_err(::anyhow::Error::new)?;

        let coll = db.collection("content");

        Ok(Self { client, coll })
    }
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

        let res = exec_transaction(transaction, (self, id, mutation_doc)).await;
        Ok(res.let_(convert_repo_err)?.let_(convert_404_or)?)
    }

    async fn get_bookmark(&self, id: UserId) -> Result<HashSet<ContentId>> {
        #[derive(::serde::Deserialize)]
        struct Model {
            bookmark: HashSet<String>,
        }

        let res = get_set(&self.coll.clone_with_type::<Model>(), id.to_string())
            .await?
            .bookmark
            .drain()
            .map(|s| s.parse::<::uuid::Uuid>().unwrap())
            .map(ContentId)
            .collect();

        Ok(res)
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

        let res = exec_transaction(transaction, (self, id)).await;
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

        let res = exec_transaction(transaction, (self, id, mutation)).await;
        Ok(res.let_(convert_repo_err)?.let_(convert_404_or)?)
    }

    async fn get_liked(&self, id: ContentId) -> Result<HashSet<UserId>> {
        #[derive(::serde::Deserialize)]
        struct Model {
            liked: HashSet<String>,
        }

        let res = get_set(&self.coll.clone_with_type::<Model>(), id.to_string())
            .await?
            .liked
            .drain()
            .map(|s| s.parse::<u64>().unwrap())
            .map(UserId)
            .collect();

        Ok(res)
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

    async fn get_pinned(&self, id: ContentId) -> Result<HashSet<UserId>> {
        #[derive(::serde::Deserialize)]
        struct Model {
            pinned: HashSet<String>,
        }

        let res = get_set(&self.coll.clone_with_type::<Model>(), id.to_string())
            .await?
            .pinned
            .drain()
            .map(|s| s.parse::<u64>().unwrap())
            .map(UserId)
            .collect();

        Ok(res)
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

        let res = exec_transaction(transaction, (self, id)).await;
        Ok(res.let_(convert_repo_err)?.let_(convert_404_or)?)
    }
}
