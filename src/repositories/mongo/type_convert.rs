use core::ops::Bound;

use mongodb::bson::{doc, Bson, Document};

use super::{
    LetChain, MongoContentAuthorModel, MongoContentModel, MongoContentPostedModel, MongoUserModel,
    UserMutation, UserQuery,
};
use crate::entities::{Author, Content, ContentId, Posted, User, UserId};
use crate::utils;

impl From<UserQuery> for Document {
    fn from(
        UserQuery {
            bookmark,
            bookmark_num,
        }: UserQuery,
    ) -> Self {
        let mut query = doc! {};

        if let Some(mut set_raw) = bookmark {
            if !set_raw.is_empty() {
                let set = set_raw.drain().map(|i| i.to_string()).collect::<Vec<_>>();
                query.insert("bookmark", doc! { "$in": set });
            }
        }

        if let Some((g, l)) = bookmark_num {
            let mut bookmark_num_q = doc! {};

            match g {
                Bound::Unbounded => (),
                Bound::Included(n) => bookmark_num_q.insert("$gte", n).let_(::core::mem::drop),
                Bound::Excluded(n) => bookmark_num_q.insert("$gt", n).let_(::core::mem::drop),
            }

            match l {
                Bound::Unbounded => (),
                Bound::Included(n) => bookmark_num_q.insert("$lte", n).let_(::core::mem::drop),
                Bound::Excluded(n) => bookmark_num_q.insert("$lt", n).let_(::core::mem::drop),
            }

            if !bookmark_num_q.is_empty() {
                query.insert("bookmark_size", bookmark_num_q);
            }
        }

        query
    }
}
impl From<UserMutation> for Document {
    fn from(UserMutation { admin, sub_admin }: UserMutation) -> Self {
        let mut mutation = doc! {};

        if let Some(val) = admin {
            mutation.insert("admin", val);
        }

        if let Some(val) = sub_admin {
            mutation.insert("sub_admin", val);
        }

        mutation
    }
}

impl From<MongoUserModel> for User {
    fn from(
        MongoUserModel {
            id,
            admin,
            sub_admin,
            bookmark,
            bookmark_size: _,
        }: MongoUserModel,
    ) -> User {
        User {
            id: id.parse::<u64>().unwrap().into(),
            admin,
            sub_admin,
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
            bookmark,
        }: User,
    ) -> Self {
        MongoUserModel {
            id: id.to_string(),
            admin,
            sub_admin,
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
            created,
            mut edited,
        }: MongoContentModel,
    ) -> Self {
        Content {
            id,
            author: author.into(),
            posted: posted.into(),
            content,
            liked: liked
                .drain()
                .map(|s| s.parse::<u64>().unwrap().into())
                .collect(),
            pinned: pinned
                .drain()
                .map(|s| s.parse::<u64>().unwrap().into())
                .collect(),
            created: utils::parse_date(created.as_str()),
            edited: edited
                .drain(..)
                .map(|s| utils::parse_date(s.as_str()))
                .collect(),
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
            created,
            mut edited,
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
            created: utils::date_to_string(created),
            edited: edited.drain(..).map(utils::date_to_string).collect(),
        }
    }
}

impl From<MongoContentAuthorModel> for Author {
    fn from(m: MongoContentAuthorModel) -> Self {
        match m {
            MongoContentAuthorModel::User { id, name, nick } => Author::User {
                id: id.parse::<u64>().unwrap().into(),
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
            id: id.parse::<u64>().unwrap().into(),
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

impl From<UserId> for Bson {
    fn from(i: UserId) -> Self { Self::String(i.to_string()) }
}
impl From<ContentId> for Bson {
    fn from(i: ContentId) -> Self { Self::String(i.to_string()) }
}
