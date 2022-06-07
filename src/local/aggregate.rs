use std::path::PathBuf;

use git2::Repository;

use tracing::{debug, error};
use walkdir::WalkDir;

use crate::forge::Project;

use super::{Repo, Repos};

#[async_trait::async_trait]
pub trait Aggregator {
    async fn from_local(root: &str, scope: &str) -> Repos;
    async fn from_forge(root: &str, projects: Vec<Project>) -> Repos;
    async fn aggregate(mut local: Repos, mut remote: Repos) -> Repos;
}

#[async_trait::async_trait]
impl Aggregator for Repos {
    #[tracing::instrument(level = "trace")]
    async fn from_local(root: &str, scope: &str) -> Repos {
        let mut repos = Vec::new();

        let path: std::path::PathBuf = [root, scope].iter().collect();

        if !path.exists() {
            return repos;
        }

        let mut walker = WalkDir::new(path).into_iter();

        loop {
            let entry = match walker.next() {
                None => break,
                Some(Err(err)) => panic!("ERROR: {}", err),
                Some(Ok(entry)) => entry,
            };

            if entry.file_type().is_dir() {
                let mut dir = std::fs::read_dir(entry.path()).unwrap();

                if let Some(_) = dir.find(|dir| {
                    if let Ok(dir) = dir {
                        dir.file_name() == ".git"
                    } else {
                        false
                    }
                }) {
                    walker.skip_current_dir();

                    match Repository::open(entry.path()) {
                        Ok(repo) => repos.push(Repo {
                            name: entry
                                .path()
                                .strip_prefix(root)
                                .unwrap()
                                .to_str()
                                .unwrap()
                                .to_string(),
                            path: entry.path().to_path_buf(),
                            repo: Some(repo),
                            ..Repo::default()
                        }),
                        Err(err) => error!("could not open repository: {}", err),
                    }
                } else {
                    continue;
                }
            }
        }

        return repos;
    }

    #[tracing::instrument(level = "trace")]
    async fn from_forge(root: &str, projects: Vec<Project>) -> Repos {
        projects
            .iter()
            .map(|project| {
                let mut repo: Repo = project.into();
                repo.path = [root, &repo.name].iter().collect();
                return repo;
            })
            .collect()
    }

    #[tracing::instrument(level = "trace", skip(local, remote))]
    async fn aggregate(mut local: Repos, mut remote: Repos) -> Repos {
        local = local
            .into_iter()
            .map(|mut left| {
                if let Some(i) = remote.iter().position(|right| *right == left) {
                    let right = remote.remove(i);
                    left.forge = right.forge;
                }

                left
            })
            .collect();

        local.append(&mut remote);
        local.sort();

        return local;
    }
}
