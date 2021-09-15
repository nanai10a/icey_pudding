use anyhow::Result;
use async_trait::async_trait;

use crate::usecases::user::{
    bookmark, edit, get, get_bookmark, gets, register, unbookmark, unregister,
};

#[async_trait]
pub trait UserRegisterPresenter {
    async fn complete(&self, data: register::Output) -> Result<()>;
}

#[async_trait]
pub trait UserGetPresenter {
    async fn complete(&self, data: get::Output) -> Result<()>;
}

#[async_trait]
pub trait UserGetsPresenter {
    async fn complete(&self, data: gets::Output) -> Result<()>;
}

#[async_trait]
pub trait UserEditPresenter {
    async fn complete(&self, data: edit::Output) -> Result<()>;
}

#[async_trait]
pub trait UserUnregisterPresenter {
    async fn complete(&self, data: unregister::Output) -> Result<()>;
}

#[async_trait]
pub trait UserBookmarkGetPresenter {
    async fn complete(&self, data: get_bookmark::Output) -> Result<()>;
}

#[async_trait]
pub trait UserBookmarkPresenter {
    async fn complete(&self, data: bookmark::Output) -> Result<()>;
}

#[async_trait]
pub trait UserUnbookmarkPresenter {
    async fn complete(&self, data: unbookmark::Output) -> Result<()>;
}
