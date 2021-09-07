use std::collections::HashMap;
use std::fmt::Debug;
use std::io::Cursor;
use std::num::NonZeroU32;
use std::ops::Bound;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use clap::ErrorKind;
use regex::Regex;
use serde::de::DeserializeOwned;
use serde_json::{json, Number, Value};
use serenity::builder::CreateEmbed;
use serenity::model::id::{ChannelId, GuildId, MessageId};
use serenity::utils::Colour;

use super::{clapcmd, Command, ContentCommand, Response, UserCommand};
use crate::conductors::PartialContentMutation;
use crate::entities::{Content, ContentId, PartialAuthor, User};
use crate::repositories::{
    AuthorQuery, ContentContentMutation, ContentQuery, PostedQuery, UserMutation, UserQuery,
};
use crate::utils::LetChain;

pub(crate) async fn parse_msg(msg: &str) -> Option<Result<Command, String>> {
    let res: Result<_> = try {
        let splitted = shell_words::split(msg)?;

        if let Some(n) = splitted.get(0) {
            if n != "*ip" {
                return None;
            }
        }

        let ams0 = match clapcmd::create_clap_app().get_matches_from_safe(splitted) {
            Ok(o) => o,
            Err(e) => match e.kind {
                ErrorKind::VersionDisplayed => Err(anyhow!({
                    let mut buf = Cursor::new(vec![]);
                    clapcmd::create_clap_app()
                        .write_long_version(&mut buf)
                        .unwrap();
                    String::from_utf8(buf.into_inner()).unwrap()
                }))?,
                _ => Err(anyhow!(e))?,
            },
        };

        let mut errs = vec![];
        let cmd = match ams0.subcommand() {
            ("user", Some(ams1)) => Command::User(match ams1.subcommand() {
                ("create", Some(_)) => UserCommand::Create,
                ("read", Some(ams2)) => {
                    let id = ams2
                        .value_of("id")
                        .map(|s| parse_num::<u64>(s, &mut errs).into());

                    UserCommand::Read { id }
                },
                ("reads", Some(ams2)) => {
                    let page = ams2
                        .value_of("page")
                        .map(|s| {
                            NonZeroU32::new(parse_num::<u32>(s, &mut errs)).unwrap_or_else(|| {
                                errs.push("page is not accept `0`".to_string());
                                NonZeroU32::new(1).unwrap() // tmp value
                            })
                        })
                        .unwrap_or_else(|| NonZeroU32::new(1).unwrap());
                    let mut query = Default::default();

                    let UserQuery {
                        posted,
                        posted_num,
                        bookmark,
                        bookmark_num,
                    } = &mut query;
                    *posted = ams2
                        .value_of("posted")
                        .map(|s| parse_array(s, &mut errs).drain(..).collect());
                    *posted_num = ams2
                        .value_of("posted_num")
                        .map(|s| parse_range(s, &mut errs));
                    *bookmark = ams2
                        .value_of("bookmark")
                        .map(|s| parse_array(s, &mut errs).drain(..).collect());
                    *bookmark_num = ams2
                        .value_of("bookmark_num")
                        .map(|s| parse_range(s, &mut errs));

                    UserCommand::Reads { page, query }
                },
                ("update", Some(ams2)) => {
                    let id = ams2
                        .value_of("id")
                        .map(|s| parse_num::<u64>(s, &mut errs).into())
                        .unwrap();
                    let mut mutation = Default::default();

                    let UserMutation { admin, sub_admin } = &mut mutation;
                    *admin = ams2.value_of("admin").map(|s| parse_bool(s, &mut errs));
                    *sub_admin = ams2.value_of("sub_admin").map(|s| parse_bool(s, &mut errs));

                    UserCommand::Update { id, mutation }
                },
                _ => return None,
            }),
            ("content", Some(ams1)) => Command::Content(match ams1.subcommand() {
                ("read", Some(ams2)) => {
                    let id = ams2
                        .value_of("id")
                        .map(|s| parse_uuid(s, &mut errs))
                        .unwrap()
                        .let_(ContentId);

                    ContentCommand::Read { id }
                },
                ("reads", Some(ams2)) => {
                    let page = ams2
                        .value_of("page")
                        .map(|s| {
                            NonZeroU32::new(parse_num::<u32>(s, &mut errs)).unwrap_or_else(|| {
                                errs.push("page is not accept `0`".to_string());
                                NonZeroU32::new(1).unwrap() // tmp value
                            })
                        })
                        .unwrap_or_else(|| NonZeroU32::new(1).unwrap());
                    let mut query = Default::default();

                    let ContentQuery {
                        author,
                        posted,
                        content,
                        liked,
                        liked_num,
                        pinned,
                        pinned_num,
                    } = &mut query;

                    *author = ams2
                        .values_of("author")
                        .map(|vs| vs.collect::<Vec<_>>())
                        .map(|mut v| match v.len() {
                            2 => (v.remove(0), v.remove(0)),
                            l => unreachable!(
                                "illegal args (expected: 2, found: {}) (impl error)",
                                l
                            ),
                        })
                        .map(|(ty, val)| {
                            match ty {
                                "id" =>
                                    AuthorQuery::UserId(parse_num::<u64>(val, &mut errs).into()),
                                "name" => AuthorQuery::UserName(parse_regex(val, &mut errs)),
                                "nick" => AuthorQuery::UserNick(parse_regex(val, &mut errs)),
                                "virt" => AuthorQuery::Virtual(parse_regex(val, &mut errs)),
                                "any" => AuthorQuery::Any(parse_regex(val, &mut errs)),
                                s => {
                                    errs.push(format!("unrecognized author_query type: {}", s));
                                    AuthorQuery::UserId(0.into()) // tmp value
                                },
                            }
                        });
                    *posted = ams2
                        .values_of("posted")
                        .map(|vs| vs.collect::<Vec<_>>())
                        .map(|mut v| match v.len() {
                            2 => (v.remove(0), v.remove(0)),
                            l => unreachable!(
                                "illegal args (expected: 2, found: {}) (impl error)",
                                l
                            ),
                        })
                        .map(|(ty, val)| match ty {
                            "id" => PostedQuery::UserId(parse_num::<u64>(val, &mut errs).into()),
                            "name" => PostedQuery::UserName(parse_regex(val, &mut errs)),
                            "nick" => PostedQuery::UserNick(parse_regex(val, &mut errs)),
                            "any" => PostedQuery::Any(parse_regex(val, &mut errs)),
                            s => {
                                errs.push(format!("unrecognized posted_query type: {}", s));

                                PostedQuery::UserId(0.into()) // tmp value
                            },
                        });
                    *content = ams2.value_of("content").map(|s| parse_regex(s, &mut errs));
                    *liked = ams2
                        .value_of("liked")
                        .map(|s| parse_array(s, &mut errs).drain(..).collect());
                    *liked_num = ams2
                        .value_of("liked_num")
                        .map(|s| parse_range(s, &mut errs));
                    *pinned = ams2
                        .value_of("pinned")
                        .map(|s| parse_array(s, &mut errs).drain(..).collect());
                    *pinned_num = ams2
                        .value_of("pinned_num")
                        .map(|s| parse_range(s, &mut errs));

                    ContentCommand::Reads { page, query }
                },
                ("update", Some(ams2)) => {
                    let id = ams2
                        .value_of("id")
                        .map(|s| parse_uuid(s, &mut errs))
                        .unwrap()
                        .let_(ContentId);
                    let mut mutation = Default::default();

                    let PartialContentMutation { author, content } = &mut mutation;
                    *author = ams2
                        .values_of("author")
                        .map(|vs| vs.collect::<Vec<_>>())
                        .map(|mut v| match v.len() {
                            2 => (v.remove(0), v.remove(0)),
                            l => unreachable!(
                                "illegal args (expected: 2, found: {}) (impl error)",
                                l
                            ),
                        })
                        .map(|(ty, val)| match ty {
                            "user" => PartialAuthor::User(parse_num::<u64>(val, &mut errs).into()),
                            "virt" => PartialAuthor::Virtual(val.to_string()),
                            s => {
                                errs.push(format!("unrecognized author_mutation type: {}", s));

                                PartialAuthor::User(0.into()) // tmp value
                            },
                        });
                    *content = ams2
                        .value_of("complete")
                        .map(|s| ContentContentMutation::Complete(s.to_string()));
                    *content = ams2
                        .values_of("sed")
                        .map(|vs| vs.collect::<Vec<_>>())
                        .map(|mut v| match v.len() {
                            2 => (v.remove(0), v.remove(0)),
                            l => unreachable!(
                                "illegal args (expected: 2, found: {}) (impl error)",
                                l
                            ),
                        })
                        .map(|(val0, val1)| ContentContentMutation::Sed {
                            capture: parse_regex(val0, &mut errs),
                            replace: val1.to_string(),
                        });

                    ContentCommand::Update { id, mutation }
                },
                ("delete", Some(ams2)) => {
                    let id = ams2
                        .value_of("id")
                        .map(|s| parse_uuid(s, &mut errs))
                        .unwrap()
                        .let_(ContentId);

                    ContentCommand::Delete { id }
                },
                _ => return None,
            }),
            ("post", Some(ams1)) => {
                let author = ams1
                    .values_of("author")
                    .map(|vs| vs.collect::<Vec<_>>())
                    .map(|mut v| match v.len() {
                        2 => (v.remove(0), v.remove(0)),
                        l => unreachable!("illegal args (expected: 2, found: {}) (impl error)", l),
                    })
                    .map(|(ty, val)| match ty {
                        "user" => PartialAuthor::User(parse_num::<u64>(val, &mut errs).into()),
                        "virt" => PartialAuthor::Virtual(val.to_string()),
                        s => {
                            errs.push(format!("unrecognized post_author type: {}", s));

                            PartialAuthor::User(0.into()) // tmp value
                        },
                    })
                    .unwrap();
                let content = ams1.value_of("content").map(|s| s.to_string()).unwrap();

                Command::Post { author, content }
            },
            ("like", Some(ams1)) => {
                let content_id = ams1
                    .value_of("content_id")
                    .map(|s| parse_uuid(s, &mut errs))
                    .unwrap()
                    .let_(ContentId);
                let undo = ams1.values_of("undo").is_some();

                Command::Like { content_id, undo }
            },
            ("pin", Some(ams1)) => {
                let content_id = ams1
                    .value_of("content_id")
                    .map(|s| parse_uuid(s, &mut errs))
                    .unwrap()
                    .let_(ContentId);
                let undo = ams1.values_of("undo").is_some();

                Command::Pin { content_id, undo }
            },
            ("bookmark", Some(ams1)) => {
                let content_id = ams1
                    .value_of("content_id")
                    .map(|s| parse_uuid(s, &mut errs))
                    .unwrap()
                    .let_(ContentId);
                let undo = ams1.values_of("undo").is_some();

                Command::Bookmark { content_id, undo }
            },
            _ => return None,
        };
        if !errs.is_empty() {
            Err(anyhow!(combine_errs(errs)))?
        }

        cmd
    };

    let tmp = match res {
        Ok(o) => Ok(o),
        Err(e) => Err(e.to_string()),
    };

    Some(tmp)
}

