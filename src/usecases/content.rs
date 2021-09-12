usecase! {
    post : {
        content: String,
        posted: entities::Posted,
        author: entities::Author,
        created: entities::Date,
    } => {
        content: entities::Content,
    }
}

usecase! {
    get : {
        content_id: entities::ContentId,
    } => {
        content: entities::Content,
    }
}

usecase! {
    gets : {
        query: crate::repositories::ContentQuery,
    } => {
        contents: Vec<entities::Content>,
    }
}

usecase! {
    edit : {
        content_id: entities::ContentId,
        mutation: crate::repositories::ContentMutation,
    } => {
        content: entities::Content,
    }
}

usecase! {
    withdraw : {
        content_id: entities::ContentId,
    } => {
        content: entities::Content,
    }
}

usecase! {
    get_like : {
        content_id: entities::ContentId,
    } => {
        like: std::collections::HashSet<entities::UserId>,
    }
}

usecase! {
    like : {
        content_id: entities::ContentId,
        user_id: entities::UserId,
    } => {
        content: entities::Content
    }
}

usecase! {
    unlike : {
        content_id: entities::ContentId,
        user_id: entities::UserId,
    } => {
        content: entities::Content
    }
}

usecase! {
    get_pin : {
        content_id: entities::ContentId,
    } => {
        like: std::collections::HashSet<entities::UserId>,
    }
}

usecase! {
    pin : {
        content_id: entities::ContentId,
        user_id: entities::UserId,
    } => {
        content: entities::Content
    }
}

usecase! {
    unpin : {
        content_id: entities::ContentId,
        user_id: entities::UserId,
    } => {
        content: entities::Content
    }
}
