use anyhow::Result;
use async_trait::async_trait;

use super::super::super::user;
use crate::usecases::user::{
    bookmark, edit, get, get_bookmark, gets, register, unbookmark, unregister,
};

pub struct SerenityUserRegisterPresenter {}
#[async_trait]
impl user::UserRegisterPresenter for SerenityUserRegisterPresenter {
    async fn complete(&self, data: register::Output) -> Result<()> { unimplemented!() }
}

pub struct SerenityUserGetPresenter {}
#[async_trait]
impl user::UserGetPresenter for SerenityUserGetPresenter {
    async fn complete(&self, data: get::Output) -> Result<()> { unimplemented!() }
}

pub struct SerenityUserGetsPresenter {}
#[async_trait]
impl user::UserGetsPresenter for SerenityUserGetsPresenter {
    async fn complete(&self, data: gets::Output) -> Result<()> { unimplemented!() }
}

pub struct SerenityUserEditPresenter {}
#[async_trait]
impl user::UserEditPresenter for SerenityUserEditPresenter {
    async fn complete(&self, data: edit::Output) -> Result<()> { unimplemented!() }
}

pub struct SerenityUserUnregisterPresenter {}
#[async_trait]
impl user::UserUnregisterPresenter for SerenityUserUnregisterPresenter {
    async fn complete(&self, data: unregister::Output) -> Result<()> { unimplemented!() }
}

pub struct SerenityUserBookmarkGetPresenter {}
#[async_trait]
impl user::UserBookmarkGetPresenter for SerenityUserBookmarkGetPresenter {
    async fn complete(&self, data: get_bookmark::Output) -> Result<()> { unimplemented!() }
}

pub struct SerenityUserBookmarkPresenter {}
#[async_trait]
impl user::UserBookmarkPresenter for SerenityUserBookmarkPresenter {
    async fn complete(&self, data: bookmark::Output) -> Result<()> { unimplemented!() }
}

pub struct SerenityUserUnbookmarkPresenter {}
#[async_trait]
impl user::UserUnbookmarkPresenter for SerenityUserUnbookmarkPresenter {
    async fn complete(&self, data: unbookmark::Output) -> Result<()> { unimplemented!() }
}
