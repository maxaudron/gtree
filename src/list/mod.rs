use anyhow::Result;

use crate::repo::Repos;

impl crate::GTree {
    pub async fn list(&self, repos: Repos) -> Result<()> {
        repos.iter().for_each(|repo| println!("{}", repo));

        Ok(())
    }
}
