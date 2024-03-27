use super::{Repo, RepoError};

use anyhow::Context;
use gix::{
    interrupt::IS_INTERRUPTED,
    progress::Discard,
    remote::{fetch::Status, Direction},
};

impl Repo {
    #[tracing::instrument(level = "trace")]
    pub fn clone(&self, url: &str) -> Result<(), RepoError> {
        std::fs::create_dir_all(&self.path).unwrap();

        let mut prepare_clone = gix::prepare_clone(url, &self.path).unwrap();

        let (mut prepare_checkout, _) = prepare_clone
            .fetch_then_checkout(Discard, &IS_INTERRUPTED)
            .unwrap();

        let (_repo, _) = prepare_checkout
            .main_worktree(Discard, &IS_INTERRUPTED)
            .unwrap();

        Ok(())
    }

    #[tracing::instrument(level = "trace")]
    pub fn fetch<'a>(&mut self) -> Result<bool, RepoError> {
        let remote = self.default_remote()?;
        let conn = remote.connect(Direction::Fetch).unwrap();
        let outcome = conn
            .prepare_fetch(Discard, gix::remote::ref_map::Options::default())
            .context("fetch: failed to prepare patch")?
            .receive(Discard, &IS_INTERRUPTED)
            .context("fetch: failed to receive")?;

        match outcome.status {
            Status::NoPackReceived {
                dry_run: _,
                negotiate: _,
                update_refs: _,
            } => Ok(false),
            Status::Change {
                negotiate: _,
                write_pack_bundle: _,
                update_refs: _,
            } => Ok(true),
        }
    }
}
