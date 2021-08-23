use anyhow::{bail, Result};
use clap::{App, AppSettings, Arg, ArgMatches, SubCommand};

pub fn create_clap_app() -> App<'static, 'static> {
    use super::command_strs::*;

    App::new(PREFIX)
        .global_setting(AppSettings::ColorNever)
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
            SubCommand::with_name(get::NAME)
                .about(get::DESC)
                .args(&vec![
                    Arg::with_name(get::id::NAME)
                        .long(get::id::NAME)
                        .help(get::id::DESC)
                        .takes_value(true)
                        .value_name(get::id::NAME),
                    Arg::with_name(get::author::NAME)
                        .long(get::author::NAME)
                        .help(get::author::DESC)
                        .takes_value(true)
                        .value_name(get::author::NAME),
                    Arg::with_name(get::posted::NAME)
                        .long(get::posted::NAME)
                        .help(get::posted::DESC)
                        .takes_value(true)
                        .value_name(get::posted::NAME),
                    Arg::with_name(get::content::NAME)
                        .long(get::content::NAME)
                        .help(get::content::DESC)
                        .takes_value(true)
                        .value_name(get::content::NAME),
                    Arg::with_name(get::liked::NAME)
                        .long(get::liked::NAME)
                        .help(get::liked::DESC)
                        .takes_value(true)
                        .value_name(get::liked::NAME),
                    Arg::with_name(get::bookmarked::NAME)
                        .long(get::bookmarked::NAME)
                        .help(get::bookmarked::DESC)
                        .takes_value(true)
                        .value_name(get::bookmarked::NAME),
                    Arg::with_name(get::pinned::NAME)
                        .long(get::pinned::NAME)
                        .help(get::pinned::DESC)
                        .takes_value(true)
                        .value_name(get::pinned::NAME),
                    Arg::with_name(get::page::NAME)
                        .long(get::page::NAME)
                        .short(get::page::S_NAME)
                        .help(get::page::DESC)
                        .takes_value(true)
                        .required(true)
                        .value_name(get::page::NAME),
                ]),
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

#[inline]
pub fn extract_clap_sams<'a>(ams: &'a ArgMatches, name: &str) -> Result<&'a ArgMatches<'a>> {
    match ams.subcommand_matches(name) {
        Some(s) => Ok(s),
        None => bail!("cannot get arg_matches: {}", name),
    }
}

#[inline]
pub fn extract_clap_arg<'a>(ams: &'a ArgMatches, name: &str) -> Result<&'a str> {
    match ams.value_of(name) {
        Some(s) => Ok(s),
        None => bail!("cannot get arg: {}", name),
    }
}
