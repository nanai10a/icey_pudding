use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;

use super::super::super::user;
use crate::entities::User;
use crate::usecases::user::get;

pub struct ReturnUserGetPresenter {
    pub ret: mpsc::Sender<User>,
}
#[async_trait]
impl user::UserGetPresenter for ReturnUserGetPresenter {
    async fn complete(&self, get::Output { user }: get::Output) -> Result<()> {
        self.ret
            .send(user)
            .await
            .map_err(|e| e.to_string())
            .unwrap();

        Ok(())
    }
}
