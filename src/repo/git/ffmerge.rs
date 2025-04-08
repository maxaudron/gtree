use super::{Repo, RepoError};

use gix::Id;

impl Repo {
    pub fn ffmerge(&self, _a: Id, _b: Id) -> Result<(), RepoError> {
        Ok(())
    }
}
