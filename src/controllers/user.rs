use alloc::sync::Arc;

use anyhow::Result;
use tokio::sync::{mpsc, Mutex};

use crate::usecases::user::{
    bookmark, edit, get, get_bookmark, gets, register, unbookmark, unregister,
};

pub struct ReturnUserController {
    register: Arc<dyn register::Usecase + Sync + Send>,
    register_lock: Mutex<()>,
    register_ret: Mutex<mpsc::Receiver<register::Output>>,

    get: Arc<dyn get::Usecase + Sync + Send>,
    get_lock: Mutex<()>,
    get_ret: Mutex<mpsc::Receiver<get::Output>>,

    gets: Arc<dyn gets::Usecase + Sync + Send>,
    gets_lock: Mutex<()>,
    gets_ret: Mutex<mpsc::Receiver<gets::Output>>,

    edit: Arc<dyn edit::Usecase + Sync + Send>,
    edit_lock: Mutex<()>,
    edit_ret: Mutex<mpsc::Receiver<edit::Output>>,

    unregister: Arc<dyn unregister::Usecase + Sync + Send>,
    unregister_lock: Mutex<()>,
    unregister_ret: Mutex<mpsc::Receiver<unregister::Output>>,

    get_bookmark: Arc<dyn get_bookmark::Usecase + Sync + Send>,
    get_bookmark_lock: Mutex<()>,
    get_bookmark_ret: Mutex<mpsc::Receiver<get_bookmark::Output>>,

    bookmark: Arc<dyn bookmark::Usecase + Sync + Send>,
    bookmark_lock: Mutex<()>,
    bookmark_ret: Mutex<mpsc::Receiver<bookmark::Output>>,

    unbookmark: Arc<dyn unbookmark::Usecase + Sync + Send>,
    unbookmark_lock: Mutex<()>,
    unbookmark_ret: Mutex<mpsc::Receiver<unbookmark::Output>>,
}
impl ReturnUserController {
    pub async fn register(&self, data: register::Input) -> Result<register::Output> {
        return_inner!(self =>
            use register,
            lock register_lock,
            ret register_ret,
            data data
        )
    }

    pub async fn get(&self, data: get::Input) -> Result<get::Output> {
        return_inner!(self =>
            use get,
            lock get_lock,
            ret get_ret,
            data data
        )
    }

    pub async fn gets(&self, data: gets::Input) -> Result<gets::Output> {
        return_inner!(self =>
            use gets,
            lock gets_lock,
            ret gets_ret,
            data data
        )
    }

    pub async fn edit(&self, data: edit::Input) -> Result<edit::Output> {
        return_inner!(self =>
            use edit,
            lock edit_lock,
            ret edit_ret,
            data data
        )
    }

    pub async fn unregister(&self, data: unregister::Input) -> Result<unregister::Output> {
        return_inner!(self =>
            use unregister,
            lock unregister_lock,
            ret unregister_ret,
            data data
        )
    }

    pub async fn get_bookmark(&self, data: get_bookmark::Input) -> Result<get_bookmark::Output> {
        return_inner!(self =>
            use get_bookmark,
            lock get_bookmark_lock,
            ret get_bookmark_ret,
            data data
        )
    }

    pub async fn bookmark(&self, data: bookmark::Input) -> Result<bookmark::Output> {
        return_inner!(self =>
            use bookmark,
            lock bookmark_lock,
            ret bookmark_ret,
            data data
        )
    }

    pub async fn unbookmark(&self, data: unbookmark::Input) -> Result<unbookmark::Output> {
        return_inner!(self =>
            use unbookmark,
            lock unbookmark_lock,
            ret unbookmark_ret,
            data data
        )
    }
}
