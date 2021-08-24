pub const VERSION: &str = env!("CARGO_PKG_VERSION");

consts::consts! {
    NAME: "icey_pudding";
    PREFIX: "*ip";
    ABOUT: "this is a ICEy_PUDDING.";

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
