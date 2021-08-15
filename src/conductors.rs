use std::fmt::Display;

use anyhow::bail;
use serde_json::Value;
use serenity::builder::CreateApplicationCommands;
use serenity::client::{Context, EventHandler};
use serenity::http::Http;
use serenity::model::channel::Message;
use serenity::model::id::GuildId;
use serenity::model::interactions::{
    ApplicationCommand, ApplicationCommandOptionType, Interaction, InteractionData,
};
use serenity::utils::Colour;
use uuid::Uuid;

use crate::entities::{Content, User};
use crate::handlers::Handler;

pub struct Conductor {
    pub handler: Handler,
}

pub enum Command {
    UserRegister,
    UserRead,
    UserUpdate(Option<bool>, Option<bool>),
    Bookmark(Uuid),
    UserDelete,
    ContentPost(String),
    ContentRead(Uuid),
    ContentUpdate(Uuid, String),
    Like(Uuid),
    Pin(Uuid),
    ContentDelete(Uuid),
}

pub struct Response {
    title: String,
    rgb: (u8, u8, u8),
    description: String,
    fields: Vec<(String, String)>,
}

macro_rules! extract_data {
    (opt $t:path => ref $v:ident in $d:ident) => {{
        let mut opt = $d
            .options
            .iter()
            .filter_map(|v| match v.name == stringify!($v) {
                false => None,
                true => match v.value {
                    Some($t(ref val)) => Some(Some(val)),
                    _ => Some(None),
                },
            })
            .collect::<Vec<_>>();

        match opt.len() {
            1 => Ok(opt.remove(0)),
            _ => Err(anyhow::anyhow!("cannot get value: `{}`", stringify!($v))),
        }
    }};
    ($t:path => ref $v:ident in $d:ident) => {{
        let mut opt = $d
            .options
            .iter()
            .filter_map(|v| match v.name == stringify!($v) {
                false => None,
                true => match v.value {
                    Some($t(ref val)) => Some(val),
                    _ => None,
                },
            })
            .collect::<Vec<_>>();

        match opt.len() {
            1 => Ok(opt.remove(0)),
            _ => Err(anyhow::anyhow!("cannot get value: `id`")),
        }
    }};
}

fn resp_from_user(
    title: impl Display,
    description: impl Display,
    rgb: (u8, u8, u8),
    User {
        id,
        admin,
        sub_admin,
        posted,
        bookmark,
    }: User,
) -> Response {
    Response {
        title: format!("{}", title),
        rgb,
        description: format!("{}", description),
        fields: vec![
            ("id:".to_string(), format!("{}", id)),
            ("is_admin?".to_string(), format!("{}", admin)),
            ("is_sub_admin?".to_string(), format!("{}", sub_admin)),
            ("posted:".to_string(), format!("{}", posted.len())),
            ("bookmarked:".to_string(), format!("{}", bookmark.len())),
        ],
    }
}

fn resp_from_content(
    title: impl Display,
    description: impl Display,
    rgb: (u8, u8, u8),
    Content {
        id,
        content,
        liked,
        bookmarked,
        pinned,
    }: Content,
) -> Response {
    Response {
        title: format!("{}", title),
        rgb,
        description: format!("{}", description),
        fields: vec![
            ("id:".to_string(), format!("{}", id)),
            ("content:".to_string(), content),
            ("liked:".to_string(), format!("{}", liked.len())),
            ("pinned:".to_string(), format!("{}", pinned.len())),
            ("bookmarked:".to_string(), format!("{}", bookmarked)),
        ],
    }
}

