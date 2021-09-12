usecase! {
    register : {
        user_id: entities::UserId,
    } => {
        user: entities::User,
    }
}

usecase! {
    get : {
        user_id: entities::UserId,
    } => {
        user: entities::User,
    }
}

// FIXME: move query and mutation to entities

usecase! {
    gets : {
        query: crate::repositories::UserQuery,
    } => {
        users: Vec<entities::User>,
    }
}

usecase! {
    edit : {
        user_id: entities::UserId,
        mutation: crate::repositories::UserMutation,
    } => {
        user: entities::User,
    }
}

usecase! {
    unregister : {
        user_id: entities::UserId,
    } => {
        user: entities::User,
    }
}

usecase! {
    get_bookmark : {
        user_id: entities::UserId,
    } => {
        bookmark: std::collections::HashSet<entities::ContentId>,
    }
}

usecase! {
    bookmark : {
        user_id: entities::UserId,
        content_id: entities::ContentId,
    } => {
        user: entities::User,
    }
}

usecase! {
    unbookmark : {
        user_id: entities::UserId,
        content_id: entities::ContentId,
    } => {
        user: entities::User,
    }
}
