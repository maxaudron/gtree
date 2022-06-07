use anyhow::{Context, Result};
use structopt::StructOpt;
use derivative::Derivative;

use tracing::{debug, info, metadata::LevelFilter, trace, Level};
use tracing_subscriber::{fmt::format::FmtSpan, prelude::*, EnvFilter};

use gtree::{
    config,
    local::{Aggregator, Repos},
};

mod list;
mod update;

#[derive(Derivative)]
#[derivative(Debug)]
struct GTree {
    figment: figment::Figment,
    config: config::Config,
    args: config::args::Args,
    forge: gtree::forge::Forge,
    #[derivative(Debug="ignore")]
    gitconfig: git2::Config,
}

impl GTree {
    pub async fn new() -> Result<GTree> {
        let args = config::args::Args::from_args();

        let figment = config::Config::figment()?;
        let config: config::Config = figment.extract()?;

        let (_name, forge_config) = config
            .iter()
            .next()
            .context("No Forge configured, please setup a forge")?;

        let forge = gtree::forge::Forge::new(forge_config).await?;

        let gitconfig = git2::Config::open_default()?;

        Ok(GTree {
            figment,
            config,
            args,
            forge,
            gitconfig,
        })
    }

    pub async fn run(self) -> Result<()> {
        let scope = self.args.scope.as_ref().map_or("", |x| x);

        let (_name, forge) = self
            .config
            .iter()
            .next()
            .context("No Forge configured, please setup a forge")?;

        let (local, remote) = tokio::join!(
            Repos::from_local(forge.root(), scope),
            Repos::from_forge(self.forge.projects(scope).await?)
        );

        let repos = Repos::aggregate(local, remote).await;

        match self.args.command {
            config::args::Commands::Sync => todo!(),
            config::args::Commands::Update => self.update(repos).await,
            config::args::Commands::List => self.list(repos).await?,
        };

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let filter = tracing_subscriber::filter::Targets::new()
        .with_default(Level::TRACE)
        .with_target("hyper", LevelFilter::OFF)
        .with_target("reqwest", LevelFilter::OFF);

    let env_filter = EnvFilter::from_default_env();

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_span_events(FmtSpan::ACTIVE))
        .with(filter)
        .with(env_filter)
        .init();

    debug!("starting");

    let gtree = GTree::new().await?;

    gtree.run().await
}
