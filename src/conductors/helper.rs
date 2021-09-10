use core::num::NonZeroU32;
use std::collections::HashSet;

use anyhow::{anyhow, Result};
use clap::ErrorKind;
use regex::Regex;
use serde::de::DeserializeOwned;
use serde_json::{json, Number, Value};
use serenity::builder::CreateEmbed;
use serenity::model::id::{ChannelId, GuildId, MessageId};
use serenity::utils::Colour;
use uuid::Uuid;

use super::{clapcmd, Command, ContentCommand, Response, UserCommand};
use crate::conductors::PartialContentMutation;
use crate::entities::{Content, ContentId, PartialAuthor, User, UserId};
use crate::repositories::{
    AuthorQuery, ContentContentMutation, ContentQuery, PostedQuery, UserMutation, UserQuery,
};
use crate::utils::{self, LetChain};

/// this is a ICEy_PUDDING.
#[derive(Debug, Clone, ::clap::Clap)]
#[clap(author, version)]
struct AppV2_1 {
    #[clap(subcommand)]
    cmd: RootMod,
}

#[derive(Debug, Clone, ::clap::Clap)]
enum RootMod {
    /// about user.
    #[clap(short_flag = 'U')]
    User {
        #[clap(subcommand)]
        cmd: UserMod,
    },

    /// about content.
    #[clap(short_flag = 'C')]
    Content {
        #[clap(subcommand)]
        cmd: ContentMod,
    },
}

#[derive(Debug, Clone, ::clap::Clap)]
enum UserMod {
    #[clap(short_flag = 'c')]
    Register(UserRegisterCmd),

    #[clap(short_flag = 'g')]
    Get(UserGetCmd),

    #[clap(short_flag = 'q')]
    Gets(UserGetsCmd),

    #[clap(short_flag = 'e')]
    Edit(UserEditCmd),

    #[clap(short_flag = 'b')]
    Bookmark(UserBookmarkCmd),

    #[clap(short_flag = 'd')]
    Unregister(UserUnregisterCmd),
}

#[derive(Debug, Clone, ::clap::Clap)]
enum ContentMod {
    #[clap(short_flag = 'c')]
    Post(ContentPostCmd),

    #[clap(short_flag = 'g')]
    Get(ContentGetCmd),

    #[clap(short_flag = 'q')]
    Gets(ContentGetsCmd),

    #[clap(short_flag = 'e')]
    Edit(ContentEditCmd),

    #[clap(short_flag = 'l')]
    Like(ContentLikeCmd),

    #[clap(short_flag = 'p')]
    Pin(ContentPinCmd),

    #[clap(short_flag = 'd')]
    Withdraw(ContentWithdrawCmd),
}

/// register user with executed user's id.
#[derive(Debug, Clone, ::clap::Clap)]
struct UserRegisterCmd;

/// get user with id.
/// if not given id, fallback to executed user's id.
#[derive(Debug, Clone, ::clap::Clap)]
struct UserGetCmd {
    /// u64,
    #[clap(name = "USER_ID")]
    user_id: Option<u64>,
}

/// get users with query.
#[derive(Debug, Clone, ::clap::Clap)]
struct UserGetsCmd {
    /// u32 (1 =< n)
    #[clap(name = "PAGE", default_value = "1", parse(try_from_str = parse_nonzero_num))]
    page: u32,

    /// json
    ///
    /// schema: {}
    #[clap(name = "QUERY", default_value = "{}", parse(try_from_str = parse_user_query))]
    query: UserQuery,
}

/// edit user with id and mutation.
#[derive(Debug, Clone, ::clap::Clap)]
struct UserEditCmd {
    /// u64
    #[clap(name = "USER_ID")]
    user_id: u64,

    /// json
    ///
    /// schema: {}
    #[clap(name = "MUTATION", default_value = "{}", parse(try_from_str = parse_user_mutation))]
    mutation: UserMutation,
}

/// about executed user's bookmark.
#[derive(Debug, Clone, ::clap::Clap)]
struct UserBookmarkCmd {
    #[clap(subcommand)]
    op: UserBookmarkOp,
}

#[derive(Debug, Clone, ::clap::Clap)]
enum UserBookmarkOp {
    /// bookmark content.
    #[clap(short_flag = 'd')]
    Do {
        /// uuid
        #[clap(name = "CONTENT_ID")]
        content_id: Uuid,
    },

