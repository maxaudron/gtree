use std::path::{Path, PathBuf};

use crate::repo::*;

use anyhow::Result;
use tracing::debug;

const REPOS: [&str; 5] = [
    "repos/site/group/repo1",
    "repos/site/group/repo2/subrepo1",
    "repos/site/group/repo2",
    "repos/site/group/subgroup/repo3",
    "repos/site/group/subgroup/subsubgroup/repo4",
];

fn prepare_repos(path: &Path) -> Result<()> {
    REPOS.iter().try_for_each(|repo| {
        let path = build_path(path, repo);
        std::fs::create_dir_all(&path)?;
        let _repo = gix::init(&path)?;

        Ok::<(), anyhow::Error>(())
    })
}

fn build_path(path: &Path, repo: &str) -> PathBuf {
    let mut path = path.to_owned();
    path.push(repo);
    return path;
}

fn build_path_string(path: &Path, repo: &str) -> String {
    let mut path = path.to_owned();
    path.push(repo);
    return path.into_os_string().into_string().unwrap();
}

#[tokio::test]
async fn search_repos() -> Result<()> {
    tracing_subscriber::fmt::init();

    let test_dir = tempfile::tempdir()?;
    let test_path = test_dir.path();
    debug!("test files placed in: {:?}", test_dir.path());

    prepare_repos(test_path)?;

    let mut left: Vec<String> = vec![
        build_path_string(test_path, "repos/site/group/repo1"),
        build_path_string(test_path, "repos/site/group/repo2"),
        build_path_string(test_path, "repos/site/group/subgroup/repo3"),
        build_path_string(test_path, "repos/site/group/subgroup/subsubgroup/repo4"),
    ];
    let right = Repos::from_local(&build_path_string(test_path, "repos"), "");

    let mut right: Vec<&str> = right.iter().map(|x| x.0.as_str()).collect();

    assert_eq!(left.sort(), right.sort_unstable());

    Ok(())
}
