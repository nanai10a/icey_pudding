use anyhow::bail;
use serenity::model::id::UserId;

use crate::entities::{Content, User};

impl UserQuery {
    #[allow(clippy::needless_lifetimes)]
    pub async fn filter<'a>(&self, mut src: Vec<&'a User>) -> anyhow::Result<Vec<&'a User>> {
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

use tokio::sync::Mutex;

pub struct InMemoryRepository<T>(Mutex<Vec<T>>);

impl<T> InMemoryRepository<T> {
    pub async fn new() -> Self { Self(Mutex::new(vec![])) }
}

#[serenity::async_trait]
pub trait UserRepository {
    async fn save(&self, item: User) -> anyhow::Result<()>;
    async fn get_matches(&self, queries: Vec<UserQuery>) -> anyhow::Result<Vec<User>>;
    async fn get_match(&self, queries: Vec<UserQuery>) -> anyhow::Result<User>;
    async fn remove_match(&self, queries: Vec<UserQuery>) -> anyhow::Result<User>;
    async fn remove_matches(&self, queries: Vec<UserQuery>) -> anyhow::Result<Vec<User>>;
}

#[serenity::async_trait]
impl UserRepository for InMemoryRepository<User> {
    async fn save(&self, item: User) -> anyhow::Result<()> {
        self.0.lock().await.push(item);
        Ok(())
    }

    async fn get_matches(&self, mut queries: Vec<UserQuery>) -> anyhow::Result<Vec<User>> {
        let locked = self.0.lock().await;
        let mut vec = locked.iter().collect();

        for q in queries.drain(..) {
            vec = q.filter(vec).await?;
        }

        Ok(vec.drain(..).cloned().collect())
    }

    async fn get_match(&self, queries: Vec<UserQuery>) -> anyhow::Result<User> {
        let mut matches = self.get_matches(queries).await?;

        if matches.len() != 1 {
            bail!("cannot find match one. matched: {}.", matches.len());
        }

        Ok(matches.remove(0))
    }

    async fn remove_match(&self, queries: Vec<UserQuery>) -> anyhow::Result<User> {
        let matched = self.get_match(queries).await?;

        let mut guard = self.0.lock().await;
        let vec: &mut Vec<_> = guard.as_mut();
        let res = try_remove_target_from_vec(vec, &matched, |v1, v2| v1.id == v2.id);
        drop(guard);

        match res {
            Ok(o) => o,
            Err(e) => bail!("{}", e),
        }

        Ok(matched)
    }

    async fn remove_matches(&self, queries: Vec<UserQuery>) -> anyhow::Result<Vec<User>> {
        let matches = self.get_matches(queries).await?;

        let mut guard = self.0.lock().await;
        let vec: &mut Vec<_> = guard.as_mut();
        let res = matches
            .iter()
            .try_for_each(|t| try_remove_target_from_vec(vec, t, |v1, v2| v1.id == v2.id));
        drop(guard);

        match res {
            Ok(o) => o,
            Err(e) => bail!("{}", e),
        }

        Ok(matches)
    }
}

fn try_remove_target_from_vec<T>(
    vec: &mut Vec<T>,
    target: &T,
    compare: impl Fn(&T, &T) -> bool,
) -> anyhow::Result<()> {
    let mut indexes = vec
        .iter()
        .enumerate()
        .filter_map(|(i, v)| match compare(target, v) {
            true => Some(i),
            false => None,
        })
        .collect::<Vec<_>>();

    let index = match indexes.len() {
        1 => indexes.remove(0),
        _ => bail!("cannot get index: got {:?}", indexes),
    };

    vec.remove(index);
    Ok(())
}

use uuid::Uuid;

pub enum UserQuery {
    Id(UserId),
    Admin(bool),
    SubAdmin(bool),
    Posted(Vec<Uuid>),
    Bookmark(Vec<Uuid>),
}

#[serenity::async_trait]
pub trait ContentRepository {
    async fn save(&self, item: Content) -> anyhow::Result<()>;
    async fn get_matches(&self, queries: Vec<ContentQuery>) -> anyhow::Result<Vec<Content>>;
    async fn get_match(&self, queries: Vec<ContentQuery>) -> anyhow::Result<Content>;
    async fn remove_matches(&self, queries: Vec<ContentQuery>) -> anyhow::Result<Vec<Content>>;
    async fn remove_match(&self, queries: Vec<ContentQuery>) -> anyhow::Result<Content>;
}

#[serenity::async_trait]
impl ContentRepository for InMemoryRepository<Content> {
    async fn save(&self, item: Content) -> anyhow::Result<()> {
        self.0.lock().await.push(item);
        Ok(())
    }

    async fn get_matches(&self, mut queries: Vec<ContentQuery>) -> anyhow::Result<Vec<Content>> {
        let locked = self.0.lock().await;
        let mut vec = locked.iter().collect::<Vec<_>>();

        for q in queries.drain(..) {
            vec = q.filter(vec).await?;
        }

        Ok(vec.drain(..).cloned().collect())
    }

    async fn get_match(&self, queries: Vec<ContentQuery>) -> anyhow::Result<Content> {
        let mut matches = self.get_matches(queries).await?;

        if matches.len() != 1 {
            bail!("cannot find match one. matched: {}.", matches.len());
        }

        Ok(matches.remove(0))
    }

    async fn remove_match(&self, queries: Vec<ContentQuery>) -> anyhow::Result<Content> {
        let matched = self.get_match(queries).await?;

        let mut guard = self.0.lock().await;
        let vec: &mut Vec<_> = guard.as_mut();
        let res = try_remove_target_from_vec(vec, &matched, |v1, v2| v1.id == v2.id);
        drop(guard);

        match res {
            Ok(o) => o,
            Err(e) => bail!("{}", e),
        }

        Ok(matched)
    }

    async fn remove_matches(&self, queries: Vec<ContentQuery>) -> anyhow::Result<Vec<Content>> {
        let matches = self.get_matches(queries).await?;

        let mut guard = self.0.lock().await;
        let vec: &mut Vec<_> = guard.as_mut();
        let res = matches
            .iter()
            .try_for_each(|t| try_remove_target_from_vec(vec, t, |v1, v2| v1.id == v2.id));
        drop(guard);

        match res {
            Ok(o) => o,
            Err(e) => bail!("{}", e),
        }

        Ok(matches)
    }
}

pub enum ContentQuery {
    Id(Uuid),
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

impl ContentQuery {
    #[allow(clippy::needless_lifetimes)]
    pub async fn filter<'a>(&self, mut src: Vec<&'a Content>) -> anyhow::Result<Vec<&'a Content>> {
        let mut c: Box<dyn FnMut(&'a Content) -> bool> = match self {
            Self::Id(f_id) => box move |Content { id, .. }| id == f_id,
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