    /// unbookmark content.
    #[clap(short_flag = 'u')]
    Undo {
        /// uuid
        #[clap(name = "CONTENT_ID")]
        content_id: Uuid,
    },

    /// get bookmarks.
    #[clap(short_flag = 's')]
    Show {
        /// u32 (1 =< n)
        #[clap(name = "PAGE", default_value = "1", parse(try_from_str = parse_nonzero_num))]
        page: u32,
    },
}

/// unregister user with executed user's id.
#[derive(Debug, Clone, ::clap::Clap)]
struct UserUnregisterCmd {
    /// u64
    #[clap(name = "USER_ID")]
    user_id: u64,
}

/// post content with executed user's id.
#[derive(Debug, Clone, ::clap::Clap)]
#[clap(group = ::clap::ArgGroup::new("author").required(true))]
struct ContentPostCmd {
    /// str
    #[clap(short = 'v', long, group = "author")]
    virt: Option<String>,

    /// u64
    #[clap(short = 'u', long, group = "author")]
    user_id: Option<u64>,

    /// str
    #[clap(short = 'c', long)]
    content: String,
}

/// get content with id.
#[derive(Debug, Clone, ::clap::Clap)]
struct ContentGetCmd {
    /// uuid
    #[clap(name = "CONTENT_ID")]
    content_id: Uuid,
}

/// get contents with query.
#[derive(Debug, Clone, ::clap::Clap)]
struct ContentGetsCmd {
    /// u32 (1 =< n)
    #[clap(name = "PAGE", default_value = "1", parse(try_from_str = parse_nonzero_num))]
    page: u32,

    /// json
    ///
    /// schema: {}
    #[clap(name = "QUERY", default_value = "{}", parse(try_from_str = parse_content_query))]
    query: ContentQuery,
}

/// edit content with id and mutation.
#[derive(Debug, Clone, ::clap::Clap)]
struct ContentEditCmd {
    /// uuid
    #[clap(name = "CONTENT_ID")]
    content_id: Uuid,

    /// json
    ///
    /// schema: {}
    #[clap(name = "MUTATION", default_value = "{}", parse(try_from_str = parse_partial_content_mutation))]
    mutation: PartialContentMutation,
}

#[derive(Debug, Clone, ::clap::Clap)]
struct ContentLikeCmd {
    #[clap(subcommand)]
    op: ContentLikeOp,
}

/// about like with executed user.
#[derive(Debug, Clone, ::clap::Clap)]
enum ContentLikeOp {
    /// like content.
    #[clap(short_flag = 'd')]
    Do {
        /// uuid
        #[clap(name = "CONTENT_ID")]
        content_id: Uuid,
    },

    /// unlike content.
    #[clap(short_flag = 'u')]
    Undo {
        /// uuid
        #[clap(name = "CONTENT_ID")]
        content_id: Uuid,
    },

    /// get liked users.
    #[clap(short_flag = 's')]
    Show {
        /// u32 (1 =< n)
        #[clap(name = "PAGE", default_value = "1", parse(try_from_str = parse_nonzero_num))]
        page: u32,

        /// uuid
        #[clap(name = "CONTENT_ID")]
        content_id: Uuid,
    },
}

/// about pin with executed user.
#[derive(Debug, Clone, ::clap::Clap)]
struct ContentPinCmd {
    #[clap(subcommand)]
    op: ContentPinOp,
}

#[derive(Debug, Clone, ::clap::Clap)]
enum ContentPinOp {
    /// pin content.
    #[clap(short_flag = 'd')]
    Do {
        /// uuid
        #[clap(name = "CONTENT_ID")]
        content_id: Uuid,
    },

    /// unpin content.
    #[clap(short_flag = 'u')]
    Undo {
        /// uuid
        #[clap(name = "CONTENT_ID")]
        content_id: Uuid,
    },

    /// get pinned users.
    #[clap(short_flag = 's')]
    Show {
        /// u32 (1 =< n)
        #[clap(name = "PAGE", default_value = "1", parse(try_from_str = parse_nonzero_num))]
        page: u32,

        /// uuid
        #[clap(name = "CONTENT_ID")]
        content_id: Uuid,
    },
}

/// withdraw content with id.
#[derive(Debug, Clone, ::clap::Clap)]
struct ContentWithdrawCmd {
    /// uuid
    #[clap(name = "CONTENT_ID")]
    content_id: Uuid,
}

