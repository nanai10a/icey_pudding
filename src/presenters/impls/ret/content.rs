use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc;

use super::super::super::content;
use crate::entities::Content;
use crate::usecases::content::get;

pub struct ReturnContentGetPresenter {
    pub ret: mpsc::Sender<Content>,
}
#[async_trait]
impl content::ContentGetPresenter for ReturnContentGetPresenter {
    async fn complete(&self, get::Output { content }: get::Output) -> Result<()> {
        self.ret
            .send(content)
            .await
            .map_err(|e| e.to_string())
            .unwrap();

        Ok(())
    }
}
