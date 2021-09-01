use std::ops::{
    Bound, Range, RangeBounds, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive,
};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serenity::builder::CreateMessage;
use serenity::client::{Context, EventHandler};
use serenity::http::CacheHttp;
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use serenity::model::prelude::User;
use serenity::utils::Colour;
use uuid::Uuid;

use crate::entities::{Author, PartialAuthor, Posted};
use crate::handlers::Handler;
use crate::repositories::{
    ContentContentMutation, ContentMutation, ContentQuery, UserMutation, UserQuery,
};

mod clapcmd;
mod command_colors;
mod helper;
mod macros;

pub struct Conductor {
    pub handler: Handler,
}

#[derive(Debug)]
pub enum Command {
    UserRegister,
    UserRead,
    UserUpdate {
        admin: Option<bool>,
        sub_admin: Option<bool>,
    },
    Bookmark {
        content_id: Uuid,
        undo: bool,
    },
    UserDelete,
    ContentPost {
        content: String,
        author: String,
    },
    ContentRead {
        queries: Vec<ContentQuery>,
        page: u32,
    },
    ContentUpdate {
        content_id: Uuid,
        new_content: String,
    },
    Like {
        content_id: Uuid,
        undo: bool,
    },
    Pin {
        content_id: Uuid,
        undo: bool,
    },
    ContentDelete {
        content_id: Uuid,
    },
}

/// command data.
#[derive(Debug, Clone)]
pub enum CommandV2 {
    /// commands about user.
    User(UserCommandV2),
    /// commands about content.
    Content(ContentCommandV2),
    /// post content with executed user's id.
    Post {
        author: PartialAuthor,
        content: String,
    },
    /// (un)like content with executed user's id.
    Like { content_id: Uuid, undo: bool },
    /// (un)pin content with executed user's id.
    Pin { content_id: Uuid, undo: bool },
    /// (un)bookmark content to executed user's id.
    Bookmark { content_id: Uuid, undo: bool },
}

#[derive(Debug, Clone)]
pub enum UserCommandV2 {
    /// create user with executed user's id.
    Create,
    /// read user with id.
    /// if not given id, fallback to executed user's id.
    Read { id: Option<u64> },
    /// read users with query.
    /// page must satisfies `1..`.
    Reads { page: u32, query: UserQuery },
    /// update user with id and mutation.
    /// it's **must** given id.
    Update { id: u64, mutation: UserMutation },
    /* delete user with executed user's id.
     * Delete, <- disabled */
}

#[derive(Debug, Clone)]
pub enum ContentCommandV2 {
    /// read content with id.
    Read { id: Uuid },
    /// read contents with query.
    /// page **must** satisfies `1..`.
    Reads { page: u32, query: ContentQuery },
    /// update content with id and mutation.
    Update {
        id: Uuid,
        mutation: PartialContentMutation,
    },
    /// delete content with id.
    Delete { id: Uuid },
}

#[derive(Debug, Clone, Default)]
pub struct PartialContentMutation {
    pub author: Option<PartialAuthor>,
    pub content: Option<ContentContentMutation>,
}

#[derive(Debug)]
pub enum MsgCommand {
    Command(Command),
    Showing(String),
}

#[derive(Debug)]
pub struct Response {
    title: String,
    rgb: (u8, u8, u8),
    description: String,
    fields: Vec<(String, String)>,
}

