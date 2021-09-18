use alloc::sync::Arc;

use anyhow::Result;
use async_recursion::async_recursion;
use smallvec::SmallVec;
use tokio::sync::{mpsc, Mutex};

use crate::presenters::impls::serenity::View;
use crate::usecases::user::{
    bookmark, edit, get, get_bookmark, gets, register, unbookmark, unregister,
};

pub struct SerenityUserController {
    pub register: Arc<dyn register::Usecase + Sync + Send>,
    pub register_lock: Mutex<()>,
    pub register_ret: Mutex<mpsc::Receiver<Box<View>>>,

    pub get: Arc<dyn get::Usecase + Sync + Send>,
    pub get_lock: Mutex<()>,
    pub get_ret: Mutex<mpsc::Receiver<Box<View>>>,

    pub gets: Arc<dyn gets::Usecase + Sync + Send>,
    pub gets_lock: Mutex<()>,
    pub gets_ret: Mutex<mpsc::Receiver<SmallVec<[Box<View>; 5]>>>,

    pub edit: Arc<dyn edit::Usecase + Sync + Send>,
    pub edit_lock: Mutex<()>,
    pub edit_ret: Mutex<mpsc::Receiver<Box<View>>>,

    pub unregister: Arc<dyn unregister::Usecase + Sync + Send>,
    pub unregister_lock: Mutex<()>,
    pub unregister_ret: Mutex<mpsc::Receiver<Box<View>>>,

    pub get_bookmark: Arc<dyn get_bookmark::Usecase + Sync + Send>,
    pub get_bookmark_lock: Mutex<()>,
    pub get_bookmark_ret: Mutex<mpsc::Receiver<SmallVec<[Box<View>; 20]>>>,

    pub bookmark: Arc<dyn bookmark::Usecase + Sync + Send>,
    pub bookmark_lock: Mutex<()>,
    pub bookmark_ret: Mutex<mpsc::Receiver<Box<View>>>,

    pub unbookmark: Arc<dyn unbookmark::Usecase + Sync + Send>,
    pub unbookmark_lock: Mutex<()>,
    pub unbookmark_ret: Mutex<mpsc::Receiver<Box<View>>>,
}
impl SerenityUserController {
    #[async_recursion]
    pub async fn register(&self, data: register::Input) -> Result<Box<View>> {
        return_inner!(self =>
            use register,
            lock register_lock,
            ret register_ret,
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
    pub async fn unregister(&self, data: unregister::Input) -> Result<Box<View>> {
        return_inner!(self =>
            use unregister,
            lock unregister_lock,
            ret unregister_ret,
            data data
        )
    }

    #[async_recursion]
    pub async fn get_bookmark(
        &self,
        data: get_bookmark::Input,
    ) -> Result<SmallVec<[Box<View>; 20]>> {
        return_inner!(self =>
            use get_bookmark,
            lock get_bookmark_lock,
            ret get_bookmark_ret,
            data data
        )
    }

    #[async_recursion]
    pub async fn bookmark(&self, data: bookmark::Input) -> Result<Box<View>> {
        return_inner!(self =>
            use bookmark,
            lock bookmark_lock,
            ret bookmark_ret,
            data data
        )
    }

    #[async_recursion]
    pub async fn unbookmark(&self, data: unbookmark::Input) -> Result<Box<View>> {
        return_inner!(self =>
            use unbookmark,
            lock unbookmark_lock,
            ret unbookmark_ret,
            data data
        )
    }
}
