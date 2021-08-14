use anyhow::bail;

pub trait Repository {
    type Item;
    type Query;

    fn save(&self, item: Self::Item) -> anyhow::Result<()>;
    fn get_matches(&self, queries: Vec<Self::Query>) -> anyhow::Result<Vec<Self::Item>>;
    fn get_match(&self, queries: Vec<Self::Query>) -> anyhow::Result<Self::Item> {
        let mut matches = self.get_matches(queries)?;

        if matches.len() != 1 {
            bail!("cannot find match one. matched: {}.", matches.len());
        }

        Ok(matches.remove(0))
    }
    fn remove_matches(&self, queries: Vec<Self::Query>) -> anyhow::Result<Vec<Self::Item>>;
    fn remove_match(&self, queries: Vec<Self::Query>) -> anyhow::Result<Self::Item>;
}