impl Conductor {
    pub async fn parse(&self, interaction: &Interaction) -> anyhow::Result<Command> {
        let data = match match interaction.data {
            Some(ref d) => d,
            None => bail!("cannot get interactiion_data."),
        } {
            InteractionData::ApplicationCommand(d) => d,
            _ => bail!(
                "cannot get application_command_interaction_data. (maybe it's message-component? \
                 sorry, not supported.)"
            ),
        };

        let com = match data.name.as_str() {
            "register" => Command::UserRegister,
            "info" => Command::UserRead,
            "change" => {
                let admin = extract_data!(opt Value::Bool => ref admin in data)?;
                let sub_admin = extract_data!(opt Value::Bool => ref sub_admin in data)?;

                Command::UserUpdate(admin.copied(), sub_admin.copied())
            },
            "bookmark" => {
                let id = extract_data!(Value::String => ref id in data)?;

                Command::Bookmark(Uuid::parse_str(id.as_str())?)
            },
            "delete_me" => Command::UserDelete,
            "post" => {
                let content = extract_data!(Value::String => ref id in data)?;

                Command::ContentPost(content.clone())
            },
            "get" => {
                let id = extract_data!(Value::String => ref id in data)?;

                Command::ContentRead(Uuid::parse_str(id.as_str())?)
            },
            "edit" => {
                let id = extract_data!(Value::String => ref id in data)?;
                let content = extract_data!(Value::String => ref content in data)?;

                Command::ContentUpdate(Uuid::parse_str(id.as_str())?, content.clone())
            },
            "like" => {
                let id = extract_data!(Value::String => ref id in data)?;

                Command::Like(Uuid::parse_str(id.as_str())?)
            },
            "pin" => {
                let id = extract_data!(Value::String => ref id in data)?;

                Command::Pin(Uuid::parse_str(id.as_str())?)
            },
            "remove" => {
                let id = extract_data!(Value::String => ref id in data)?;

                Command::ContentDelete(Uuid::parse_str(id.as_str())?)
            },
            _ => bail!("unrecognized application_command name."),
        };

        Ok(com)
    }

