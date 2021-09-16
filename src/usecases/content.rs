usecase! {
    post : {
        pub content: String,
        pub posted: entities::Posted,
        pub author: entities::Author,
        pub created: entities::Date,
    } => {
        pub content: entities::Content,
    }
}

usecase! {
    get : {
        pub content_id: entities::ContentId,
    } => {
        pub content: entities::Content,
    }
}

usecase! {
    gets : {
        pub query: super::ContentQuery,
        pub page: u32,
    } => {
        pub contents: [(u32, entities::Content); 5],
        pub page: u32,
    }
}

usecase! {
    edit : {
        pub content_id: entities::ContentId,
        pub mutation: super::ContentMutation,
    } => {
        pub content: entities::Content,
    }
}

usecase! {
    withdraw : {
        pub content_id: entities::ContentId,
    } => {
        pub content: entities::Content,
    }
}

usecase! {
    get_like : {
        pub content_id: entities::ContentId,
        pub page: u32,
    } => {
        pub like: [(u32, entities::UserId); 20],
        pub page: u32,
    }
}

usecase! {
    like : {
        pub content_id: entities::ContentId,
        pub user_id: entities::UserId,
    } => {
        pub content: entities::Content
    }
}

usecase! {
    unlike : {
        pub content_id: entities::ContentId,
        pub user_id: entities::UserId,
    } => {
        pub content: entities::Content
    }
}

usecase! {
    get_pin : {
        pub content_id: entities::ContentId,
        pub page: u32,
    } => {
        pub pin: [(u32, entities::UserId); 20]
        pub page: u32,
    }
}

usecase! {
    pin : {
        pub content_id: entities::ContentId,
        pub user_id: entities::UserId,
    } => {
        pub content: entities::Content
    }
}

usecase! {
    unpin : {
        pub content_id: entities::ContentId,
        pub user_id: entities::UserId,
    } => {
        pub content: entities::Content
    }
}

use core::ops::Bound;
use std::collections::HashSet;

use regex::Regex;

use crate::entities::{Author, Date, UserId};

#[derive(Debug, Clone, Default)]
pub struct ContentQuery {
    pub author: Option<AuthorQuery>,
    pub posted: Option<PostedQuery>,
    pub content: Option<Regex>,
    pub liked: Option<HashSet<UserId>>,
    pub liked_num: Option<(Bound<u32>, Bound<u32>)>,
    pub pinned: Option<HashSet<UserId>>,
    pub pinned_num: Option<(Bound<u32>, Bound<u32>)>,
    // FiF: times query
}

#[derive(Debug, Clone)]
pub enum AuthorQuery {
    UserId(UserId),
    UserName(Regex),
    UserNick(Regex),
    Virtual(Regex),
    Any(Regex),
}

#[derive(Debug, Clone)]
pub enum PostedQuery {
    UserId(UserId),
    UserName(Regex),
    UserNick(Regex),
    Any(Regex),
}

#[derive(Debug, Clone)]
pub struct ContentMutation {
    pub author: Option<Author>,
    pub content: Option<ContentContentMutation>,
    pub edited: Date,
}

#[derive(Debug, Clone)]
pub enum ContentContentMutation {
    Complete(String),
    Sed { capture: Regex, replace: String },
}
