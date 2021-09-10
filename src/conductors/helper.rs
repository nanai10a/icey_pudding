use core::num::NonZeroU32;
use std::collections::HashSet;

use anyhow::Result;
use regex::Regex;
use serde_json::{json, Number, Value};
use serenity::builder::CreateEmbed;
use serenity::model::id::{ChannelId, GuildId, MessageId};
use serenity::utils::Colour;
use uuid::Uuid;

use super::Response;
use crate::conductors::PartialContentMutation;
use crate::entities::{Content, ContentId, PartialAuthor, User, UserId};
use crate::repositories::{
    AuthorQuery, ContentContentMutation, ContentQuery, PostedQuery, UserMutation, UserQuery,
};
use crate::utils::{self, LetChain};

/// this is a ICEy_PUDDING.
#[derive(Debug, Clone, ::clap::Clap)]
#[clap(author, version)]
pub struct App {
    #[clap(subcommand)]
    pub cmd: RootMod,
}

#[derive(Debug, Clone, ::clap::Clap)]
pub enum RootMod {
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
pub enum UserMod {
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
pub enum ContentMod {
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
pub struct UserRegisterCmd;

/// get user with id.
/// if not given id, fallback to executed user's id.
#[derive(Debug, Clone, ::clap::Clap)]
pub struct UserGetCmd {
    /// u64,
    #[clap(name = "USER_ID")]
    pub user_id: Option<u64>,
}

/// get users with query.
#[derive(Debug, Clone, ::clap::Clap)]
pub struct UserGetsCmd {
    /// u32 (1 =< n)
    #[clap(name = "PAGE", default_value = "1", parse(try_from_str = parse_nonzero_num))]
    pub page: u32,

    /// json
    ///
    /// schema: {
    ///   bookmark?: [uuid],
    ///   bookmark_num?: range<u32>,
    /// }
    #[clap(name = "QUERY", default_value = "{}", parse(try_from_str = parse_user_query))]
    pub query: UserQuery,
}

/// edit user with id and mutation.
#[derive(Debug, Clone, ::clap::Clap)]
pub struct UserEditCmd {
    /// u64
    #[clap(name = "USER_ID")]
    pub user_id: u64,

    /// json
    ///
    /// schema: {
    ///   admin?: bool,
    ///   sub_admin?: bool,
    /// }
    #[clap(name = "MUTATION", default_value = "{}", parse(try_from_str = parse_user_mutation))]
    pub mutation: UserMutation,
}

/// about executed user's bookmark.
#[derive(Debug, Clone, ::clap::Clap)]
pub struct UserBookmarkCmd {
    #[clap(subcommand)]
    pub op: UserBookmarkOp,
}

#[derive(Debug, Clone, ::clap::Clap)]
pub enum UserBookmarkOp {
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

        /// u64
        #[clap(name = "USER_ID")]
        user_id: Option<u64>,
    },
}

/// unregister user with executed user's id.
#[derive(Debug, Clone, ::clap::Clap)]
pub struct UserUnregisterCmd {
    /// u64
    #[clap(name = "USER_ID")]
    pub user_id: u64,
}

/// post content with executed user's id.
#[derive(Debug, Clone, ::clap::Clap)]
#[clap(group = ::clap::ArgGroup::new("author").required(true))]
pub struct ContentPostCmd {
    /// str
    #[clap(short = 'v', long, group = "author")]
    pub virt: Option<String>,

    /// u64
    #[clap(short = 'u', long, group = "author")]
    pub user_id: Option<u64>,

    /// str
    #[clap(short = 'c', long)]
    pub content: String,
}

/// get content with id.
#[derive(Debug, Clone, ::clap::Clap)]
pub struct ContentGetCmd {
    /// uuid
    #[clap(name = "CONTENT_ID")]
    pub content_id: Uuid,
}

/// get contents with query.
#[derive(Debug, Clone, ::clap::Clap)]
pub struct ContentGetsCmd {
    /// u32 (1 =< n)
    #[clap(name = "PAGE", default_value = "1", parse(try_from_str = parse_nonzero_num))]
    pub page: u32,

    /// json
    ///
    /// schema: {
    ///   author?: Author,
    ///   posted?: Posted,
    ///   content?: regex,
    ///   liked?: [u64],
    ///   liked_num?: range<u32>,
    ///   pinned: [u64],
    ///   pinned_num?: range<u32>,
    /// }
    ///
    /// enum Author {
    ///   UserId(u64),
    ///   UserName(regex),
    ///   UserNick(regex),
    ///   Any(regex),
    /// }
    ///
    /// enum Posted {
    ///   UserId(u64),
    ///   UserName(regex),
    ///   UserNick(regex),
    ///   Any(regex)
    /// }
    ///
    /// # example
    ///
    /// {
    ///   "author": {
    ///     "Any": "username"
    ///   },
    ///   "pinned_num": "10.."
    /// }
    #[clap(name = "QUERY", default_value = "{}", parse(try_from_str = parse_content_query))]
    pub query: ContentQuery,
}

/// edit content with id and mutation.
#[derive(Debug, Clone, ::clap::Clap)]
pub struct ContentEditCmd {
    /// uuid
    #[clap(name = "CONTENT_ID")]
    pub content_id: Uuid,

    /// json
    ///
    /// schema: {
    ///   "author": Author,
    ///   "content": Content,
    /// }
    ///
    /// enum Author {
    ///   User(u64),
    ///   Virtual(regex),
    /// }
    ///
    /// enum Content {
    ///   Complete(str),
    ///   Sed { capture: regex, replace: str }
    /// }
    #[clap(name = "MUTATION", default_value = "{}", parse(try_from_str = parse_partial_content_mutation))]
    pub mutation: PartialContentMutation,
}

#[derive(Debug, Clone, ::clap::Clap)]
pub struct ContentLikeCmd {
    #[clap(subcommand)]
    pub op: ContentLikeOp,
}

/// about like with executed user.
#[derive(Debug, Clone, ::clap::Clap)]
pub enum ContentLikeOp {
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
pub struct ContentPinCmd {
    #[clap(subcommand)]
    pub op: ContentPinOp,
}

#[derive(Debug, Clone, ::clap::Clap)]
pub enum ContentPinOp {
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
pub struct ContentWithdrawCmd {
    /// uuid
    #[clap(name = "CONTENT_ID")]
    pub content_id: Uuid,
}

pub fn parse_msg(msg: &str) -> Option<Result<App, String>> {
    let split_res = shell_words::split(msg)
        .map(|mut v| {
            v.drain(..)
                .map(|s| s.into())
                .collect::<Vec<::std::ffi::OsString>>()
        })
        .map_err(|e| e.to_string());

    let splitted = match split_res {
        Ok(o) => o,
        Err(e) => return Some(Err(e)),
    };

    if let Some("*ip") = splitted.get(0).map(|s| s.to_str().unwrap()) {
    } else {
        return None;
    }

    use clap::Clap;

    App::try_parse_from(splitted)
        .map_err(|e| e.to_string())
        .let_(Some)
}

// --- public helpers ---

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

// --- parsing helpers ---

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
