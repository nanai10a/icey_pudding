use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serenity::builder::CreateMessage;
use serenity::client::{Context, EventHandler};
use serenity::http::CacheHttp;
use serenity::model::channel::Message;
use serenity::model::prelude::User;

use crate::conductors::helper::{
    App, ContentEditCmd, ContentGetCmd, ContentGetsCmd, ContentLikeCmd, ContentLikeOp, ContentMod,
    ContentPinCmd, ContentPinOp, ContentPostCmd, ContentWithdrawCmd, RootMod, UserBookmarkCmd,
    UserBookmarkOp, UserEditCmd, UserGetCmd, UserGetsCmd, UserMod, UserRegisterCmd,
    UserUnregisterCmd,
};
use crate::entities::{Author, ContentId, PartialAuthor, Posted, UserId};
use crate::handlers::Handler;
use crate::repositories::{ContentContentMutation, ContentMutation};
use crate::utils::LetChain;

mod command_colors;
mod helper;

pub struct Conductor {
    pub handler: Handler,
}

#[derive(Debug, Clone, Default)]
pub struct PartialContentMutation {
    pub author: Option<PartialAuthor>,
    pub content: Option<ContentContentMutation>,
}

#[derive(Debug)]
pub struct Response {
    title: String,
    rgb: (u8, u8, u8),
    description: String,
    fields: Vec<(String, String)>,
}

macro_rules! inner_op_handler {
    ($n:literal, $c:path, $s:expr, $i:expr, $p:expr, $d:expr) => {{
        if $s.is_empty() {
            Err(anyhow!("{}: {}", $n, $s.len()))?
        }

        const ITEMS: usize = $i; // show num [users]/[page]

        let lim = {
            let full = ..$s.len();
            let lim = (ITEMS * ($p as usize - 1))..(ITEMS + ITEMS * ($p as usize - 1));

            if !full.contains(&lim.start) {
                Err(anyhow!("out of range (0..{} !< {:?})", $s.len(), lim))?
            }

            if !full.contains(&lim.end) {
                (lim.start..).to_turple()
            } else {
                lim.to_turple()
            }
        };

        let mut resp_fields = vec![];

        let all_pages = (($s.len() as f32) / (ITEMS as f32)).ceil();
        resp_fields.push(("page".to_string(), format!("{}/{}", $p, all_pages)));

        $s.drain()
            .enumerate()
            .collect::<Vec<_>>()
            .drain(lim)
            .enumerate()
            .for_each(|(s, (i, c))| resp_fields.push((format!("{}[{}]", i, s), c.to_string())));

        Response {
            title: format!("showing {}", $n),
            description: $d,
            rgb: $c,
            fields: resp_fields,
        }
        .let_(|r| vec![r])
    }};
}

