use std::collections::HashMap;
use std::fmt::Display;

use anyhow::bail;
use clap::ErrorKind;
use serde_json::{json, Number, Value};
use serenity::builder::{CreateApplicationCommands, CreateEmbed, CreateMessage};
use serenity::client::{Context, EventHandler};
use serenity::http::Http;
use serenity::model::channel::Message;
use serenity::model::id::{ChannelId, GuildId, MessageId, UserId};
use serenity::model::interactions::application_command::{
    ApplicationCommand, ApplicationCommandInteractionData, ApplicationCommandOptionType,
};
use serenity::model::interactions::Interaction;
use serenity::model::prelude::User;
use serenity::utils::Colour;
use uuid::Uuid;

use crate::entities::{self, Content};
use crate::handlers::Handler;

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

macro_rules! extract_option {
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

macro_rules! extract_clap_sams {
    ($n:expr; in $a:expr) => {
        match $a.subcommand_matches($n) {
            Some(s) => s,
            None => bail!("cannot get arg_matches: {}", $n),
        }
    };
}

macro_rules! extract_clap_arg {
    ($n:expr; in $a:expr) => {
        match $a.value_of($n) {
            Some(s) => s,
            None => bail!("cannot get arg: {}", $n),
        }
    };
}

fn resp_from_user(
    title: impl Display,
    description: impl Display,
    rgb: (u8, u8, u8),
    entities::User {
        id,
        admin,
        sub_admin,
        posted,
        bookmark,
    }: entities::User,
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
        author,
        posted,
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
            ("author".to_string(), author),
            ("posted".to_string(), format!("{}", posted)),
            ("content:".to_string(), content),
            ("liked:".to_string(), format!("{}", liked.len())),
            ("pinned:".to_string(), format!("{}", pinned.len())),
            ("bookmarked:".to_string(), format!("{}", bookmarked)),
        ],
    }
}

impl Conductor {
    pub async fn parse_ia(
        &self,
        acid: &ApplicationCommandInteractionData,
    ) -> anyhow::Result<Command> {
        let com = match acid.name.as_str() {
            "register" => Command::UserRegister,
            "info" => Command::UserRead,
            "change" => {
                let admin = extract_option!(opt Value::Bool => ref admin in acid)?;
                let sub_admin = extract_option!(opt Value::Bool => ref sub_admin in acid)?;

                Command::UserUpdate(admin.copied(), sub_admin.copied())
            },
            "bookmark" => {
                let id = extract_option!(Value::String => ref id in acid)?;

                Command::Bookmark(Uuid::parse_str(id.as_str())?)
            },
            "delete_me" => Command::UserDelete,
            "post" => {
                let content = extract_option!(Value::String => ref id in acid)?;
                let author = extract_option!(Value::String => ref author in acid)?;

                Command::ContentPost(content.clone(), author.clone())
            },
            "get" => {
                let id = extract_option!(Value::String => ref id in acid)?;

                Command::ContentRead(Uuid::parse_str(id.as_str())?)
            },
            "edit" => {
                let id = extract_option!(Value::String => ref id in acid)?;
                let content = extract_option!(Value::String => ref content in acid)?;

                Command::ContentUpdate(Uuid::parse_str(id.as_str())?, content.clone())
            },
            "like" => {
                let id = extract_option!(Value::String => ref id in acid)?;

                Command::Like(Uuid::parse_str(id.as_str())?)
            },
            "pin" => {
                let id = extract_option!(Value::String => ref id in acid)?;

                Command::Pin(Uuid::parse_str(id.as_str())?)
            },
            "remove" => {
                let id = extract_option!(Value::String => ref id in acid)?;

                Command::ContentDelete(Uuid::parse_str(id.as_str())?)
            },
            _ => bail!("unrecognized application_command name."),
        };

        Ok(com)
    }

