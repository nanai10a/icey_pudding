use core::num::NonZeroU32;
use std::collections::HashSet;

use regex::Regex;
use uuid::Uuid;

use crate::entities::{ContentId, PartialAuthor, UserId};
use crate::usecases::content::{AuthorQuery, ContentContentMutation, ContentQuery, PostedQuery};
use crate::usecases::user::{UserMutation, UserQuery};
use crate::utils::LetChain;

#[derive(Debug, Clone, Default)]
pub struct PartialContentMutation {
    pub author: Option<PartialAuthor>,
    pub content: Option<ContentContentMutation>,
}

pub fn parse_nonzero_num(
    s: &str,
) -> ::core::result::Result<u32, <NonZeroU32 as ::core::str::FromStr>::Err> {
    Ok(s.parse::<::core::num::NonZeroU32>()?.get())
}

pub fn parse_user_query(s: &str) -> ::core::result::Result<UserQuery, String> {
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

pub fn parse_user_mutation(s: &str) -> ::core::result::Result<UserMutation, String> {
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

pub fn parse_content_query(s: &str) -> ::core::result::Result<ContentQuery, String> {
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

pub fn parse_partial_content_mutation(
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
