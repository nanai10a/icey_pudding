use alloc::sync::Arc;
use std::collections::HashSet;

use anyhow::{bail, Result};
use async_trait::async_trait;
use smallvec::SmallVec;

use super::*;
use crate::entities::Content;
use crate::presenters::content::{
    ContentEditPresenter, ContentGetPresenter, ContentGetsPresenter, ContentLikeGetPresenter,
    ContentLikePresenter, ContentPinGetPresenter, ContentPinPresenter, ContentPostPresenter,
    ContentUnlikePresenter, ContentUnpinPresenter, ContentWithdrawPresenter,
};
use crate::repositories::{ContentRepository, UserRepository};
use crate::usecases::content::{
    edit, get, get_like, get_pin, gets, like, pin, post, unlike, unpin, withdraw,
};
use crate::utils::{AlsoChain, LetChain};

pub struct ContentPostInteractor {
    pub user_repository: Arc<dyn UserRepository + Sync + Send>,
    pub content_repository: Arc<dyn ContentRepository + Sync + Send>,
    pub pres: Arc<dyn ContentPostPresenter + Sync + Send>,
}
#[async_trait]
impl post::Usecase for ContentPostInteractor {
    #[tracing::instrument(skip(self))]
    async fn handle(&self, data: post::Input) -> Result<()> {
        tracing::trace!("input - {:?}", data);

        let post::Input {
            content,
            posted,
            author,
            created,
        } = data;

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

        post::Output {
            content: new_content,
        }
        .also_(|o| tracing::trace!("output - {:?}", o))
        .let_(|r| self.pres.complete(r))
        .await
        .unwrap();

        Ok(())
    }
}

pub struct ContentGetInteractor {
    pub content_repository: Arc<dyn ContentRepository + Sync + Send>,
    pub pres: Arc<dyn ContentGetPresenter + Sync + Send>,
}
#[async_trait]
impl get::Usecase for ContentGetInteractor {
    #[tracing::instrument(skip(self))]
    async fn handle(&self, data: get::Input) -> anyhow::Result<()> {
        tracing::trace!("input - {:?}", data);

        let get::Input { content_id } = data;

        self.content_repository
            .find(content_id)
            .await
            .map_err(content_err_fmt)?
            .let_(|content| get::Output { content })
            .also_(|o| tracing::trace!("output - {:?}", o))
            .let_(|r| self.pres.complete(r))
            .await
            .unwrap();

        Ok(())
    }
}

pub struct ContentGetsInteractor {
    pub content_repository: Arc<dyn ContentRepository + Sync + Send>,
    pub pres: Arc<dyn ContentGetsPresenter + Sync + Send>,
}
#[async_trait]
impl gets::Usecase for ContentGetsInteractor {
    #[tracing::instrument(skip(self))]
    async fn handle(&self, data: gets::Input) -> anyhow::Result<()> {
        tracing::trace!("input - {:?}", data);

        let gets::Input { query, page } = data;

        self.content_repository
            .finds(query)
            .await
            .map_err(content_err_fmt)?
            .let_(|mut v| {
                calc_paging(0..v.len(), 5, page as usize).map(move |lim| {
                    v.drain(lim)
                        .enumerate()
                        .map(|(i, c)| (i as u32, c))
                        .collect::<SmallVec<[_; 5]>>()
                })
            })?
            .let_(|contents| gets::Output { contents, page })
            .also_(|o| tracing::trace!("output - {:?}", o))
            .let_(|r| self.pres.complete(r))
            .await
            .unwrap();

        Ok(())
    }
}

pub struct ContentEditInteractor {
    pub content_repository: Arc<dyn ContentRepository + Sync + Send>,
    pub pres: Arc<dyn ContentEditPresenter + Sync + Send>,
}
#[async_trait]
impl edit::Usecase for ContentEditInteractor {
    #[tracing::instrument(skip(self))]
    async fn handle(&self, data: edit::Input) -> anyhow::Result<()> {
        tracing::trace!("input - {:?}", data);

        let edit::Input {
            content_id,
            mutation,
        } = data;

        self.content_repository
            .update(content_id, mutation)
            .await
            .map_err(content_err_fmt)?
            .let_(|content| edit::Output { content })
            .also_(|o| tracing::trace!("output - {:?}", o))
            .let_(|r| self.pres.complete(r))
            .await
            .unwrap();

        Ok(())
    }
}