pub(crate) fn resp_from_user(
    title: impl ToString,
    description: impl ToString,
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
        title: title.to_string(),
        rgb,
        description: description.to_string(),
        fields: vec![
            ("id:".to_string(), id.to_string()),
            ("is_admin?".to_string(), admin.to_string()),
            ("is_sub_admin?".to_string(), sub_admin.to_string()),
            ("posted:".to_string(), posted.len().to_string()),
            ("bookmarked:".to_string(), bookmark.len().to_string()),
        ],
    }
}

pub(crate) fn resp_from_content(
    title: impl ToString,
    description: impl ToString,
    rgb: (u8, u8, u8),
    Content {
        id,
        content,
        author,
        posted,
        liked,
        pinned,
    }: Content,
) -> Response {
    Response {
        title: title.to_string(),
        rgb,
        description: description.to_string(),
        fields: vec![
            ("id:".to_string(), id.to_string()),
            ("author".to_string(), author.to_string()),
            ("posted".to_string(), posted.to_string()),
            ("content:".to_string(), content),
            ("liked:".to_string(), liked.len().to_string()),
            ("pinned:".to_string(), pinned.len().to_string()),
        ],
    }
}

pub(crate) fn build_embed_from_resp(
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
                .map(|(s1, s2)| (s1, s2, true))
                .collect::<Vec<_>>(),
        )
}