impl Conductor {
    pub async fn conduct(
        &self,
        cmd: App,
        http: impl CacheHttp + Clone,
        msg: &Message,
    ) -> Vec<Response> {
        let user_nick = msg.author_nick(&http).await;

        let Message {
            guild_id: guild_id_raw,
            author:
                User {
                    id: executed_user_id_raw,
                    name: user_name,
                    ..
                },
            timestamp,
            ..
        } = msg;

        let executed_user_id = UserId(executed_user_id_raw.0);
        let guild_id = guild_id_raw.as_ref().map(|r| r.0);

        let from_user_shows = format!(
            "from: {} ({})",
            user_name,
            user_nick.as_ref().unwrap_or(&"".to_string())
        );

        use command_colors::*;

        let res: Result<Vec<Response>> = try {
            let App { cmd } = self
                .authorize_cmd(cmd, executed_user_id)
                .await
                .map_err(|e| anyhow!(e))?;

            match cmd {
                RootMod::User { cmd } => match cmd {
                    UserMod::Register(UserRegisterCmd) => {
                        let user = self.handler.register_user(executed_user_id).await?;

                        helper::resp_from_user(
                            "registered user",
                            from_user_shows,
                            USER_REGISTER,
                            user,
                        )
                        .let_(|r| vec![r])
                    },

                    UserMod::Get(UserGetCmd { user_id }) => {
                        let user = self
                            .handler
                            .get_user(user_id.map(UserId).unwrap_or(executed_user_id))
                            .await?;

                        helper::resp_from_user("showing user", from_user_shows, USER_GET, user)
                            .let_(|r| vec![r])
                    },

                    UserMod::Gets(UserGetsCmd { page, query }) => {
                        let mut users = self.handler.get_users(query).await?;

                        if users.is_empty() {
                            Err(anyhow!("matched: {}", users.len()))?
                        }

                        const ITEMS: usize = 5; // show 5[users]/[page]

                        let lim = {
                            let full = ..users.len();
                            let lim = (ITEMS * (page as usize - 1))
                                ..(ITEMS + ITEMS * (page as usize - 1));

                            if !full.contains(&lim.start) {
                                Err(anyhow!("out of range (0..{} !< {:?})", users.len(), lim))?
                            }

                            if !full.contains(&lim.end) {
                                (lim.start..).to_turple()
                            } else {
                                lim.to_turple()
                            }
                        };
                        let all_pages = ((users.len() as f32) / (ITEMS as f32)).ceil();
                        users
                            .drain(..)
                            .enumerate()
                            .collect::<Vec<_>>()
                            .drain(lim)
                            .enumerate()
                            .map(|(s, (i, u))| {
                                helper::resp_from_user(
                                    format!("showing users: {}[{}] | {}/{}", i, s, page, all_pages),
                                    from_user_shows.to_string(),
                                    USER_GET,
                                    u,
                                )
                            })
                            .collect()
                    },

                    UserMod::Edit(UserEditCmd { user_id, mutation }) => {
                        let user = self
                            .handler
                            .edit_user(user_id.let_(UserId), mutation)
                            .await?;

                        helper::resp_from_user("updated user", from_user_shows, USER_EDIT, user)
                            .let_(|r| vec![r])
                    },

                    UserMod::Bookmark(UserBookmarkCmd { op }) => match op {
                        UserBookmarkOp::Do { content_id } => {
                            let (user, _) = self
                                .handler
                                .user_bookmark_op(
                                    executed_user_id,
                                    content_id.let_(ContentId),
                                    false,
                                )
                                .await?;

                            helper::resp_from_user(
                                "bookmarked",
                                from_user_shows,
                                USER_BOOKMARK,
                                user,
                            )
                            .let_(|r| vec![r])
                        },

                        UserBookmarkOp::Undo { content_id } => {
                            let (user, _) = self
                                .handler
                                .user_bookmark_op(
                                    executed_user_id,
                                    content_id.let_(ContentId),
                                    true,
                                )
                                .await?;

                            helper::resp_from_user(
                                "unbookmarked",
                                from_user_shows,
                                USER_BOOKMARK,
                                user,
                            )
                            .let_(|r| vec![r])
                        },

                        UserBookmarkOp::Show { user_id, page } => {
                            let mut bookmark = self
                                .handler
                                .get_user_bookmark(user_id.map(UserId).unwrap_or(executed_user_id))
                                .await?;

                            inner_op_handler!(
                                "bookmark",
                                USER_BOOKMARK,
                                bookmark,
                                20,
                                page,
                                from_user_shows
                            )
                        },
                    },

                    UserMod::Unregister(UserUnregisterCmd { user_id }) => {
                        let user = self.handler.unregister_user(user_id.let_(UserId)).await?;

                        helper::resp_from_user(
                            "deleted user.",
                            from_user_shows,
                            USER_UNREGISTER,
                            user,
                        )
                        .let_(|r| vec![r])
                    },
                },

                RootMod::Content { cmd } => match cmd {
                    ContentMod::Post(ContentPostCmd {
                        virt,
                        user_id,
                        content,
                    }) => {
                        let author = match (user_id, virt) {
                            (Some(v), None) => {
                                let user = http.http().get_user(v).await?;
                                let nick = match guild_id {
                                    Some(i) => user.nick_in(http, i).await,
                                    None => None,
                                };

                                Author::User {
                                    id: v.let_(UserId),
                                    name: user.name,
                                    nick,
                                }
                            },

                            (None, Some(v)) => Author::Virtual(v),
                            v => unreachable!("found: {:?}", v),
                        };

                        let content = self
                            .handler
                            .content_post(
                                content,
                                Posted {
                                    id: executed_user_id,
                                    name: user_name.clone(),
                                    nick: user_nick,
                                },
                                author,
                                *timestamp,
                            )
                            .await?;

                        helper::resp_from_content(
                            "posted content",
                            from_user_shows,
                            CONTENT_POST,
                            content,
                        )
                        .let_(|r| vec![r])
                    },

                    ContentMod::Get(ContentGetCmd { content_id }) => {
                        let content = self.handler.get_content(content_id.let_(ContentId)).await?;

                        helper::resp_from_content(
                            "showing content",
                            from_user_shows,
                            CONTENT_GET,
                            content,
                        )
                        .let_(|r| vec![r])
                    },

                    ContentMod::Gets(ContentGetsCmd { page, query }) => {
                        let mut contents = self.handler.get_contents(query).await?;

                        if contents.is_empty() {
                            Err(anyhow!("matched: {}", contents.len()))?
                        }

                        const ITEMS: usize = 5; // show 5[contents]/[page]

                        let lim = {
                            let full = ..contents.len();
                            let lim = (ITEMS * (page as usize - 1))
                                ..(ITEMS + ITEMS * (page as usize - 1));

                            if !full.contains(&lim.start) {
                                Err(anyhow!("out of range (0..{} !< {:?})", contents.len(), lim))?
                            }

                            if !full.contains(&lim.end) {
                                (lim.start..).to_turple()
                            } else {
                                lim.to_turple()
                            }
                        };
                        let all_pages = ((contents.len() as f32) / (ITEMS as f32)).ceil();
                        contents
                            .drain(..)
                            .enumerate()
                            .collect::<Vec<_>>()
                            .drain(lim)
                            .enumerate()
                            .map(|(s, (i, c))| {
                                helper::resp_from_content(
                                    format!(
                                        "showing contents: {}[{}] | {}/{}",
                                        i, s, page, all_pages
                                    ),
                                    from_user_shows.to_string(),
                                    CONTENT_GET,
                                    c,
                                )
                            })
                            .collect()
                    },

                    ContentMod::Edit(ContentEditCmd {
                        content_id,
                        mutation:
                            PartialContentMutation {
                                author: p_author,
                                content,
                            },
                    }) => {
                        let author = match p_author {
                            Some(PartialAuthor::Virtual(s)) => Some(Author::Virtual(s)),
                            Some(PartialAuthor::User(id)) => {
                                let user = http.http().get_user(id.0).await?;

                                let nick = match guild_id {
                                    Some(i) => user.nick_in(http, i).await,
                                    None => None,
                                };
                                let name = user.name;

                                Some(Author::User { id, name, nick })
                            },
                            None => None,
                        };

                        let mutation = ContentMutation {
                            author,
                            content,
                            edited: *timestamp,
                        };
                        let content = self
                            .handler
                            .edit_content(content_id.let_(ContentId), mutation)
                            .await?;

                        helper::resp_from_content(
                            "updated user",
                            from_user_shows,
                            CONTENT_EDIT,
                            content,
                        )
                        .let_(|r| vec![r])
                    },

                    ContentMod::Like(ContentLikeCmd { op }) => match op {
                        ContentLikeOp::Do { content_id } => {
                            let content = self
                                .handler
                                .content_like_op(
                                    content_id.let_(ContentId),
                                    executed_user_id,
                                    false,
                                )
                                .await?;

                            helper::resp_from_content(
                                "liked",
                                from_user_shows,
                                CONTENT_LIKE,
                                content,
                            )
                            .let_(|r| vec![r])
                        },

                        ContentLikeOp::Undo { content_id } => {
                            let content = self
                                .handler
                                .content_like_op(content_id.let_(ContentId), executed_user_id, true)
                                .await?;

                            helper::resp_from_content(
                                "unliked",
                                from_user_shows,
                                CONTENT_LIKE,
                                content,
                            )
                            .let_(|r| vec![r])
                        },

                        ContentLikeOp::Show { page, content_id } => {
                            let mut like = self
                                .handler
                                .get_content_like(content_id.let_(ContentId))
                                .await?;

                            inner_op_handler!("like", CONTENT_LIKE, like, 20, page, from_user_shows)
                        },
                    },

                    ContentMod::Pin(ContentPinCmd { op }) => match op {
                        ContentPinOp::Do { content_id } => {
                            let content = self
                                .handler
                                .content_pin_op(content_id.let_(ContentId), executed_user_id, false)
                                .await?;

                            helper::resp_from_content(
                                "pinned",
                                from_user_shows,
                                CONTENT_PIN,
                                content,
                            )
                            .let_(|r| vec![r])
                        },

                        ContentPinOp::Undo { content_id } => {
                            let content = self
                                .handler
                                .content_pin_op(content_id.let_(ContentId), executed_user_id, true)
                                .await?;

                            helper::resp_from_content(
                                "unpinned",
                                from_user_shows,
                                CONTENT_PIN,
                                content,
                            )
                            .let_(|r| vec![r])
                        },

                        ContentPinOp::Show { page, content_id } => {
                            let mut pin = self
                                .handler
                                .get_content_pin(content_id.let_(ContentId))
                                .await?;

                            inner_op_handler!("pin", CONTENT_PIN, pin, 20, page, from_user_shows)
                        },
                    },

                    ContentMod::Withdraw(ContentWithdrawCmd { content_id }) => {
                        let content = self
                            .handler
                            .withdraw_content(content_id.let_(ContentId))
                            .await?;

                        helper::resp_from_content(
                            "deleted content",
                            from_user_shows,
                            CONTENT_WITHDRAW,
                            content,
                        )
                        .let_(|r| vec![r])
                    },
                },
            }
        };

        res.unwrap_or_else(|e| {
            Response {
                title: "response".to_string(),
                rgb: ERROR,
                description: e.to_string(),
                fields: vec![],
            }
            .let_(|r| vec![r])
        })
    }

