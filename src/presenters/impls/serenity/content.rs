use anyhow::Result;
use async_trait::async_trait;
use smallvec::SmallVec;
use tokio::sync::mpsc;

use super::super::super::content;
use super::View;
use crate::entities::Content;
use crate::usecases::content::{
    edit, get, get_like, get_pin, gets, like, pin, post, unlike, unpin, withdraw,
};
use crate::utils::date_to_string;

pub struct SerenityContentPostPresenter {
    pub out: mpsc::Sender<Box<View>>,
}
#[async_trait]
impl content::ContentPostPresenter for SerenityContentPostPresenter {
    async fn complete(
        &self,
        post::Output {
            content:
                Content {
                    id,
                    author,
                    posted,
                    content,
                    liked: _,
                    pinned: _,
                    created,
                    edited: _,
                },
        }: post::Output,
    ) -> Result<()> {
        const COLOR: (u8, u8, u8) = (0xfb, 0xf1, 0xc7);

        self.out
            .send(box move |ce| {
                ce.title("posted content")
                    .colour(COLOR)
                    .description(id)
                    .fields([
                        ("author", author.to_string(), true),
                        ("posted", posted.to_string(), true),
                        ("created", created.to_string(), true),
                        ("content", content, true),
                    ])
            })
            .await
            .map_err(|e| e.to_string())
            .unwrap();

        Ok(())
    }
}

pub struct SerenityContentGetPresenter {
    pub out: mpsc::Sender<Box<View>>,
}
#[async_trait]
impl content::ContentGetPresenter for SerenityContentGetPresenter {
    async fn complete(
        &self,
        get::Output {
            content:
                Content {
                    id,
                    author,
                    posted,
                    content,
                    liked,
                    pinned,
                    created,
                    mut edited,
                },
        }: get::Output,
    ) -> Result<()> {
        const COLOR: (u8, u8, u8) = (0xfa, 0xdb, 0x2f);

        self.out
            .send(box move |ce| {
                ce.title("showing content")
                    .colour(COLOR)
                    .description(id)
                    .fields([
                        ("author", author.to_string(), true),
                        ("posted", posted.to_string(), true),
                        ("created", created.to_string(), true),
                        ("edited_times", edited.len().to_string(), true),
                        (
                            "last_edited",
                            edited
                                .pop()
                                .map(date_to_string)
                                .unwrap_or_else(|| "None".to_string()),
                            true,
                        ),
                        ("like", liked.len().to_string(), true),
                        ("pin", pinned.len().to_string(), true),
                        ("content", content, true),
                    ])
            })
            .await
            .map_err(|e| e.to_string())
            .unwrap();

        Ok(())
    }
}