pub struct ContentWithdrawInteractor {
    pub content_repository: Arc<dyn ContentRepository + Sync + Send>,
    pub pres: Arc<dyn ContentWithdrawPresenter + Sync + Send>,
}
#[async_trait]
impl withdraw::Usecase for ContentWithdrawInteractor {
    #[tracing::instrument(skip(self))]
    async fn handle(&self, data: withdraw::Input) -> anyhow::Result<()> {
        tracing::trace!("input - {:?}", data);

        let withdraw::Input { content_id } = data;

        self.content_repository
            .delete(content_id)
            .await
            .map_err(content_err_fmt)?
            .let_(|content| withdraw::Output { content })
            .also_(|o| tracing::trace!("output - {:?}", o))
            .let_(|r| self.pres.complete(r))
            .await
            .unwrap();

        Ok(())
    }
}

pub struct ContentLikeGetInteractor {
    pub content_repository: Arc<dyn ContentRepository + Sync + Send>,
    pub pres: Arc<dyn ContentLikeGetPresenter + Sync + Send>,
}
#[async_trait]
impl get_like::Usecase for ContentLikeGetInteractor {
    #[tracing::instrument(skip(self))]
    async fn handle(&self, data: get_like::Input) -> anyhow::Result<()> {
        tracing::trace!("input - {:?}", data);

        let get_like::Input { content_id, page } = data;

        self.content_repository
            .get_liked(content_id)
            .await
            .map_err(content_err_fmt)?
            .drain()
            .collect::<Vec<_>>()
            .let_(|mut v| {
                calc_paging(0..v.len(), 20, page as usize).map(|lim| {
                    v.drain(lim)
                        .enumerate()
                        .map(|(idx, id)| (idx as u32, id))
                        .collect::<SmallVec<[_; 20]>>()
                })
            })?
            .let_(|like| get_like::Output { like, page })
            .also_(|o| tracing::trace!("output - {:?}", o))
            .let_(|r| self.pres.complete(r))
            .await
            .unwrap();

        Ok(())
    }
}

pub struct ContentLikeInteractor {
    pub content_repository: Arc<dyn ContentRepository + Sync + Send>,
    pub pres: Arc<dyn ContentLikePresenter + Sync + Send>,
}
#[async_trait]
impl like::Usecase for ContentLikeInteractor {
    #[tracing::instrument(skip(self))]
    async fn handle(&self, data: like::Input) -> anyhow::Result<()> {
        tracing::trace!("input - {:?}", data);

        let like::Input {
            content_id,
            user_id,
        } = data;

        let can_insert = self
            .content_repository
            .insert_liked(content_id, user_id)
            .await
            .map_err(content_err_fmt)?;

        if !can_insert {
            bail!("already liked.");
        }

        self.content_repository
            .find(content_id)
            .await
            .map_err(content_err_fmt)?
            .let_(|content| like::Output {
                content,
                id: user_id,
            })
            .also_(|o| tracing::trace!("output - {:?}", o))
            .let_(|r| self.pres.complete(r))
            .await
            .unwrap();

        Ok(())
    }
}

