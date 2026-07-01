use std::{sync::Arc, thread};

use anyhow::{Context, Result};
use clap::Parser;

use tokio::runtime::Runtime;
use tracing::{Level, debug, metadata::LevelFilter};
use tracing_subscriber::{EnvFilter, fmt::format::FmtSpan, prelude::*};

use crate::repo::{Aggregator, Repos};

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

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct GTree {
    figment: figment::Figment,
    config: config::Config,
    args: config::args::Args,
    forge: forge::Forge,
}

impl GTree {
    #[tracing::instrument(level = "trace")]
    pub fn new() -> Result<GTree> {
        let args = config::args::Args::parse();

        let figment = config::Config::figment()?;
        let config: config::Config = figment.extract()?;

        let (_name, forge_config) = config
            .forge
            .iter()
            .next()
            .context("No Forge configured, please setup a forge")?;

        RUNTIME.set(Runtime::new()?).unwrap();

        let forge = RUNTIME
            .get()
            .unwrap()
            .block_on(forge::Forge::new(forge_config))?;

        Ok(GTree {
            figment,
            config,
            args,
            forge,
        })
    }

    fn get_repos(&self, scope: config::url::GitUrl) -> Result<Repos, anyhow::Error> {
        // TODO select a specific forge
        let forge = Arc::new(
            self.config
                .forge
                .get(&scope.domain)
                .context("No Forge configured, please setup a forge")?
                .clone(),
        );

        let scope_path = scope.full_path()?;
        let forge_t = forge.clone();
        let handle = thread::spawn(move || Repos::from_local(forge_t.root(), &scope_path));

        let scope_path = scope.full_path()?;
        let projects = RUNTIME
            .get()
            .unwrap()
            .block_on(self.forge.projects(&scope_path))?;
        let remote = Repos::from_forge(forge.root(), projects);

        let local = handle.join().unwrap();
        Ok(Repos::aggregate(local, remote, forge.known_hosts()))
    }

    // #[tracing::instrument(level = "trace")]
    pub fn run(self) -> Result<()> {
        match &self.args.command {
            config::args::Commands::Clone(args) => self
                .git_clone(
                    self.config.make_git_url(
                        args.scope
                            .as_deref()
                            .ok_or(anyhow::anyhow!("no scope provided"))?,
                    )?,
                )
                .unwrap(),
            config::args::Commands::Sync(args) => self.sync(
                self.get_repos(
                    self.config.make_git_url(
                        args.scope
                            .as_deref()
                            .ok_or(anyhow::anyhow!("no scope provided"))?,
                    )?,
                )?,
            ),
            config::args::Commands::Update(args) => self.update(
                self.get_repos(
                    self.config.make_git_url(
                        args.scope
                            .as_deref()
                            .ok_or(anyhow::anyhow!("no scope provided"))?,
                    )?,
                )?,
            ),
            config::args::Commands::List(args) => self.list(
                self.get_repos(
                    self.config.make_git_url(
                        args.scope
                            .as_deref()
                            .ok_or(anyhow::anyhow!("no scope provided"))?,
                    )?,
                )?,
            )?,
        };

        Ok(())
    }
}

fn main() -> Result<()> {
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
