use alloc::sync::Arc;

use anyhow::Result;
use async_recursion::async_recursion;
use smallvec::SmallVec;
use tokio::sync::{mpsc, Mutex};

use crate::presenters::impls::serenity::View;
use crate::usecases::content::{
    edit, get, get_like, get_pin, gets, like, pin, post, unlike, unpin, withdraw,
};

pub struct SerenityContentController {
    pub post: Arc<dyn post::Usecase + Sync + Send>,
    pub post_lock: Mutex<()>,
    pub post_ret: Mutex<mpsc::Receiver<Box<View>>>,

    pub get: Arc<dyn get::Usecase + Sync + Send>,
    pub get_lock: Mutex<()>,
    pub get_ret: Mutex<mpsc::Receiver<Box<View>>>,

    pub gets: Arc<dyn gets::Usecase + Sync + Send>,
    pub gets_lock: Mutex<()>,
    pub gets_ret: Mutex<mpsc::Receiver<SmallVec<[Box<View>; 5]>>>,

    pub edit: Arc<dyn edit::Usecase + Sync + Send>,
    pub edit_lock: Mutex<()>,
    pub edit_ret: Mutex<mpsc::Receiver<Box<View>>>,

    pub withdraw: Arc<dyn withdraw::Usecase + Sync + Send>,
    pub withdraw_lock: Mutex<()>,
    pub withdraw_ret: Mutex<mpsc::Receiver<Box<View>>>,

    pub get_like: Arc<dyn get_like::Usecase + Sync + Send>,
    pub get_like_lock: Mutex<()>,
    pub get_like_ret: Mutex<mpsc::Receiver<SmallVec<[Box<View>; 20]>>>,

    pub like: Arc<dyn like::Usecase + Sync + Send>,
    pub like_lock: Mutex<()>,
    pub like_ret: Mutex<mpsc::Receiver<Box<View>>>,

    pub unlike: Arc<dyn unlike::Usecase + Sync + Send>,
    pub unlike_lock: Mutex<()>,
    pub unlike_ret: Mutex<mpsc::Receiver<Box<View>>>,

    pub get_pin: Arc<dyn get_pin::Usecase + Sync + Send>,
    pub get_pin_lock: Mutex<()>,
    pub get_pin_ret: Mutex<mpsc::Receiver<SmallVec<[Box<View>; 20]>>>,

    pub pin: Arc<dyn pin::Usecase + Sync + Send>,
    pub pin_lock: Mutex<()>,
    pub pin_ret: Mutex<mpsc::Receiver<Box<View>>>,

    pub unpin: Arc<dyn unpin::Usecase + Sync + Send>,
    pub unpin_lock: Mutex<()>,
    pub unpin_ret: Mutex<mpsc::Receiver<Box<View>>>,
}

impl SerenityContentController {
    #[async_recursion]
    pub async fn post(&self, data: post::Input) -> Result<Box<View>> {
        return_inner!(self =>
            use post,
            lock post_lock,
            ret post_ret,
            data data
        )
    }

    #[async_recursion]
    pub async fn get(&self, data: get::Input) -> Result<Box<View>> {
        return_inner!(self =>
            use get,
            lock get_lock,
            ret get_ret,
            data data
        )
    }

    #[async_recursion]
    pub async fn gets(&self, data: gets::Input) -> Result<SmallVec<[Box<View>; 5]>> {
        return_inner!(self =>
            use gets,
            lock gets_lock,
            ret gets_ret,
            data data
        )
    }

    #[async_recursion]
    pub async fn edit(&self, data: edit::Input) -> Result<Box<View>> {
        return_inner!(self =>
            use edit,
            lock edit_lock,
            ret edit_ret,
            data data
        )
    }

    #[async_recursion]
    pub async fn withdraw(&self, data: withdraw::Input) -> Result<Box<View>> {
        return_inner!(self =>
            use withdraw,
            lock withdraw_lock,
            ret withdraw_ret,
            data data
        )
    }

    #[async_recursion]
    pub async fn get_like(&self, data: get_like::Input) -> Result<SmallVec<[Box<View>; 20]>> {
        return_inner!(self =>
            use get_like,
            lock get_like_lock,
            ret get_like_ret,
            data data
        )
    }

    #[async_recursion]
    pub async fn like(&self, data: like::Input) -> Result<Box<View>> {
        return_inner!(self =>
            use like,
            lock like_lock,
            ret like_ret,
            data data
        )
    }

    #[async_recursion]
    pub async fn unlike(&self, data: unlike::Input) -> Result<Box<View>> {
        return_inner!(self =>
            use unlike,
            lock unlike_lock,
            ret unlike_ret,
            data data
        )
    }

    #[async_recursion]
    pub async fn get_pin(&self, data: get_pin::Input) -> Result<SmallVec<[Box<View>; 20]>> {
        return_inner!(self =>
            use get_pin,
            lock get_pin_lock,
            ret get_pin_ret,
            data data
        )
    }

    #[async_recursion]
    pub async fn pin(&self, data: pin::Input) -> Result<Box<View>> {
        return_inner!(self =>
            use pin,
            lock pin_lock,
            ret pin_ret,
            data data
        )
    }

    #[async_recursion]
    pub async fn unpin(&self, data: unpin::Input) -> Result<Box<View>> {
        return_inner!(self =>
            use unpin,
            lock unpin_lock,
            ret unpin_ret,
            data data
        )
    }
}
