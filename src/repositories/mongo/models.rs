use std::collections::HashSet;

use crate::entities::ContentId;

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MongoUserModel {
    pub id: String,
    pub admin: bool,
    pub sub_admin: bool,
    pub bookmark: HashSet<ContentId>,
    pub bookmark_size: i64,
}

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MongoContentModel {
    pub id: ContentId,
    pub author: MongoContentAuthorModel,
    pub posted: MongoContentPostedModel,
    pub content: String,
    pub liked: HashSet<String>,
    pub liked_size: i64,
    pub pinned: HashSet<String>,
    pub pinned_size: i64,
    pub created: String,
    pub edited: Vec<String>,
}

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub enum MongoContentAuthorModel {
    User {
        id: String,
        name: String,
        nick: Option<String>,
    },
    Virtual(String),
}

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize)]
pub struct MongoContentPostedModel {
    pub id: String,
    pub name: String,
    pub nick: Option<String>,
}
