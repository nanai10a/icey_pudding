use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;

use anyhow::{bail, Result};
use serde_json::{json, Number, Value};
use serenity::builder::CreateEmbed;
use serenity::model::id::{ChannelId, GuildId, MessageId};
use serenity::model::interactions::application_command::ApplicationCommandInteractionData;
use serenity::utils::Colour;
use uuid::Uuid;

use super::{clapcmd, command_strs, Command, MsgCommand, Response};
use crate::entities::{Content, User};

pub async fn parse_ia(acid: &ApplicationCommandInteractionData) -> Result<Command> {
    use crate::extract_option;

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

pub async fn parse_msg(msg: &str) -> Result<MsgCommand> {
    let splitted = shell_words::split(msg)?;

    if let Some(n) = splitted.get(0) {
        if n != command_strs::PREFIX {
            bail!("not command, abort.")
        }
    }

    let ams = match clapcmd::create_clap_app().get_matches_from_safe(splitted) {
        Ok(o) => o,
        Err(e) => return Ok(MsgCommand::Showing(e.message)),
    };

    use command_strs::*;

    use crate::{extract_clap_arg, extract_clap_sams};

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

pub fn resp_from_user(
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

pub fn resp_from_content(
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

pub fn append_message_reference(
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
