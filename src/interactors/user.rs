use alloc::sync::Arc;
use std::collections::HashSet;

use anyhow::{bail, Result};
use async_trait::async_trait;
use smallvec::SmallVec;

use super::*;
use crate::entities::User;
use crate::presenters::user::{
    UserBookmarkGetPresenter, UserBookmarkPresenter, UserEditPresenter, UserGetPresenter,
    UserGetsPresenter, UserRegisterPresenter, UserUnbookmarkPresenter, UserUnregisterPresenter,
};
use crate::repositories::UserRepository;
use crate::usecases::user::{
    bookmark, edit, get, get_bookmark, gets, register, unbookmark, unregister,
};
use crate::utils::{AlsoChain, LetChain};

pub struct UserRegisterInteractor {
    pub user_repository: Arc<dyn UserRepository + Sync + Send>,
    pub pres: Arc<dyn UserRegisterPresenter + Sync + Send>,
}
#[async_trait]
impl register::Usecase for UserRegisterInteractor {
    #[tracing::instrument(skip(self))]
    async fn handle(&self, data: register::Input) -> Result<()> {
        tracing::trace!("input - {:?}", data);

        let register::Input { user_id } = data;

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

        register::Output { user: new_user }
            .also_(|o| tracing::trace!("output - {:?}", o))
            .let_(|r| self.pres.complete(r))
            .await
            .unwrap();

        Ok(())
    }
}

pub struct UserGetInteractor {
    pub user_repository: Arc<dyn UserRepository + Sync + Send>,
    pub pres: Arc<dyn UserGetPresenter + Sync + Send>,
}
#[async_trait]
impl get::Usecase for UserGetInteractor {
    #[tracing::instrument(skip(self))]
    async fn handle(&self, data: get::Input) -> Result<()> {
        tracing::trace!("input - {:?}", data);

        let get::Input { user_id } = data;

        self.user_repository
            .find(user_id)
            .await
            .map_err(user_err_fmt)?
            .let_(|user| get::Output { user })
            .also_(|o| tracing::trace!("output - {:?}", o))
            .let_(|r| self.pres.complete(r))
            .await
            .unwrap();

        Ok(())
    }
}

pub struct UserGetsInteractor {
    pub user_repository: Arc<dyn UserRepository + Sync + Send>,
    pub pres: Arc<dyn UserGetsPresenter + Sync + Send>,
}
#[async_trait]
impl gets::Usecase for UserGetsInteractor {
    #[tracing::instrument(skip(self))]
    async fn handle(&self, data: gets::Input) -> Result<()> {
        tracing::trace!("input - {:?}", data);

        let gets::Input { query, page } = data;

        self.user_repository
            .finds(query)
            .await
            .map_err(user_err_fmt)?
            .let_(|mut v| {
                calc_paging(0..v.len(), 5, page as usize).map(move |lim| {
                    v.drain(lim)
                        .enumerate()
                        .map(|(i, u)| (i as u32, u))
                        .collect::<SmallVec<[_; 5]>>()
                })
            })?
            .let_(|users| gets::Output { users, page })
            .also_(|o| tracing::trace!("output - {:?}", o))
            .let_(|r| self.pres.complete(r))
            .await
            .unwrap();

        Ok(())
    }
}

pub struct UserEditInteractor {
    pub user_repository: Arc<dyn UserRepository + Sync + Send>,
    pub pres: Arc<dyn UserEditPresenter + Sync + Send>,
}
#[async_trait]
impl edit::Usecase for UserEditInteractor {
    #[tracing::instrument(skip(self))]
    async fn handle(&self, data: edit::Input) -> Result<()> {
        tracing::trace!("input - {:?}", data);

        let edit::Input { user_id, mutation } = data;

        self.user_repository
            .update(user_id, mutation)
            .await
            .map_err(user_err_fmt)?
            .let_(|user| edit::Output { user })
            .also_(|o| tracing::trace!("output - {:?}", o))
            .let_(|r| self.pres.complete(r))
            .await
            .unwrap();

        Ok(())
    }
}