    pub async fn parse_msg(&self, msg: &str) -> anyhow::Result<MsgCommand> {
        let splitted = shell_words::split(msg)?;

        if let Some(n) = splitted.get(0){
            if n != command_strs::PREFIX { bail!("not command, abort.") }
        }

        let ams = match create_clap_app().get_matches_from_safe(splitted) {
            Ok(o) => o,
            Err(e) => return Ok(MsgCommand::Showing(e.message)),
        };

        use std::str::FromStr;

        use command_strs::*;

        let cmd = match match ams.subcommand_name() {
            None => bail!("cannot get subcommand."),
            Some(s) => s,
        } {
            // let name = extract_clap_arg!(register::name::NAME; in sams);
            register::NAME => Command::UserRegister,
            info::NAME => Command::UserRead,
            change::NAME => {
                let sams = extract_clap_sams!(change::NAME; in ams);
                let admin_raw = sams.value_of(change::admin::NAME);
                let sub_admin_raw = sams.value_of(change::sub_admin::NAME);

                let admin = match admin_raw.map(|s| bool::from_str(s)) {
                    Some(Ok(b)) => Some(b),
                    None => None,
                    Some(Err(e)) => bail!("{}", e),
                };

                let sub_admin = match sub_admin_raw.map(|s| bool::from_str(s)) {
                    Some(Ok(b)) => Some(b),
                    None => None,
                    Some(Err(e)) => bail!("{}", e),
                };

                Command::UserUpdate(admin, sub_admin)
            },
            bookmark::NAME => {
                let sams = extract_clap_sams!(bookmark::NAME; in ams);
                let id_raw = extract_clap_arg!(bookmark::id::NAME; in sams);

                let id = Uuid::from_str(id_raw)?;

                Command::Bookmark(id)
            },
            delete_me::NAME => Command::UserDelete,
            post::NAME => {
                let sams = extract_clap_sams!(post::NAME; in ams);
                let content = extract_clap_arg!(post::content::NAME; in sams);
                let author = extract_clap_arg!(post::author::NAME; in sams);

                Command::ContentPost(content.to_string(), author.to_string())
            },
            get::NAME => {
                let sams = extract_clap_sams!(get::NAME; in ams);
                let id_raw = extract_clap_arg!(get::id::NAME; in sams);

                let id = Uuid::from_str(id_raw)?;

                Command::ContentRead(id)
            },
            edit::NAME => {
                let sams = extract_clap_sams!(edit::NAME; in ams);
                let id_raw = extract_clap_arg!(edit::id::NAME; in sams);
                let content = extract_clap_arg!(edit::content::NAME; in sams);

                let id = Uuid::from_str(id_raw)?;

                Command::ContentUpdate(id, content.to_string())
            },
            like::NAME => {
                let sams = extract_clap_sams!(like::NAME; in ams);
                let id_raw = extract_clap_arg!(like::id::NAME; in sams);

                let id = Uuid::from_str(id_raw)?;

                Command::Like(id)
            },
            pin::NAME => {
                let sams = extract_clap_sams!(pin::NAME; in ams);
                let id_raw = extract_clap_arg!(pin::id::NAME; in sams);

                let id = Uuid::from_str(id_raw)?;

                Command::Pin(id)
            },
            remove::NAME => {
                let sams = extract_clap_sams!(remove::NAME; in ams);
                let id_raw = extract_clap_arg!(remove::id::NAME; in sams);

                let id = Uuid::from_str(id_raw)?;

                Command::ContentDelete(id)
            },
            _ => bail!("unrecognized subcommand."),
        };

        Ok(MsgCommand::Command(cmd))
    }