    pub async fn authorize_cmd(&self, cmd: App, user_id: UserId) -> Result<App, String> {
        let user_res = self
            .handler
            .get_user(user_id)
            .await
            .map_err(|e| format!("auth error: {}", e));

        let res = match &cmd.cmd {
            RootMod::User { cmd } => match cmd {
                UserMod::Edit(..) | UserMod::Unregister(..) => user_res?.admin,
                _ => true,
            },
            RootMod::Content { cmd } => match cmd {
                ContentMod::Edit(ContentEditCmd { content_id, .. })
                | ContentMod::Withdraw(ContentWithdrawCmd { content_id, .. }) => {
                    let user = user_res?;
                    let content = self
                        .handler
                        .get_content(content_id.let_(|r| *r).let_(ContentId))
                        .await
                        .map_err(|e| format!("auth error: {}", e))?;

                    content.posted.id == user_id || user.admin || user.sub_admin
                },
                _ => true,
            },
        };

        match res {
            true => Ok(cmd),
            false => Err("not permitted operation".to_string()),
        }
    }
}

trait ConvertRange<T>: ::core::ops::RangeBounds<T> {
    fn to_turple(self) -> (::core::ops::Bound<T>, ::core::ops::Bound<T>);
}
impl<T> ConvertRange<T> for ::core::ops::Range<T> {
    fn to_turple(self) -> (::core::ops::Bound<T>, ::core::ops::Bound<T>) {
        let ::core::ops::Range { start, end } = self;
        (
            ::core::ops::Bound::Included(start),
            ::core::ops::Bound::Excluded(end),
        )
    }
}
impl<T> ConvertRange<T> for ::core::ops::RangeFrom<T> {
    fn to_turple(self) -> (::core::ops::Bound<T>, ::core::ops::Bound<T>) {
        let ::core::ops::RangeFrom { start } = self;
        (
            ::core::ops::Bound::Included(start),
            ::core::ops::Bound::Unbounded,
        )
    }
}
impl<T> ConvertRange<T> for ::core::ops::RangeFull {
    fn to_turple(self) -> (::core::ops::Bound<T>, ::core::ops::Bound<T>) {
        (::core::ops::Bound::Unbounded, ::core::ops::Bound::Unbounded)
    }
}
impl<T> ConvertRange<T> for ::core::ops::RangeInclusive<T> {
    fn to_turple(self) -> (::core::ops::Bound<T>, ::core::ops::Bound<T>) {
        let (start, end) = self.into_inner();
        (
            ::core::ops::Bound::Included(start),
            ::core::ops::Bound::Included(end),
        )
    }
}
impl<T> ConvertRange<T> for ::core::ops::RangeTo<T> {
    fn to_turple(self) -> (::core::ops::Bound<T>, ::core::ops::Bound<T>) {
        let ::core::ops::RangeTo { end } = self;
        (
            ::core::ops::Bound::Unbounded,
            ::core::ops::Bound::Excluded(end),
        )
    }
}
impl<T> ConvertRange<T> for ::core::ops::RangeToInclusive<T> {
    fn to_turple(self) -> (::core::ops::Bound<T>, ::core::ops::Bound<T>) {
        let ::core::ops::RangeToInclusive { end } = self;
        (
            ::core::ops::Bound::Unbounded,
            ::core::ops::Bound::Included(end),
        )
    }
}