pub struct SerenityContentGetsPresenter {
    pub out: mpsc::Sender<SmallVec<[Box<View>; 5]>>,
}
#[async_trait]
impl content::ContentGetsPresenter for SerenityContentGetsPresenter {
    async fn complete(&self, gets::Output { mut contents, page }: gets::Output) -> Result<()> {
        const COLOR: (u8, u8, u8) = (0xfa, 0xdb, 0x2f);

        self.out
            .send(
                contents
                    .drain(..)
                    .map::<Box<View>, _>(
                        |(
                            idx,
                            Content {
                                id,
                                author,
                                posted,
                                content,
                                liked,
                                pinned,
                                created,
                                mut edited,
                            },
                        )| {
                            box move |ce| {
                                ce.title("showing contents.")
                                    .colour(COLOR)
                                    .description(format!("{} in {} | {}", idx, page, id))
                                    .fields([
                                        ("author", author.to_string(), true),
                                        ("posted", posted.to_string(), true),
                                        ("created", created.to_string(), true),
                                        ("edited_times", edited.len().to_string(), true),
                                        (
                                            "last_edited",
                                            edited
                                                .pop()
                                                .map(date_to_string)
                                                .unwrap_or_else(|| "None".to_string()),
                                            true,
                                        ),
                                        ("like", liked.len().to_string(), true),
                                        ("pin", pinned.len().to_string(), true),
                                        ("content", content, true),
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

pub struct SerenityContentEditPresenter {
    pub out: mpsc::Sender<Box<View>>,
}
#[async_trait]
impl content::ContentEditPresenter for SerenityContentEditPresenter {
    async fn complete(
        &self,
        edit::Output {
            content:
                Content {
                    id,
                    author,
                    posted,
                    content,
                    liked,
                    pinned,
                    created,
                    mut edited,
                },
        }: edit::Output,
    ) -> Result<()> {
        const COLOR: (u8, u8, u8) = (0x8e, 0xc0, 0x7c);

        self.out
            .send(box move |ce| {
                ce.title("updated content.")
                    .colour(COLOR)
                    .description(id)
                    .fields([
                        ("author", author.to_string(), true),
                        ("posted", posted.to_string(), true),
                        ("created", created.to_string(), true),
                        ("edited_times", edited.len().to_string(), true),
                        (
                            "last_edited",
                            edited
                                .pop()
                                .map(date_to_string)
                                .unwrap_or_else(|| "None".to_string()),
                            true,
                        ),
                        ("like", liked.len().to_string(), true),
                        ("pin", pinned.len().to_string(), true),
                        ("content", content, true),
                    ])
            })
            .await
            .map_err(|e| e.to_string())
            .unwrap();

        Ok(())
    }
}

pub struct SerenityContentWithdrawPresenter {
    pub out: mpsc::Sender<Box<View>>,
}
#[async_trait]
impl content::ContentWithdrawPresenter for SerenityContentWithdrawPresenter {
    async fn complete(
        &self,
        withdraw::Output {
            content:
                Content {
                    id,
                    author,
                    posted,
                    content,
                    mut liked,
                    mut pinned,
                    created,
                    mut edited,
                },
        }: withdraw::Output,
    ) -> Result<()> {
        const COLOR: (u8, u8, u8) = (0x66, 0x5c, 0x54);

        self.out
            .send(box move |ce| {
                ce.title("deleted content.")
                    .colour(COLOR)
                    .description(id)
                    .fields([
                        ("author", author.to_string(), true),
                        ("posted", posted.to_string(), true),
                        ("created", created.to_string(), true),
                        ("edited_times", edited.len().to_string(), true),
                        (
                            "edit_history",
                            edited
                                .drain(..)
                                .map(date_to_string)
                                .collect::<Vec<_>>()
                                .join(", "),
                            true,
                        ),
                        ("like_times", liked.len().to_string(), true),
                        (
                            "liked",
                            liked
                                .drain()
                                .map(|i| i.to_string())
                                .collect::<Vec<_>>()
                                .join(", "),
                            true,
                        ),
                        ("pinned_times", pinned.len().to_string(), true),
                        (
                            "pinned",
                            pinned
                                .drain()
                                .map(|i| i.to_string())
                                .collect::<Vec<_>>()
                                .join(", "),
                            true,
                        ),
                        ("content", content, true),
                    ])
            })
            .await
            .map_err(|e| e.to_string())
            .unwrap();

        Ok(())
    }
}

pub struct SerenityContentLikeGetPresenter {
    pub out: mpsc::Sender<SmallVec<[Box<View>; 20]>>,
}
#[async_trait]
impl content::ContentLikeGetPresenter for SerenityContentLikeGetPresenter {
    async fn complete(&self, get_like::Output { mut like, page }: get_like::Output) -> Result<()> {
        const COLOR: (u8, u8, u8) = (0xd3, 0x86, 0x9b);

        self.out
            .send(
                like.drain(..)
                    .map::<Box<View>, _>(|(idx, id)| {
                        box move |ce| {
                            ce.title("showing like")
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

pub struct SerenityContentLikePresenter {
    pub out: mpsc::Sender<Box<View>>,
}
#[async_trait]
impl content::ContentLikePresenter for SerenityContentLikePresenter {
    async fn complete(
        &self,
        like::Output {
            content:
                Content {
                    id: content_id,
                    author: _,
                    posted: _,
                    content: _,
                    liked,
                    pinned: _,
                    created: _,
                    edited: _,
                },
            id,
        }: like::Output,
    ) -> Result<()> {
        const COLOR: (u8, u8, u8) = (0xd3, 0x86, 0x9b);

        self.out
            .send(box move |ce| {
                ce.title("like")
                    .colour(COLOR)
                    .description(format!("{} => {}", id, content_id))
                    .fields([("like", liked.len(), true)])
            })
            .await
            .map_err(|e| e.to_string())
            .unwrap();

        Ok(())
    }
}

pub struct SerenityContentUnlikePresenter {
    pub out: mpsc::Sender<Box<View>>,
}
#[async_trait]
impl content::ContentUnlikePresenter for SerenityContentUnlikePresenter {
    async fn complete(
        &self,
        unlike::Output {
            content:
                Content {
                    id: content_id,
                    author: _,
                    posted: _,
                    content: _,
                    liked,
                    pinned: _,
                    created: _,
                    edited: _,
                },
            id,
        }: unlike::Output,
    ) -> Result<()> {
        const COLOR: (u8, u8, u8) = (0xd3, 0x86, 0x9b);

        self.out
            .send(box move |ce| {
                ce.title("unlike")
                    .colour(COLOR)
                    .description(format!("{} =/> {}", id, content_id))
                    .fields([("like", liked.len(), true)])
            })
            .await
            .map_err(|e| e.to_string())
            .unwrap();

        Ok(())
    }
}

pub struct SerenityContentPinGetPresenter {
    pub out: mpsc::Sender<SmallVec<[Box<View>; 20]>>,
}
#[async_trait]
impl content::ContentPinGetPresenter for SerenityContentPinGetPresenter {
    async fn complete(&self, get_pin::Output { mut pin, page }: get_pin::Output) -> Result<()> {
        const COLOR: (u8, u8, u8) = (0xfb, 0x49, 0x34);

        self.out
            .send(
                pin.drain(..)
                    .map::<Box<View>, _>(|(idx, id)| {
                        box move |ce| {
                            ce.title("showing pin")
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

pub struct SerenityContentPinPresenter {
    pub out: mpsc::Sender<Box<View>>,
}
#[async_trait]
impl content::ContentPinPresenter for SerenityContentPinPresenter {
    async fn complete(
        &self,
        pin::Output {
            content:
                Content {
                    id: content_id,
                    author: _,
                    posted: _,
                    content: _,
                    liked: _,
                    pinned,
                    created: _,
                    edited: _,
                },
            id,
        }: pin::Output,
    ) -> Result<()> {
        const COLOR: (u8, u8, u8) = (0xfb, 0x49, 0x34);

        self.out
            .send(box move |ce| {
                ce.title("pin")
                    .colour(COLOR)
                    .description(format!("{} => {}", id, content_id))
                    .fields([("pin", pinned.len(), true)])
            })
            .await
            .map_err(|e| e.to_string())
            .unwrap();

        Ok(())
    }
}

pub struct SerenityContentUnpinPresenter {
    pub out: mpsc::Sender<Box<View>>,
}
#[async_trait]
impl content::ContentUnpinPresenter for SerenityContentUnpinPresenter {
    async fn complete(
        &self,
        unpin::Output {
            content:
                Content {
                    id: content_id,
                    author: _,
                    posted: _,
                    content: _,
                    liked: _,
                    pinned,
                    created: _,
                    edited: _,
                },
            id,
        }: unpin::Output,
    ) -> Result<()> {
        const COLOR: (u8, u8, u8) = (0xfb, 0x49, 0x34);

        self.out
            .send(box move |ce| {
                ce.title("unpin")
                    .colour(COLOR)
                    .description(format!("{} =/> {}", id, content_id))
                    .fields([("pin", pinned.len(), true)])
            })
            .await
            .map_err(|e| e.to_string())
            .unwrap();

        Ok(())
    }
}
