use std::collections::HashSet;

use anyhow::{bail, Error, Result};

use crate::entities::{Author, Content, ContentId, Posted, User, UserId};
use crate::repositories::{
    ContentMutation, ContentQuery, ContentRepository, RepositoryError, UserMutation, UserQuery,
    UserRepository,
};

pub(crate) struct Handler {
    pub(crate) user_repository: Box<dyn UserRepository + Sync + Send>,
    pub(crate) content_repository: Box<dyn ContentRepository + Sync + Send>,
}

#[inline]
fn user_err_fmt(e: RepositoryError) -> Error {
    use anyhow::anyhow;

    match e {
        RepositoryError::NotFound => unreachable!("illegal error. (impl error)"),
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

impl Handler {
    pub(crate) async fn create_user(&self, user_id: UserId) -> Result<User> {
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

    pub(crate) async fn read_user(&self, user_id: UserId) -> Result<User> {
        self.user_repository
            .find(user_id)
            .await
            .map_err(user_err_fmt)
    }

    pub(crate) async fn read_users(&self, query: UserQuery) -> Result<Vec<User>> {
        self.user_repository
            .finds(query)
            .await
            .map_err(user_err_fmt)
    }

    pub(crate) async fn update_user(
        &self,
        user_id: UserId,
        mutation: UserMutation,
    ) -> Result<User> {
        self.user_repository
            .update(user_id, mutation)
            .await
            .map_err(user_err_fmt)
    }

    pub(crate) async fn bookmark(
        &self,
        user_id: UserId,
        content_id: ContentId,
        undo: bool,
    ) -> Result<(User, Content)> {
        let can_insert = match undo {
            false =>
                self.user_repository
                    .insert_bookmark(user_id, content_id)
                    .await,
            true =>
                self.user_repository
                    .delete_bookmark(user_id, content_id)
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

    pub(crate) async fn post(
        &self,
        content: String,
        posted: Posted,
        author: Author,
    ) -> Result<Content> {
        let user_is_exists = self
            .user_repository
            .is_exists(posted.id)
            .await
            .map_err(user_err_fmt)?;
        if !user_is_exists {
            bail!("cannot find user. not registered?");
        }

        let posted_id = posted.id;
        let new_content = Content {
            id: ::uuid::Uuid::new_v4().into(),
            content,
            author,
            posted,
            liked: HashSet::new(),
            pinned: HashSet::new(),
        };

        let user_posted_can_insert = self
            .user_repository
            .insert_posted(posted_id, new_content.id)
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

    pub(crate) async fn read_content(&self, content_id: ContentId) -> Result<Content> {
        self.content_repository
            .find(content_id)
            .await
            .map_err(content_err_fmt)
    }

    pub(crate) async fn read_contents(&self, query: ContentQuery) -> Result<Vec<Content>> {
        self.content_repository
            .finds(query)
            .await
            .map_err(content_err_fmt)
    }

    pub(crate) async fn update_content(
        &self,
        content_id: ContentId,
        mutation: ContentMutation,
    ) -> Result<Content> {
        self.content_repository
            .update(content_id, mutation)
            .await
            .map_err(content_err_fmt)
    }

    pub(crate) async fn like(
        &self,
        content_id: ContentId,
        user_id: UserId,
        undo: bool,
    ) -> Result<Content> {
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

    pub(crate) async fn pin(
        &self,
        content_id: ContentId,
        user_id: UserId,
        undo: bool,
    ) -> Result<Content> {
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

    // FIXME: unsyncronized `user#posted`
    // rename to `withdraw` (<=?=> `post`)
    pub(crate) async fn delete_content(&self, content_id: ContentId) -> Result<Content> {
        self.content_repository
            .delete(content_id)
            .await
            .map_err(content_err_fmt)
    }
}
