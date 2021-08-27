#![allow(non_upper_case_globals)]

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

consts::consts! {
    NAME: "icey_pudding";
    PREFIX: "*ip";
    ABOUT: "this is a ICEy_PUDDING.";

    user: "user";
    user {
        DESC: "about users.";

        create: "create";

        read: "read";

        update: "update";
        update {
            DESC: "owner can only use.";

            id: "id";

            admin: "admin";
            admin {
                DESC: "`admin` privileges.";
            }

            sub_admin: "sub_admin";
            sub_admin {
                DESC: "`sub_admin` privileges.";
            }
        }

        delete: "delete";
    }

    content: "content";
    content {
        DESC: "about contents.";

        read: "read";
        read {
            id: "id";
        }

        reads: "reads";
        reads {
            DESC: "queries contents. see `post` for details.";

            author_id: "author-id";
            author: "author";
            content: "content";
        }

        update: "update";
        update {
            DESC: "see `post` for details.";

            id: "id";
            author_id: "author-id";
            author: "author";
            content: "content";
        }

        delete: "delete";
        delete {
            id: "id";
        }
    }

    post_v2: "post";
    post_v2 {
        DESC: "post content.";

        author_id: "author-id";
        author_id {
            DESC: "author user id. (u64)";
        }

        author: "author";
        author {
            DESC: "author name. (str)";
        }

        content: "content";
        content {
            DESC: "content's content.";
        }
    }

    like_v2: "like";
    like_v2 {
        DESC: "do thing like twitter's fav.";

        id: "id";
        undo: "undo";
        undo {
            DESC: "is `unlike`?";
        }
    }

    pin_v2: "pin";
    pin_v2 {
        DESC: "imitates twitter's RT.";

        id: "id";
        undo {
            DESC: "is `unpin`?";
        }
    }

    bookmark_v2: "bookmark";
    bookmark_v2 {
        DESC: "bookmark content.";

        id: "id";
        undo {
            DESC: "is `unbookmark`?";
        }
    }



    register {
        NAME: "register";
        DESC: "register user.";
    }

    info {
        NAME: "info";
        DESC: "get your user data.";
    }


    change {
        NAME: "change";
        DESC: "change your user data.";
        admin {
            NAME: "admin";
            DESC: "set bot's admin.";
        }
        sub_admin {
            NAME: "sub_admin";
            DESC: "set bot's sub_admin.";
        }
    }

    bookmark {
        NAME: "bookmark";
        DESC: "bookmark content.";
        id {
            NAME: "id";
            DESC: "content's id.";
        }
        undo {
            NAME: "undo";
            DESC: "is `un-bookmark`?";
        }
    }

    delete_me {
        NAME: "delete_me";
        DESC: "delete user.";
    }

    post {
        NAME: "post";
        DESC: "post content.";
        author {
            NAME: "author";
            DESC: "who said conntent.";
        }
        content {
            NAME: "content";
            DESC: "content's content.";
        }
    }

    get {
        NAME: "get";
        DESC: "get content.";
        page {
            NAME: "page";
            DESC: "showing page num.";
        }
        id {
            NAME: "id";
            DESC: "content's id.";
        }
        author {
            NAME: "author";
            DESC: "author name.";
        }
        posted {
            NAME: "posted";
            DESC: "posted id.";
        }
        content {
            NAME: "content";
            DESC: "content.";
        }
        liked {
            NAME: "liked";
            DESC: "liked num.";
        }
        bookmarked {
            NAME: "bookmarked";
            DESC: "bookmarked num.";
        }
        pinned {
            NAME: "pinned";
            DESC: "pinned num.";
        }
    }

    edit {
        NAME: "edit";
        DESC: "edit content.";
        id {
            NAME: "id";
            DESC: "content's id.";
        }
        content {
            NAME: "content";
            DESC: "replace content.";
        }
    }

    like {
        NAME: "like";
        DESC: "like content.";
        id {
            NAME: "id";
            DESC: "content's id.";
        }
        undo {
            NAME: "undo";
            DESC: "is `un-like`?";
        }
    }

    pin {
        NAME: "pin";
        DESC: "pin content.";
        id {
            NAME: "id";
            DESC: "content's id.";
        }
        undo {
            NAME: "undo";
            DESC: "is `un-pin`?";
        }
    }

    remove {
        NAME: "remove";
        DESC: "remove content.";
        id {
            NAME: "id";
            DESC: "content's id.";
        }
    }
}
