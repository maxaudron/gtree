use std::{collections::HashMap, path::Path, sync::RwLock};

use tracing::{debug, error};
use walkdir::WalkDir;

use crate::forge::Project;

use super::{Repo, Repos};

#[async_trait::async_trait]
pub trait Aggregator {
    fn from_local(root: &Path, scope: &str) -> Repos;
    fn from_forge(root: &Path, projects: Vec<Project>) -> Repos;
    fn aggregate(local: Repos, remote: Repos, known_hosts: Vec<ssh_key::PublicKey>) -> Repos;
}

#[async_trait::async_trait]
impl Aggregator for Repos {
    #[tracing::instrument(level = "trace", ret)]
    fn from_local(root: &Path, scope: &str) -> Repos {
        let mut repos = HashMap::new();

        let path: std::path::PathBuf = root.to_owned();

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

            if entry.file_type().is_dir() && entry.path().to_str().unwrap().contains(scope) {
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

                    match git2::Repository::open(entry.path()) {
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

    #[tracing::instrument(level = "trace", ret)]
    fn from_forge(root: &Path, projects: Vec<Project>) -> Repos {
        projects
            .iter()
            .map(|project| {
                let mut repo: Repo = project.into();
                repo.path = root.join(&repo.name);
                debug!("repo path: {:#?}", repo.path);
                (repo.name.clone(), RwLock::new(repo))
            })
            .collect()
    }

    // TODO optimise this func
    //
    // the iteration is currently quite inefficient as
    // it's constantly removing stuff from `remote`
    #[tracing::instrument(level = "trace", skip(local, remote))]
    fn aggregate(
        mut local: Repos,
        mut remote: Repos,
        known_hosts: Vec<ssh_key::PublicKey>,
    ) -> Repos {
        let known_hosts: Vec<[u8; 32]> = known_hosts
            .iter()
            .map(|k| k.fingerprint(ssh_key::HashAlg::Sha256).sha256().unwrap())
            .collect();
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
        local.iter_mut().for_each(|(_, r)| {
            r.write().unwrap().known_hosts = known_hosts.clone();
        });
        // local.sort();

        local
    }
}
