use std::fmt::Display;

#[derive(Debug)]
pub enum UpdateResult {
    NoChanges {
        name: String,
    },
    Dirty {
        name: String,
    },
    Merged {
        name: String,
    },
    Error {
        name: String,
        error: super::RepoError,
    },
}

impl UpdateResult {
    pub fn err(name: String, error: super::RepoError) -> UpdateResult {
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
            UpdateResult::Error { name, error } => {
                f.write_fmt(format_args!("{} {} [{}]", Red.paint("ERROR  "), name, error))
            }
        }
    }
}
