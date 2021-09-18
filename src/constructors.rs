use alloc::sync::Arc;

use serenity::client::EventHandler;
use tokio::sync::{mpsc, Mutex};

use crate::conductors::Conductor;
use crate::controllers::ret::content::ReturnContentController;
use crate::controllers::ret::user::ReturnUserController;
use crate::controllers::serenity::content::SerenityContentController;
use crate::controllers::serenity::user::SerenityUserController;
use crate::controllers::serenity::SerenityReturnController;
use crate::entities::*;
use crate::interactors::content::*;
use crate::interactors::user::*;
use crate::presenters::impls::ret::content::ReturnContentGetPresenter;
use crate::presenters::impls::ret::user::ReturnUserGetPresenter;
use crate::presenters::impls::serenity::content::*;
use crate::presenters::impls::serenity::user::*;
use crate::repositories::*;

fn contr(
    user_contr: SerenityUserController,
    content_contr: SerenityContentController,
    user_repo: Arc<dyn UserRepository + Sync + Send>,
    content_repo: Arc<dyn ContentRepository + Sync + Send>,
) -> SerenityReturnController {
    let (user_in, user_out) = mpsc::channel(1);
    let (content_in, content_out) = mpsc::channel(1);

    SerenityReturnController {
        user: user_contr,
        content: content_contr,
        return_user_contr: ReturnUserController {
            usecase: Arc::new(UserGetInteractor {
                user_repository: user_repo.clone(),
                pres: Arc::new(ReturnUserGetPresenter { ret: user_in }),
            }),
            lock: Mutex::new(()),
            ret: Mutex::new(user_out),
        },
        return_content_contr: ReturnContentController {
            usecase: Arc::new(ContentGetInteractor {
                content_repository: content_repo.clone(),
                pres: Arc::new(ReturnContentGetPresenter { ret: content_in }),
            }),
            lock: Mutex::new(()),
            ret: Mutex::new(content_out),
        },
    }
}

fn user(repo: Arc<dyn UserRepository + Sync + Send>) -> SerenityUserController {
    let (register_in, register_out) = mpsc::channel(1);
    let (get_in, get_out) = mpsc::channel(1);
    let (gets_in, gets_out) = mpsc::channel(1);
    let (edit_in, edit_out) = mpsc::channel(1);
    let (unregister_in, unregister_out) = mpsc::channel(1);
    let (get_bookmark_in, get_bookmark_out) = mpsc::channel(1);
    let (bookmark_in, bookmark_out) = mpsc::channel(1);
    let (unbookmark_in, unbookmark_out) = mpsc::channel(1);

    SerenityUserController {
        register: Arc::new(UserRegisterInteractor {
            user_repository: repo.clone(),
            pres: Arc::new(SerenityUserRegisterPresenter { out: register_in }),
        }),
        register_ret: Mutex::new(register_out),
        register_lock: Mutex::new(()),

        get: Arc::new(UserGetInteractor {
            user_repository: repo.clone(),
            pres: Arc::new(SerenityUserGetPresenter { out: get_in }),
        }),
        get_ret: Mutex::new(get_out),
        get_lock: Mutex::new(()),

        gets: Arc::new(UserGetsInteractor {
            user_repository: repo.clone(),
            pres: Arc::new(SerenityUserGetsPresenter { out: gets_in }),
        }),
        gets_ret: Mutex::new(gets_out),
        gets_lock: Mutex::new(()),

        edit: Arc::new(UserEditInteractor {
            user_repository: repo.clone(),
            pres: Arc::new(SerenityUserEditPresenter { out: edit_in }),
        }),
        edit_ret: Mutex::new(edit_out),
        edit_lock: Mutex::new(()),

        unregister: Arc::new(UserUnregisterInteractor {
            user_repository: repo.clone(),
            pres: Arc::new(SerenityUserUnregisterPresenter { out: unregister_in }),
        }),
        unregister_ret: Mutex::new(unregister_out),
        unregister_lock: Mutex::new(()),

        get_bookmark: Arc::new(UserBookmarkGetInteractor {
            user_repository: repo.clone(),
            pres: Arc::new(SerenityUserBookmarkGetPresenter {
                out: get_bookmark_in,
            }),
        }),
        get_bookmark_ret: Mutex::new(get_bookmark_out),
        get_bookmark_lock: Mutex::new(()),

        bookmark: Arc::new(UserBookmarkInteractor {
            user_repository: repo.clone(),
            pres: Arc::new(SerenityUserBookmarkPresenter { out: bookmark_in }),
        }),
        bookmark_ret: Mutex::new(bookmark_out),
        bookmark_lock: Mutex::new(()),

        unbookmark: Arc::new(UserUnbookmarkInteractor {
            user_repository: repo.clone(),
            pres: Arc::new(SerenityUserUnbookmarkPresenter { out: unbookmark_in }),
        }),
        unbookmark_ret: Mutex::new(unbookmark_out),
        unbookmark_lock: Mutex::new(()),
    }
}