pub struct UserUnregisterInteractor {
    pub user_repository: Arc<dyn UserRepository + Sync + Send>,
    pub pres: Arc<dyn UserUnregisterPresenter + Sync + Send>,
}
#[async_trait]
impl unregister::Usecase for UserUnregisterInteractor {
    #[tracing::instrument(skip(self))]
    async fn handle(&self, data: unregister::Input) -> Result<()> {
        tracing::trace!("input - {:?}", data);

        let unregister::Input { user_id } = data;

        self.user_repository
            .delete(user_id)
            .await
            .map_err(content_err_fmt)?
            .let_(|user| unregister::Output { user })
            .also_(|o| tracing::trace!("output - {:?}", o))
            .let_(|r| self.pres.complete(r))
            .await
            .unwrap();

        Ok(())
    }
}

pub struct UserBookmarkGetInteractor {
    pub user_repository: Arc<dyn UserRepository + Sync + Send>,
    pub pres: Arc<dyn UserBookmarkGetPresenter + Sync + Send>,
}
#[async_trait]
impl get_bookmark::Usecase for UserBookmarkGetInteractor {
    #[tracing::instrument(skip(self))]
    async fn handle(&self, data: get_bookmark::Input) -> Result<()> {
        tracing::trace!("input - {:?}", data);

        let get_bookmark::Input { user_id, page } = data;

        self.user_repository
            .get_bookmark(user_id)
            .await
            .map_err(content_err_fmt)?
            .drain()
            .collect::<Vec<_>>()
            .let_(|mut v| {
                calc_paging(0..v.len(), 20, page as usize).map(move |lim| {
                    v.drain(lim)
                        .enumerate()
                        .map(|(i, d)| (i as u32, d))
                        .collect::<SmallVec<[_; 20]>>()
                })
            })?
            .let_(|bookmark| get_bookmark::Output { bookmark, page })
            .also_(|o| tracing::trace!("output - {:?}", o))
            .let_(|r| self.pres.complete(r))
            .await
            .unwrap();

        Ok(())
    }
}

pub struct UserBookmarkInteractor {
    pub user_repository: Arc<dyn UserRepository + Sync + Send>,
    pub pres: Arc<dyn UserBookmarkPresenter + Sync + Send>,
}
#[async_trait]
impl bookmark::Usecase for UserBookmarkInteractor {
    #[tracing::instrument(skip(self))]
    async fn handle(&self, data: bookmark::Input) -> Result<()> {
        tracing::trace!("input - {:?}", data);

        let bookmark::Input {
            user_id,
            content_id,
        } = data;

        let can_insert = self
            .user_repository
            .insert_bookmark(user_id, content_id)
            .await
            .map_err(user_err_fmt)?;

        if !can_insert {
            bail!("already bookmarked.");
        }

        self.user_repository
            .find(user_id)
            .await
            .map_err(user_err_fmt)?
            .let_(|user| bookmark::Output {
                user,
                id: content_id,
            })
            .also_(|o| tracing::trace!("output - {:?}", o))
            .let_(|r| self.pres.complete(r))
            .await
            .unwrap();

        Ok(())
    }
}

pub struct UserUnbookmarkInteractor {
    pub user_repository: Arc<dyn UserRepository + Sync + Send>,
    pub pres: Arc<dyn UserUnbookmarkPresenter + Sync + Send>,
}
#[async_trait]
impl unbookmark::Usecase for UserUnbookmarkInteractor {
    #[tracing::instrument(skip(self))]
    async fn handle(&self, data: unbookmark::Input) -> Result<()> {
        tracing::trace!("input - {:?}", data);

        let unbookmark::Input {
            user_id,
            content_id,
        } = data;

        let can_insert = self
            .user_repository
            .delete_bookmark(user_id, content_id)
            .await
            .map_err(user_err_fmt)?;

        if !can_insert {
            bail!("didn't bookmarked.");
        }

        self.user_repository
            .find(user_id)
            .await
            .map_err(user_err_fmt)?
            .let_(|user| unbookmark::Output {
                user,
                id: content_id,
            })
            .also_(|o| tracing::trace!("output - {:?}", o))
            .let_(|r| self.pres.complete(r))
            .await
            .unwrap();

        Ok(())
    }
}