    pub async fn handle_ia(&self, interaction: &Interaction) -> Response {
        let res: anyhow::Result<Response> = try {
            let user = match interaction.user {
                Some(ref u) => Ok(u),
                None => Err(anyhow::anyhow!(
                    "cannot get user info. (maybe it's DM? sorry, not supported.)"
                )),
            }?;

            let resp: Response = match self.parse(interaction).await? {
                Command::UserRegister => resp_from_user(
                    "registered user",
                    format!("from: [unimplemented]"),
                    (0, 0, 0),
                    self.handler.create_user(user.id).await?,
                ),
                Command::UserRead => resp_from_user(
                    "showing user",
                    format!("from: [unimplemented]"),
                    (0, 0, 0),
                    self.handler.read_user(user.id).await?,
                ),
                Command::UserUpdate(new_admin, new_sub_admin) => resp_from_user(
                    "updated user",
                    format!("from: [unimplemented]"),
                    (0, 0, 0),
                    self.handler
                        .update_user(user.id, new_admin, new_sub_admin)
                        .await?,
                ),
                Command::Bookmark(id) => {
                    self.handler.read_content(id).await?;
                    self.handler.bookmark_update_user(user.id, id).await?;
                    let Content { bookmarked, .. } = self.handler.read_content(id).await?;

                    Response {
                        title: "bookmarked".to_string(),
                        rgb: (0, 0, 0),
                        description: format!("from: [unimplemented]"),
                        fields: vec![
                            ("id:".to_string(), format!("{}", id)),
                            ("bookmarked:".to_string(), format!("{}", bookmarked)),
                        ],
                    }
                },
                Command::UserDelete => {
                    self.handler.delete_user(user.id).await?;

                    Response {
                        title: "deleted user".to_string(),
                        rgb: (0, 0, 0), // TODO: 色決め
                        description: "see you!".to_string(),
                        fields: vec![],
                    }
                },
                Command::ContentPost(content) => resp_from_content(
                    "posted content",
                    format!("from: [unimplemented]"),
                    (0, 0, 0),
                    self.handler
                        .create_content_and_posted_update_user(content, user.id)
                        .await?,
                ),
                Command::ContentRead(id) => resp_from_content(
                    "showing content",
                    format!("from [unimplemented]"),
                    (0, 0, 0),
                    self.handler.read_content(id).await?,
                ),
                Command::ContentUpdate(id, new_content) => resp_from_content(
                    "updated content",
                    format!("from: [unimplemented]"),
                    (0, 0, 0),
                    self.handler.update_content(id, new_content).await?,
                ),

                Command::Like(id) => {
                    self.handler.read_content(id).await?;
                    self.handler.like_update_content(id, user.id).await?;
                    let Content { liked, .. } = self.handler.read_content(id).await?;

                    Response {
                        title: "liked".to_string(),
                        rgb: (0, 0, 0),
                        description: format!("from: [unimplemented]"),
                        fields: vec![
                            ("id:".to_string(), format!("{}", id)),
                            ("liked:".to_string(), format!("{}", liked.len())),
                        ],
                    }
                },
                Command::Pin(id) => {
                    self.handler.read_content(id).await?;
                    self.handler.pin_update_content(id, user.id).await?;
                    let Content { pinned, .. } = self.handler.read_content(id).await?;

                    Response {
                        title: "pinned".to_string(),
                        rgb: (0, 0, 0),
                        description: format!("from: [unimplemented]"),
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
                        rgb: (0, 0, 0), // TODO: 色決め
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
                rgb: (Colour::RED.r(), Colour::RED.g(), Colour::RED.b()),
                description: format!("{}", e),
                fields: vec![],
            },
        }
    }

    pub async fn handle_msg(&self, msg: &Message) -> Response { unimplemented!() }
}

#[serenity::async_trait]
impl EventHandler for Conductor {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let Response {
            title,
            rgb,
            description,
            mut fields,
        } = self.handle_ia(&interaction).await;
        let (r, g, b) = rgb;

        let res = interaction
            .create_interaction_response(ctx.http, |cir| {
                cir.interaction_response_data(|cird| {
                    cird.create_embed(|ce| {
                        ce.title(title)
                            .colour(Colour::from_rgb(r, g, b))
                            .description(description)
                            .fields(
                                fields
                                    .drain(..)
                                    .map(|(s1, s2)| (s1, s2, false))
                                    .collect::<Vec<_>>(),
                            )
                    })
                })
            })
            .await;

        match res {
            Ok(o) => o,
            Err(e) => eprintln!("{}", e),
        };
    }

    async fn message(&self, ctx: Context, msg: Message) { self.handle_msg(&msg).await; }
}

pub async fn application_command_create(
    http: impl AsRef<Http>,
    guild_id: Option<GuildId>,
) -> anyhow::Result<Vec<ApplicationCommand>> {
    let map = application_commands_create_inner().await;

    let ac = match guild_id {
        Some(GuildId(id)) =>
            http.as_ref()
                .create_guild_application_commands(id, &map)
                .await?,
        None =>
            http.as_ref()
                .create_global_application_commands(&map)
                .await?,
    };

    Ok(ac)
}

async fn application_commands_create_inner() -> Value {
    let mut cacs = CreateApplicationCommands::default();

    cacs.create_application_command(|cac| cac.name("register").description("register user."))
        .create_application_command(|cac| cac.name("info").description("get your user data."))
        .create_application_command(|cac| {
            cac.name("change")
                .description("change your user data.")
                .create_option(|caco| {
                    caco.name("admin")
                        .description("set bot's admin.")
                        .required(false)
                        .kind(ApplicationCommandOptionType::Boolean)
                })
                .create_option(|caco| {
                    caco.name("sub_admin")
                        .description("set bot's sub_admin.")
                        .required(false)
                        .kind(ApplicationCommandOptionType::Boolean)
                })
        })
        .create_application_command(|cac| {
            cac.name("bookmark")
                .description("bookmark content.")
                .create_option(|caco| {
                    caco.name("id")
                        .description("content's id.")
                        .required(true)
                        .kind(ApplicationCommandOptionType::String)
                })
        })
        .create_application_command(|cac| cac.name("delete_me").description("delete user."))
        .create_application_command(|cac| {
            cac.name("post")
                .description("post content.")
                .create_option(|caco| {
                    caco.name("content")
                        .description("content's content.")
                        .required(true)
                        .kind(ApplicationCommandOptionType::String)
                })
        })
        .create_application_command(|cac| {
            cac.name("get")
                .description("get content.")
                .create_option(|caco| {
                    caco.name("id")
                        .description("content's id.")
                        .required(true)
                        .kind(ApplicationCommandOptionType::String)
                })
        })
        .create_application_command(|cac| {
            cac.name("edit")
                .description("edit content.")
                .create_option(|caco| {
                    caco.name("id")
                        .description("content's id.")
                        .required(true)
                        .kind(ApplicationCommandOptionType::String)
                })
                .create_option(|caco| {
                    caco.name("content")
                        .description("replace content.")
                        .required(true)
                        .kind(ApplicationCommandOptionType::String)
                })
        })
        .create_application_command(|cac| {
            cac.name("like")
                .description("like content.")
                .create_option(|caco| {
                    caco.name("id")
                        .description("content's id.")
                        .required(true)
                        .kind(ApplicationCommandOptionType::String)
                })
        })
        .create_application_command(|cac| {
            cac.name("pin")
                .description("pin content.")
                .create_option(|caco| {
                    caco.name("id")
                        .description("content's id.")
                        .required(true)
                        .kind(ApplicationCommandOptionType::String)
                })
        })
        .create_application_command(|cac| {
            cac.name("remove")
                .description("remove content.")
                .create_option(|caco| {
                    caco.name("id")
                        .description("content's id.")
                        .required(true)
                        .kind(ApplicationCommandOptionType::String)
                })
        });

    Value::Array(cacs.0)
}
