use super::{Repo, RepoError};

use anyhow::Context;
use gix::{
    clone::checkout::main_worktree::ProgressId, interrupt::IS_INTERRUPTED, progress, remote, Id,
    Progress,
};

use gix_index::{File, State};
use tracing::debug;

impl Repo {
    pub fn ffmerge(&self, a: Id, b: Id) -> Result<(), RepoError> {

        Ok(())
    }
}
