macro_rules! return_inner {
    ($s:ident => use $u:ident,lock $l:ident,ret $r:ident,data $d:ident) => {{
        let guard = $s.$l.lock().await;

        $s.$u.handle($d).await?;
        let ret = $s.$r.lock().await.recv().await.unwrap();

        drop(guard);

        Ok(ret)
    }};
}

pub mod content;
pub mod user;

use anyhow::{anyhow, bail, Result};
use serenity::http::CacheHttp;
use serenity::model::channel::Message;
use smallvec::{smallvec, SmallVec};

use crate::cmds::{
    Cmd, ContentEditCmd, ContentGetCmd, ContentGetsCmd, ContentLikeCmd, ContentLikeOp, ContentMod,
    ContentPinCmd, ContentPinOp, ContentPostCmd, ContentWithdrawCmd, PartialContentMutation,
    RootMod, UserBookmarkCmd, UserBookmarkOp, UserEditCmd, UserGetCmd, UserGetsCmd, UserMod,
    UserRegisterCmd, UserUnregisterCmd,
};
use crate::entities::{Author, ContentId, PartialAuthor, Posted, UserId};
use crate::presenters::impls::serenity::View;
use crate::usecases;
use crate::usecases::content::ContentMutation;
use crate::utils::LetChain;

use super::ret::content::ReturnContentController;
use super::ret::user::ReturnUserController;

pub struct SerenityReturnController {
    pub user: user::SerenityUserController,
    pub content: content::SerenityContentController,
    pub return_user_contr: ReturnUserController,
    pub return_content_contr: ReturnContentController,
}

impl SerenityReturnController {
    pub async fn parse(
        &self,
        msg: &Message,
        http: impl CacheHttp + Clone,
    ) -> Option<Result<SmallVec<[Box<View>; 20]>>> {
        let parsed = match match Self::parse_str(msg.content.as_str()).await {
            Some(r) => r,
            None => return None,
        } {
            Ok(o) => o,
            Err(e) => return Some(Err(anyhow!(e))),
        };

        let res = match self.handle_cmd(parsed, msg, http).await {
            Ok(o) => o,
            Err(e) => return Some(Err(e)),
        };

        Some(Ok(res))
    }

    async fn parse_str(raw: &str) -> Option<Result<Cmd>> {
        let split_res = ::shell_words::split(raw)
            .map(|mut v| {
                v.drain(..)
                    .map(|s| s.into())
                    .collect::<Vec<::std::ffi::OsString>>()
            })
            .map_err(|e| e.to_string());

        let splitted = match split_res {
            Ok(o) => o,
            Err(e) => return Some(Err(anyhow!(e))),
        };

        if let Some("*ip") = splitted.get(0).map(|s| s.to_str().unwrap()) {
        } else {
            return None;
        }

        use clap::Clap;

        Cmd::try_parse_from(splitted)
            .map_err(|e| anyhow!(e.to_string()))
            .let_(Some)
    }

