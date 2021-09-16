use alloc::sync::Arc;
use std::collections::HashSet;

use anyhow::{bail, Result};
use async_trait::async_trait;
use smallvec::SmallVec;
use tokio::sync::mpsc;

// FIXME: move to interactors::
use crate::conductors::calc_paging;
use crate::entities::User;
// FIXME: move to interactors::
use crate::handlers::helpers::*;
use crate::repositories::UserRepository;
use crate::usecases::user::{
    bookmark, edit, get, get_bookmark, gets, register, unbookmark, unregister,
};
use crate::utils::LetChain;

// FIXME: Sender not required shared reference
// FIXME: replace `ret` to `presenter:
//     Arc<dyn [entity][op]Presenter + Sync + Send>`
pub struct ReturnUserRegisterInteractor {
    pub user_repository: Arc<dyn UserRepository + Sync + Send>,
    pub ret: Arc<mpsc::Sender<register::Output>>,
}
#[async_trait]
impl register::Usecase for ReturnUserRegisterInteractor {
    async fn handle(&self, register::Input { user_id }: register::Input) -> Result<()> {
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
            .let_(|r| self.ret.send(r))
            .await
            .unwrap();

        Ok(())
    }
}

pub struct ReturnUserGetInteractor {
    pub user_repository: Arc<dyn UserRepository + Sync + Send>,
    pub ret: Arc<mpsc::Sender<get::Output>>,
}
#[async_trait]
impl get::Usecase for ReturnUserGetInteractor {
    async fn handle(&self, get::Input { user_id }: get::Input) -> Result<()> {
        self.user_repository
            .find(user_id)
            .await
            .map_err(user_err_fmt)?
            .let_(|user| get::Output { user })
            .let_(|r| self.ret.send(r))
            .await
            .unwrap();

        Ok(())
    }
}

pub struct ReturnUserGetsInteractor {
    pub user_repository: Arc<dyn UserRepository + Sync + Send>,
    pub ret: Arc<mpsc::Sender<gets::Output>>,
}
#[async_trait]
impl gets::Usecase for ReturnUserGetsInteractor {
    async fn handle(&self, gets::Input { query, page }: gets::Input) -> Result<()> {
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
            .let_(|r| self.ret.send(r))
            .await
            .unwrap();

        Ok(())
    }
}

pub struct ReturnUserEditInteractor {
    pub user_repository: Arc<dyn UserRepository + Sync + Send>,
    pub ret: Arc<mpsc::Sender<edit::Output>>,
}
#[async_trait]
impl edit::Usecase for ReturnUserEditInteractor {
    async fn handle(&self, edit::Input { user_id, mutation }: edit::Input) -> Result<()> {
        self.user_repository
            .update(user_id, mutation)
            .await
            .map_err(user_err_fmt)?
            .let_(|user| edit::Output { user })
            .let_(|r| self.ret.send(r))
            .await
            .unwrap();

        Ok(())
    }
}

pub struct ReturnUserUnregisterInteractor {
    pub user_repository: Arc<dyn UserRepository + Sync + Send>,
    pub ret: Arc<mpsc::Sender<unregister::Output>>,
}
#[async_trait]
impl unregister::Usecase for ReturnUserUnregisterInteractor {
    async fn handle(&self, unregister::Input { user_id }: unregister::Input) -> Result<()> {
        self.user_repository
            .delete(user_id)
            .await
            .map_err(content_err_fmt)?
            .let_(|user| unregister::Output { user })
            .let_(|r| self.ret.send(r))
            .await
            .unwrap();

        Ok(())
    }
}

pub struct ReturnUserBookmarkGetInteractor {
    pub user_repository: Arc<dyn UserRepository + Sync + Send>,
    pub ret: Arc<mpsc::Sender<get_bookmark::Output>>,
}
#[async_trait]
impl get_bookmark::Usecase for ReturnUserBookmarkGetInteractor {
    async fn handle(
        &self,
        get_bookmark::Input { user_id, page }: get_bookmark::Input,
    ) -> Result<()> {
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
            .let_(|r| self.ret.send(r))
            .await
            .unwrap();

        Ok(())
    }
}

pub struct ReturnUserBookmarkInteractor {
    pub user_repository: Arc<dyn UserRepository + Sync + Send>,
    pub ret: Arc<mpsc::Sender<bookmark::Output>>,
}
#[async_trait]
impl bookmark::Usecase for ReturnUserBookmarkInteractor {
    async fn handle(
        &self,
        bookmark::Input {
            user_id,
            content_id,
        }: bookmark::Input,
    ) -> Result<()> {
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
            .let_(|r| self.ret.send(r))
            .await
            .unwrap();

        Ok(())
    }
}

pub struct ReturnUserUnbookmarkInteractor {
    pub user_repository: Arc<dyn UserRepository + Sync + Send>,
    pub ret: Arc<mpsc::Sender<unbookmark::Output>>,
}
#[async_trait]
impl unbookmark::Usecase for ReturnUserUnbookmarkInteractor {
    async fn handle(
        &self,
        unbookmark::Input {
            user_id,
            content_id,
        }: unbookmark::Input,
    ) -> Result<()> {
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
            .let_(|r| self.ret.send(r))
            .await
            .unwrap();

        Ok(())
    }
}