fn content(
    repo: Arc<dyn ContentRepository + Sync + Send>,
    user_repo: Arc<dyn UserRepository + Sync + Send>,
) -> SerenityContentController {
    let (post_in, post_out) = mpsc::channel(1);
    let (get_in, get_out) = mpsc::channel(1);
    let (gets_in, gets_out) = mpsc::channel(1);
    let (edit_in, edit_out) = mpsc::channel(1);
    let (withdraw_in, withdraw_out) = mpsc::channel(1);
    let (get_like_in, get_like_out) = mpsc::channel(1);
    let (like_in, like_out) = mpsc::channel(1);
    let (unlike_in, unlike_out) = mpsc::channel(1);
    let (get_pin_in, get_pin_out) = mpsc::channel(1);
    let (pin_in, pin_out) = mpsc::channel(1);
    let (unpin_in, unpin_out) = mpsc::channel(1);

    SerenityContentController {
        post: Arc::new(ContentPostInteractor {
            user_repository: user_repo.clone(),
            content_repository: repo.clone(),
            pres: Arc::new(SerenityContentPostPresenter { out: post_in }),
        }),
        post_ret: Mutex::new(post_out),
        post_lock: Mutex::new(()),

        get: Arc::new(ContentGetInteractor {
            content_repository: repo.clone(),
            pres: Arc::new(SerenityContentGetPresenter { out: get_in }),
        }),
        get_ret: Mutex::new(get_out),
        get_lock: Mutex::new(()),

        gets: Arc::new(ContentGetsInteractor {
            content_repository: repo.clone(),
            pres: Arc::new(SerenityContentGetsPresenter { out: gets_in }),
        }),
        gets_ret: Mutex::new(gets_out),
        gets_lock: Mutex::new(()),

        edit: Arc::new(ContentEditInteractor {
            content_repository: repo.clone(),
            pres: Arc::new(SerenityContentEditPresenter { out: edit_in }),
        }),
        edit_ret: Mutex::new(edit_out),
        edit_lock: Mutex::new(()),

        withdraw: Arc::new(ContentWithdrawInteractor {
            content_repository: repo.clone(),
            pres: Arc::new(SerenityContentWithdrawPresenter { out: withdraw_in }),
        }),
        withdraw_ret: Mutex::new(withdraw_out),
        withdraw_lock: Mutex::new(()),

        get_like: Arc::new(ContentLikeGetInteractor {
            content_repository: repo.clone(),
            pres: Arc::new(SerenityContentLikeGetPresenter { out: get_like_in }),
        }),
        get_like_ret: Mutex::new(get_like_out),
        get_like_lock: Mutex::new(()),

        like: Arc::new(ContentLikeInteractor {
            content_repository: repo.clone(),
            pres: Arc::new(SerenityContentLikePresenter { out: like_in }),
        }),
        like_ret: Mutex::new(like_out),
        like_lock: Mutex::new(()),

        unlike: Arc::new(ContentUnlikeInteractor {
            content_repository: repo.clone(),
            pres: Arc::new(SerenityContentUnlikePresenter { out: unlike_in }),
        }),
        unlike_ret: Mutex::new(unlike_out),
        unlike_lock: Mutex::new(()),

        get_pin: Arc::new(ContentPinGetInteractor {
            content_repository: repo.clone(),
            pres: Arc::new(SerenityContentPinGetPresenter { out: get_pin_in }),
        }),
        get_pin_ret: Mutex::new(get_pin_out),
        get_pin_lock: Mutex::new(()),

        pin: Arc::new(ContentPinInteractor {
            content_repository: repo.clone(),
            pres: Arc::new(SerenityContentPinPresenter { out: pin_in }),
        }),
        pin_ret: Mutex::new(pin_out),
        pin_lock: Mutex::new(()),

        unpin: Arc::new(ContentUnpinInteractor {
            content_repository: repo.clone(),
            pres: Arc::new(SerenityContentUnpinPresenter { out: unpin_in }),
        }),
        unpin_ret: Mutex::new(unpin_out),
        unpin_lock: Mutex::new(()),
    }
}

pub fn in_memory() -> impl EventHandler {
    let ur = Arc::new(InMemoryRepository::<User>::new());
    let cr = Arc::new(InMemoryRepository::<Content>::new());

    Conductor {
        contr: contr(user(ur.clone()), content(cr.clone(), ur.clone()), ur, cr),
    }
}

pub async fn mongo(
    uri_str: impl AsRef<str>,
    db_name: impl AsRef<str>,
) -> ::anyhow::Result<impl EventHandler> {
    let c = ::mongodb::Client::with_uri_str(uri_str).await?;
    let db = c.database(db_name.as_ref());

    let ur = Arc::new(MongoUserRepository::new_with(c.clone(), db.clone()).await?);
    let cr = Arc::new(MongoContentRepository::new_with(c, db).await?);

    let eh = Conductor {
        contr: contr(user(ur.clone()), content(cr.clone(), ur.clone()), ur, cr),
    };

    Ok(eh)
}
