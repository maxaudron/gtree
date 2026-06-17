use std::fmt::{Debug, Display};

use crate::{
    batch::batch,
    repo::{LocalRepoState, Repo, RepoError, Repos},
};

impl crate::GTree {
    pub fn sync(&self, repos: Repos) {
        batch(repos, |mut repo| {
            match repo.sync() {
                Ok(u) => println!("{}", u),
                Err(u) => println!("{}", u),
            };
        });
    }
}

impl Repo {
    /// Clone repos from forge and push new repos to forge
    #[tracing::instrument(level = "trace")]
    pub fn sync(&mut self) -> Result<SyncResult, SyncResult> {
        let repo_name = self.name.clone();

        let repo_state = self
            .is_clean()
            .map_err(|err| SyncResult::err(repo_name.clone(), err))?;
        if self.repo.is_some() && repo_state != LocalRepoState::Clean {
            return Ok(SyncResult::dirty(repo_name, repo_state));
        };

        if self.repo.is_some() && self.forge.is_some() {
            Ok(SyncResult::no_changes(repo_name))
        } else if self.repo.is_some() {
            // TODO do push stuff
            Ok(SyncResult::pushed(repo_name))
        } else if self.forge.is_some() {
            let url = self
                .forge
                .as_ref()
                .unwrap()
                .ssh_clone_url
                .as_ref()
                .ok_or_else(|| SyncResult::err(self.name.clone(), RepoError::NoRemoteFound))?.clone();

            self.clone(&url)
                .map_err(|err| SyncResult::err(repo_name.clone(), err))?;

            // TODO detect moved repos based on first commit
            // ???? how to detect and not move forks?

            Ok(SyncResult::cloned(repo_name))
        } else {
            Ok(SyncResult::no_changes(repo_name))
        }
    }
}

#[derive(Debug)]
pub enum SyncResult {
    NoChanges { name: String },
    Dirty { name: String, state: LocalRepoState },
    Cloned { name: String },
    Pushed { name: String },
    Error { name: String, error: RepoError },
}

impl SyncResult {
    pub fn err(name: String, error: RepoError) -> SyncResult {
        SyncResult::Error { name, error }
    }

    pub fn cloned(name: String) -> SyncResult {
        SyncResult::Cloned { name }
    }

    pub fn pushed(name: String) -> SyncResult {
        SyncResult::Pushed { name }
    }

    pub fn dirty(name: String, state: LocalRepoState) -> SyncResult {
        SyncResult::Dirty { name, state }
    }

    pub fn no_changes(name: String) -> SyncResult {
        SyncResult::NoChanges { name }
    }
}

impl Display for SyncResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ansi_term::Colour::{Blue, Green, Red, Yellow};

        match self {
            SyncResult::NoChanges { name } => {
                f.write_fmt(format_args!("{} {}", Blue.paint("NOCHANGE"), name))
            }
            SyncResult::Dirty { name, state } => f.write_fmt(format_args!(
                "{} {} [{}]",
                Yellow.paint("DIRTY   "),
                name,
                state
            )),
            SyncResult::Cloned { name } => {
                f.write_fmt(format_args!("{} {}", Green.paint("CLONED  "), name))
            }
            SyncResult::Pushed { name } => {
                f.write_fmt(format_args!("{} {}", Green.paint("PUSHED  "), name))
            }
            SyncResult::Error { name, error } => f.write_fmt(format_args!(
                "{} {} [{}]",
                Red.paint("ERROR  "),
                name,
                error
            )),
        }
    }
}
