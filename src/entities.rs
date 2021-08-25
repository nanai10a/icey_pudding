use std::collections::HashSet;

use serenity::model::id::UserId;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub admin: bool,
    pub sub_admin: bool,
    pub posted: HashSet<Uuid>,
    pub bookmark: HashSet<Uuid>,
}

#[derive(Debug, Clone)]
pub struct Content {
    pub id: Uuid,
    pub author: String, /* TODO: `Discordに存在する人物(UserID) || 何らかの人物(String)` */
    pub posted: UserId,
    pub content: String,
    pub liked: HashSet<UserId>,
    pub bookmarked: u32,
    pub pinned: HashSet<UserId>,
}
