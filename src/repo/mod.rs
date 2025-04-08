use std::{collections::HashMap, fmt::Debug, path::PathBuf, sync::RwLock};

use thiserror::Error;

use gix::Repository;
use tracing::error;

use crate::forge::Project;

mod aggregate;
mod git;
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
}

#[derive(Error, Debug)]
pub enum RepoError {
    #[error("repo is not cloned locally")]
    NoLocalRepo,
    #[error("local git repo does not have a remote")]
    NoRemoteFound,
    #[error("no head found")]
    NoHead,
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
    #[error("currently checked out branch is not default")]
    NonDefaultBranch,
    #[error("{0} unpushed commits")]
    UnpushedCommits(usize),
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
