use anyhow::bail;
use serenity::model::id::UserId;

use crate::entities::User;

#[serenity::async_trait]
pub trait Repository {
    type Item: Send + Sync;
    type Query: Send + Sync;

    async fn save(&self, item: Self::Item) -> anyhow::Result<()>;
    async fn get_matches(&self, queries: Vec<Self::Query>) -> anyhow::Result<Vec<Self::Item>>;
    async fn get_match(&self, queries: Vec<Self::Query>) -> anyhow::Result<Self::Item> {
        let mut matches = self.get_matches(queries).await?;

        if matches.len() != 1 {
            bail!("cannot find match one. matched: {}.", matches.len());
        }

        Ok(matches.remove(0))
    }
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

    async fn remove_match(&self, queries: Vec<Self::Query>) -> anyhow::Result<Self::Item> {
        let mut guard = self.0.lock().await;
        let vec: &mut Vec<_> = guard.as_mut();

        let matched = self.get_match(queries).await?;
        let res = try_remove_target_from_vec(vec, &matched);

        match res {
            Ok(o) => o,
            Err(e) => bail!("{}", e),
        }

        Ok(matched)
    }

    async fn remove_matches(&self, queries: Vec<Self::Query>) -> anyhow::Result<Vec<Self::Item>> {
        let mut guard = self.0.lock().await;
        let vec: &mut Vec<_> = guard.as_mut();

        let matches = self.get_matches(queries).await?;
        let res = matches
            .iter()
            .try_for_each(|t| try_remove_target_from_vec(vec, t));

        match res {
            Ok(o) => o,
            Err(e) => bail!("{}", e),
        }

        Ok(matches)
    }
}

fn try_remove_target_from_vec(vec: &mut Vec<User>, target: &User) -> anyhow::Result<()> {
    let mut indexes = vec
        .iter()
        .enumerate()
        .filter_map(|(i, v)| match target.id == v.id {
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
