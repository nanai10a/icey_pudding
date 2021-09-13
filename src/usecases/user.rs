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

// FIXME: move query and mutation to entities

usecase! {
    gets : {
        pub query: crate::repositories::UserQuery,
    } => {
        pub users: Vec<entities::User>,
    }
}

usecase! {
    edit : {
        pub user_id: entities::UserId,
        pub mutation: crate::repositories::UserMutation,
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
