use git2::Oid;
use tracing::debug;

use super::{Repo, RepoError};

impl Repo {
    #[tracing::instrument(level = "debug", ret, err)]
    pub fn checkout(&self, head: Oid) -> Result<(), RepoError> {
        let repo = self.repo()?;
        let head = repo.find_object(head, None)?;
        debug!("object kind {:?}", head.kind());
        repo.reset(&head, git2::ResetType::Hard, None)?;

        Ok(())
    }
}
