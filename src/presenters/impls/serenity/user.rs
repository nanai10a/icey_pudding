use anyhow::Result;
use async_trait::async_trait;
use smallvec::SmallVec;
use tokio::sync::mpsc;

use super::super::super::user;
use super::{View, EMPTY_FIELD};
use crate::entities::User;
use crate::usecases::user::{
    bookmark, edit, get, get_bookmark, gets, register, unbookmark, unregister,
};

pub struct SerenityUserRegisterPresenter {
    pub out: mpsc::Sender<Box<View>>,
}
#[async_trait]
impl user::UserRegisterPresenter for SerenityUserRegisterPresenter {
    async fn complete(
        &self,
        register::Output {
            user:
                User {
                    id,
                    admin: _,
                    sub_admin: _,
                    bookmark: _,
                },
        }: register::Output,
    ) -> Result<()> {
        const COLOR: (u8, u8, u8) = (0xd5, 0xc4, 0xa1);

        self.out
            .send(box move |ce| ce.title("registered user").color(COLOR).description(id))
            .await
            .map_err(|e| e.to_string())
            .unwrap();
        Ok(())
    }
}

pub struct SerenityUserGetPresenter {
    pub out: mpsc::Sender<Box<View>>,
}
#[async_trait]
impl user::UserGetPresenter for SerenityUserGetPresenter {
    async fn complete(
        &self,
        get::Output {
            user:
                User {
                    id,
                    admin,
                    sub_admin,
                    bookmark,
                },
        }: get::Output,
    ) -> Result<()> {
        const COLOR: (u8, u8, u8) = (0x83, 0xa5, 0x98);

        self.out
            .send(box move |ce| {
                ce.title("showing user")
                    .color(COLOR)
                    .description(id)
                    .fields([
                        ("admin", admin.to_string(), true),
                        ("sub_admin", sub_admin.to_string(), true),
                        (EMPTY_FIELD.0, EMPTY_FIELD.1.into(), EMPTY_FIELD.2),
                        ("bookmark", bookmark.len().to_string(), true),
                    ])
            })
            .await
            .map_err(|e| e.to_string())
            .unwrap();
        Ok(())
    }
}

pub struct SerenityUserGetsPresenter {
    pub out: mpsc::Sender<SmallVec<[Box<View>; 5]>>,
}
#[async_trait]
impl user::UserGetsPresenter for SerenityUserGetsPresenter {
    async fn complete(&self, gets::Output { mut users, page }: gets::Output) -> Result<()> {
        const COLOR: (u8, u8, u8) = (0x83, 0xa5, 0x98);

        self.out
            .send(
                users
                    .drain(..)
                    .map::<Box<View>, _>(
                        |(
                            idx,
                            User {
                                id,
                                admin,
                                sub_admin,
                                bookmark,
                            },
                        )| {
                            box move |ce| {
                                ce.title("showing users")
                                    .color(COLOR)
                                    .description(format!("{} in {} | {}", idx, page, id))
                                    .fields([
                                        ("admin", admin.to_string(), true),
                                        ("sub_admin", sub_admin.to_string(), true),
                                        (EMPTY_FIELD.0, EMPTY_FIELD.1.into(), EMPTY_FIELD.2),
                                        ("bookmark", bookmark.len().to_string(), true),
                                    ])
                            }
                        },
                    )
                    .collect(),
            )
            .await
            .map_err(|e| e.to_string())
            .unwrap();

        Ok(())
    }
}

pub struct SerenityUserEditPresenter {
    pub out: mpsc::Sender<Box<View>>,
}
#[async_trait]
impl user::UserEditPresenter for SerenityUserEditPresenter {
    async fn complete(
        &self,
        edit::Output {
            user:
                User {
                    id,
                    admin,
                    sub_admin,
                    bookmark,
                },
        }: edit::Output,
    ) -> Result<()> {
        const COLOR: (u8, u8, u8) = (0xb8, 0xb2, 0x26);

        self.out
            .send(box move |ce| {
                ce.title("updated user")
                    .color(COLOR)
                    .description(id)
                    .fields([
                        ("admin", admin.to_string(), true),
                        ("sub_admin", sub_admin.to_string(), true),
                        (EMPTY_FIELD.0, EMPTY_FIELD.1.into(), EMPTY_FIELD.2),
                        ("bookmark", bookmark.len().to_string(), true),
                    ])
            })
            .await
            .map_err(|e| e.to_string())
            .unwrap();
        Ok(())
    }
}

