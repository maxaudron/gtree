use super::{LocalRepoState, Repo, RepoError};

use anyhow::Context;
use git2::{Direction, Reference, Remote, RepositoryState};
use tracing::{debug, instrument};

mod checkout;
mod fetch;

impl Repo {
    #[instrument(level = "debug", ret, err)]
    pub fn is_clean(&self) -> Result<LocalRepoState, RepoError> {
        let repo = self.repo()?;

        if repo.state() != RepositoryState::Clean {
            Ok(LocalRepoState::Other(repo.state()))
        } else {
            if repo.head_detached()? {
                return Ok(LocalRepoState::DetachedHead);
            }

            if repo.state() != RepositoryState::Clean {
                return Ok(LocalRepoState::Other(repo.state()));
            }

            let head = self.repo()?.head()?;
            let branch = head.shorthand()?;
            let default_branch = self.default_branch()?;

            debug!("branch: {branch}, default_branch: {default_branch}");
            if !branch.contains(&default_branch) {
                return Ok(LocalRepoState::NonDefaultBranch);
            }

            let default_ref = self.default_remote_ref()?.peel_to_commit()?;

            let head_ref = head.peel_to_commit()?;

            let unpushed_commits = default_ref
                .parents()
                .take_while(|c| c.id() != head_ref.id())
                .count();

            if default_ref.id() != head_ref.id() && unpushed_commits > 0 {
                return Ok(LocalRepoState::UnpushedCommits(unpushed_commits));
            }

            Ok(LocalRepoState::Clean)
        }
    }

    #[instrument(level = "debug", err)]
    pub fn default_remote(&self) -> Result<Remote<'_>, RepoError> {
        let remotes = self.repo()?.remotes()?;

        if remotes.is_empty() {
            return Err(RepoError::NoRemoteFound);
        }

        let remote_name = if remotes.len() == 1
            && let Some(Ok(Some(remote_name))) = remotes.into_iter().next()
        {
            remote_name
        } else {
            "origin"
        };

        self.repo()?
            .find_remote(remote_name)
            .map_err(RepoError::from)
    }

    #[instrument(level = "debug", err)]
    pub fn default_remote_ref(&self) -> Result<Reference<'_>, RepoError> {
        let repo = self.repo()?;
        let remote = self.default_remote()?;
        let remote_name = remote.name()?.ok_or(RepoError::NoRemoteFound)?;

        let origin_ref = repo
            .find_reference(&format!("refs/remotes/{}/HEAD", remote_name))
            .context("the remotes HEAD references does not exist")?;

        debug!("got ref to origin: {:?}", origin_ref.shorthand()?);

        Ok(origin_ref)
    }

    #[instrument(level = "debug", err)]
    pub fn default_branch(&self) -> Result<String, RepoError> {
        let mut remote = self.default_remote()?;
        let (cb, _) = self.remote_callbacks();
        remote.connect_auth(Direction::Fetch, Some(cb), None)?;
        remote
            .default_branch()
            .and_then(|s| Ok(s.as_str()?.to_owned()))
            .map_err(RepoError::from)
            .map(|s| s.strip_prefix("refs/heads/").unwrap_or(&s).to_string())
    }
}