pub(crate) fn append_message_reference(
    raw: &mut HashMap<&str, Value>,
    id: MessageId,
    channel_id: ChannelId,
    guild_id: Option<GuildId>,
) {
    let mr = json!({
        "message_id": id,
        "channel_id": channel_id,
        "guild_id": match guild_id {
            Some(GuildId(i)) => Value::Number(Number::from(i)),
            None => Value::Null
        },
    });

    raw.insert("message_reference", mr);
}

#[inline]
fn combine_errs(mut errs: Vec<String>) -> String {
    let mut s = vec![];
    let len = errs.len();
    errs.drain(..)
        .enumerate()
        .map(|(i, s)| match (i + 1) == len {
            true => format!("err ({}): {}", i, s),
            false => format!("err ({}): {}\n", i, s),
        })
        .map(|v| v.into_bytes())
        .for_each(|mut v| s.append(&mut v));

    String::from_utf8(s).unwrap()
}

#[inline]
fn parse_num<N>(s: &str, errs: &mut Vec<String>) -> N
where
    N: Default + FromStr,
    <N as FromStr>::Err: ToString,
{
    match s.parse() {
        Ok(o) => o,
        Err(e) => {
            errs.push(e.to_string());
            Default::default() // tmp value
        },
    }
}

#[inline]
fn parse_range<N>(s: &str, errs: &mut Vec<String>) -> (Bound<N>, Bound<N>)
where
    N: range_parser::Num + Default + FromStr + Debug,
    <N as FromStr>::Err: Debug + PartialEq + Eq,
{
    match range_parser::parse(s.to_string()).map_err(|e| anyhow::anyhow!("{:?}", e)) {
        Ok(o) => o,
        Err(e) => {
            errs.push(e.to_string());
            (Bound::Unbounded, Bound::Unbounded) // tmp value
        },
    }
}

#[inline]
fn parse_array<T>(s: &str, errs: &mut Vec<String>) -> Vec<T>
where T: DeserializeOwned {
    match serde_json::from_str(s) {
        Ok(o) => o,
        Err(e) => {
            errs.push(e.to_string());
            Default::default() // tmp value
        },
    }
}

#[inline]
fn parse_uuid(s: &str, errs: &mut Vec<String>) -> ::uuid::Uuid {
    match s.parse() {
        Ok(o) => o,
        Err(e) => {
            errs.push(e.to_string());
            Default::default() // tmp value
        },
    }
}

#[inline]
fn parse_bool(s: &str, errs: &mut Vec<String>) -> bool {
    match s.parse() {
        Ok(o) => o,
        Err(e) => {
            errs.push(e.to_string());
            Default::default() // tmp value
        },
    }
}

#[inline]
fn parse_regex(s: &str, errs: &mut Vec<String>) -> Regex {
    match s.parse() {
        Ok(o) => o,
        Err(e) => {
            errs.push(e.to_string());
            "".parse().unwrap() // tmp value
        },
    }
}
