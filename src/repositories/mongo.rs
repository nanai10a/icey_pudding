use mongodb::Collection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::entities::Author;

pub struct MongoUserRepository {
    main_coll: Collection<MongoUserModel>,
}

pub struct MongoContentRepository {
    main_coll: Collection<MongoContentModel>,
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
