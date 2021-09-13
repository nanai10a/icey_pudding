usecase! {
    register : {
        pub user_id: entities::UserId,
    } => {
        pub user: entities::User,
    }
}

usecase! {
    get : {
        pub user_id: entities::UserId,
    } => {
        pub user: entities::User,
    }
}

usecase! {
    gets : {
        pub query: super::UserQuery,
    } => {
        pub users: Vec<entities::User>,
    }
}

usecase! {
    edit : {
        pub user_id: entities::UserId,
        pub mutation: super::UserMutation,
    } => {
        pub user: entities::User,
    }
}

usecase! {
    unregister : {
        pub user_id: entities::UserId,
    } => {
        pub user: entities::User,
    }
}

usecase! {
    get_bookmark : {
        pub user_id: entities::UserId,
    } => {
        pub bookmark: std::collections::HashSet<entities::ContentId>,
    }
}

usecase! {
    bookmark : {
        pub user_id: entities::UserId,
        pub content_id: entities::ContentId,
    } => {
        pub user: entities::User,
    }
}

usecase! {
    unbookmark : {
        pub user_id: entities::UserId,
        pub content_id: entities::ContentId,
    } => {
        pub user: entities::User,
    }
}

use core::ops::Bound;
use std::collections::HashSet;

use crate::entities::ContentId;

#[derive(Debug, Clone, Default)]
pub struct UserQuery {
    pub bookmark: Option<HashSet<ContentId>>,
    pub bookmark_num: Option<(Bound<u32>, Bound<u32>)>,
}

#[derive(Debug, Clone, Default)]
pub struct UserMutation {
    pub admin: Option<bool>,
    pub sub_admin: Option<bool>,
}
