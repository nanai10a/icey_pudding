use super::{MongoContentAuthorModel, MongoContentModel, MongoContentPostedModel, MongoUserModel};
use crate::entities::{Author, Content, Posted, User};

impl From<MongoUserModel> for User {
    fn from(
        MongoUserModel {
            id,
            admin,
            sub_admin,
            posted,
            posted_size: _,
            bookmark,
            bookmark_size: _,
        }: MongoUserModel,
    ) -> User {
        User {
            id: id.parse().unwrap(),
            admin,
            sub_admin,
            posted,
            bookmark,
        }
    }
}
impl From<User> for MongoUserModel {
    fn from(
        User {
            id,
            admin,
            sub_admin,
            posted,
            bookmark,
        }: User,
    ) -> Self {
        MongoUserModel {
            id: id.to_string(),
            admin,
            sub_admin,
            posted_size: posted.len() as i64,
            posted,
            bookmark_size: bookmark.len() as i64,
            bookmark,
        }
    }
}

impl From<MongoContentModel> for Content {
    fn from(
        MongoContentModel {
            id,
            author,
            posted,
            content,
            mut liked,
            liked_size: _,
            mut pinned,
            pinned_size: _,
        }: MongoContentModel,
    ) -> Self {
        Content {
            id,
            author: author.into(),
            posted: posted.into(),
            content,
            liked: liked.drain().map(|s| s.parse().unwrap()).collect(),
            pinned: pinned.drain().map(|s| s.parse().unwrap()).collect(),
        }
    }
}
impl From<Content> for MongoContentModel {
    fn from(
        Content {
            id,
            author,
            posted,
            content,
            mut liked,
            mut pinned,
        }: Content,
    ) -> Self {
        MongoContentModel {
            id,
            author: author.into(),
            posted: posted.into(),
            content,
            liked_size: liked.len() as i64,
            liked: liked.drain().map(|n| n.to_string()).collect(),
            pinned_size: pinned.len() as i64,
            pinned: pinned.drain().map(|n| n.to_string()).collect(),
        }
    }
}

impl From<MongoContentAuthorModel> for Author {
    fn from(m: MongoContentAuthorModel) -> Self {
        match m {
            MongoContentAuthorModel::User { id, name, nick } => Author::User {
                id: id.parse().unwrap(),
                name,
                nick,
            },
            MongoContentAuthorModel::Virtual(s) => Author::Virtual(s),
        }
    }
}
impl From<Author> for MongoContentAuthorModel {
    fn from(a: Author) -> Self {
        match a {
            Author::User { id, name, nick } => MongoContentAuthorModel::User {
                id: id.to_string(),
                name,
                nick,
            },
            Author::Virtual(s) => MongoContentAuthorModel::Virtual(s),
        }
    }
}

impl From<MongoContentPostedModel> for Posted {
    fn from(MongoContentPostedModel { id, name, nick }: MongoContentPostedModel) -> Self {
        Posted {
            id: id.parse().unwrap(),
            name,
            nick,
        }
    }
}
impl From<Posted> for MongoContentPostedModel {
    fn from(Posted { id, name, nick }: Posted) -> Self {
        MongoContentPostedModel {
            id: id.to_string(),
            name,
            nick,
        }
    }
}
