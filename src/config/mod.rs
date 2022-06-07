pub mod args;

use serde::{Deserialize, Serialize};

use std::{collections::BTreeMap, ops::Deref, path::Path};

use figment::{
    providers::{Format, Toml},
    value::{Dict, Map},
    Error, Figment, Metadata, Profile, Provider,
};

use anyhow::{Context, Result};

/// Configuration for the Bot
#[derive(Clone, Debug, Deserialize, Serialize)]
// pub struct Config();

// TODO make forge optional
pub struct Config {
    #[serde(flatten)]
    config: BTreeMap<String, ForgeConfig>
}

impl Deref for Config {
    type Target = BTreeMap<String, ForgeConfig>;

    fn deref(&self) -> &Self::Target {
        &self.config
    }
}

impl Config {
    // Allow the configuration to be extracted from any `Provider`.
    fn from<T: Provider>(provider: T) -> Result<Config, Error> {
        Figment::from(provider).extract()
    }

    // Provide a default provider, a `Figment`.
    pub fn figment() -> Result<Figment> {
        use figment::providers::Env;

        let dirs = xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME")).unwrap();

        Ok(Figment::from(Toml::file(
            dirs.place_config_file("config.toml")
                .context("failed to create config directory")?,
        ))
        .merge(Toml::file(
            dirs.place_config_file("config.yaml")
                .context("failed to create config directory")?,
        ))
        .merge(Env::prefixed("GTREE_")))
    }
}

// Make `Config` a provider itself for composability.
impl Provider for Config {
    fn metadata(&self) -> Metadata {
        Metadata::named("Library Config")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        figment::providers::Serialized::defaults(self).data()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum ForgeConfig {
    #[serde(alias = "gitlab")]
    Gitlab(crate::forge::gitlab::config::Gitlab),
}

pub trait ForgeConfigTrait {
    fn root(&self) -> &str;
}

impl Deref for ForgeConfig {
    type Target = dyn ForgeConfigTrait;

    fn deref(&self) -> &Self::Target {
        match self {
            ForgeConfig::Gitlab(conf) => conf,
        }
    }
}
