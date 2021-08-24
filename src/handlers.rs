use anyhow::{bail, Result};
use serenity::model::id::UserId;
use uuid::Uuid;

use crate::entities::{Content, User};
use crate::repositories::{ContentQuery, Query, Repository, UserQuery};

pub struct Handler {
    pub user_repository: Box<dyn Repository<User> + Send + Sync>,
    pub content_repository: Box<dyn Repository<Content> + Send + Sync>,
}

impl Handler {
    pub async fn create_user(&self, user_id: UserId) -> Result<User> {
        if self.verify_user(user_id).await?.is_some() {
            bail!("already registered.");
        }

        let new_user = User {
            id: user_id,
            admin: false,
            sub_admin: false,
            posted: vec![],
            bookmark: vec![],
        };

        self.user_repository.save(new_user.clone()).await?;

        Ok(new_user)
    }

    pub async fn read_user(&self, user_id: UserId) -> Result<User> {
        match self.verify_user(user_id).await? {
            Some(u) => Ok(u),
            None => bail!("cannot find user. not registered?"),
        }
    }

    pub async fn update_user(
        &self,
        user_id: UserId,
        admin: Option<bool>,
        sub_admin: Option<bool>,
    ) -> Result<User> {
        if self.verify_user(user_id).await?.is_none() {
            bail!("cannot find user. not registered?");
        }

        let mut user = self
            .user_repository
            .remove_match(vec![&UserQuery::Id(user_id)])
            .await?;

        if let Some(b) = admin {
            user.admin = b;
        }

        if let Some(b) = sub_admin {
            user.sub_admin = b;
        }

        self.user_repository.save(user.clone()).await?;

        Ok(user)
    }

    pub async fn bookmark_update_user(&self, user_id: UserId, content_id: Uuid) -> Result<()> {
        if self.verify_user(user_id).await?.is_none() {
            bail!("cannot find user. not registered?");
        }

        if self.verify_content(content_id).await?.is_none() {
            bail!("cannot find content.");
        }

        let mut user = self
            .user_repository
            .remove_match(vec![&UserQuery::Id(user_id)])
            .await?;
        let mut content = self
            .content_repository
            .remove_match(vec![&ContentQuery::Id(content_id)])
            .await?;

        user.bookmark.push(content_id);
        content.bookmarked += 1;

        self.user_repository.save(user).await?;
        self.content_repository.save(content).await?;

        Ok(())
    }

    pub async fn delete_user(&self, user_id: UserId) -> Result<()> {
        if self.verify_user(user_id).await?.is_none() {
            bail!("cannot find user. not registered?");
        }

        self.user_repository
            .remove_match(vec![&UserQuery::Id(user_id)])
            .await?;
        Ok(())
    }

    pub async fn create_content_and_posted_update_user(
        &self,
        content: String,
        posted: UserId,
        author: String,
    ) -> Result<Content> {
        if self.verify_user(posted).await?.is_none() {
            bail!("cannot find user. not registered?");
        }

        let mut posted_user = self
            .user_repository
            .remove_match(vec![&UserQuery::Id(posted)])
            .await?;

        let new_content = Content {
            id: uuid::Uuid::new_v4(),
            content,
            author,
            posted,
            liked: vec![],
            bookmarked: 0,
            pinned: vec![],
        };

        posted_user.posted.push(new_content.id);

        self.content_repository.save(new_content.clone()).await?;
        self.user_repository.save(posted_user).await?;

        Ok(new_content)
    }

    pub async fn read_content(&self, content_query: Vec<ContentQuery>) -> Result<Vec<Content>> {
        crate::convert_query!(ref content_query);
        Ok(self.content_repository.get_matches(content_query).await?)
    }

    pub async fn update_content(&self, content_id: Uuid, content: String) -> Result<Content> {
        if self.verify_content(content_id).await?.is_none() {
            bail!("cannot find content.");
        }

        let mut current_content = self
            .content_repository
            .remove_match(vec![&ContentQuery::Id(content_id)])
            .await?;

        current_content.content = content;

        self.content_repository
            .save(current_content.clone())
            .await?;

        Ok(current_content)
    }

    pub async fn like_update_content(&self, content_id: Uuid, user_id: UserId) -> Result<()> {
        if self.verify_user(user_id).await?.is_none() {
            bail!("cannot find user. not registered?");
        }

        if self.verify_content(content_id).await?.is_none() {
            bail!("cannot find content.");
        }

        let mut current_content = self
            .content_repository
            .remove_match(vec![&ContentQuery::Id(content_id)])
            .await?;

        current_content.liked.push(user_id);

        self.content_repository.save(current_content).await?;

        Ok(())
    }

    pub async fn pin_update_content(&self, content_id: Uuid, user_id: UserId) -> Result<()> {
        if self.verify_user(user_id).await?.is_none() {
            bail!("cannot find user. not registered?")
        }

        if self.verify_content(content_id).await?.is_none() {
            bail!("cannot find content.")
        }

        let mut current_content = self
            .content_repository
            .remove_match(vec![&ContentQuery::Id(content_id)])
            .await?;

        current_content.pinned.push(user_id);

        self.content_repository.save(current_content).await?;

        Ok(())
    }

    pub async fn delete_content(&self, content_id: Uuid) -> Result<()> {
        if self.verify_content(content_id).await?.is_none() {
            bail!("cannot find content.")
        }

        self.content_repository
            .remove_match(vec![&ContentQuery::Id(content_id)])
            .await?;
        Ok(())
    }

    async fn verify_user(&self, user_id: UserId) -> Result<Option<User>> {
        let mut matched = match self
            .user_repository
            .get_matches(vec![&UserQuery::Id(user_id)])
            .await
        {
            Ok(o) => o,
            Err(e) => bail!("repository error: {}", e),
        };

        match matched.len() {
            0 => Ok(None),
            1 => Ok(Some(matched.remove(0))),
            _ => bail!("matched: {} (internal error)", matched.len()),
        }
    }

    async fn verify_content(&self, content_id: Uuid) -> Result<Option<Content>> {
        let mut matched = match self
            .content_repository
            .get_matches(vec![&ContentQuery::Id(content_id)])
            .await
        {
            Ok(o) => o,
            Err(e) => bail!("repository error: {}", e),
        };

        match matched.len() {
            0 => Ok(None),
            1 => Ok(Some(matched.remove(0))),
            _ => bail!("matched: {} (internal error)", matched.len()),
        }
    }
}

#[macro_export]
macro_rules! convert_query {
    (ref $q:ident) => {
        let $q = {
            let mut convert = Vec::<&(dyn Query<_> + Sync + Send)>::new();
            $q.iter().for_each(|q| convert.push(q));
            convert
        };
    };
}
