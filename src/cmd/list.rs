use itertools::Itertools;

use crate::{GTreeError, repo::Repos};

impl crate::GTree {
    pub fn list(&self, repos: Repos) -> Result<(), GTreeError> {
        repos.iter().sorted_by_key(|x| x.0).for_each(|(_, repo)| {
            let repo = repo.read().unwrap();
            println!("{}", repo)
        });

        Ok(())
    }
}
