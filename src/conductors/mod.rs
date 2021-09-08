use std::num::NonZeroU32;
use std::ops::{
    Bound, Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serenity::builder::CreateMessage;
use serenity::client::{Context, EventHandler};
use serenity::http::CacheHttp;
use serenity::model::channel::Message;
use serenity::model::prelude::User;

use crate::entities::{Author, ContentId, PartialAuthor, Posted, UserId};
use crate::handlers::Handler;
use crate::repositories::{
    ContentContentMutation, ContentMutation, ContentQuery, UserMutation, UserQuery,
};
use crate::utils::LetChain;

mod clapcmd;
mod command_colors;
mod helper;

pub(crate) struct Conductor {
    pub(crate) handler: Handler,
}

/// command data.
#[derive(Debug, Clone)]
pub(crate) enum Command {
    /// commands about user.
    User(UserCommand),
    /// commands about content.
    Content(ContentCommand),
    /// post content with executed user's id.
    Post {
        author: PartialAuthor,
        content: String,
    },
    /// (un)like content with executed user's id.
    Like { content_id: ContentId, undo: bool },
    /// (un)pin content with executed user's id.
    Pin { content_id: ContentId, undo: bool },
    /// (un)bookmark content to executed user's id.
    Bookmark { content_id: ContentId, undo: bool },
}

// TODO: can show user's bookmark and posted
#[derive(Debug, Clone)]
pub(crate) enum UserCommand {
    /// create user with executed user's id.
    Create,
    /// read user with id.
    /// if not given id, fallback to executed user's id.
    Read { id: Option<UserId> },
    /// read users with query.
    /// page **must** satisfies `1..`.
    Reads { page: NonZeroU32, query: UserQuery },
    /// update user with id and mutation.
    /// it's **must** given id.
    Update { id: UserId, mutation: UserMutation },
    /* delete user with executed user's id. (only accepted from user and admin)
     * Delete, <- TODO: do implement */
}

// TODO: can show content's liked and pinned
#[derive(Debug, Clone)]
pub(crate) enum ContentCommand {
    /// read content with id.
    Read { id: ContentId },
    /// read contents with query.
    /// page **must** satisfies `1..`.
    Reads {
        page: NonZeroU32,
        query: ContentQuery,
    },
    /// update content with id and mutation.
    Update {
        id: ContentId,
        mutation: PartialContentMutation,
    },
    /// delete content with id.
    Delete { id: ContentId },
}

#[derive(Debug, Clone, Default)]
pub(crate) struct PartialContentMutation {
    pub(crate) author: Option<PartialAuthor>,
    pub(crate) content: Option<ContentContentMutation>,
}

#[derive(Debug)]
pub(crate) struct Response {
    title: String,
    rgb: (u8, u8, u8),
    description: String,
    fields: Vec<(String, String)>,
}

impl Conductor {
    pub(crate) async fn conduct(
        &self,
        cmd: Command,
        http: impl CacheHttp + Clone,
        msg: &Message,
    ) -> Vec<Response> {
        let user_nick = msg.author_nick(&http).await;

        let Message {
            guild_id: guild_id_raw,
            author:
                User {
                    id: user_id_raw,
                    name: user_name,
                    ..
                },
            timestamp,
            ..
        } = msg;

        let user_id = UserId(user_id_raw.0);
        let guild_id = guild_id_raw.as_ref().map(|r| r.0);

        let from_user_shows = format!(
            "from: {} ({})",
            user_name,
            user_nick.as_ref().unwrap_or(&"".to_string())
        );

        use command_colors::*;

        let res: Result<Vec<Response>> = try {
            let cmd = self
                .authorize_cmd(cmd, user_id)
                .await
                .map_err(|e| anyhow!(e))?;

            match cmd {
                Command::User(UserCommand::Create) => {
                    let user = self.handler.create_user(user_id).await?;

                    helper::resp_from_user("registered user", from_user_shows, USER_CREATE, user)
                        .let_(|r| vec![r])
                },
                Command::User(UserCommand::Read { id }) => {
                    let user = self.handler.read_user(id.unwrap_or(user_id)).await?;

                    helper::resp_from_user("showing user", from_user_shows, USER_READ, user)
                        .let_(|r| vec![r])
                },
                Command::User(UserCommand::Reads { page, query }) => {
                    let mut users = self.handler.read_users(query).await?;

                    if users.is_empty() {
                        Err(anyhow!("matched: {}", users.len()))?
                    }

                    const ITEMS: usize = 5; // show 5[users]/[page]

                    let lim = {
                        let full = ..users.len();
                        let lim = (ITEMS * (page.get() as usize - 1))
                            ..(ITEMS + ITEMS * (page.get() as usize - 1));

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
                                USER_READ,
                                u,
                            )
                        })
                        .collect()
                },
                Command::User(UserCommand::Update { id, mutation }) => {
                    let user = self.handler.update_user(id, mutation).await?;

                    helper::resp_from_user("updated user", from_user_shows, USER_UPDATE, user)
                        .let_(|r| vec![r])
                },
                Command::Content(ContentCommand::Read { id }) => {
                    let content = self.handler.read_content(id).await?;

                    helper::resp_from_content(
                        "showing content",
                        from_user_shows,
                        CONTENT_READ,
                        content,
                    )
                    .let_(|r| vec![r])
                },
                Command::Content(ContentCommand::Reads { page, query }) => {
                    let mut contents = self.handler.read_contents(query).await?;

                    if contents.is_empty() {
                        Err(anyhow!("matched: {}", contents.len()))?
                    }

                    const ITEMS: usize = 5; // show 5[contents]/[page]

                    let lim = {
                        let full = ..contents.len();
                        let lim = (ITEMS * (page.get() as usize - 1))
                            ..(ITEMS + ITEMS * (page.get() as usize - 1));

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
                                format!("showing contents: {}[{}] | {}/{}", i, s, page, all_pages),
                                from_user_shows.to_string(),
                                CONTENT_READ,
                                c,
                            )
                        })
                        .collect()
                },
                Command::Content(ContentCommand::Update {
                    id,
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
                    let content = self.handler.update_content(id, mutation).await?;

                    helper::resp_from_content(
                        "updated user",
                        from_user_shows,
                        CONTENT_UPDATE,
                        content,
                    )
                    .let_(|r| vec![r])
                },
                Command::Content(ContentCommand::Delete { id }) => {
                    let content = self.handler.delete_content(id).await?;
                    helper::resp_from_content(
                        "deleted content",
                        from_user_shows,
                        CONTENT_DELETE,
                        content,
                    )
                    .let_(|r| vec![r])
                },
                Command::Post {
                    author: partial_author,
                    content,
                } => {
                    let author = match partial_author {
                        PartialAuthor::Virtual(s) => Author::Virtual(s),
                        PartialAuthor::User(id) => {
                            let user = http.http().get_user(id.0).await?;
                            let nick = match guild_id {
                                Some(i) => user.nick_in(http, i).await,
                                None => None,
                            };

                            Author::User {
                                id,
                                name: user.name,
                                nick,
                            }
                        },
                    };

                    let content = self
                        .handler
                        .post(
                            content,
                            Posted {
                                id: user_id,
                                name: user_name.clone(),
                                nick: user_nick,
                            },
                            author,
                            *timestamp,
                        )
                        .await?;

                    helper::resp_from_content("posted content", from_user_shows, POST, content)
                        .let_(|r| vec![r])
                },
                Command::Like { content_id, undo } => {
                    let content = self.handler.like(content_id, user_id, undo).await?;

                    let title = match undo {
                        false => "liked",
                        true => "unliked",
                    };
                    helper::resp_from_content(title, from_user_shows, LIKE, content)
                        .let_(|r| vec![r])
                },
                Command::Pin { content_id, undo } => {
                    let content = self.handler.pin(content_id, user_id, undo).await?;

                    let title = match undo {
                        false => "pinned",
                        true => "unpinned",
                    };
                    helper::resp_from_content(title, from_user_shows, PIN, content)
                        .let_(|r| vec![r])
                },
                Command::Bookmark { content_id, undo } => {
                    let (user, _) = self.handler.bookmark(user_id, content_id, undo).await?;

                    let title = match undo {
                        false => "bookmarked",
                        true => "unbookmarked",
                    };
                    helper::resp_from_user(title, from_user_shows, BOOKMARK, user).let_(|r| vec![r])
                },
            }
        };

        match res {
            Ok(r) => r,
            Err(e) => Response {
                title: "response".to_string(),
                rgb: ERROR,
                description: format!("{}", e),
                fields: vec![],
            }
            .let_(|r| vec![r]),
        }
    }

    async fn authorize_cmd(&self, cmd: Command, user_id: UserId) -> Result<Command, String> {
        let user_res = self
            .handler
            .read_user(user_id)
            .await
            .map_err(|e| format!("auth error: {}", e));

        let res = match &cmd {
            Command::User(UserCommand::Update { .. }) => user_res?.admin,
            // Command::User(UserComamnd::Delete { id }) => unimplemented!(),
            Command::Content(ContentCommand::Update { id, .. }) => {
                let user = user_res?;
                let content = self
                    .handler
                    .read_content(*id)
                    .await
                    .map_err(|e| format!("auth error: {}", e))?;

                content.posted.id == user_id || user.admin || user.sub_admin
            },
            Command::Content(ContentCommand::Delete { id }) => {
                let user = user_res?;
                let content = self
                    .handler
                    .read_content(*id)
                    .await
                    .map_err(|e| format!("auth error: {}", e))?;

                content.posted.id == user_id || user.admin || user.sub_admin
            },

            _ => true,
        };

        match res {
            true => Ok(cmd),
            false => Err("not permitted operation".to_string()),
        }
    }
}

