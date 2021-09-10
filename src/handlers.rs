use std::collections::HashSet;

use anyhow::{bail, Error, Result};

use crate::entities::{Author, Content, ContentId, Date, Posted, User, UserId};
use crate::repositories::{
    ContentMutation, ContentQuery, ContentRepository, RepositoryError, UserMutation, UserQuery,
    UserRepository,
};

pub struct Handler {
    pub user_repository: Box<dyn UserRepository + Sync + Send>,
    pub content_repository: Box<dyn ContentRepository + Sync + Send>,
}

// FIXME: rename to new_op_names
impl Handler {
    pub async fn create_user(&self, user_id: UserId) -> Result<User> {
        let new_user = User {
            id: user_id,
            admin: false,
            sub_admin: false,
            bookmark: HashSet::new(),
        };

        let can_insert = self.user_repository.insert(new_user.clone()).await?;

        if !can_insert {
            bail!("already registered.");
        }

        Ok(new_user)
    }

    pub async fn read_user(&self, user_id: UserId) -> Result<User> {
        self.user_repository
            .find(user_id)
            .await
            .map_err(user_err_fmt)
    }

    pub async fn read_users(&self, query: UserQuery) -> Result<Vec<User>> {
        self.user_repository
            .finds(query)
            .await
            .map_err(user_err_fmt)
    }

    pub async fn update_user(&self, user_id: UserId, mutation: UserMutation) -> Result<User> {
        self.user_repository
            .update(user_id, mutation)
            .await
            .map_err(user_err_fmt)
    }

    pub async fn delete_user(&self, user_id: UserId) -> Result<User> {
        self.user_repository
            .delete(user_id)
            .await
            .map_err(content_err_fmt)
    }

    pub async fn read_bookmark(&self, user_id: UserId) -> Result<HashSet<ContentId>> {
        self.user_repository
            .get_bookmark(user_id)
            .await
            .map_err(content_err_fmt)
    }

    pub async fn bookmark(
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

    pub async fn post(
        &self,
        content: String,
        posted: Posted,
        author: Author,
        created: Date,
    ) -> Result<Content> {
        let user_is_exists = self
            .user_repository
            .is_exists(posted.id)
            .await
            .map_err(user_err_fmt)?;
        if !user_is_exists {
            bail!("cannot find user. not registered?");
        }

        let new_content = Content {
            id: ::uuid::Uuid::new_v4().into(),
            content,
            author,
            posted,
            liked: HashSet::new(),
            pinned: HashSet::new(),
            created,
            edited: vec![],
        };

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

    pub async fn read_content(&self, content_id: ContentId) -> Result<Content> {
        self.content_repository
            .find(content_id)
            .await
            .map_err(content_err_fmt)
    }

    pub async fn read_contents(&self, query: ContentQuery) -> Result<Vec<Content>> {
        self.content_repository
            .finds(query)
            .await
            .map_err(content_err_fmt)
    }

    pub async fn update_content(
        &self,
        content_id: ContentId,
        mutation: ContentMutation,
    ) -> Result<Content> {
        self.content_repository
            .update(content_id, mutation)
            .await
            .map_err(content_err_fmt)
    }

    pub async fn read_like(&self, content_id: ContentId) -> Result<HashSet<UserId>> {
        self.content_repository
            .get_liked(content_id)
            .await
            .map_err(content_err_fmt)
    }

    pub async fn like(
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

    pub async fn read_pin(&self, content_id: ContentId) -> Result<HashSet<UserId>> {
        self.content_repository
            .get_pinned(content_id)
            .await
            .map_err(content_err_fmt)
    }

    pub async fn pin(&self, content_id: ContentId, user_id: UserId, undo: bool) -> Result<Content> {
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

    pub async fn delete_content(&self, content_id: ContentId) -> Result<Content> {
        self.content_repository
            .delete(content_id)
            .await
            .map_err(content_err_fmt)
    }
}

fn user_err_fmt(e: RepositoryError) -> Error {
    use anyhow::anyhow;

    match e {
        RepositoryError::NotFound => anyhow!("cannot find user. not registered?"),
        e => anyhow!("repository error: {}", e),
    }
}

fn content_err_fmt(e: RepositoryError) -> Error {
    use anyhow::anyhow;

    match e {
        RepositoryError::NotFound => anyhow!("cannot find content."),
        e => anyhow!("repository error: {}", e),
    }
}
