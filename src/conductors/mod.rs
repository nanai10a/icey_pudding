use std::fmt::Display;

use anyhow::Result;
use async_trait::async_trait;
use serenity::builder::CreateMessage;
use serenity::client::{Context, EventHandler};
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use serenity::model::prelude::User;
use serenity::utils::Colour;
use uuid::Uuid;

use crate::entities::{Content, PartialAuthor};
use crate::handlers::Handler;
use crate::repositories::{ContentMutation, ContentQuery, UserMutation, UserQuery};

mod appcmd;
mod clapcmd;
mod command_colors;
#[deprecated]
mod command_strs;
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
    Update { id: Uuid, mutation: ContentMutation },
    /// delete content with id.
    Delete { id: Uuid },
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
        user_name: impl Display,
        user_nick: Option<&str>,
    ) -> Vec<Response> {
        let from_user_shows = format!("from: {} ({})", user_name, user_nick.unwrap_or(""));

        use command_colors::*;

        let res: Result<Vec<Response>> = try {
            match cmd {
                CommandV2::User(UserCommandV2::Create) => unimplemented!(),
                CommandV2::User(UserCommandV2::Read { id }) => unimplemented!(),
                CommandV2::User(UserCommandV2::Reads { page, query }) => unimplemented!(),
                CommandV2::User(UserCommandV2::Update { id, mutation }) => unimplemented!(),
                CommandV2::Content(ContentCommandV2::Read { id }) => unimplemented!(),
                CommandV2::Content(ContentCommandV2::Reads { page, query }) => unimplemented!(),
                CommandV2::Content(ContentCommandV2::Update { id, mutation }) => unimplemented!(),
                CommandV2::Content(ContentCommandV2::Delete { id }) => unimplemented!(),
                CommandV2::Post { author, content } => unimplemented!(),
                CommandV2::Like { content_id, undo } => unimplemented!(),
                CommandV2::Pin { content_id, undo } => unimplemented!(),
                CommandV2::Bookmark { content_id, undo } => unimplemented!(),
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

        let nick_opt_string = msg.author_nick(&ctx).await;

        let nick_opt = nick_opt_string.as_deref();

        let Message {
            id: message_id,
            channel_id,
            guild_id: guild_id_opt,
            author,
            ..
        } = msg;
        let User {
            id: user_id, name, ..
        } = author;

        let mut resps = self.handle(cmd, user_id, name, nick_opt).await;

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
