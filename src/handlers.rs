use serenity::model::id::UserId;
use uuid::Uuid;

use crate::entities::{Content, User};
use crate::repositories::{ContentQuery, Repository, UserQuery};

pub struct Handler {
    pub user_repository: Box<dyn Repository<Item = User, Query = UserQuery> + Sync + Send>,
    pub content_repository: Box<dyn Repository<Item = Content, Query = ContentQuery> + Sync + Send>,
}

impl Handler {
    pub async fn create_user(&self, id: UserId) -> anyhow::Result<User> {
        let new_user = User {
            id,
            admin: false,
            sub_admin: false,
            posted: vec![],
            bookmark: vec![],
        };

        self.user_repository.save(new_user.clone()).await?;

        Ok(new_user)
    }

    pub async fn read_user(&self, id: UserId) -> anyhow::Result<User> {
        Ok(self
            .user_repository
            .get_match(vec![UserQuery::Id(id)])
            .await?)
    }

    pub async fn update_user(
        &self,
        id: UserId,
        admin: Option<bool>,
        sub_admin: Option<bool>,
    ) -> anyhow::Result<User> {
        let mut user = self
            .user_repository
            .remove_match(vec![UserQuery::Id(id)])
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

    pub async fn bookmark_update_user(&self, id: UserId, content: Uuid) -> anyhow::Result<()> {
        self.content_repository
            .get_match(vec![ContentQuery::Id(content)])
            .await?;

        let mut user = self
            .user_repository
            .remove_match(vec![UserQuery::Id(id)])
            .await?;

        user.bookmark.push(content);

        self.user_repository.save(user).await?;

        Ok(())
    }

    pub async fn delete_user(&self, id: UserId) -> anyhow::Result<()> {
        self.user_repository
            .remove_match(vec![UserQuery::Id(id)])
            .await?;
        Ok(())
    }

    pub async fn create_content_and_posted_update_user(
        &self,
        content: String,
        posted: UserId,
        author: String,
    ) -> anyhow::Result<Content> {
        let new_content = Content {
            id: uuid::Uuid::new_v4(),
            content,
            author,
            posted,
            liked: vec![],
            bookmarked: 0,
            pinned: vec![],
        };

        let mut current_user = self
            .user_repository
            .remove_match(vec![UserQuery::Id(posted)])
            .await?;

        current_user.posted.push(new_content.id);

        self.user_repository.save(current_user).await?;

        self.content_repository.save(new_content.clone()).await?;

        Ok(new_content)
    }

    pub async fn read_content(&self, id: Uuid) -> anyhow::Result<Content> {
        self.content_repository
            .get_match(vec![ContentQuery::Id(id)])
            .await
    }

    pub async fn update_content(&self, id: Uuid, content: String) -> anyhow::Result<Content> {
        let mut current_content = self
            .content_repository
            .remove_match(vec![ContentQuery::Id(id)])
            .await?;

        current_content.content = content;

        self.content_repository
            .save(current_content.clone())
            .await?;

        Ok(current_content)
    }

    pub async fn like_update_content(&self, id: Uuid, user: UserId) -> anyhow::Result<()> {
        self.user_repository
            .get_match(vec![UserQuery::Id(user)])
            .await?;
        let mut current_content = self
            .content_repository
            .remove_match(vec![ContentQuery::Id(id)])
            .await?;

        current_content.liked.push(user);

        self.content_repository.save(current_content).await?;

        Ok(())
    }

    pub async fn pin_update_content(&self, id: Uuid, user: UserId) -> anyhow::Result<()> {
        self.user_repository
            .get_match(vec![UserQuery::Id(user)])
            .await?;
        let mut current_content = self
            .content_repository
            .remove_match(vec![ContentQuery::Id(id)])
            .await?;

        current_content.pinned.push(user);

        self.content_repository.save(current_content).await?;

        Ok(())
    }

    pub async fn delete_content(&self, id: Uuid) -> anyhow::Result<()> {
        self.content_repository
            .remove_match(vec![ContentQuery::Id(id)])
            .await?;
        Ok(())
    }
}
