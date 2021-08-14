use serenity::model::id::UserId;
use uuid::Uuid;

pub struct User {
    pub id: UserId,
    pub admin: bool,
    pub sub_admin: bool,
}

pub struct Content {
    pub id: Uuid,
    pub content: String,
    pub liked: Vec<UserId>,
    pub bookmarked: u32,
    pub pinned: Vec<UserId>,
}
