use std::{thread, sync::Arc};

use anyhow::{Context, Result};
use clap::Parser;

use tokio::runtime::Runtime;
use tracing::{debug, metadata::LevelFilter, Level};
use tracing_subscriber::{fmt::format::FmtSpan, prelude::*, EnvFilter};

use crate::repo::{Aggregator, Repos};

pub mod config;
pub mod forge;
pub mod git;
pub mod repo;

mod list;
mod sync;
mod update;

#[cfg(test)]
mod tests;

#[derive(Debug)]
#[allow(dead_code)]
struct GTree {
    figment: figment::Figment,
    config: config::Config,
    args: config::args::Args,
    forge: forge::Forge,
    rt: Runtime,
}

impl GTree {
    #[tracing::instrument(level = "trace")]
    pub fn new() -> Result<GTree> {
        let args = config::args::Args::parse();

        let figment = config::Config::figment()?;
        let config: config::Config = figment.extract()?;

        let (_name, forge_config) = config
            .iter()
            .next()
            .context("No Forge configured, please setup a forge")?;

        let rt = Runtime::new()?;

        let forge = rt.block_on(forge::Forge::new(forge_config))?;

        Ok(GTree {
            figment,
            config,
            args,
            forge,
            rt
        })
    }

    #[tracing::instrument(level = "trace")]
    pub fn run(self) -> Result<()> {
        let scope = Arc::new(self.args.scope.as_ref().map_or("", |x| x).to_string());

        // TODO select a specific forge
        let forge = Arc::new(self
            .config
            .iter()
            .next()
            .context("No Forge configured, please setup a forge")?.1.clone());

        let scope_t = scope.clone();
        let forge_t = forge.clone();
        let handle = thread::spawn(move || {
            Repos::from_local(forge_t.root(), &scope_t)
        });

        let projects = self.rt.block_on(self.forge.projects(&scope))?;
        let remote = Repos::from_forge(forge.root(), projects);

        let local = handle.join().unwrap();
        let repos = Repos::aggregate(local, remote);

        match self.args.command {
            config::args::Commands::Sync => self.sync(repos),
            config::args::Commands::Update => self.update(repos),
            config::args::Commands::List => self.list(repos)?,
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

    gtree.run()?;

    Ok(())
}
