use uuid::Uuid;

use crate::usecases::content::ContentQuery;
use crate::usecases::user::{UserMutation, UserQuery};

pub mod parser;

pub use parser::PartialContentMutation;
use parser::*;

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