pub struct ContentUnlikeInteractor {
    pub content_repository: Arc<dyn ContentRepository + Sync + Send>,
    pub pres: Arc<dyn ContentUnlikePresenter + Sync + Send>,
}
#[async_trait]
impl unlike::Usecase for ContentUnlikeInteractor {
    #[tracing::instrument(skip(self))]
    async fn handle(&self, data: unlike::Input) -> anyhow::Result<()> {
        tracing::trace!("input - {:?}", data);

        let unlike::Input {
            content_id,
            user_id,
        } = data;

        let can_insert = self
            .content_repository
            .delete_liked(content_id, user_id)
            .await
            .map_err(content_err_fmt)?;

        if !can_insert {
            bail!("didn't liked.")
        }

        self.content_repository
            .find(content_id)
            .await
            .map_err(content_err_fmt)?
            .let_(|content| unlike::Output {
                content,
                id: user_id,
            })
            .also_(|o| tracing::trace!("output - {:?}", o))
            .let_(|r| self.pres.complete(r))
            .await
            .unwrap();

        Ok(())
    }
}

pub struct ContentPinGetInteractor {
    pub content_repository: Arc<dyn ContentRepository + Sync + Send>,
    pub pres: Arc<dyn ContentPinGetPresenter + Sync + Send>,
}
#[async_trait]
impl get_pin::Usecase for ContentPinGetInteractor {
    #[tracing::instrument(skip(self))]
    async fn handle(&self, data: get_pin::Input) -> anyhow::Result<()> {
        tracing::trace!("input - {:?}", data);

        let get_pin::Input { content_id, page } = data;

        self.content_repository
            .get_pinned(content_id)
            .await
            .map_err(content_err_fmt)?
            .drain()
            .collect::<Vec<_>>()
            .let_(|mut v| {
                calc_paging(0..v.len(), 20, page as usize).map(move |lim| {
                    v.drain(lim)
                        .enumerate()
                        .map(|(idx, id)| (idx as u32, id))
                        .collect::<SmallVec<[_; 20]>>()
                })
            })?
            .let_(|pin| get_pin::Output { pin, page })
            .also_(|o| tracing::trace!("output - {:?}", o))
            .let_(|r| self.pres.complete(r))
            .await
            .unwrap();

        Ok(())
    }
}

pub struct ContentPinInteractor {
    pub content_repository: Arc<dyn ContentRepository + Sync + Send>,
    pub pres: Arc<dyn ContentPinPresenter + Sync + Send>,
}
#[async_trait]
impl pin::Usecase for ContentPinInteractor {
    #[tracing::instrument(skip(self))]
    async fn handle(&self, data: pin::Input) -> anyhow::Result<()> {
        tracing::trace!("input - {:?}", data);

        let pin::Input {
            content_id,
            user_id,
        } = data;

        let can_insert = self
            .content_repository
            .insert_pinned(content_id, user_id)
            .await
            .map_err(content_err_fmt)?;

        if !can_insert {
            bail!("already pinned.");
        }

        self.content_repository
            .find(content_id)
            .await
            .map_err(content_err_fmt)?
            .let_(|content| pin::Output {
                content,
                id: user_id,
            })
            .also_(|o| tracing::trace!("output - {:?}", o))
            .let_(|r| self.pres.complete(r))
            .await
            .unwrap();

        Ok(())
    }
}

pub struct ContentUnpinInteractor {
    pub content_repository: Arc<dyn ContentRepository + Sync + Send>,
    pub pres: Arc<dyn ContentUnpinPresenter + Sync + Send>,
}
#[async_trait]
impl unpin::Usecase for ContentUnpinInteractor {
    #[tracing::instrument(skip(self))]
    async fn handle(&self, data: unpin::Input) -> anyhow::Result<()> {
        tracing::trace!("input - {:?}", data);

        let unpin::Input {
            content_id,
            user_id,
        } = data;

        let can_insert = self
            .content_repository
            .delete_pinned(content_id, user_id)
            .await
            .map_err(content_err_fmt)?;

        if !can_insert {
            bail!("didn't pinned.");
        }

        self.content_repository
            .find(content_id)
            .await
            .map_err(content_err_fmt)?
            .let_(|content| unpin::Output {
                content,
                id: user_id,
            })
            .also_(|o| tracing::trace!("output - {:?}", o))
            .let_(|r| self.pres.complete(r))
            .await
            .unwrap();

        Ok(())
    }
}
