use alloc::sync::Arc;
use std::collections::HashSet;

use anyhow::{bail, Result};
use async_trait::async_trait;
use tokio::sync::Mutex;

use crate::entities::Content;
// FIXME: move to interactors::
use crate::handlers::helpers::*;
use crate::repositories::{ContentRepository, UserRepository};
use crate::usecases::content::{
    edit, get, get_like, get_pin, gets, like, pin, post, unlike, unpin, withdraw,
};
use crate::utils::LetChain;

pub struct ReturnContentPostInteractor {
    pub user_repository: Arc<dyn UserRepository + Sync + Send>,
    pub content_repository: Arc<dyn ContentRepository + Sync + Send>,
    pub lock: Arc<Mutex<()>>,
    pub ret: Arc<Mutex<Option<post::Output>>>,
}
#[async_trait]
impl post::Usecase for ReturnContentPostInteractor {
    async fn handle(
        &self,
        post::Input {
            content,
            posted,
            author,
            created,
        }: post::Input,
    ) -> Result<()> {
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

        *self.ret.lock().await = post::Output {
            content: new_content,
        }
        .let_(Some);

        Ok(())
    }
}

pub struct ReturnContentGetInteractor {
    pub content_repository: Arc<dyn ContentRepository + Sync + Send>,
    pub lock: Arc<Mutex<()>>,
    pub ret: Arc<Mutex<Option<get::Output>>>,
}
#[async_trait]
impl get::Usecase for ReturnContentGetInteractor {
    async fn handle(&self, get::Input { content_id }: get::Input) -> anyhow::Result<()> {
        *self.ret.lock().await = self
            .content_repository
            .find(content_id)
            .await
            .map_err(content_err_fmt)?
            .let_(|content| get::Output { content })
            .let_(Some);

        Ok(())
    }
}

pub struct ReturnContentGetsInteractor {
    pub content_repository: Arc<dyn ContentRepository + Sync + Send>,
    pub lock: Arc<Mutex<()>>,
    pub ret: Arc<Mutex<Option<gets::Output>>>,
}
#[async_trait]
impl gets::Usecase for ReturnContentGetsInteractor {
    async fn handle(&self, gets::Input { query }: gets::Input) -> anyhow::Result<()> {
        *self.ret.lock().await = self
            .content_repository
            .finds(query)
            .await
            .map_err(content_err_fmt)?
            .let_(|contents| gets::Output { contents })
            .let_(Some);

        Ok(())
    }
}

pub struct ReturnContentEditInteractor {
    pub content_repository: Arc<dyn ContentRepository + Sync + Send>,
    pub lock: Arc<Mutex<()>>,
    pub ret: Arc<Mutex<Option<edit::Output>>>,
}
#[async_trait]
impl edit::Usecase for ReturnContentEditInteractor {
    async fn handle(
        &self,
        edit::Input {
            content_id,
            mutation,
        }: edit::Input,
    ) -> anyhow::Result<()> {
        *self.ret.lock().await = self
            .content_repository
            .update(content_id, mutation)
            .await
            .map_err(content_err_fmt)?
            .let_(|content| edit::Output { content })
            .let_(Some);

        Ok(())
    }
}

pub struct ReturnContentWithdrawInteractor {
    pub content_repository: Arc<dyn ContentRepository + Sync + Send>,
    pub lock: Arc<Mutex<()>>,
    pub ret: Arc<Mutex<Option<withdraw::Output>>>,
}
#[async_trait]
impl withdraw::Usecase for ReturnContentWithdrawInteractor {
    async fn handle(&self, withdraw::Input { content_id }: withdraw::Input) -> anyhow::Result<()> {
        *self.ret.lock().await = self
            .content_repository
            .delete(content_id)
            .await
            .map_err(content_err_fmt)?
            .let_(|content| withdraw::Output { content })
            .let_(Some);

        Ok(())
    }
}

pub struct ReturnContentLikeGetInteractor {
    pub content_repository: Arc<dyn ContentRepository + Sync + Send>,
    pub lock: Arc<Mutex<()>>,
    pub ret: Arc<Mutex<Option<get_like::Output>>>,
}
#[async_trait]
impl get_like::Usecase for ReturnContentLikeGetInteractor {
    async fn handle(&self, get_like::Input { content_id }: get_like::Input) -> anyhow::Result<()> {
        *self.ret.lock().await = self
            .content_repository
            .get_liked(content_id)
            .await
            .map_err(content_err_fmt)?
            .let_(|like| get_like::Output { like })
            .let_(Some);

        Ok(())
    }
}

