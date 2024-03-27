use std::{collections::HashMap, os::unix::ffi::OsStrExt, sync::RwLock};

use gix::bstr::ByteSlice;
use tracing::{debug, error};
use walkdir::WalkDir;

use crate::forge::Project;

use super::{Repo, Repos};

#[async_trait::async_trait]
pub trait Aggregator {
    fn from_local(root: &str, scope: &str) -> Repos;
    fn from_forge(root: &str, projects: Vec<Project>) -> Repos;
    fn aggregate(local: Repos, remote: Repos) -> Repos;
}

#[async_trait::async_trait]
impl Aggregator for Repos {
    #[tracing::instrument(level = "trace")]
    fn from_local(root: &str, scope: &str) -> Repos {
        let mut repos = HashMap::new();

        let path: std::path::PathBuf = root.into();

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

            if entry.file_type().is_dir() && entry.path().as_os_str().as_bytes().contains_str(scope)
            {
                let mut dir = std::fs::read_dir(entry.path()).unwrap();

                if dir.any(|dir| {
                    if let Ok(dir) = dir {
                        dir.file_name() == ".git"
                    } else {
                        false
                    }
                }) {
                    walker.skip_current_dir();

                    debug!("found git repo {:?} trying to open...", entry.path());

                    match gix::open(entry.path()) {
                        Ok(repo) => {
                            let name = entry
                                .path()
                                .strip_prefix(root)
                                .unwrap()
                                .to_str()
                                .unwrap()
                                .to_string();

                            repos.insert(
                                name.clone(),
                                RwLock::new(Repo {
                                    name,
                                    path: entry.path().to_path_buf(),
                                    repo: Some(repo),
                                    ..Repo::default()
                                }),
                            );
                        }
                        Err(err) => error!("could not open repository: {}", err),
                    }
                } else {
                    continue;
                }
            }
        }

        repos
    }

    #[tracing::instrument(level = "trace")]
    fn from_forge(root: &str, projects: Vec<Project>) -> Repos {
        projects
            .iter()
            .map(|project| {
                let mut repo: Repo = project.into();
                repo.path = [root, &repo.name].iter().collect();
                (repo.name.clone(), RwLock::new(repo))
            })
            .collect()
    }

    // TODO optimise this func
    //
    // the iteration is currently quite inefficient as
    // it's constantly removing stuff from `remote`
    #[tracing::instrument(level = "trace", skip(local, remote))]
    fn aggregate(mut local: Repos, mut remote: Repos) -> Repos {
        local = local
            .into_iter()
            .map(|(left_name, left)| {
                if let Some(right) = remote.remove(&left_name) {
                    left.write().unwrap().forge = right.into_inner().unwrap().forge;
                }

                (left_name, left)
            })
            .collect();

        local.extend(remote.into_iter());
        // local.sort();

        local
    }
}