pub struct SerenityUserUnregisterPresenter {
    pub out: mpsc::Sender<Box<View>>,
}
#[async_trait]
impl user::UserUnregisterPresenter for SerenityUserUnregisterPresenter {
    async fn complete(
        &self,
        unregister::Output {
            user:
                User {
                    id,
                    admin,
                    sub_admin,
                    mut bookmark,
                },
        }: unregister::Output,
    ) -> Result<()> {
        const COLOR: (u8, u8, u8) = (0x1d, 0x20, 0x21);

        self.out
            .send(box move |ce| {
                ce.title("deleted user")
                    .color(COLOR)
                    .description(id)
                    .fields([
                        ("admin", admin.to_string(), true),
                        ("sub_admin", sub_admin.to_string(), true),
                        (EMPTY_FIELD.0, EMPTY_FIELD.1.into(), EMPTY_FIELD.2),
                        ("bookmark", bookmark.len().to_string(), false),
                        (
                            "bookmark",
                            bookmark
                                .drain()
                                .map(|i| i.to_string())
                                .collect::<Vec<_>>()
                                .join(", "),
                            true,
                        ),
                    ])
            })
            .await
            .map_err(|e| e.to_string())
            .unwrap();
        Ok(())
    }
}

pub struct SerenityUserBookmarkGetPresenter {
    pub out: mpsc::Sender<SmallVec<[Box<View>; 20]>>,
}
#[async_trait]
impl user::UserBookmarkGetPresenter for SerenityUserBookmarkGetPresenter {
    async fn complete(
        &self,
        get_bookmark::Output { mut bookmark, page }: get_bookmark::Output,
    ) -> Result<()> {
        const COLOR: (u8, u8, u8) = (0x83, 0xa5, 0x98);

        self.out
            .send(
                bookmark
                    .drain(..)
                    .map::<Box<View>, _>(|(idx, id)| {
                        box move |ce| {
                            ce.title("showing bookmark")
                                .color(COLOR)
                                .description(format!("{} in {}", idx, page))
                                .fields([("id", id, true)])
                        }
                    })
                    .collect(),
            )
            .await
            .map_err(|e| e.to_string())
            .unwrap();
        Ok(())
    }
}

pub struct SerenityUserBookmarkPresenter {
    pub out: mpsc::Sender<Box<View>>,
}
#[async_trait]
impl user::UserBookmarkPresenter for SerenityUserBookmarkPresenter {
    async fn complete(
        &self,
        bookmark::Output {
            user:
                User {
                    id: user_id,
                    admin: _,
                    sub_admin: _,
                    bookmark,
                },
            id,
        }: bookmark::Output,
    ) -> Result<()> {
        const COLOR: (u8, u8, u8) = (0x83, 0xa5, 0x98);

        self.out
            .send(box move |ce| {
                ce.title("bookmarked")
                    .color(COLOR)
                    .description(format!("{} => {}", user_id, id))
                    .fields([("bookmark", bookmark.len(), true)])
            })
            .await
            .map_err(|e| e.to_string())
            .unwrap();
        Ok(())
    }
}

pub struct SerenityUserUnbookmarkPresenter {
    pub out: mpsc::Sender<Box<View>>,
}
#[async_trait]
impl user::UserUnbookmarkPresenter for SerenityUserUnbookmarkPresenter {
    async fn complete(
        &self,
        unbookmark::Output {
            user:
                User {
                    id: user_id,
                    admin: _,
                    sub_admin: _,
                    bookmark,
                },
            id,
        }: unbookmark::Output,
    ) -> Result<()> {
        const COLOR: (u8, u8, u8) = (0x83, 0xa5, 0x98);

        self.out
            .send(box move |ce| {
                ce.title("unbookmarked")
                    .color(COLOR)
                    .description(format!("{} =/> {}", user_id, id))
                    .fields([("bookmark", bookmark.len(), true)])
            })
            .await
            .map_err(|e| e.to_string())
            .unwrap();

        Ok(())
    }
}
