use std::fmt::Display;

use anyhow::Result;
use serenity::builder::CreateMessage;
use serenity::client::{Context, EventHandler};
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use serenity::model::interactions::Interaction;
use serenity::model::prelude::User;
use serenity::utils::Colour;
use uuid::Uuid;

use crate::entities::Content;
use crate::handlers::Handler;

mod appcmd;
mod clapcmd;
mod command_colors;
mod command_strs;
mod helper;
mod macros;

pub use appcmd::application_command_create;

pub struct Conductor {
    pub handler: Handler,
}

#[derive(Debug)]
pub enum Command {
    UserRegister,
    UserRead,
    UserUpdate(Option<bool>, Option<bool>),
    Bookmark(Uuid),
    UserDelete,
    ContentPost(String, String),
    ContentRead(Uuid),
    ContentUpdate(Uuid, String),
    Like(Uuid),
    Pin(Uuid),
    ContentDelete(Uuid),
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
        cmd: Command,
        user_id: UserId,
        user_name: impl Display,
        user_nick: Option<&str>,
    ) -> Response {
        let from_user_shows = format!("from: {} ({})", user_name, user_nick.unwrap_or(""));

        use command_colors::*;

        let res: Result<Response> = try {
            let resp: Response = match cmd {
                Command::UserRegister => helper::resp_from_user(
                    "registered user",
                    from_user_shows,
                    REGISTER,
                    self.handler.create_user(user_id).await?,
                ),
                Command::UserRead => helper::resp_from_user(
                    "showing user",
                    from_user_shows,
                    INFO,
                    self.handler.read_user(user_id).await?,
                ),
                Command::UserUpdate(new_admin, new_sub_admin) => helper::resp_from_user(
                    "updated user",
                    from_user_shows,
                    CHANGE,
                    self.handler
                        .update_user(user_id, new_admin, new_sub_admin)
                        .await?,
                ),
                Command::Bookmark(id) => {
                    self.handler.bookmark_update_user(user_id, id).await?;

                    Response {
                        title: "bookmarked".to_string(),
                        rgb: BOOKMARK,
                        description: from_user_shows,
                        fields: vec![
                            ("id:".to_string(), format!("{}", id)),
                            (
                                "bookmarked:".to_string(),
                                format!("{}", self.handler.read_content(id).await?.bookmarked),
                            ),
                        ],
                    }
                },
                Command::UserDelete => {
                    self.handler.delete_user(user_id).await?;

                    Response {
                        title: "deleted user".to_string(),
                        rgb: DELETE_ME,
                        description: "see you!".to_string(),
                        fields: vec![],
                    }
                },
                Command::ContentPost(content, author) => helper::resp_from_content(
                    "posted content",
                    from_user_shows,
                    POST,
                    self.handler
                        .create_content_and_posted_update_user(content, user_id, author)
                        .await?,
                ),
                Command::ContentRead(id) => helper::resp_from_content(
                    "showing content",
                    from_user_shows,
                    GET,
                    self.handler.read_content(id).await?,
                ),
                Command::ContentUpdate(id, new_content) => helper::resp_from_content(
                    "updated content",
                    from_user_shows,
                    EDIT,
                    self.handler.update_content(id, new_content).await?,
                ),

                Command::Like(id) => {
                    self.handler.like_update_content(id, user_id).await?;

                    Response {
                        title: "liked".to_string(),
                        rgb: LIKE,
                        description: from_user_shows,
                        fields: vec![
                            ("id:".to_string(), format!("{}", id)),
                            (
                                "liked:".to_string(),
                                format!("{}", self.handler.read_content(id).await?.liked.len()),
                            ),
                        ],
                    }
                },
                Command::Pin(id) => {
                    self.handler.pin_update_content(id, user_id).await?;
                    let Content { pinned, .. } = self.handler.read_content(id).await?;

                    Response {
                        title: "pinned".to_string(),
                        rgb: PIN,
                        description: from_user_shows,
                        fields: vec![
                            ("id:".to_string(), format!("{}", id)),
                            ("pinned:".to_string(), format!("{}", pinned.len())),
                        ],
                    }
                },
                Command::ContentDelete(id) => {
                    self.handler.delete_content(id).await?;

                    Response {
                        title: "deleted content".to_string(),
                        description: "i'm sad...".to_string(),
                        rgb: REMOVE,
                        fields: vec![("id:".to_string(), format!("{}", id))],
                    }
                },
            };

            resp
        };

        match res {
            Ok(r) => r,
            Err(e) => Response {
                title: "error occurred".to_string(),
                rgb: ERROR,
                description: format!("{}", e),
                fields: vec![],
            },
        }
    }
}

#[serenity::async_trait]
impl EventHandler for Conductor {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let aci = match interaction {
            Interaction::ApplicationCommand(aci) => aci,
            _ => return eprintln!("received not `application command`"),
        };

        let cmd = match helper::parse_ia(&aci.data).await {
            Ok(c) => c,
            Err(e) => return eprintln!("{}", e),
        };

        let nick_opt_string = match aci.guild_id {
            Some(gi) => aci.user.nick_in(&ctx, gi).await,
            None => None,
        };

        let nick_opt = nick_opt_string.as_deref();

        let resp = self
            .handle(cmd, aci.user.id, aci.user.name.clone(), nick_opt)
            .await;

        let res = aci
            .create_interaction_response(&ctx, |cir| {
                cir.interaction_response_data(|cird| {
                    cird.create_embed(|ce| helper::build_embed_from_resp(ce, resp))
                })
            })
            .await;

        match res {
            Ok(o) => o,
            Err(e) => eprintln!("{}", e),
        };
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let mcmd = match helper::parse_msg(msg.content.as_str()).await {
            Ok(o) => o,
            Err(e) => return eprintln!("err: {}", e),
        };

        let cmd = match mcmd {
            MsgCommand::Command(c) => c,
            MsgCommand::Showing(s) => {
                let res = msg
                    .channel_id
                    .send_message(ctx.http, |cm| {
                        cm.add_embed(|ce| {
                            ce.title("error occurred")
                                .colour(Colour::RED)
                                .description(format!("```{}```", s))
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

        let resp = self.handle(cmd, user_id, name, nick_opt).await;

        let res = channel_id
            .send_message(ctx.http, |cm| {
                cm.add_embed(|ce| helper::build_embed_from_resp(ce, resp));

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
