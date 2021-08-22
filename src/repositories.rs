use serenity::model::id::UserId;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::entities::{Content, User};

type StdResult<T, E> = ::std::result::Result<T, E>;
type Result<T> = ::std::result::Result<T, RepositoryError>;

#[serenity::async_trait]
pub trait Repository<T> {
    async fn save(&self, item: T) -> Result<()>;
    async fn get_matches(&self, queries: Vec<&(dyn Query<T> + Sync + Send)>) -> Result<Vec<T>>;
    async fn get_match(&self, queries: Vec<&(dyn Query<T> + Sync + Send)>) -> Result<T>;
    async fn remove_match(&self, queries: Vec<&(dyn Query<T> + Sync + Send)>) -> Result<T>;
}

#[serenity::async_trait]
pub trait Query<T> {
    async fn filter<'a>(&self, src: Vec<&'a T>) -> StdResult<Vec<&'a T>, anyhow::Error>;
}

pub trait Same {
    fn is_same(&self, other: &Self) -> bool;
}

#[serenity::async_trait]
impl<T: Send + Sync + Clone + Same> Repository<T> for InMemoryRepository<T> {
    async fn save(&self, item: T) -> Result<()> {
        self.0.lock().await.push(item);

        Ok(())
    }

    async fn get_matches(&self, mut queries: Vec<&(dyn Query<T> + Sync + Send)>) -> Result<Vec<T>> {
        let guard = self.0.lock().await;
        let mut vec = guard.iter().collect();

        for q in queries.drain(..) {
            vec = q.filter(vec).await.map_err(RepositoryError::Internal)?;
        }

        Ok(vec.drain(..).cloned().collect())
    }

    async fn get_match(&self, queries: Vec<&(dyn Query<T> + Sync + Send)>) -> Result<T> {
        let mut matches = self.get_matches(queries).await?;

        match matches.len() {
            1 => Ok(matches.remove(0)),
            _ => Err(RepositoryError::NoUnique {
                matched: matches.len() as u32,
            }),
        }
    }

    async fn remove_match(&self, queries: Vec<&(dyn Query<T> + Sync + Send)>) -> Result<T> {
        let matched = self.get_match(queries).await?;

        let mut guard = self.0.lock().await;
        let vec = guard.as_mut();

        try_remove_target_from_vec(vec, |v| matched.is_same(v))
            .map_err(|e| RepositoryError::NoUnique { matched: e as u32 })
    }
}

#[derive(Debug)]
pub enum RepositoryError {
    NotFound,
    NoUnique { matched: u32 },
    Internal(anyhow::Error),
}

impl ::std::fmt::Display for RepositoryError {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            RepositoryError::NotFound => write!(f, "cannot find object."),
            RepositoryError::NoUnique { matched } => write!(
                f,
                "expected unique object, found non-unique objects (matched: {})",
                matched
            ),
            RepositoryError::Internal(e) => write!(f, "internal error: {}", e),
        }
    }
}

impl ::std::error::Error for RepositoryError {}

fn try_remove_target_from_vec<T>(
    vec: &mut Vec<T>,
    is_target: impl Fn(&T) -> bool,
) -> ::std::result::Result<T, usize> {
    let mut indexes: Vec<_> = vec
        .iter()
        .enumerate()
        .filter_map(|(i, v)| match is_target(v) {
            true => Some(i),
            false => None,
        })
        .collect();

    match indexes.len() {
        1 => Ok(vec.remove(indexes.remove(0))),
        _ => Err(indexes.len()),
    }
}

pub struct InMemoryRepository<T>(Mutex<Vec<T>>);

impl<T> InMemoryRepository<T> {
    pub async fn new() -> Self { Self(Mutex::new(vec![])) }
}

impl Same for User {
    fn is_same(&self, other: &Self) -> bool { self.id == other.id }
}

pub enum UserQuery {
    Id(UserId),
    Admin(bool),
    SubAdmin(bool),
    Posted(Vec<Uuid>),
    Bookmark(Vec<Uuid>),
}

#[serenity::async_trait]
impl Query<User> for UserQuery {
    #[allow(clippy::needless_lifetimes)]
    async fn filter<'a>(&self, mut src: Vec<&'a User>) -> anyhow::Result<Vec<&'a User>> {
        let mut c: Box<dyn FnMut(&'a User) -> bool> = match self {
            // FIXME: `User`変更時にQueryの変更をしていないので, 足りないfieldがある
            Self::Id(f_id) => box move |User { id, .. }| id == f_id,
            Self::Admin(f_admin) => box move |User { admin, .. }| admin == f_admin,
            Self::SubAdmin(f_sub_admin) =>
                box move |User { sub_admin, .. }| sub_admin == f_sub_admin,
            Self::Posted(f_posted) => box move |User { posted, .. }| {
                f_posted.iter().filter(|elem| posted.contains(elem)).count() == posted.len()
            },
            Self::Bookmark(f_bookmark) => box move |User { bookmark, .. }| {
                f_bookmark
                    .iter()
                    .filter(|elem| bookmark.contains(elem))
                    .count()
                    == bookmark.len()
            },
        };

        Ok(src.drain_filter(|v| c.call_mut((v,))).collect())
    }
}

impl Same for Content {
    fn is_same(&self, other: &Self) -> bool { self.id == other.id }
}

pub enum ContentQuery {
    Id(Uuid),
    IdHead(u32),
    Author(String),
    Posted(UserId),
    Content(String),
    Liked(Vec<UserId>),
    Bookmarked(u32, Comparison),
    Pinned(Vec<UserId>),
}

pub enum Comparison {
    Over,
    Eq,
    Under,
}

#[serenity::async_trait]
impl Query<Content> for ContentQuery {
    #[allow(clippy::needless_lifetimes)]
    async fn filter<'a>(&self, mut src: Vec<&'a Content>) -> anyhow::Result<Vec<&'a Content>> {
        let mut c: Box<dyn FnMut(&'a Content) -> bool> = match self {
            Self::Id(f_id) => box move |Content { id, .. }| id == f_id,
            Self::IdHead(f_id_head) => box move |Content { id, .. }| id.as_fields().0 == *f_id_head,
            Self::Author(f_author) => {
                let r = regex::Regex::new(f_author)?;
                box move |Content { author, .. }| r.is_match(author)
            },
            Self::Posted(f_posted) => box move |Content { posted, .. }| posted == f_posted,
            Self::Content(f_content) => {
                let rx = regex::Regex::new(f_content)?;
                box move |Content { content, .. }| rx.is_match(content)
            },
            Self::Liked(f_liked) => box move |Content { liked, .. }| {
                f_liked.iter().filter(|elem| liked.contains(elem)).count() == liked.len()
            },
            Self::Bookmarked(f_bookmarked, comp) =>
                box move |Content { bookmarked, .. }| match comp {
                    Comparison::Over => bookmarked >= f_bookmarked,
                    Comparison::Eq => bookmarked == f_bookmarked,
                    Comparison::Under => bookmarked <= f_bookmarked,
                },
            Self::Pinned(f_pinned) => box move |Content { pinned, .. }| {
                f_pinned.iter().filter(|elem| pinned.contains(elem)).count() == pinned.len()
            },
        };

        Ok(src.drain_filter(|v| c.call_mut((v,))).collect())
    }
}
