use serenity::client::EventHandler;

use crate::conductors::Conductor;
use crate::entities::{Content, User};
use crate::handlers::Handler;
use crate::repositories::mock::InMemoryRepository;
use crate::repositories::mongo::{MongoContentRepository, MongoUserRepository};

pub fn in_memory() -> impl EventHandler {
    Conductor {
        handler: Handler {
            user_repository: Box::new(InMemoryRepository::<User>::new()),
            content_repository: Box::new(InMemoryRepository::<Content>::new()),
        },
    }
}

pub async fn mongo(uri_str: impl AsRef<str>, db_name: impl AsRef<str>) -> impl EventHandler {
    let c = ::mongodb::Client::with_uri_str(uri_str).await.unwrap();
    let db = c.database(db_name.as_ref());

    Conductor {
        handler: Handler {
            user_repository: box MongoUserRepository::new_with(c.clone(), db.clone())
                .await
                .unwrap(),
            content_repository: box MongoContentRepository::new_with(c, db).await.unwrap(),
        },
    }
}