pub struct ReturnContentLikeInteractor {
    pub content_repository: Arc<dyn ContentRepository + Sync + Send>,
    pub lock: Arc<Mutex<()>>,
    pub ret: Arc<Mutex<Option<like::Output>>>,
}
#[async_trait]
impl like::Usecase for ReturnContentLikeInteractor {
    async fn handle(
        &self,
        like::Input {
            content_id,
            user_id,
        }: like::Input,
    ) -> anyhow::Result<()> {
        let can_insert = self
            .content_repository
            .insert_liked(content_id, user_id)
            .await
            .map_err(content_err_fmt)?;

        if !can_insert {
            bail!("already liked.");
        }

        *self.ret.lock().await = self
            .content_repository
            .find(content_id)
            .await
            .map_err(content_err_fmt)?
            .let_(|content| like::Output { content })
            .let_(Some);

        Ok(())
    }
}

pub struct ReturnContentUnlikeInteractor {
    pub content_repository: Arc<dyn ContentRepository + Sync + Send>,
    pub lock: Arc<Mutex<()>>,
    pub ret: Arc<Mutex<Option<unlike::Output>>>,
}
#[async_trait]
impl unlike::Usecase for ReturnContentUnlikeInteractor {
    async fn handle(
        &self,
        unlike::Input {
            content_id,
            user_id,
        }: unlike::Input,
    ) -> anyhow::Result<()> {
        let can_insert = self
            .content_repository
            .delete_liked(content_id, user_id)
            .await
            .map_err(content_err_fmt)?;

        if !can_insert {
            bail!("didn't liked.")
        }

        *self.ret.lock().await = self
            .content_repository
            .find(content_id)
            .await
            .map_err(content_err_fmt)?
            .let_(|content| unlike::Output { content })
            .let_(Some);

        Ok(())
    }
}

pub struct ReturnContentPinGetInteractor {
    pub content_repository: Arc<dyn ContentRepository + Sync + Send>,
    pub lock: Arc<Mutex<()>>,
    pub ret: Arc<Mutex<Option<get_pin::Output>>>,
}
#[async_trait]
impl get_pin::Usecase for ReturnContentPinGetInteractor {
    async fn handle(&self, get_pin::Input { content_id }: get_pin::Input) -> anyhow::Result<()> {
        *self.ret.lock().await = self
            .content_repository
            .get_pinned(content_id)
            .await
            .map_err(content_err_fmt)?
            .let_(|pin| get_pin::Output { pin })
            .let_(Some);

        Ok(())
    }
}

pub struct ReturnContentPinInteractor {
    pub content_repository: Arc<dyn ContentRepository + Sync + Send>,
    pub lock: Arc<Mutex<()>>,
    pub ret: Arc<Mutex<Option<pin::Output>>>,
}
#[async_trait]
impl pin::Usecase for ReturnContentPinInteractor {
    async fn handle(
        &self,
        pin::Input {
            content_id,
            user_id,
        }: pin::Input,
    ) -> anyhow::Result<()> {
        let can_insert = self
            .content_repository
            .insert_pinned(content_id, user_id)
            .await
            .map_err(content_err_fmt)?;

        if !can_insert {
            bail!("already pinned.");
        }

        *self.ret.lock().await = self
            .content_repository
            .find(content_id)
            .await
            .map_err(content_err_fmt)?
            .let_(|content| pin::Output { content })
            .let_(Some);

        Ok(())
    }
}

pub struct ReturnContentUnpinInteractor {
    pub content_repository: Arc<dyn ContentRepository + Sync + Send>,
    pub lock: Arc<Mutex<()>>,
    pub ret: Arc<Mutex<Option<unpin::Output>>>,
}
#[async_trait]
impl unpin::Usecase for ReturnContentUnpinInteractor {
    async fn handle(
        &self,
        unpin::Input {
            content_id,
            user_id,
        }: unpin::Input,
    ) -> anyhow::Result<()> {
        let can_insert = self
            .content_repository
            .delete_pinned(content_id, user_id)
            .await
            .map_err(content_err_fmt)?;

        if !can_insert {
            bail!("didn't pinned.");
        }

        *self.ret.lock().await = self
            .content_repository
            .find(content_id)
            .await
            .map_err(content_err_fmt)?
            .let_(|content| unpin::Output { content })
            .let_(Some);

        Ok(())
    }
}
