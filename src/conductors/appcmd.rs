use anyhow::Result;
use serde_json::Value;
use serenity::builder::CreateApplicationCommands;
use serenity::http::Http;
use serenity::model::id::GuildId;
use serenity::model::interactions::application_command::{
    ApplicationCommand, ApplicationCommandOptionType,
};

use super::command_strs::*;

#[deprecated]
pub async fn application_command_create(
    http: impl AsRef<Http>,
    guild_id: Option<GuildId>,
) -> Result<Vec<ApplicationCommand>> {
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
