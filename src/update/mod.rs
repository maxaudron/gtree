use std::fmt::{Debug, Display};

use git2::BranchType;
use tracing::debug;

use crate::{
    batch::batch,
    repo::{Repo, RepoError, Repos},
};

impl crate::GTree {
    pub fn update(&self, repos: Repos) {
        batch(repos, |mut repo| {
            if repo.repo.is_some() {
                match repo.update() {
                    Ok(u) => println!("{}", u),
                    Err(u) => println!("{}", u),
                };
            }
        });
    }
}

impl Repo {
    /// Fetch any new state from the remote and fast forward merge changes into local branches
    #[tracing::instrument(level = "trace")]
    pub fn update(&mut self) -> Result<UpdateResult, UpdateResult> {
        let repo_name = self.name.clone();
        if self.repo.is_some() {
            self.update_inner()
                .map_err(|e| UpdateResult::err(repo_name, e))
        } else {
            Ok(UpdateResult::err(repo_name, RepoError::NoLocalRepo))
        }
    }

    fn update_inner(&mut self) -> Result<UpdateResult, RepoError> {
        let repo = self.repo.as_ref().unwrap();
        let mut remote = self.main_remote(repo)?;

        self.fetch(&mut remote)?;

        self.default_branch = remote.default_branch()?.as_str().unwrap().to_string();

        debug!("default branch: {}", self.default_branch);

        if self.is_clean()? {
            debug!("repo is clean");

            let merged = repo.branches(Some(BranchType::Local))?
                    .filter_map(|x| x.ok())
                    .try_fold(false, |mut merged, (mut branch, _)| {
                        let name = format!("refs/heads/{}", Repo::branch_name(&branch));

                        if branch.upstream().is_ok() {
                            let upstream = branch.upstream().unwrap();

                            debug!("branch: {}", name);

                            merged |= self.merge(repo, &mut branch, &upstream)?;
                            Ok::<bool, RepoError>(merged)
                        } else {
                            debug!("not updating branch: {}: branch does not have upstream tracking branch set", name);
                            Ok(merged)
                        }
                    })?;

            if merged {
                Ok(UpdateResult::merged(self.name.clone()))
            } else {
                Ok(UpdateResult::no_changes(self.name.clone()))
            }
        } else {
            Ok(UpdateResult::dirty(self.name.clone()))
        }
    }
}

#[derive(Debug)]
pub enum UpdateResult {
    NoChanges { name: String },
    Dirty { name: String },
    Merged { name: String },
    Error { name: String, error: RepoError },
}

impl UpdateResult {
    pub fn err(name: String, error: RepoError) -> UpdateResult {
        UpdateResult::Error { name, error }
    }

    pub fn merged(name: String) -> UpdateResult {
        UpdateResult::Merged { name }
    }

    pub fn dirty(name: String) -> UpdateResult {
        UpdateResult::Dirty { name }
    }

    pub fn no_changes(name: String) -> UpdateResult {
        UpdateResult::NoChanges { name }
    }
}

impl Display for UpdateResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ansi_term::Colour::{Blue, Green, Red, Yellow};

        match self {
            UpdateResult::NoChanges { name } => {
                f.write_fmt(format_args!("{} {}", Blue.paint("FETCHED"), name))
            }
            UpdateResult::Dirty { name } => {
                f.write_fmt(format_args!("{} {}", Yellow.paint("DIRTY  "), name))
            }
            UpdateResult::Merged { name } => {
                f.write_fmt(format_args!("{} {}", Green.paint("PULLED "), name))
            }
            UpdateResult::Error { name, error } => f.write_fmt(format_args!(
                "{} {} [{}]",
                Red.paint("ERROR  "),
                name,
                error
            )),
        }
    }
}
