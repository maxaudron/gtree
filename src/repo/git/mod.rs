use super::{LocalRepoState, Repo, RepoError};

use anyhow::Context;
use gix::{
    bstr::BString,
    refs::{
        transaction::{LogChange, PreviousValue, RefEdit},
        FullName,
    },
    remote, Id, ObjectId, Remote,
};
use tracing::debug;

mod checkout;
mod ffmerge;
mod fetch;

impl Repo {
    #[tracing::instrument(level = "debug")]
    pub fn is_clean(&self) -> Result<LocalRepoState, RepoError> {
        let repo = self.repo()?;

        if let Some(state) = repo.state() {
            Ok(LocalRepoState::InProgress(state))
        } else {
            let head = repo.head().unwrap();

            if head.is_detached() {
                return Ok(LocalRepoState::DetachedHead);
            }

            if head.is_unborn() {
                return Ok(LocalRepoState::UnbornHead);
            }

            Ok(LocalRepoState::Clean)
        }
    }

    pub fn default_remote(&self) -> Result<Remote, RepoError> {
        Ok(self
            .repo()?
            .find_default_remote(gix::remote::Direction::Fetch)
            .ok_or(RepoError::NoRemoteFound)?
            .context("fetch: failed to find default remote")?)
    }

    pub fn default_branch(&self) -> Result<BString, RepoError> {
        let repo = self.repo()?;
        let remote = self.default_remote()?;
        let remote_name = remote.name().context("remote does not have name")?;

        let origin_ref = repo
            .find_reference(&format!("remotes/{}/HEAD", remote_name.as_bstr()))
            .context("the remotes HEAD references does not exist")?;

        debug!("got ref to origin: {:?}", origin_ref);

        if let Some(origin_ref) = origin_ref.target().try_name() {
            let shortened = origin_ref.shorten().to_owned();
            Ok(shortened.split(|x| *x == b'/').last().unwrap().into())
        } else {
            Err(RepoError::NoDefaultBranch)
        }
    }

    pub fn refedit(target: ObjectId, name: &str, message: &str) -> RefEdit {
        RefEdit {
            change: gix::refs::transaction::Change::Update {
                log: LogChange {
                    mode: gix::refs::transaction::RefLog::AndReference,
                    force_create_reflog: false,
                    message: message.into(),
                },
                expected: PreviousValue::Any,
                new: gix::refs::Target::Peeled(target),
            },
            name: FullName::try_from(name).unwrap(),
            deref: true,
        }
    }

    pub fn update_default_branch_ref(
        &self,
        remote: &remote::Name,
        head: Id,
    ) -> Result<(), RepoError> {
        let default_branch = self.default_branch()?;
        debug!("default branch: {:?}", default_branch);

        let repo = self.repo()?;

        let edits = repo.edit_reference(Repo::refedit(
            head.into(),
            &format!("refs/heads/{}", default_branch),
            &format!("checkout: {}/HEAD with gtree", remote.as_bstr()),
        ))
        .context("checkout: failed to edit ref")?;

        debug!("ref edits: {:?}", edits);

        Ok(())
    }

    pub fn default_remote_head(&self) -> Result<(remote::Name, Id), RepoError> {
        let repo = self.repo()?;

        let remote = repo
            .find_fetch_remote(None)
            .context("could not find remote to fetch")?;
        let remote = remote.name().context("remote does not have name")?;

        let head_ref = repo
            .find_reference(&format!("remotes/{}/HEAD", remote.as_bstr()))
            .context("the remotes HEAD references does not exist")?;
        let head = head_ref
            .into_fully_peeled_id()
            .context("failed to peel ref")?;

        Ok((remote.to_owned(), head.to_owned()))
    }
}
