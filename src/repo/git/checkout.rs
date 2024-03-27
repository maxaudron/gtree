use super::{Repo, RepoError};

use anyhow::Context;
use gix::{
    clone::checkout::main_worktree::ProgressId, interrupt::IS_INTERRUPTED, progress, remote, Id,
    Progress,
};

use gix_index::{File, State};
use tracing::debug;

impl Repo {
    #[tracing::instrument(level = "trace", skip(progress))]
    pub fn checkout(
        &self,
        remote: remote::Name,
        head: Id,
        progress: &mut dyn progress::DynNestedProgress,
    ) -> Result<(), RepoError> {
        let repo = self.repo()?;

        let workdir = repo.work_dir().ok_or(RepoError::NoWorktree)?;
        let head_tree = head
            .object()
            .context("could not find object HEAD points to")?
            .peel_to_tree()
            .context("failed to peel HEAD object")?
            .id();

        let index = State::from_tree(&head_tree, &repo.objects).context("index from tree")?;
        let mut index = File::from_state(index, repo.index_path());

        let mut files =
            progress.add_child_with_id("checkout".to_string(), ProgressId::CheckoutFiles.into());
        let mut bytes =
            progress.add_child_with_id("writing".to_string(), ProgressId::BytesWritten.into());

        files.init(Some(index.entries().len()), progress::count("files"));
        bytes.init(None, progress::bytes());

        let start = std::time::Instant::now();

        debug!("workdir: {:?}", workdir);
        let opts = gix_worktree_state::checkout::Options::default();
        let outcome = gix_worktree_state::checkout(
            &mut index,
            workdir,
            repo.objects.clone().into_arc().unwrap(),
            &files,
            &bytes,
            &IS_INTERRUPTED,
            opts,
        )
        .context("checkout: failed");

        files.show_throughput(start);
        bytes.show_throughput(start);

        debug!("outcome: {:?}", outcome);
        debug!("is interrupted: {:?}", &gix::interrupt::IS_INTERRUPTED);

        index
            .write(Default::default())
            .context("checkout: write index")?;

        Ok(())
    }
}
