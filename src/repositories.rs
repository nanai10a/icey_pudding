use anyhow::bail;
use serenity::model::id::UserId;

use crate::entities::{Content, User};

#[serenity::async_trait]
pub trait Repository {
    type Item: Send + Sync;
    type Query: Send + Sync;

    async fn save(&self, item: Self::Item) -> anyhow::Result<()>;
    async fn get_matches(&self, queries: Vec<Self::Query>) -> anyhow::Result<Vec<Self::Item>>;
    async fn get_match(&self, queries: Vec<Self::Query>) -> anyhow::Result<Self::Item>;
    async fn remove_matches(&self, queries: Vec<Self::Query>) -> anyhow::Result<Vec<Self::Item>>;
    async fn remove_match(&self, queries: Vec<Self::Query>) -> anyhow::Result<Self::Item>;
}

use tokio::sync::Mutex;

pub struct InMemoryRepository<T>(Mutex<Vec<T>>);

impl<T> InMemoryRepository<T> {
    pub async fn new() -> Self { Self(Mutex::new(vec![])) }
}
#[serenity::async_trait]
impl Repository for InMemoryRepository<User> {
    type Item = User;
    type Query = UserQuery;

    async fn save(&self, item: Self::Item) -> anyhow::Result<()> {
        self.0.lock().await.push(item);
        Ok(())
    }

    async fn get_matches(&self, mut queries: Vec<Self::Query>) -> anyhow::Result<Vec<Self::Item>> {
        let locked = self.0.lock().await;
        let mut vec = locked.iter().collect::<Vec<_>>();

        queries.drain(..).for_each(|q| match q {
            UserQuery::Id(i) => vec = vec.drain(..).filter(|v| v.id == i).collect(),
            UserQuery::Admin(a) => {
                vec = vec.drain(..).filter(|v| v.admin == a).collect();
            },
            UserQuery::SubAdmin(sa) => {
                vec = vec.drain(..).filter(|v| v.sub_admin == sa).collect();
            },
            UserQuery::Posted(p) => {
                vec = vec
                    .drain(..)
                    .filter(|v| p.iter().filter(|i| v.posted.contains(i)).count() == p.len())
                    .collect();
            },
            UserQuery::Bookmark(b) => {
                vec = vec
                    .drain(..)
                    .filter(|v| b.iter().filter(|i| v.bookmark.contains(i)).count() == b.len())
                    .collect();
            },
        });

        Ok(vec.drain(..).cloned().collect())
    }

    async fn get_match(&self, queries: Vec<Self::Query>) -> anyhow::Result<Self::Item> {
        let mut matches = self.get_matches(queries).await?;

        if matches.len() != 1 {
            bail!("cannot find match one. matched: {}.", matches.len());
        }

        Ok(matches.remove(0))
    }

    async fn remove_match(&self, queries: Vec<Self::Query>) -> anyhow::Result<Self::Item> {
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

    async fn remove_matches(&self, queries: Vec<Self::Query>) -> anyhow::Result<Vec<Self::Item>> {
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
impl Repository for InMemoryRepository<Content> {
    type Item = Content;
    type Query = ContentQuery;

    async fn save(&self, item: Self::Item) -> anyhow::Result<()> {
        self.0.lock().await.push(item);
        Ok(())
    }

    async fn get_matches(&self, mut queries: Vec<Self::Query>) -> anyhow::Result<Vec<Self::Item>> {
        let locked = self.0.lock().await;
        let mut vec = locked.iter().collect::<Vec<_>>();

        let res = queries
            .drain(..)
            .try_for_each::<_, anyhow::Result<()>>(|q| {
                match q {
                    ContentQuery::Id(i) => vec = vec.drain(..).filter(|v| v.id == i).collect(),
                    ContentQuery::Content(s) => {
                        let r = regex::Regex::new(s.as_str())?;
                        vec = vec
                            .drain(..)
                            .filter(|v| r.is_match(v.content.as_str()))
                            .collect();
                    },

                    ContentQuery::Liked(l) => {
                        vec = vec
                            .drain(..)
                            .filter(|v| l.iter().filter(|i| v.liked.contains(i)).count() == l.len())
                            .collect();
                    },
                    ContentQuery::Bookmarked(b, c) => {
                        vec = vec
                            .drain(..)
                            .filter(|v| match c {
                                Comparison::Over => v.bookmarked >= b,
                                Comparison::Eq => v.bookmarked == b,
                                Comparison::Under => v.bookmarked <= b,
                            })
                            .collect();
                    },
                    ContentQuery::Pinned(p) => {
                        vec = vec
                            .drain(..)
                            .filter(|v| {
                                p.iter().filter(|i| v.pinned.contains(i)).count() == p.len()
                            })
                            .collect();
                    },
                }

                Ok(())
            });

        match res {
            Ok(o) => o,
            Err(e) => bail!("{}", e),
        }

        Ok(vec.drain(..).cloned().collect())
    }

    async fn get_match(&self, queries: Vec<Self::Query>) -> anyhow::Result<Self::Item> {
        let mut matches = self.get_matches(queries).await?;

        if matches.len() != 1 {
            bail!("cannot find match one. matched: {}.", matches.len());
        }

        Ok(matches.remove(0))
    }

    async fn remove_match(&self, queries: Vec<Self::Query>) -> anyhow::Result<Self::Item> {
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

    async fn remove_matches(&self, queries: Vec<Self::Query>) -> anyhow::Result<Vec<Self::Item>> {
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