pub async fn parse_msg(msg: &str) -> Option<Result<Command, String>> {
    let res: Result<_> = try {
        let splitted = shell_words::split(msg)?;

        if let Some(n) = splitted.get(0) {
            if n != "*ip" {
                return None;
            }
        }

        let ams0 = match clapcmd::create_clap_app().try_get_matches_from(splitted) {
            Ok(o) => o,
            Err(e) => match e.kind {
                ErrorKind::DisplayVersion => Err(anyhow!(CLAP_VERSION.clone()))?,
                _ => Err(anyhow!(e.to_string()))?, // FIXME: is ok?
            },
        };

        let mut errs = vec![];
        let cmd = match ams0.subcommand() {
            Some(("user", ams1)) => Command::User(match ams1.subcommand() {
                Some(("create", _)) => UserCommand::Create,
                Some(("read", ams2)) => {
                    let id = ams2
                        .value_of("id")
                        .map(|s| parse_num::<u64>(s, &mut errs).into());

                    UserCommand::Read { id }
                },
                Some(("reads", ams2)) => {
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
                        bookmark,
                        bookmark_num,
                    } = &mut query;
                    *bookmark = ams2
                        .value_of("bookmark")
                        .map(|s| parse_array(s, &mut errs).drain(..).collect());
                    *bookmark_num = ams2
                        .value_of("bookmark_num")
                        .map(|s| parse_range(s, &mut errs));

                    UserCommand::Reads { page, query }
                },
                Some(("update", ams2)) => {
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
                Some(("delete", ams2)) => {
                    let id = ams2
                        .value_of("id")
                        .map(|s| parse_num::<u64>(s, &mut errs).into())
                        .unwrap();

                    UserCommand::Delete { id }
                },
                _ => Err(anyhow!(CLAP_HELP.clone()))?,
            }),
            Some(("content", ams1)) => Command::Content(match ams1.subcommand() {
                Some(("read", ams2)) => {
                    let id = ams2
                        .value_of("id")
                        .map(|s| parse_uuid(s, &mut errs))
                        .unwrap()
                        .let_(ContentId);

                    ContentCommand::Read { id }
                },
                Some(("reads", ams2)) => {
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
                Some(("update", ams2)) => {
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
                Some(("delete", ams2)) => {
                    let id = ams2
                        .value_of("id")
                        .map(|s| parse_uuid(s, &mut errs))
                        .unwrap()
                        .let_(ContentId);

                    ContentCommand::Delete { id }
                },
                _ => Err(anyhow!(CLAP_HELP.clone()))?,
            }),
            Some(("post", ams1)) => {
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
            Some(("like", ams1)) => {
                let content_id = ams1
                    .value_of("content_id")
                    .map(|s| parse_uuid(s, &mut errs))
                    .unwrap()
                    .let_(ContentId);
                let undo = ams1.values_of("undo").is_some();

                Command::Like { content_id, undo }
            },
            Some(("pin", ams1)) => {
                let content_id = ams1
                    .value_of("content_id")
                    .map(|s| parse_uuid(s, &mut errs))
                    .unwrap()
                    .let_(ContentId);
                let undo = ams1.values_of("undo").is_some();

                Command::Pin { content_id, undo }
            },
            Some(("bookmark", ams1)) => {
                let content_id = ams1
                    .value_of("content_id")
                    .map(|s| parse_uuid(s, &mut errs))
                    .unwrap()
                    .let_(ContentId);
                let undo = ams1.values_of("undo").is_some();

                Command::Bookmark { content_id, undo }
            },
            _ => Err(anyhow!(CLAP_HELP.clone()))?,
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

pub fn resp_from_user(
    title: impl ToString,
    description: impl ToString,
    rgb: (u8, u8, u8),
    User {
        id,
        admin,
        sub_admin,
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
            ("bookmarked:".to_string(), bookmark.len().to_string()),
        ],
    }
}

pub fn resp_from_content(
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
        created,
        mut edited,
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
            ("created:".to_string(), created.to_string()),
            (
                "last_edited:".to_string(),
                edited
                    .pop()
                    .map_or_else(|| "no edited".to_string(), utils::date_to_string),
            ),
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
                .map(|(s1, s2)| (s1, s2, true))
                .collect::<Vec<_>>(),
        )
}

pub fn append_message_reference(
    raw: &mut ::std::collections::HashMap<&str, Value>,
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

fn parse_num<N>(s: &str, errs: &mut Vec<String>) -> N
where
    N: Default + ::core::str::FromStr,
    <N as ::core::str::FromStr>::Err: ToString,
{
    match s.parse() {
        Ok(o) => o,
        Err(e) => {
            errs.push(e.to_string());
            Default::default() // tmp value
        },
    }
}

fn parse_range<N>(s: &str, errs: &mut Vec<String>) -> (::core::ops::Bound<N>, ::core::ops::Bound<N>)
where
    N: range_parser::Num + Default + ::core::str::FromStr + ::core::fmt::Debug,
    <N as ::core::str::FromStr>::Err: ::core::fmt::Debug + PartialEq + Eq,
{
    match range_parser::parse(s.to_string()).map_err(|e| anyhow::anyhow!("{:?}", e)) {
        Ok(o) => o,
        Err(e) => {
            errs.push(e.to_string());
            (::core::ops::Bound::Unbounded, ::core::ops::Bound::Unbounded) // tmp value
        },
    }
}

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

fn parse_uuid(s: &str, errs: &mut Vec<String>) -> ::uuid::Uuid {
    match s.parse() {
        Ok(o) => o,
        Err(e) => {
            errs.push(e.to_string());
            Default::default() // tmp value
        },
    }
}

fn parse_bool(s: &str, errs: &mut Vec<String>) -> bool {
    match s.parse() {
        Ok(o) => o,
        Err(e) => {
            errs.push(e.to_string());
            Default::default() // tmp value
        },
    }
}

fn parse_regex(s: &str, errs: &mut Vec<String>) -> Regex {
    match s.parse() {
        Ok(o) => o,
        Err(e) => {
            errs.push(e.to_string());
            "".parse().unwrap() // tmp value
        },
    }
}

fn parse_nonzero_num(
    s: &str,
) -> ::core::result::Result<u32, <NonZeroU32 as ::core::str::FromStr>::Err> {
    Ok(s.parse::<::core::num::NonZeroU32>()?.get())
}

fn parse_user_query(s: &str) -> ::core::result::Result<UserQuery, String> {
    #[derive(::serde::Deserialize)]
    struct UserQueryModel {
        bookmark: Option<HashSet<Uuid>>,
        bookmark_num: Option<String>,
    }

    // --- parsing json ---

    let UserQueryModel {
        bookmark: bookmark_raw,
        bookmark_num: bookmark_num_raw,
    } = serde_json::from_str(s).map_err(|e| e.to_string())?;

    // --- converting ---

    let bookmark = bookmark_raw.map(|mut s| s.drain().map(ContentId).collect());

    let bookmark_num = bookmark_num_raw
        .map(|s| range_parser::parse(s).map_err(|e| format!("{:?}", e)))
        .transpose()?;

    // --- finalize ---

    Ok(UserQuery {
        bookmark,
        bookmark_num,
    })
}

fn parse_user_mutation(s: &str) -> ::core::result::Result<UserMutation, String> {
    #[derive(::serde::Deserialize)]
    struct UserMutationModel {
        admin: Option<bool>,
        sub_admin: Option<bool>,
    }

    // --- parsing json ---

    let UserMutationModel { admin, sub_admin } =
        serde_json::from_str(s).map_err(|e| e.to_string())?;

    // --- finalize ---

    Ok(UserMutation { admin, sub_admin })
}

fn parse_content_query(s: &str) -> ::core::result::Result<ContentQuery, String> {
    #[derive(::serde::Deserialize)]
    struct ContentQueryModel<'a> {
        pub author: Option<AuthorQueryModel<'a>>,
        pub posted: Option<PostedQueryModel<'a>>,
        pub content: Option<&'a str>,
        pub liked: Option<HashSet<u64>>,
        pub liked_num: Option<&'a str>,
        pub pinned: Option<HashSet<u64>>,
        pub pinned_num: Option<&'a str>,
    }
    #[derive(::serde::Deserialize)]
    pub enum AuthorQueryModel<'a> {
        UserId(u64),
        UserName(&'a str),
        UserNick(&'a str),
        Virtual(&'a str),
        Any(&'a str),
    }
    #[derive(::serde::Deserialize)]
    pub enum PostedQueryModel<'a> {
        UserId(u64),
        UserName(&'a str),
        UserNick(&'a str),
        Any(&'a str),
    }

    // --- parsing json ---

    let ContentQueryModel {
        author: author_raw,
        posted: posted_raw,
        content: content_raw,
        liked: liked_raw,
        liked_num: liked_num_raw,
        pinned: pinned_raw,
        pinned_num: pinned_num_raw,
    } = serde_json::from_str(s).map_err(|e| e.to_string())?;

    // --- converting ---

    let author = author_raw
        .map(|m| match m {
            AuthorQueryModel::UserId(n) => n.let_(Ok).map(UserId).map(AuthorQuery::UserId),
            AuthorQueryModel::UserName(s) => Regex::new(s)
                .map(AuthorQuery::UserName)
                .map_err(|e| e.to_string()),
            AuthorQueryModel::UserNick(s) => Regex::new(s)
                .map(AuthorQuery::UserNick)
                .map_err(|e| e.to_string()),
            AuthorQueryModel::Virtual(s) => Regex::new(s)
                .map(AuthorQuery::Virtual)
                .map_err(|e| e.to_string()),
            AuthorQueryModel::Any(s) => Regex::new(s)
                .map(AuthorQuery::Any)
                .map_err(|e| e.to_string()),
        })
        .transpose()?;

    let posted = posted_raw
        .map(|m| match m {
            PostedQueryModel::UserId(n) => n.let_(Ok).map(UserId).map(PostedQuery::UserId),
            PostedQueryModel::UserName(s) => Regex::new(s)
                .map(PostedQuery::UserName)
                .map_err(|e| e.to_string()),
            PostedQueryModel::UserNick(s) => Regex::new(s)
                .map(PostedQuery::UserNick)
                .map_err(|e| e.to_string()),
            PostedQueryModel::Any(s) => Regex::new(s)
                .map(PostedQuery::Any)
                .map_err(|e| e.to_string()),
        })
        .transpose()?;

    let content = content_raw
        .map(|s| Regex::new(s).map_err(|e| e.to_string()))
        .transpose()?;

    let liked = liked_raw.map(|mut s| s.drain().map(UserId).collect());

    let liked_num = liked_num_raw
        .map(|s| range_parser::parse(s.to_string()).map_err(|e| e.to_string()))
        .transpose()?;

    let pinned = pinned_raw.map(|mut s| s.drain().map(UserId).collect());

    let pinned_num = pinned_num_raw
        .map(|s| range_parser::parse(s.to_string()).map_err(|e| e.to_string()))
        .transpose()?;

    // --- finalize ---

    Ok(ContentQuery {
        author,
        posted,
        content,
        liked,
        liked_num,
        pinned,
        pinned_num,
    })
}

fn parse_partial_content_mutation(
    s: &str,
) -> ::core::result::Result<PartialContentMutation, String> {
    #[derive(::serde::Deserialize)]
    struct PartialContentMutationModel {
        author: Option<PartialAuthorModel>,
        content: Option<ContentContentMutationModel>,
    }
    #[derive(::serde::Deserialize)]
    enum PartialAuthorModel {
        User(u64),
        Virtual(String),
    }
    #[derive(::serde::Deserialize)]
    enum ContentContentMutationModel {
        Complete(String),
        Sed { capture: String, replace: String },
    }

    // --- parsing json ---

    let PartialContentMutationModel {
        author: author_raw,
        content: content_raw,
    } = serde_json::from_str(s).map_err(|e| e.to_string())?;

    // --- converting ---

    let author = author_raw.map(|m| match m {
        PartialAuthorModel::User(n) => n.let_(UserId).let_(PartialAuthor::User),
        PartialAuthorModel::Virtual(s) => s.let_(PartialAuthor::Virtual),
    });

    let content = content_raw
        .map(|m| match m {
            ContentContentMutationModel::Complete(s) =>
                s.let_(ContentContentMutation::Complete).let_(Ok),
            ContentContentMutationModel::Sed {
                capture: capture_raw,
                replace,
            } => (&capture_raw)
                .let_(|s| s.as_str())
                .let_(Regex::new)
                .map(|capture| ContentContentMutation::Sed { capture, replace })
                .map_err(|e| e.to_string()),
        })
        .transpose()?;

    // --- finalize ---

    Ok(PartialContentMutation { author, content })
}

lazy_static::lazy_static! {
    static ref CLAP_HELP: String = {
        let mut buf = ::std::io::Cursor::new(vec![]);
        clapcmd::create_clap_app()
            .write_long_help(&mut buf)
            .unwrap();

        String::from_utf8(buf.into_inner()).unwrap()
    };

    static ref CLAP_VERSION: String = clapcmd::create_clap_app().render_long_version();
}