#[async_trait]
impl EventHandler for Conductor {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        let parse_res = match helper::parse_msg(msg.content.as_str()) {
            Some(o) => o,
            None => return,
        };

        let cmd = match parse_res {
            Ok(o) => o,
            Err(e) => {
                let res = msg
                    .channel_id
                    .send_message(ctx.http, |cm| {
                        cm.add_embed(|ce| {
                            ce.title("response")
                                .colour(command_colors::ERROR)
                                .description(format!("```{}```", e))
                        });

                        let CreateMessage(ref mut raw, ..) = cm;
                        helper::append_message_reference(raw, msg.id, msg.channel_id, msg.guild_id);

                        cm
                    })
                    .await;

                return match res {
                    Ok(_) => (),
                    Err(e) => eprintln!("err: {}", e),
                };
            },
        };

        let mut resps = self.conduct(cmd, ctx.clone(), &msg).await;

        let res = msg
            .channel_id
            .send_message(ctx.http, |cm| {
                resps.drain(..).for_each(|resp| {
                    cm.add_embed(|ce| helper::build_embed_from_resp(ce, resp));
                });

                let CreateMessage(ref mut raw, ..) = cm;
                helper::append_message_reference(raw, msg.id, msg.channel_id, msg.guild_id);

                cm
            })
            .await;

        match res {
            Ok(_) => (),
            Err(e) => eprintln!("{}", e),
        }
    }
}
