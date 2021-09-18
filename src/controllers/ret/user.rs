use alloc::sync::Arc;

use anyhow::Result;
use tokio::sync::{mpsc, Mutex};

use crate::entities::{User, UserId};
use crate::usecases::user::get;

pub struct ReturnUserController {
    pub usecase: Arc<dyn get::Usecase + Sync + Send>,
    pub lock: Mutex<()>,
    pub ret: Mutex<mpsc::Receiver<User>>,
}
impl ReturnUserController {
    pub async fn get(&self, user_id: UserId) -> Result<User> {
        let guard = self.lock.lock().await;

        self.usecase.handle(get::Input { user_id }).await?;
        let user = self.ret.lock().await.recv().await.unwrap();

        drop(guard);

        Ok(user)
    }
}
