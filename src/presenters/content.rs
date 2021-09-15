use anyhow::Result;
use async_trait::async_trait;

use crate::usecases::content::{
    edit, get, get_like, get_pin, gets, like, pin, post, unlike, unpin, withdraw,
};

#[async_trait]
pub trait ContentPostPresenter {
    async fn complete(&self, data: post::Output) -> Result<()>;
}

#[async_trait]
pub trait ContentGetPresenter {
    async fn complete(&self, data: get::Output) -> Result<()>;
}

#[async_trait]
pub trait ContentGetsPresenter {
    async fn complete(&self, data: gets::Output) -> Result<()>;
}

#[async_trait]
pub trait ContentEditPresenter {
    async fn complete(&self, data: edit::Output) -> Result<()>;
}

#[async_trait]
pub trait ContentWithdrawPresenter {
    async fn complete(&self, data: withdraw::Output) -> Result<()>;
}

#[async_trait]
pub trait ContentLikeGetPresenter {
    async fn complete(&self, data: get_like::Output) -> Result<()>;
}

#[async_trait]
pub trait ContentLikePresenter {
    async fn complete(&self, data: like::Output) -> Result<()>;
}

#[async_trait]
pub trait ContentUnlikePresenter {
    async fn complete(&self, data: unlike::Output) -> Result<()>;
}

#[async_trait]
pub trait ContentPinGetPresenter {
    async fn complete(&self, data: get_pin::Output) -> Result<()>;
}

#[async_trait]
pub trait ContentPinPresenter {
    async fn complete(&self, data: pin::Output) -> Result<()>;
}

#[async_trait]
pub trait ContentUnpinPresenter {
    async fn complete(&self, data: unpin::Output) -> Result<()>;
}
