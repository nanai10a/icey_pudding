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
        pub query: crate::repositories::ContentQuery,
    } => {
        pub contents: Vec<entities::Content>,
    }
}

usecase! {
    edit : {
        pub content_id: entities::ContentId,
        pub mutation: crate::repositories::ContentMutation,
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
    } => {
        pub like: std::collections::HashSet<entities::UserId>,
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
    } => {
        pub like: std::collections::HashSet<entities::UserId>,
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
