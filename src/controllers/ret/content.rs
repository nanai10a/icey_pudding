use alloc::sync::Arc;

use anyhow::Result;
use tokio::sync::{mpsc, Mutex};

use crate::entities::{Content, ContentId};
use crate::usecases::content::get;

pub struct ReturnContentController {
    pub usecase: Arc<dyn get::Usecase + Sync + Send>,
    pub lock: Mutex<()>,
    pub ret: Mutex<mpsc::Receiver<Content>>,
}
impl ReturnContentController {
    pub async fn get(&self, content_id: ContentId) -> Result<Content> {
        let guard = self.lock.lock().await;

        self.usecase.handle(get::Input { content_id }).await?;
        let content = self.ret.lock().await.recv().await.unwrap();

        drop(guard);

        Ok(content)
    }
}