trait ConvertRange<T>: RangeBounds<T> {
    fn to_turple(self) -> (Bound<T>, Bound<T>);
}
impl<T> ConvertRange<T> for Range<T> {
    fn to_turple(self) -> (Bound<T>, Bound<T>) {
        let Range { start, end } = self;
        (Bound::Included(start), Bound::Excluded(end))
    }
}
impl<T> ConvertRange<T> for RangeFrom<T> {
    fn to_turple(self) -> (Bound<T>, Bound<T>) {
        let RangeFrom { start } = self;
        (Bound::Included(start), Bound::Unbounded)
    }
}
impl<T> ConvertRange<T> for RangeFull {
    fn to_turple(self) -> (Bound<T>, Bound<T>) { (Bound::Unbounded, Bound::Unbounded) }
}
impl<T> ConvertRange<T> for RangeInclusive<T> {
    fn to_turple(self) -> (Bound<T>, Bound<T>) {
        let (start, end) = self.into_inner();
        (Bound::Included(start), Bound::Included(end))
    }
}
impl<T> ConvertRange<T> for RangeTo<T> {
    fn to_turple(self) -> (Bound<T>, Bound<T>) {
        let RangeTo { end } = self;
        (Bound::Unbounded, Bound::Excluded(end))
    }
}
impl<T> ConvertRange<T> for RangeToInclusive<T> {
    fn to_turple(self) -> (Bound<T>, Bound<T>) {
        let RangeToInclusive { end } = self;
        (Bound::Unbounded, Bound::Included(end))
    }
}

#[async_trait]
impl EventHandler for Conductor {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        let parse_res = match helper::parse_msg(msg.content.as_str()).await {
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
