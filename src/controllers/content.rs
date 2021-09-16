use alloc::sync::Arc;

use anyhow::Result;
use async_recursion::async_recursion;
use smallvec::SmallVec;
use tokio::sync::{mpsc, Mutex};

use crate::presenters::impls::serenity::View;
use crate::usecases::content::{
    edit, get, get_like, get_pin, gets, like, pin, post, unlike, unpin, withdraw,
};

pub struct ReturnContentController {
    post: Arc<dyn post::Usecase + Sync + Send>,
    post_lock: Mutex<()>,
    post_ret: Mutex<mpsc::Receiver<Box<View>>>,

    get: Arc<dyn get::Usecase + Sync + Send>,
    get_lock: Mutex<()>,
    get_ret: Mutex<mpsc::Receiver<Box<View>>>,

    gets: Arc<dyn gets::Usecase + Sync + Send>,
    gets_lock: Mutex<()>,
    gets_ret: Mutex<mpsc::Receiver<SmallVec<[Box<View>; 5]>>>,

    edit: Arc<dyn edit::Usecase + Sync + Send>,
    edit_lock: Mutex<()>,
    edit_ret: Mutex<mpsc::Receiver<Box<View>>>,

    withdraw: Arc<dyn withdraw::Usecase + Sync + Send>,
    withdraw_lock: Mutex<()>,
    withdraw_ret: Mutex<mpsc::Receiver<Box<View>>>,

    get_like: Arc<dyn get_like::Usecase + Sync + Send>,
    get_like_lock: Mutex<()>,
    get_like_ret: Mutex<mpsc::Receiver<SmallVec<[Box<View>; 20]>>>,

    like: Arc<dyn like::Usecase + Sync + Send>,
    like_lock: Mutex<()>,
    like_ret: Mutex<mpsc::Receiver<Box<View>>>,

    unlike: Arc<dyn unlike::Usecase + Sync + Send>,
    unlike_lock: Mutex<()>,
    unlike_ret: Mutex<mpsc::Receiver<Box<View>>>,

    get_pin: Arc<dyn get_pin::Usecase + Sync + Send>,
    get_pin_lock: Mutex<()>,
    get_pin_ret: Mutex<mpsc::Receiver<SmallVec<[Box<View>; 20]>>>,

    pin: Arc<dyn pin::Usecase + Sync + Send>,
    pin_lock: Mutex<()>,
    pin_ret: Mutex<mpsc::Receiver<Box<View>>>,

    unpin: Arc<dyn unpin::Usecase + Sync + Send>,
    unpin_lock: Mutex<()>,
    unpin_ret: Mutex<mpsc::Receiver<Box<View>>>,
}

impl ReturnContentController {
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
