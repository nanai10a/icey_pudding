use mongodb::{Collection, Database};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub struct MongoUserRepository {
    main_coll: Collection<MongoUserModel>,
    db: Database,
}

impl MongoUserRepository {
    #[inline]
    fn posted_coll(&self, id: u64) -> Collection<Uuid> {
        self.db.collection(format!("user:{}#posted", id).as_str())
    }

    #[inline]
    fn bookmarked_coll(&self, id: u64) -> Collection<Uuid> {
        self.db.collection(format!("user:{}#bookmarked", id).as_str())
    }
}

pub struct MongoContentRepository {
    main_coll: Collection<MongoContentModel>,
    db: Database,
}

impl MongoContentRepository {
    #[inline]
    fn liked_coll(&self, id: Uuid) -> Collection<u64> {
        self.db.collection(format!("content:{}#liked", id.as_u128()).as_str())
    }

    #[inline]
    fn pinned_coll(&self, id: Uuid) -> Collection<u64> {
        self.db.collection(format!("content:{}#pinned", id.as_u128()).as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MongoUserModel {
    id: u64,
    admin: bool,
    sub_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MongoContentModel {
    id: Uuid,
    author: Author,
    posted: u64,
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum Author {
    User {
        id: u64,
        name: String,
        nick: Option<String>,
    },
    Virtual(String),
}
