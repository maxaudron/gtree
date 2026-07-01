#![allow(clippy::result_large_err)]

use std::{sync::Arc, thread};

use clap::Parser;

use tokio::runtime::Runtime;
use tracing::{Level, debug, metadata::LevelFilter};
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan, prelude::*};

use crate::{
    config::{ConfigError, url::{GitUrl, GitUrlError}},
    forge::{Forge, ForgeError, Project},
    repo::{Aggregator, RepoError, Repos},
};

pub mod config;
pub mod forge;
pub mod repo;

mod batch;
mod cmd;

#[cfg(test)]
mod tests;

use once_cell::sync::OnceCell;

static GTREE: OnceCell<GTree> = OnceCell::new();
static RUNTIME: OnceCell<Runtime> = OnceCell::new();

#[derive(Debug, thiserror::Error)]
enum GTreeError {
    #[error("repo error: {0}")]
    Repo(#[from] RepoError),
    #[error("git url error: {0}")]
    GitUrl(#[from] GitUrlError),
    #[error("error in configuration: {0}")]
    Config(#[from] ConfigError),
    #[error("error in forge: {0}")]
    Forge(#[from] ForgeError),

    #[error("io error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("no forge could be found in the configuration for name: {0}")]
    NoForge(String),
    #[error("no scope was provided on the command line, don't know what to do")]
    NoScope,
    #[error("no projects were found in the forge with the scope {0:?}")]
    NoProjects(GitUrl)
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct GTree {
    figment: figment::Figment,
    config: config::Config,
    args: config::args::Args,
}

impl GTree {
    #[tracing::instrument(level = "trace")]
    pub fn new() -> Result<GTree, GTreeError> {
        let args = config::args::Args::parse();

        let figment = config::Config::figment()?;
        let config: config::Config = figment.extract().map_err(ConfigError::from)?;

        RUNTIME.set(Runtime::new()?).unwrap();

        Ok(GTree {
            figment,
            config,
            args,
        })
    }

    fn get_repos(&self, scope: config::url::GitUrl) -> Result<Repos, GTreeError> {
        // TODO select a specific forge
        let forge_config = Arc::new(
            self.config
                .forge
                .get(&scope.domain)
                .ok_or(GTreeError::NoForge(scope.domain.clone()))?
                .clone(),
        );

        let scope_path = scope.full_path()?;
        let forge_t = forge_config.clone();
        let handle = thread::spawn(move || Repos::from_local(forge_t.root(), &scope_path));

        let scope_path = scope.full_path()?;
        let projects = RUNTIME.get().unwrap().block_on(async {
            let forge = Forge::new(&forge_config).await?;
            Ok::<Vec<Project>, GTreeError>(forge.projects(&scope_path).await?)
        })?;

        if projects.is_empty() {
            return Err(GTreeError::NoProjects(scope))
        }

        let remote = Repos::from_forge(forge_config.root(), projects);

        let local = handle.join().unwrap();
        Ok(Repos::aggregate(local, remote, forge_config.known_hosts()))
    }

    // #[tracing::instrument(level = "trace")]
    pub fn run(self) -> Result<(), GTreeError> {
        match &self.args.command {
            config::args::Commands::Clone(args) => self
                .git_clone(
                    self.config
                        .make_git_url(args.scope.as_deref().ok_or(GTreeError::NoScope)?)?,
                )
                .unwrap(),
            config::args::Commands::Sync(args) => self.sync(
                self.get_repos(
                    self.config
                        .make_git_url(args.scope.as_deref().ok_or(GTreeError::NoScope)?)?,
                )?,
            ),
            config::args::Commands::Update(args) => self.update(
                self.get_repos(
                    self.config
                        .make_git_url(args.scope.as_deref().ok_or(GTreeError::NoScope)?)?,
                )?,
            ),
            config::args::Commands::List(args) => self.list(
                self.get_repos(
                    self.config
                        .make_git_url(args.scope.as_deref().ok_or(GTreeError::NoScope)?)?,
                )?,
            )?,
        };

        Ok(())
    }
}

fn main() -> Result<(), GTreeError> {
    let filter = tracing_subscriber::filter::Targets::new()
        .with_default(Level::TRACE)
        .with_target("hyper", LevelFilter::OFF)
        .with_target("hyper", LevelFilter::OFF)
        .with_target("reqwest", LevelFilter::OFF);

    let env_filter = EnvFilter::from_default_env();

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_span_events(FmtSpan::ACTIVE))
        .with(filter)
        .with(env_filter)
        .init();

    debug!("starting");

    let gtree = GTree::new()?;
    GTREE.set(gtree.clone()).unwrap();

    gtree.run()?;

    Ok(())
}
