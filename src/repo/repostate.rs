use super::Repo;

#[derive(Clone, Debug)]
pub enum RepoState {
    Local,
    Remote,
    Synced,
    Unknown,
}

impl std::fmt::Display for RepoState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ansi_term::Colour::{Blue, Green, Red, Yellow};

        match self {
            RepoState::Local => f.write_str(&Yellow.paint("LOCAL  ").to_string()),
            RepoState::Remote => f.write_str(&Blue.paint("REMOTE ").to_string()),
            RepoState::Synced => f.write_str(&Green.paint("SYNCED ").to_string()),
            RepoState::Unknown => f.write_str(&Red.paint("UNKNOWN").to_string()),
        }
    }
}

impl From<&Repo> for RepoState {
    fn from(repo: &Repo) -> Self {
        if repo.repo.is_some() && repo.forge.is_some() {
            RepoState::Synced
        } else if repo.repo.is_some() {
            RepoState::Local
        } else if repo.forge.is_some() {
            RepoState::Remote
        } else {
            RepoState::Unknown
        }
    }
}
