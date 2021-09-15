use anyhow::Result;
use async_trait::async_trait;

use super::super::super::content;
use crate::usecases::content::{
    edit, get, get_like, get_pin, gets, like, pin, post, unlike, unpin, withdraw,
};

pub struct SerenityContentPostPresenter {}
#[async_trait]
impl content::ContentPostPresenter for SerenityContentPostPresenter {
    async fn complete(&self, data: post::Output) -> Result<()> { unimplemented!() }
}

pub struct SerenityContentGetPresenter {}
#[async_trait]
impl content::ContentGetPresenter for SerenityContentGetPresenter {
    async fn complete(&self, data: get::Output) -> Result<()> { unimplemented!() }
}

pub struct SerenityContentGetsPresenter {}
#[async_trait]
impl content::ContentGetsPresenter for SerenityContentGetsPresenter {
    async fn complete(&self, data: gets::Output) -> Result<()> { unimplemented!() }
}

pub struct SerenityContentEditPresenter {}
#[async_trait]
impl content::ContentEditPresenter for SerenityContentEditPresenter {
    async fn complete(&self, data: edit::Output) -> Result<()> { unimplemented!() }
}

pub struct SerenityContentWithdrawPresenter {}
#[async_trait]
impl content::ContentWithdrawPresenter for SerenityContentWithdrawPresenter {
    async fn complete(&self, data: withdraw::Output) -> Result<()> { unimplemented!() }
}

pub struct SerenityContentLikeGetPresenter {}
#[async_trait]
impl content::ContentLikeGetPresenter for SerenityContentLikeGetPresenter {
    async fn complete(&self, data: get_like::Output) -> Result<()> { unimplemented!() }
}

pub struct SerenityContentLikePresenter {}
#[async_trait]
impl content::ContentLikePresenter for SerenityContentLikePresenter {
    async fn complete(&self, data: like::Output) -> Result<()> { unimplemented!() }
}

pub struct SerenityContentUnlikePresenter {}
#[async_trait]
impl content::ContentUnlikePresenter for SerenityContentUnlikePresenter {
    async fn complete(&self, data: unlike::Output) -> Result<()> { unimplemented!() }
}

pub struct SerenityContentPinGetPresenter {}
#[async_trait]
impl content::ContentPinGetPresenter for SerenityContentPinGetPresenter {
    async fn complete(&self, data: get_pin::Output) -> Result<()> { unimplemented!() }
}

pub struct SerenityContentPinPresenter {}
#[async_trait]
impl content::ContentPinPresenter for SerenityContentPinPresenter {
    async fn complete(&self, data: pin::Output) -> Result<()> { unimplemented!() }
}

pub struct SerenityContentUnpinPresenter {}
#[async_trait]
impl content::ContentUnpinPresenter for SerenityContentUnpinPresenter {
    async fn complete(&self, data: unpin::Output) -> Result<()> { unimplemented!() }
}