impl Conductor {
    pub async fn handle(
        &self,
        cmd: CommandV2,
        user_id: UserId,
        user_name: String,
        user_nick: Option<String>,
        http: impl CacheHttp,
        guild_id: Option<u64>,
    ) -> Vec<Response> {
        let from_user_shows = format!(
            "from: {} ({})",
            user_name,
            user_nick.as_ref().unwrap_or(&"".to_string())
        );

        use command_colors::*;

        let res: Result<Vec<Response>> = try {
            match cmd {
                CommandV2::User(UserCommandV2::Create) => {
                    let user = self.handler.create_user_v2(*user_id.as_u64()).await?;

                    vec![helper::resp_from_user(
                        "registered user",
                        from_user_shows,
                        USER_CREATE,
                        user,
                    )]
                },
                CommandV2::User(UserCommandV2::Read { id }) => {
                    let user = self.handler.read_user_v2(id.unwrap_or(user_id.0)).await?;

                    vec![helper::resp_from_user(
                        "showing user",
                        from_user_shows,
                        USER_READ,
                        user,
                    )]
                },
                CommandV2::User(UserCommandV2::Reads { page, query }) => {
                    let mut users = self.handler.read_users_v2(query).await?;

                    if users.is_empty() {
                        Err(anyhow!("matched: {}", users.len()))?
                    }

                    const ITEMS: usize = 5; // show 5[users]/[page]

                    let lim = {
                        let full = ..users.len();
                        let lim =
                            (ITEMS * (page as usize - 1))..(ITEMS + ITEMS * (page as usize - 1));

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
                CommandV2::User(UserCommandV2::Update { id, mutation }) => {
                    let user = self.handler.update_user_v2(id, mutation).await?;

                    vec![helper::resp_from_user(
                        "updated user",
                        from_user_shows,
                        USER_UPDATE,
                        user,
                    )]
                },
                CommandV2::Content(ContentCommandV2::Read { id }) => {
                    let content = self.handler.read_content_v2(id).await?;

                    vec![helper::resp_from_content(
                        "showing content",
                        from_user_shows,
                        CONTENT_READ,
                        content,
                    )]
                },
                CommandV2::Content(ContentCommandV2::Reads { page, query }) => {
                    let mut contents = self.handler.read_contents_v2(query).await?;

                    if contents.is_empty() {
                        Err(anyhow!("matched: {}", contents.len()))?
                    }

                    const ITEMS: usize = 5; // show 5[contents]/[page]

                    let lim = {
                        let full = ..contents.len();
                        let lim =
                            (ITEMS * (page as usize - 1))..(ITEMS + ITEMS * (page as usize - 1));

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
                CommandV2::Content(ContentCommandV2::Update {
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
                            let user = http.http().get_user(id).await?;

                            let nick = match guild_id {
                                Some(i) => user.nick_in(http, i).await,
                                None => None,
                            };
                            let name = user.name;

                            Some(Author::User { id, name, nick })
                        },
                        None => None,
                    };

                    let mutation = ContentMutation { author, content };
                    let content = self.handler.update_content_v2(id, mutation).await?;

                    vec![helper::resp_from_content(
                        "updated user",
                        from_user_shows,
                        CONTENT_UPDATE,
                        content,
                    )]
                },
                CommandV2::Content(ContentCommandV2::Delete { id }) => {
                    let content = self.handler.delete_content_v2(id).await?;

                    vec![helper::resp_from_content(
                        "deleted content",
                        from_user_shows,
                        CONTENT_DELETE,
                        content,
                    )]
                },
                CommandV2::Post {
                    author: partial_author,
                    content,
                } => {
                    let author = match partial_author {
                        PartialAuthor::Virtual(s) => Author::Virtual(s),
                        PartialAuthor::User(id) => {
                            let user = http.http().get_user(id).await?;
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
                        .post_v2(
                            content,
                            Posted {
                                id: user_id.0,
                                name: user_name,
                                nick: user_nick,
                            },
                            author,
                        )
                        .await?;

                    vec![helper::resp_from_content(
                        "posted content",
                        from_user_shows,
                        POST,
                        content,
                    )]
                },
                CommandV2::Like { content_id, undo } => {
                    let content = self.handler.like_v2(content_id, user_id.0, undo).await?;

                    let title = match undo {
                        false => "liked",
                        true => "unliked",
                    };
                    vec![helper::resp_from_content(
                        title,
                        from_user_shows,
                        LIKE,
                        content,
                    )]
                },
                CommandV2::Pin { content_id, undo } => {
                    let content = self.handler.pin_v2(content_id, user_id.0, undo).await?;

                    let title = match undo {
                        false => "pinned",
                        true => "unpinned",
                    };
                    vec![helper::resp_from_content(
                        title,
                        from_user_shows,
                        PIN,
                        content,
                    )]
                },
                CommandV2::Bookmark { content_id, undo } => {
                    let (user, _) = self
                        .handler
                        .bookmark_v2(user_id.0, content_id, undo)
                        .await?;

                    let title = match undo {
                        false => "bookmarked",
                        true => "unbookmarked",
                    };
                    vec![helper::resp_from_user(
                        title,
                        from_user_shows,
                        BOOKMARK,
                        user,
                    )]
                },
            }
        };

        match res {
            Ok(r) => r,
            Err(e) => vec![Response {
                title: "response".to_string(),
                rgb: ERROR,
                description: format!("{}", e),
                fields: vec![],
            }],
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
        let parse_res = match helper::parse_msg_v2(msg.content.as_str()).await {
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
                                .colour(Colour::RED)
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

        let user_nick = msg.author_nick(&ctx).await;

        let Message {
            id: message_id,
            channel_id,
            guild_id: guild_id_opt,
            author,
            ..
        } = msg;
        let User {
            id: user_id,
            name: user_name,
            ..
        } = author;

        let mut resps = self
            .handle(
                cmd,
                user_id,
                user_name,
                user_nick,
                ctx.clone(),
                guild_id_opt.map(|i| i.0),
            )
            .await;

        let res = channel_id
            .send_message(ctx.http, |cm| {
                resps.drain(..).for_each(|resp| {
                    cm.add_embed(|ce| helper::build_embed_from_resp(ce, resp));
                });

                let CreateMessage(ref mut raw, ..) = cm;
                helper::append_message_reference(raw, message_id, channel_id, guild_id_opt);

                cm
            })
            .await;

        match res {
            Ok(_) => (),
            Err(e) => eprintln!("{}", e),
        }
    }
}
