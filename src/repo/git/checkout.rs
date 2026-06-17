use git2::Oid;

use super::{Repo, RepoError};

impl Repo {
    #[tracing::instrument(level = "trace")]
    pub fn checkout(&self, head: Oid) -> Result<(), RepoError> {
        let repo = self.repo()?;
        let head = repo.find_tree(head)?;
        repo.checkout_tree(head.as_object(), None)?;

        Ok(())
    }
}
