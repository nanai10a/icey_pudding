use anyhow::{bail, Result};
use clap::{load_yaml, App, AppSettings, Arg, ArgMatches, SubCommand};

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
                        .value_name(change::admin::NAME),
                    Arg::with_name(change::sub_admin::NAME)
                        .help(change::sub_admin::DESC)
                        .value_name(change::sub_admin::NAME),
                ]),
            SubCommand::with_name(bookmark::NAME)
                .about(bookmark::DESC)
                .args(&vec![
                    Arg::with_name(bookmark::id::NAME)
                        .help(bookmark::id::DESC)
                        .value_name(bookmark::id::NAME)
                        .required(true),
                    Arg::with_name(bookmark::undo::NAME)
                        .long(bookmark::undo::NAME)
                        .help(bookmark::undo::DESC),
                ]),
            SubCommand::with_name(delete_me::NAME).about(delete_me::DESC),
            SubCommand::with_name(post::NAME)
                .about(post::DESC)
                .args(&vec![
                    Arg::with_name(post::author::NAME)
                        .help(post::author::DESC)
                        .value_name(post::author::NAME)
                        .required(true),
                    Arg::with_name(post::content::NAME)
                        .help(post::content::DESC)
                        .value_name(post::content::NAME)
                        .required(true),
                ]),
            SubCommand::with_name(get::NAME)
                .about(get::DESC)
                .args(&vec![
                    Arg::with_name(get::page::NAME)
                        .help(get::page::DESC)
                        .value_name(get::page::NAME)
                        .required(true),
                    Arg::with_name(get::id::NAME)
                        .long(get::id::NAME)
                        .help(get::id::DESC)
                        .value_name(get::id::NAME),
                    Arg::with_name(get::author::NAME)
                        .long(get::author::NAME)
                        .help(get::author::DESC)
                        .value_name(get::author::NAME),
                    Arg::with_name(get::posted::NAME)
                        .long(get::posted::NAME)
                        .help(get::posted::DESC)
                        .value_name(get::posted::NAME),
                    Arg::with_name(get::content::NAME)
                        .long(get::content::NAME)
                        .help(get::content::DESC)
                        .value_name(get::content::NAME),
                    Arg::with_name(get::liked::NAME)
                        .long(get::liked::NAME)
                        .help(get::liked::DESC)
                        .value_name(get::liked::NAME),
                    Arg::with_name(get::bookmarked::NAME)
                        .long(get::bookmarked::NAME)
                        .help(get::bookmarked::DESC)
                        .value_name(get::bookmarked::NAME),
                    Arg::with_name(get::pinned::NAME)
                        .long(get::pinned::NAME)
                        .help(get::pinned::DESC)
                        .value_name(get::pinned::NAME),
                ]),
            SubCommand::with_name(edit::NAME)
                .about(edit::DESC)
                .args(&vec![
                    Arg::with_name(edit::id::NAME)
                        .help(edit::id::DESC)
                        .value_name(edit::id::NAME)
                        .required(true),
                    Arg::with_name(edit::content::NAME)
                        .help(edit::content::DESC)
                        .value_name(edit::content::NAME)
                        .required(true),
                ]),
            SubCommand::with_name(like::NAME)
                .about(like::DESC)
                .args(&vec![
                    Arg::with_name(like::id::NAME)
                        .help(like::id::DESC)
                        .value_name(like::id::NAME)
                        .required(true),
                    Arg::with_name(like::undo::NAME)
                        .long(like::undo::NAME)
                        .help(like::undo::DESC),
                ]),
            SubCommand::with_name(pin::NAME)
                .about(pin::DESC)
                .args(&vec![
                    Arg::with_name(pin::id::NAME)
                        .help(pin::id::DESC)
                        .value_name(pin::id::NAME)
                        .required(true),
                    Arg::with_name(pin::undo::NAME)
                        .long(pin::undo::NAME)
                        .help(pin::undo::DESC),
                ]),
            SubCommand::with_name(remove::NAME).about(remove::DESC).arg(
                Arg::with_name(remove::id::NAME)
                    .help(remove::id::DESC)
                    .value_name(remove::id::NAME),
            ),
        ])
}

pub fn create_clap_app_v2() -> App<'static, 'static> {
    let yml = load_yaml!("clap.yml");
    App::from_yaml(yml).version(env!("CARGO_PKG_VERSION"))
}

#[test]
fn load_clap_yaml() {
    create_clap_app_v2()
        .get_matches_from_safe(vec!["*ip", "--help"])
        .unwrap();
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