    async fn handle_cmd(
        &self,
        app: Cmd,
        msg: &Message,
        http: impl CacheHttp + Clone,
    ) -> Result<SmallVec<[Box<View>; 20]>> {
        let ex_guild_id = msg.guild_id.as_ref().map(|i| i.0);
        let ex_timestamp = &msg.timestamp;

        let ex_user_id = (&msg.author.id).let_(|i| i.0).let_(UserId);
        let ex_user_name = &msg.author.name;
        let ex_user_nick = msg.author_nick(&http).await;

        use usecases::{content, user};
        let Cmd { cmd } = self.authorize_cmd(app, ex_user_id).await?;
        match cmd {
            RootMod::User { cmd } => match cmd {
                UserMod::Register(UserRegisterCmd) => self
                    .user
                    .register(user::register::Input {
                        user_id: ex_user_id,
                    })
                    .await
                    .map(|v| smallvec![v]),

                UserMod::Get(UserGetCmd { user_id }) => self
                    .user
                    .get(user::get::Input {
                        user_id: user_id.map(UserId).unwrap_or(ex_user_id),
                    })
                    .await
                    .map(|v| smallvec![v]),

                UserMod::Gets(UserGetsCmd { page, query }) => self
                    .user
                    .gets(user::gets::Input { query, page })
                    .await
                    .map(|mut v| v.drain(..).collect()),

                UserMod::Edit(UserEditCmd { user_id, mutation }) => self
                    .user
                    .edit(user::edit::Input {
                        user_id: user_id.let_(UserId),
                        mutation,
                    })
                    .await
                    .map(|v| smallvec![v]),

                UserMod::Unregister(UserUnregisterCmd { user_id }) => self
                    .user
                    .unregister(user::unregister::Input {
                        user_id: user_id.let_(UserId),
                    })
                    .await
                    .map(|v| smallvec![v]),

                UserMod::Bookmark(UserBookmarkCmd { op }) => match op {
                    UserBookmarkOp::Do { content_id } => self
                        .user
                        .bookmark(user::bookmark::Input {
                            user_id: ex_user_id,
                            content_id: content_id.let_(ContentId),
                        })
                        .await
                        .map(|v| smallvec![v]),

                    UserBookmarkOp::Undo { content_id } => self
                        .user
                        .unbookmark(user::unbookmark::Input {
                            user_id: ex_user_id,
                            content_id: content_id.let_(ContentId),
                        })
                        .await
                        .map(|v| smallvec![v]),

                    UserBookmarkOp::Show { page, user_id } =>
                        self.user
                            .get_bookmark(user::get_bookmark::Input {
                                user_id: user_id.map(UserId).unwrap_or(ex_user_id),
                                page,
                            })
                            .await,
                },
            },

            RootMod::Content { cmd } => match cmd {
                ContentMod::Post(ContentPostCmd {
                    virt,
                    user_id,
                    content,
                }) => {
                    let posted = Posted {
                        id: ex_user_id,
                        name: ex_user_name.clone(),
                        nick: ex_user_nick,
                    };
                    let author = match (user_id, virt) {
                        (Some(i), None) => {
                            let user = http
                                .http()
                                .get_user(i)
                                .await
                                .map_err(|e| anyhow!("cannot get author: {}", e))?;

                            let nick = match ex_guild_id {
                                Some(i) => user.nick_in(http, i).await,
                                None => None,
                            };
                            let id = user.id.let_(|i| i.0).let_(UserId);
                            let name = user.name;

                            Author::User { id, name, nick }
                        },
                        (None, Some(s)) => Author::Virtual(s),
                        _ => bail!("internal processing error"),
                    };

                    self.content
                        .post(content::post::Input {
                            content,
                            posted,
                            author,
                            created: *ex_timestamp,
                        })
                        .await
                        .map(|v| smallvec![v])
                },

                ContentMod::Get(ContentGetCmd { content_id }) => self
                    .content
                    .get(content::get::Input {
                        content_id: content_id.let_(ContentId),
                    })
                    .await
                    .map(|v| smallvec![v]),

                ContentMod::Gets(ContentGetsCmd { page, query }) => self
                    .content
                    .gets(content::gets::Input { query, page })
                    .await
                    .map(|mut v| v.drain(..).collect()),

                ContentMod::Edit(ContentEditCmd {
                    content_id,
                    mutation: p,
                }) => {
                    let PartialContentMutation { author, content } = p;
                    let author = match author {
                        Some(PartialAuthor::Virtual(s)) => Some(Author::Virtual(s)),
                        Some(PartialAuthor::User(i)) => {
                            let user = http
                                .http()
                                .get_user(i.0)
                                .await
                                .map_err(|e| anyhow!("cannot get author: {}", e))?;

                            let nick = match ex_guild_id {
                                Some(i) => user.nick_in(http, i).await,
                                None => None,
                            };
                            let id = user.id.let_(|i| i.0).let_(UserId);
                            let name = user.name;

                            Some(Author::User { id, name, nick })
                        },
                        None => None,
                    };
                    let mutation = ContentMutation {
                        author,
                        content,
                        edited: *ex_timestamp,
                    };

                    self.content
                        .edit(content::edit::Input {
                            content_id: content_id.let_(ContentId),
                            mutation,
                        })
                        .await
                        .map(|v| smallvec![v])
                },

                ContentMod::Withdraw(ContentWithdrawCmd { content_id }) => self
                    .content
                    .withdraw(content::withdraw::Input {
                        content_id: content_id.let_(ContentId),
                    })
                    .await
                    .map(|v| smallvec![v]),

                ContentMod::Like(ContentLikeCmd { op }) => match op {
                    ContentLikeOp::Do { content_id } => self
                        .content
                        .like(content::like::Input {
                            content_id: content_id.let_(ContentId),
                            user_id: ex_user_id,
                        })
                        .await
                        .map(|v| smallvec![v]),

                    ContentLikeOp::Undo { content_id } => self
                        .content
                        .unlike(content::unlike::Input {
                            content_id: content_id.let_(ContentId),
                            user_id: ex_user_id,
                        })
                        .await
                        .map(|v| smallvec![v]),

                    ContentLikeOp::Show { page, content_id } =>
                        self.content
                            .get_like(content::get_like::Input {
                                content_id: content_id.let_(ContentId),
                                page,
                            })
                            .await,
                },

                ContentMod::Pin(ContentPinCmd { op }) => match op {
                    ContentPinOp::Do { content_id } => self
                        .content
                        .pin(content::pin::Input {
                            content_id: content_id.let_(ContentId),
                            user_id: ex_user_id,
                        })
                        .await
                        .map(|v| smallvec![v]),

                    ContentPinOp::Undo { content_id } => self
                        .content
                        .unpin(content::unpin::Input {
                            content_id: content_id.let_(ContentId),
                            user_id: ex_user_id,
                        })
                        .await
                        .map(|v| smallvec![v]),

                    ContentPinOp::Show { page, content_id } =>
                        self.content
                            .get_pin(content::get_pin::Input {
                                content_id: content_id.let_(ContentId),
                                page,
                            })
                            .await,
                },
            },
        }
    }

    async fn authorize_cmd(&self, cmd: Cmd, ex_user_id: UserId) -> Result<Cmd> {
        let ex_user_res = self.return_user_contr.get(ex_user_id).await;

        let res = match &cmd.cmd {
            RootMod::User { cmd } => match cmd {
                UserMod::Edit(_) | UserMod::Unregister(_) => ex_user_res?.admin,
                _ => true,
            },
            RootMod::Content { cmd } => match cmd {
                ContentMod::Edit(ContentEditCmd { content_id, .. })
                | ContentMod::Withdraw(ContentWithdrawCmd { content_id, .. }) => {
                    let ex_user = ex_user_res?;

                    let content = self
                        .return_content_contr
                        .get((*content_id).let_(ContentId))
                        .await?;

                    content.posted.id == ex_user_id || ex_user.admin || ex_user.sub_admin
                },
                _ => true,
            },
        };

        match res {
            true => Ok(cmd),
            false => Err(anyhow!("not permitted operation")),
        }
    }
}
