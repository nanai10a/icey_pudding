use serenity::model::id::UserId;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub admin: bool,
    pub sub_admin: bool,
    pub posted: Vec<Uuid>,
    pub bookmark: Vec<Uuid>,
}

#[derive(Debug, Clone)]
pub struct Content {
    pub id: Uuid,
    pub content: String,
    pub liked: Vec<UserId>,
    pub bookmarked: u32,
    pub pinned: Vec<UserId>,
}
