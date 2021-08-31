use std::collections::HashSet;

use anyhow::{bail, Error, Result};
use serenity::model::id::UserId;
use uuid::Uuid;

use crate::entities::{Author, Content, User};
use crate::repositories::{
    ContentMutation, ContentQuery, ContentRepository, RepositoryError, UserMutation, UserQuery,
    UserRepository,
};

pub struct Handler {
    pub user_repository: Box<dyn UserRepository + Sync + Send>,
    pub content_repository: Box<dyn ContentRepository + Sync + Send>,
}

#[inline]
fn user_err_fmt(e: RepositoryError) -> Error {
    use anyhow::anyhow;

    match e {
        RepositoryError::NotFound => anyhow!("cannot find user. not registered?"),
        e => anyhow!("repository error: {}", e),
    }
}

#[inline]
fn content_err_fmt(e: RepositoryError) -> Error {
    use anyhow::anyhow;

    match e {
        RepositoryError::NotFound => anyhow!("cannot find content."),
        e => anyhow!("repository error: {}", e),
    }
}

// FIXME: `v2`の接尾辞を削除
impl Handler {
    #[deprecated]
    pub async fn create_user(&self, _: UserId) -> Result<User> { unimplemented!() }

    pub async fn create_user_v2(&self, user_id: u64) -> Result<User> {
        let new_user = User {
            id: user_id,
            admin: false,
            sub_admin: false,
            posted: HashSet::new(),
            bookmark: HashSet::new(),
        };

        let can_insert = self.user_repository.insert(new_user.clone()).await?;

        if !can_insert {
            bail!("already registered.");
        }

        Ok(new_user)
    }

    #[deprecated]
    pub async fn read_user(&self, _: UserId) -> Result<User> { unimplemented!() }

    pub async fn read_user_v2(&self, user_id: u64) -> Result<User> {
        self.user_repository
            .find(user_id)
            .await
            .map_err(user_err_fmt)
    }

    pub async fn read_users_v2(&self, query: UserQuery) -> Result<Vec<User>> {
        self.user_repository
            .finds(query)
            .await
            .map_err(user_err_fmt)
    }

    #[deprecated]
    pub async fn update_user(&self, _: UserId, _: Option<bool>, _: Option<bool>) -> Result<User> {
        unimplemented!()
    }

    pub async fn update_user_v2(&self, user_id: u64, mutation: UserMutation) -> Result<User> {
        self.user_repository
            .update(user_id, mutation)
            .await
            .map_err(user_err_fmt)
    }

    #[deprecated]
    pub async fn bookmark_update_user(
        &self,
        _: UserId,
        _: Uuid,
        _: bool,
    ) -> Result<(User, Content)> {
        unimplemented!()
    }

    pub async fn bookmark_v2(
        &self,
        user_id: u64,
        content_id: Uuid,
        undo: bool,
    ) -> Result<(User, Content)> {
        let can_insert = match undo {
            false =>
                self.user_repository
                    .insert_bookmarked(user_id, content_id)
                    .await,
            true =>
                self.user_repository
                    .delete_bookmarked(user_id, content_id)
                    .await,
        }
        .map_err(user_err_fmt)?;

        match (undo, can_insert) {
            (false, false) => bail!("already bookmarked."),
            (true, false) => bail!("didn't bookmarked."),
            (_, true) => (),
        }

        let user = self
            .user_repository
            .find(user_id)
            .await
            .map_err(user_err_fmt)?;
        let content = self
            .content_repository
            .find(content_id)
            .await
            .map_err(content_err_fmt)?;

        Ok((user, content))
    }

    #[deprecated]
    pub async fn delete_user(&self, _: UserId) -> Result<()> { unimplemented!() }

    #[deprecated]
    pub async fn delete_user_v2(&self, user_id: u64) -> Result<User> {
        self.user_repository
            .delete(user_id)
            .await
            .map_err(user_err_fmt)
    }