    pub async fn handle(
        &self,
        cmd: Command,
        user_id: UserId,
        user_name: impl Display,
        user_nick: Option<&str>,
    ) -> Response {
        let from_user_shows = format!("from: {} ({})", user_name, user_nick.unwrap_or(""));

        use command_colors::*;

        let res: anyhow::Result<Response> = try {
            let resp: Response = match cmd {
                Command::UserRegister => resp_from_user(
                    "registered user",
                    from_user_shows,
                    REGISTER,
                    self.handler.create_user(user_id).await?,
                ),
                Command::UserRead => resp_from_user(
                    "showing user",
                    from_user_shows,
                    INFO,
                    self.handler.read_user(user_id).await?,
                ),
                Command::UserUpdate(new_admin, new_sub_admin) => resp_from_user(
                    "updated user",
                    from_user_shows,
                    CHANGE,
                    self.handler
                        .update_user(user_id, new_admin, new_sub_admin)
                        .await?,
                ),
                Command::Bookmark(id) => {
                    self.handler.read_content(id).await?;
                    self.handler.bookmark_update_user(user_id, id).await?;
                    let Content { bookmarked, .. } = self.handler.read_content(id).await?;

                    Response {
                        title: "bookmarked".to_string(),
                        rgb: BOOKMARK,
                        description: from_user_shows,
                        fields: vec![
                            ("id:".to_string(), format!("{}", id)),
                            ("bookmarked:".to_string(), format!("{}", bookmarked)),
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
                Command::ContentPost(content, author) => resp_from_content(
                    "posted content",
                    from_user_shows,
                    POST,
                    self.handler
                        .create_content_and_posted_update_user(content, user_id, author)
                        .await?,
                ),
                Command::ContentRead(id) => resp_from_content(
                    "showing content",
                    from_user_shows,
                    GET,
                    self.handler.read_content(id).await?,
                ),
                Command::ContentUpdate(id, new_content) => resp_from_content(
                    "updated content",
                    from_user_shows,
                    EDIT,
                    self.handler.update_content(id, new_content).await?,
                ),

                Command::Like(id) => {
                    self.handler.read_content(id).await?;
                    self.handler.like_update_content(id, user_id).await?;
                    let Content { liked, .. } = self.handler.read_content(id).await?;

                    Response {
                        title: "liked".to_string(),
                        rgb: LIKE,
                        description: from_user_shows,
                        fields: vec![
                            ("id:".to_string(), format!("{}", id)),
                            ("liked:".to_string(), format!("{}", liked.len())),
                        ],
                    }
                },
                Command::Pin(id) => {
                    self.handler.read_content(id).await?;
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

pub fn build_embed_from_resp(
    ce: &mut CreateEmbed,
    Response {
        title,
        rgb,
        description,
        mut fields,
    }: Response,
) -> &mut CreateEmbed {
    let (r, g, b) = rgb;

    ce.title(title)
        .colour(Colour::from_rgb(r, g, b))
        .description(description)
        .fields(
            fields
                .drain(..)
                .map(|(s1, s2)| (s1, s2, false))
                .collect::<Vec<_>>(),
        )
}

fn append_message_reference(
    raw: &mut HashMap<&str, Value>,
    id: MessageId,
    channel_id: ChannelId,
    guild_id: Option<GuildId>,
) {
    let mr = dbg!(json!({
        "message_id": id,
        "channel_id": channel_id,
        "guild_id": match guild_id {
            Some(i) => Value::Number(Number::from(i.0)),
            None => Value::Null
        },
    }));

    dbg!(raw.insert("message_reference", mr));
}

#[serenity::async_trait]
impl EventHandler for Conductor {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        let aci = match interaction {
            Interaction::ApplicationCommand(aci) => aci,
            _ => return eprintln!("received not `application command`"),
        };

        let cmd = match self.parse_ia(&aci.data).await {
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
                    cird.create_embed(|ce| build_embed_from_resp(ce, resp))
                })
            })
            .await;

        match res {
            Ok(o) => o,
            Err(e) => eprintln!("{}", e),
        };
    }

    async fn message(&self, ctx: Context, msg: Message) {
        let mcmd = match self.parse_msg(msg.content.as_str()).await {
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
                        append_message_reference(raw, msg.id, msg.channel_id, msg.guild_id);

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
                cm.add_embed(|ce| build_embed_from_resp(ce, resp));

                let CreateMessage(ref mut raw, ..) = cm;
                append_message_reference(raw, message_id, channel_id, guild_id_opt);

                cm
            })
            .await;

        match res {
            Ok(_) => (),
            Err(e) => eprintln!("{}", e),
        }
    }
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

mod command_colors {
    pub const REGISTER: (u8, u8, u8) = (0xd5, 0xc4, 0xa1);
    pub const INFO: (u8, u8, u8) = (0x83, 0xa5, 0x98);
    pub const CHANGE: (u8, u8, u8) = (0xb8, 0xb2, 0x26);
    pub const BOOKMARK: (u8, u8, u8) = (0x83, 0xa5, 0x98);
    pub const DELETE_ME: (u8, u8, u8) = (0x1d, 0x20, 0x21);
    pub const POST: (u8, u8, u8) = (0xfb, 0xf1, 0xc7);
    pub const GET: (u8, u8, u8) = (0xfa, 0xdb, 0x2f);
    pub const EDIT: (u8, u8, u8) = (0x8e, 0xc0, 0x7c);
    pub const LIKE: (u8, u8, u8) = (0xd3, 0x86, 0x9b);
    pub const PIN: (u8, u8, u8) = (0xfb, 0x49, 0x34);
    pub const REMOVE: (u8, u8, u8) = (0x66, 0x5c, 0x54);

    pub const ERROR: (u8, u8, u8) = (0xfe, 0x80, 0x19);
}

mod command_strs {
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");

    consts::consts! {
        NAME: "icey_pudding";
        PREFIX: "*ip";
        ABOUT: "this is a ICEy_PUDDING.";

        register {
            NAME: "register";
            DESC: "register user.";
        }

        info {
            NAME: "info";
            DESC: "get your user data.";
        }


        change {
            NAME: "change";
            DESC: "change your user data.";
            admin {
                NAME: "admin";
                DESC: "set bot's admin.";
            }
            sub_admin {
                NAME: "sub_admin";
                DESC: "set bot's sub_admin.";
            }
        }

        bookmark {
            NAME: "bookmark";
            DESC: "bookmark content.";
            id {
                NAME: "id";
                DESC: "content's id.";
            }
        }

        delete_me {
            NAME: "delete_me";
            DESC: "delete user.";
        }

        post {
            NAME: "post";
            DESC: "post content.";
            author {
                NAME: "author";
                DESC: "who said conntent.";
            }
            content {
                NAME: "content";
                DESC: "content's content.";
            }
        }

        get {
            NAME: "get";
            DESC: "get content.";
            id {
                NAME: "id";
                DESC: "content's id.";
            }
        }

        edit {
            NAME: "edit";
            DESC: "edit content.";
            id {
                NAME: "id";
                DESC: "content's id.";
            }
            content {
                NAME: "content";
                DESC: "replace content.";
            }
        }

        like {
            NAME: "like";
            DESC: "like content.";
            id {
                NAME: "id";
                DESC: "content's id.";
            }
        }

        pin {
            NAME: "pin";
            DESC: "pin content.";
            id {
                NAME: "id";
                DESC: "content's id.";
            }
        }

        remove {
            NAME: "remove";
            DESC: "remove content.";
            id {
                NAME: "id";
                DESC: "content's id.";
            }
        }
    }
}

async fn application_commands_create_inner() -> Value {
    let mut cacs = CreateApplicationCommands::default();

    use command_strs::*;

    cacs.create_application_command(|cac| cac.name(register::NAME).description(register::DESC))
        .create_application_command(|cac| cac.name(info::NAME).description(info::DESC))
        .create_application_command(|cac| {
            cac.name(change::NAME)
                .description(change::DESC)
                .create_option(|caco| {
                    caco.name(change::admin::NAME)
                        .description(change::admin::DESC)
                        .required(false)
                        .kind(ApplicationCommandOptionType::Boolean)
                })
                .create_option(|caco| {
                    caco.name(change::sub_admin::NAME)
                        .description(change::sub_admin::DESC)
                        .required(false)
                        .kind(ApplicationCommandOptionType::Boolean)
                })
        })
        .create_application_command(|cac| {
            cac.name(bookmark::NAME)
                .description(bookmark::DESC)
                .create_option(|caco| {
                    caco.name(bookmark::id::NAME)
                        .description(bookmark::id::DESC)
                        .required(true)
                        .kind(ApplicationCommandOptionType::String)
                })
        })
        .create_application_command(|cac| cac.name(delete_me::NAME).description(delete_me::DESC))
        .create_application_command(|cac| {
            cac.name(post::NAME)
                .description(post::DESC)
                .create_option(|caco| {
                    caco.name(post::author::NAME)
                        .description(post::author::DESC)
                        .required(true)
                        .kind(ApplicationCommandOptionType::String)
                })
                .create_option(|caco| {
                    caco.name(post::content::NAME)
                        .description(post::content::DESC)
                        .required(true)
                        .kind(ApplicationCommandOptionType::String)
                })
        })
        .create_application_command(|cac| {
            cac.name(get::NAME)
                .description(get::DESC)
                .create_option(|caco| {
                    caco.name(get::id::NAME)
                        .description(get::id::DESC)
                        .required(true)
                        .kind(ApplicationCommandOptionType::String)
                })
        })
        .create_application_command(|cac| {
            cac.name(edit::NAME)
                .description(edit::DESC)
                .create_option(|caco| {
                    caco.name(edit::id::NAME)
                        .description(edit::id::DESC)
                        .required(true)
                        .kind(ApplicationCommandOptionType::String)
                })
                .create_option(|caco| {
                    caco.name(edit::content::NAME)
                        .description(edit::content::DESC)
                        .required(true)
                        .kind(ApplicationCommandOptionType::String)
                })
        })
        .create_application_command(|cac| {
            cac.name(like::NAME)
                .description(like::DESC)
                .create_option(|caco| {
                    caco.name(like::id::NAME)
                        .description(like::id::DESC)
                        .required(true)
                        .kind(ApplicationCommandOptionType::String)
                })
        })
        .create_application_command(|cac| {
            cac.name(pin::NAME)
                .description(pin::DESC)
                .create_option(|caco| {
                    caco.name(pin::id::NAME)
                        .description(pin::id::DESC)
                        .required(true)
                        .kind(ApplicationCommandOptionType::String)
                })
        })
        .create_application_command(|cac| {
            cac.name(remove::NAME)
                .description(remove::DESC)
                .create_option(|caco| {
                    caco.name(remove::id::NAME)
                        .description(remove::id::DESC)
                        .required(true)
                        .kind(ApplicationCommandOptionType::String)
                })
        });

    Value::Array(cacs.0)
}

pub fn create_clap_app() -> clap::App<'static, 'static> {
    use clap::{App, Arg, SubCommand};
    use command_strs::*;

    App::new(PREFIX)
        .name(NAME)
        .about(ABOUT)
        .version(VERSION)
        .subcommands(vec![
            SubCommand::with_name(register::NAME).about(register::DESC),
            SubCommand::with_name(info::NAME).about(info::DESC),
            SubCommand::with_name(change::NAME)
                .about(change::DESC)
                .args(&vec![
                    Arg::with_name(change::admin::NAME)
                        .help(change::admin::DESC)
                        .required(false)
                        .takes_value(true)
                        .value_name(change::admin::NAME),
                    Arg::with_name(change::sub_admin::NAME)
                        .help(change::sub_admin::DESC)
                        .required(false)
                        .takes_value(true)
                        .value_name(change::sub_admin::NAME),
                ]),
            SubCommand::with_name(bookmark::NAME)
                .about(bookmark::DESC)
                .arg(
                    Arg::with_name(bookmark::id::NAME)
                        .help(bookmark::id::DESC)
                        .required(true)
                        .takes_value(true)
                        .value_name(bookmark::id::NAME),
                ),
            SubCommand::with_name(delete_me::NAME).about(delete_me::DESC),
            SubCommand::with_name(post::NAME)
                .about(post::DESC)
                .args(&vec![
                    Arg::with_name(post::author::NAME)
                        .help(post::author::DESC)
                        .required(true)
                        .takes_value(true)
                        .value_name(post::author::NAME),
                    Arg::with_name(post::content::NAME)
                        .help(post::content::DESC)
                        .required(true)
                        .takes_value(true)
                        .value_name(post::content::NAME),
                ]),
            SubCommand::with_name(get::NAME).about(get::DESC).arg(
                Arg::with_name(get::id::NAME)
                    .help(get::id::DESC)
                    .required(true)
                    .takes_value(true)
                    .value_name(get::id::NAME),
            ),
            SubCommand::with_name(edit::NAME)
                .about(edit::DESC)
                .args(&vec![
                    Arg::with_name(edit::id::NAME)
                        .help(edit::id::DESC)
                        .required(true)
                        .takes_value(true)
                        .value_name(edit::id::NAME),
                    Arg::with_name(edit::content::NAME)
                        .help(edit::content::DESC)
                        .required(true)
                        .takes_value(true)
                        .value_name(edit::content::NAME),
                ]),
            SubCommand::with_name(like::NAME).about(like::DESC).arg(
                Arg::with_name(like::id::NAME)
                    .help(like::id::DESC)
                    .required(true)
                    .takes_value(true)
                    .value_name(like::id::NAME),
            ),
            SubCommand::with_name(pin::NAME).about(pin::DESC).arg(
                Arg::with_name(pin::id::NAME)
                    .help(pin::id::DESC)
                    .required(true)
                    .takes_value(true)
                    .value_name(pin::id::NAME),
            ),
            SubCommand::with_name(remove::NAME).about(remove::DESC).arg(
                Arg::with_name(remove::id::NAME)
                    .help(remove::id::DESC)
                    .required(true)
                    .takes_value(true)
                    .value_name(remove::id::NAME),
            ),
        ])
}
