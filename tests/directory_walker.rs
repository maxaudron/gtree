use gtree::local::*;

use anyhow::Result;
use git2::Repository;

const TEST_DIR: &str = env!("CARGO_TARGET_TMPDIR");

const REPOS: [&str; 5] = [
    "repos/site/group/repo1",
    "repos/site/group/repo2/subrepo1",
    "repos/site/group/repo2",
    "repos/site/group/subgroup/repo3",
    "repos/site/group/subgroup/subsubgroup/repo4",
];

fn prepare_repos() -> Result<()> {
    REPOS.iter().try_for_each(|repo| {
        let path = format!("{}/{}", TEST_DIR, repo);
        std::fs::create_dir_all(&path)?;
        let _repo = Repository::init(&path)?;

        Ok::<(), anyhow::Error>(())
    })
}

fn clean_repos() -> Result<()> {
    REPOS.iter().try_for_each(|repo| {
        let path = format!("{}/{}", TEST_DIR, repo);
        std::fs::remove_dir_all(&path)?;

        Ok::<(), anyhow::Error>(())
    })
}

#[tokio::test]
async fn search_repos() -> Result<()> {
    tracing_subscriber::fmt::init();

    prepare_repos()?;

    let mut left: Vec<String> = vec![
        format!("{}/repos/site/group/repo1", TEST_DIR),
        format!("{}/repos/site/group/repo2", TEST_DIR),
        format!("{}/repos/site/group/subgroup/repo3", TEST_DIR),
        format!("{}/repos/site/group/subgroup/subsubgroup/repo4", TEST_DIR),
    ];
    let right = Repos::from_local(&format!("{}/repos", TEST_DIR), "").await;

    let mut right: Vec<&str> = right.iter().map(|x| x.name.as_str()).collect();

    assert_eq!(left.sort(), right.sort());

    clean_repos()?;
    Ok(())
}