    #[deprecated]
    pub async fn create_content_and_posted_update_user(
        &self,
        _: String,
        _: UserId,
        _: String,
    ) -> Result<Content> {
        unimplemented!()
    }

    pub async fn post_v2(&self, content: String, posted: u64, author: Author) -> Result<Content> {
        let user_is_exists = !self
            .user_repository
            .is_exists(posted)
            .await
            .map_err(user_err_fmt)?;
        if !user_is_exists {
            bail!("cannot find user. not registered?");
        }

        let new_content = Content {
            id: uuid::Uuid::new_v4(),
            content,
            author,
            posted,
            liked: HashSet::new(),
            pinned: HashSet::new(),
        };

        let user_posted_can_insert = self
            .user_repository
            .insert_posted(posted, new_content.id)
            .await
            .map_err(user_err_fmt)?;
        if !user_posted_can_insert {
            panic!("content_id duplicated!");
        }

        let content_can_insert = self
            .content_repository
            .insert(new_content.clone())
            .await
            .map_err(content_err_fmt)?;

        if !content_can_insert {
            panic!("content_id duplicated!");
        }

        Ok(new_content)
    }

    #[deprecated]
    pub async fn read_content(&self, _: Vec<ContentQuery>) -> Result<Vec<Content>> {
        unimplemented!()
    }

    pub async fn read_content_v2(&self, content_id: Uuid) -> Result<Content> {
        self.content_repository
            .find(content_id)
            .await
            .map_err(content_err_fmt)
    }

    pub async fn read_contents_v2(&self, query: ContentQuery) -> Result<Vec<Content>> {
        self.content_repository
            .finds(query)
            .await
            .map_err(content_err_fmt)
    }

    #[deprecated]
    pub async fn update_content(&self, _: Uuid, _: String) -> Result<Content> { unimplemented!() }

    pub async fn update_content_v2(
        &self,
        content_id: Uuid,
        mutation: ContentMutation,
    ) -> Result<Content> {
        self.content_repository
            .update(content_id, mutation)
            .await
            .map_err(content_err_fmt)
    }

    #[deprecated]
    pub async fn like_update_content(&self, _: Uuid, _: UserId, _: bool) -> Result<Content> {
        unimplemented!()
    }

    pub async fn like_v2(&self, content_id: Uuid, user_id: u64, undo: bool) -> Result<Content> {
        let can_insert = match undo {
            false =>
                self.content_repository
                    .insert_liked(content_id, user_id)
                    .await,
            true =>
                self.content_repository
                    .delete_liked(content_id, user_id)
                    .await,
        }
        .map_err(content_err_fmt)?;

        match (undo, can_insert) {
            (false, false) => bail!("already liked."),
            (true, false) => bail!("didn't liked."),
            (_, true) => (),
        }

        self.content_repository
            .find(content_id)
            .await
            .map_err(content_err_fmt)
    }

    #[deprecated]
    pub async fn pin_update_content(&self, _: Uuid, _: UserId, _: bool) -> Result<Content> {
        unimplemented!()
    }

    pub async fn pin_v2(&self, content_id: Uuid, user_id: u64, undo: bool) -> Result<Content> {
        let can_insert = match undo {
            false =>
                self.content_repository
                    .insert_pinned(content_id, user_id)
                    .await,
            true =>
                self.content_repository
                    .delete_pinned(content_id, user_id)
                    .await,
        }
        .map_err(content_err_fmt)?;

        match (undo, can_insert) {
            (false, false) => bail!("already pinned."),
            (true, false) => bail!("didn't pinned."),
            (_, true) => (),
        }

        self.content_repository
            .find(content_id)
            .await
            .map_err(content_err_fmt)
    }

    #[deprecated]
    pub async fn delete_content(&self, _: Uuid) -> Result<()> { unimplemented!() }

    pub async fn delete_content_v2(&self, content_id: Uuid) -> Result<Content> {
        self.content_repository
            .delete(content_id)
            .await
            .map_err(content_err_fmt)
    }
}
