use std::{collections::HashMap, fmt::Debug, path::PathBuf, sync::RwLock};

use anyhow::Context;
use thiserror::Error;

use gix::{
    bstr::BString,
    clone::checkout::main_worktree::ProgressId,
    refs::{
        transaction::{LogChange, PreviousValue, RefEdit},
        FullName,
    },
    remote, Id, ObjectId, Progress, Remote, Repository,
};
use tracing::{debug, error};

use crate::forge::Project;

mod aggregate;
mod repostate;

pub use aggregate::*;
pub use repostate::*;

// pub type Repos = Vec<Repo>;
pub type Repos = HashMap<String, RwLock<Repo>>;

pub struct Repo {
    pub name: String,
    pub path: PathBuf,
    pub repo: Option<Repository>,
    pub forge: Option<Project>,
    pub default_branch: String,
}

impl Repo {
    pub fn repo(&self) -> Result<&Repository, RepoError> {
        match &self.repo {
            Some(repo) => Ok(repo),
            None => Err(RepoError::NoLocalRepo),
        }
    }

    pub fn repo_mut(&mut self) -> Result<&mut Repository, RepoError> {
        match &mut self.repo {
            Some(repo) => Ok(repo),
            None => Err(RepoError::NoLocalRepo),
        }
    }

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

        if let Some(origin_ref) = origin_ref.target().try_name() {
            Ok(origin_ref.shorten().to_owned())
        } else {
            Err(RepoError::NoDefaultBranch)
        }
    }

    #[tracing::instrument(level = "trace")]
    pub fn clone(&self, url: &str) -> Result<(), RepoError> {
        std::fs::create_dir_all(&self.path).unwrap();

        let mut prepare_clone = gix::prepare_clone(url, &self.path).unwrap();

        let (mut prepare_checkout, _) = prepare_clone
            .fetch_then_checkout(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
            .unwrap();

        let (_repo, _) = prepare_checkout
            .main_worktree(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
            .unwrap();

        Ok(())
    }

    #[tracing::instrument(level = "trace")]
    pub fn fetch<'a>(&mut self) -> Result<bool, RepoError> {
        let remote = self.default_remote()?;
        let conn = remote.connect(gix::remote::Direction::Fetch).unwrap();
        let outcome = conn
            .prepare_fetch(
                gix::progress::Discard,
                gix::remote::ref_map::Options::default(),
            )
            .context("fetch: failed to prepare patch")?
            .receive(gix::progress::Discard, &gix::interrupt::IS_INTERRUPTED)
            .context("fetch: failed to receive")?;

        match outcome.status {
            gix::remote::fetch::Status::NoPackReceived {
                dry_run: _,
                negotiate: _,
                update_refs: _,
            } => Ok(false),
            gix::remote::fetch::Status::Change {
                negotiate: _,
                write_pack_bundle: _,
                update_refs: _,
            } => Ok(true),
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
        remote: remote::Name,
        head: Id,
    ) -> Result<(), RepoError> {
        let default_branch = self.default_branch()?;
        let repo = self.repo()?;

        repo.edit_reference(Repo::refedit(
            head.into(),
            &format!("heads/{}", default_branch),
            &format!("checkout: {}/HEAD with gtree", remote.as_bstr()),
        ))
        .context("checkout: failed to edit ref")?;

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

    #[tracing::instrument(level = "trace", skip(progress))]
    pub fn checkout(
        &self,
        remote: remote::Name,
        head: Id,
        progress: &mut dyn gix::progress::DynNestedProgress,
    ) -> Result<(), RepoError> {
        let repo = self.repo()?;

        let workdir = repo.work_dir().ok_or(RepoError::NoWorktree)?;
        let head_tree = head
            .object()
            .context("could not find object HEAD points to")?
            .peel_to_tree()
            .context("failed to peel HEAD object")?
            .id();

        let index =
            gix_index::State::from_tree(&head_tree, &repo.objects).context("index from tree")?;
        let mut index = gix_index::File::from_state(index, repo.index_path());

        let mut files =
            progress.add_child_with_id("checkout".to_string(), ProgressId::CheckoutFiles.into());
        let mut bytes =
            progress.add_child_with_id("writing".to_string(), ProgressId::BytesWritten.into());

        files.init(Some(index.entries().len()), gix::progress::count("files"));
        bytes.init(None, gix::progress::bytes());

        let start = std::time::Instant::now();

        debug!("workdir: {:?}", workdir);
        let opts = gix_worktree_state::checkout::Options::default();
        let outcome = gix_worktree_state::checkout(
            &mut index,
            workdir,
            repo.objects.clone().into_arc().unwrap(),
            &files,
            &bytes,
            &gix::interrupt::IS_INTERRUPTED,
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

#[derive(Error, Debug)]
pub enum RepoError {
    #[error("repo is not cloned locally")]
    NoLocalRepo,
    #[error("local git repo does not have a remote")]
    NoRemoteFound,
    #[error("could not determine default branch based on remote HEAD")]
    NoDefaultBranch,
    #[error("repo is not checked out")]
    NoWorktree,
    #[error("repository is dirty: {0}")]
    Dirty(LocalRepoState),
    #[error("fast-forward merge was not possible")]
    NoFF,
    #[error("error: {0}")]
    Anyhow(#[from] anyhow::Error),
    #[error("unknown repo error")]
    Unknown,
}

#[derive(Error, Debug, PartialEq)]
pub enum LocalRepoState {
    #[error("operation in progress: {0:?}")]
    InProgress(gix::state::InProgress),
    #[error("head is detached")]
    DetachedHead,
    #[error("head is unborn")]
    UnbornHead,
    #[error("repo is clean")]
    Clean,
}

impl Ord for Repo {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl Eq for Repo {}

impl PartialOrd for Repo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl PartialEq for Repo {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl From<Project> for Repo {
    fn from(project: Project) -> Self {
        Self {
            name: project.path.clone(),
            forge: Some(project),
            ..Repo::default()
        }
    }
}

impl From<&Project> for Repo {
    fn from(project: &Project) -> Self {
        Self {
            name: project.path.clone(),
            forge: Some(project.to_owned()),
            ..Repo::default()
        }
    }
}

impl std::fmt::Display for Repo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} {}", RepoState::from(self), self.name))
    }
}

impl Debug for Repo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Repo").field("path", &self.name).finish()
    }
}

impl Default for Repo {
    fn default() -> Self {
        Self {
            name: Default::default(),
            path: Default::default(),
            repo: Default::default(),
            forge: Default::default(),
            default_branch: "main".to_string(),
        }
    }
}
