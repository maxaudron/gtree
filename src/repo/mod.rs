use std::{
    collections::HashMap,
    fmt::Debug,
    path::PathBuf,
    sync::RwLock,
};

use thiserror::Error;

use git2::{Branch, Remote, Repository};
use tracing::{debug, trace};

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
    pub fn is_clean(&self) -> Result<bool, RepoError> {
        if let Some(repo) = &self.repo {
            debug!("repo state: {:?}", repo.state());
            let statuses: Vec<git2::Status> = repo
                .statuses(None)?
                .iter()
                .filter_map(|status| {
                    if status.status().is_ignored() {
                        None
                    } else {
                        Some(status.status())
                    }
                })
                .collect();

            debug!("got repo statuses: {:?}", statuses);

            if repo.state() == git2::RepositoryState::Clean && statuses.is_empty() {
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err(RepoError::NoLocalRepo)
        }
    }

    pub fn main_remote<'a>(&self, repo: &'a Repository) -> Result<git2::Remote<'a>, RepoError> {
        let remotes = repo.remotes()?;

        let remote = if remotes.iter().any(|x| x == Some("origin")) {
            "origin"
        } else if let Some(remote) = remotes.get(0) {
            remote
        } else {
            return Err(RepoError::NoRemoteFound);
        };

        return Ok(repo.find_remote(remote)?);
    }

    #[tracing::instrument(level = "trace", skip(remote))]
    pub fn fetch<'a>(&self, remote: &mut Remote) -> Result<(), RepoError> {
        // Pass an empty array as the refspec to fetch to fetch the "default" refspecs
        // Type annotation is needed because type can't be guessed from the empty array
        remote.fetch::<&str>(
            &[],
            Some(&mut crate::git::fetch_options()),
            Some("gtree fetch"),
        )?;

        Ok(())
    }

    #[tracing::instrument(level = "trace")]
    pub fn clone(&self, url: &str) -> Result<Repository, RepoError> {
        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(crate::git::fetch_options());

        builder.clone(url, &self.path).map_err(RepoError::GitError)
    }

    #[tracing::instrument(level = "trace")]
    pub fn checkout(&self) -> Result<(), RepoError> {
        if let Some(repo) = &self.repo {
            repo.checkout_head(None).map_err(|e| e.into())
        } else {
            Err(RepoError::NoLocalRepo)
        }
    }

    pub fn branch_name(branch: &Branch) -> String {
        match branch.name().unwrap() {
            Some(s) => s.to_string(),
            None => String::from_utf8_lossy(branch.name_bytes().unwrap()).to_string(),
        }
    }

    #[tracing::instrument(level = "trace", skip(repo, local, upstream))]
    pub fn merge(
        &self,
        repo: &Repository,
        local: &mut Branch,
        upstream: &Branch,
    ) -> Result<bool, RepoError> {
        let local_name = Repo::branch_name(local);
        let upstream_name = Repo::branch_name(upstream);

        let local_ref = local.get_mut();
        let upstream_ref = upstream.get();

        let analysis = repo.merge_analysis_for_ref(
            local_ref,
            &[&repo.reference_to_annotated_commit(upstream_ref)?],
        )?;

        if analysis.0.is_fast_forward() {
            trace!("Doing a fast forward");

            let msg = format!(
                "gtree: update repo branch: {} to {}",
                local_name, upstream_name
            );
            debug!("{}", msg);

            // sets the branch to target the new commit
            // of the remote branch it's tracking
            local_ref.set_target(upstream_ref.target().unwrap(), &msg)?;
            // Apply these changes in the working dir if the branch is currently checked out.
            if format!("refs/heads/{}", local_name) == self.default_branch {
                repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?
            }

            Ok(true)
        } else if analysis.0.is_up_to_date() {
            Ok(false)
        } else {
            Err(RepoError::NoFF)
        }
    }
}

#[derive(Error, Debug)]
pub enum RepoError {
    #[error("repo is not cloned locally")]
    NoLocalRepo,
    #[error("local git repo does not have a remote")]
    NoRemoteFound,
    #[error("repository is dirty")]
    Dirty,
    #[error("fast-forward merge was not possible")]
    NoFF,
    #[error("error during git operation {0}")]
    GitError(#[from] git2::Error),
    #[error("unknown repo error")]
    Unknown,
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
