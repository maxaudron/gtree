use super::{Repo, RepoError};

use git2::Repository;

impl Repo {
    #[tracing::instrument(level = "trace")]
    pub fn clone(&mut self, url: &str) -> Result<(), RepoError> {
        std::fs::create_dir_all(&self.path).unwrap();

        // TODO credential setup? ssh-agent?
        let repo = Repository::clone(url, &self.path)?;
        self.repo = Some(repo);

        Ok(())
    }

    #[tracing::instrument(level = "trace")]
    pub fn fetch<'a>(&mut self) -> Result<bool, RepoError> {
        let mut remote = self.default_remote()?;
        // TODO FetchOpts for info if refs were updated or not
        remote.fetch(&[&self.default_branch], None, Some("gtree fetch"))?;

        Ok(true)
    }
}
