pub mod args;
pub mod url;

use serde::{Deserialize, Deserializer, Serialize};

use std::{
    collections::BTreeMap,
    ops::Deref,
    path::{Path, PathBuf},
};

use figment::{
    Error, Figment, Metadata, Profile, Provider,
    providers::{Format, Toml},
    value::{Dict, Map},
};

use crate::forge::{ForgeType, github, gitlab};

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("configuration failure: {0}")]
    Figment(#[from] figment::Error),
    #[error("failed to create config directory")]
    IOError(#[from] std::io::Error),
}

/// Configuration for the Bot
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub forge: BTreeMap<String, ForgeConfig>,
    #[serde(default)]
    pub alias: BTreeMap<String, String>,
    #[serde(default)]
    pub settings: Settings,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Settings {
    pub default_forge: Option<String>,
}

impl<'a> Config {
    // Allow the configuration to be extracted from any `Provider`.
    #[allow(clippy::result_large_err)]
    pub fn from<T: Provider>(provider: T) -> Result<Config, ConfigError> {
        Ok(Figment::from(provider).extract()?)
    }

    // Provide a default provider, a `Figment`.
    pub fn figment() -> Result<Figment, ConfigError> {
        use figment::providers::Env;

        let dirs = xdg::BaseDirectories::with_prefix(env!("CARGO_PKG_NAME"));

        Ok(
            Figment::from(Toml::file(dirs.place_config_file("config.toml")?))
                .merge(Toml::file(dirs.place_config_file("config.yaml")?))
                .merge(Env::prefixed("GTREE_")),
        )
    }

    /// Resolve a shorthand, name or domain to the
    /// key of the forge in our config
    pub fn resolve_forge(&'a self, forge: &'a str) -> Option<&'a str> {
        if self.forge.contains_key(forge) {
            Some(forge)
        } else if let Some(forge) = self.alias.get(forge) {
            Some(forge.as_str())
        } else {
            None
        }
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
    Gitlab(gitlab::Config),
    #[serde(alias = "github")]
    Github(github::Config),
}

impl From<&ForgeConfig> for ForgeType {
    fn from(val: &ForgeConfig) -> Self {
        match val {
            ForgeConfig::Gitlab(_) => ForgeType::Gitlab,
            ForgeConfig::Github(_) => ForgeType::Github,
        }
    }
}

pub trait ForgeConfigTrait {
    fn root(&self) -> &Path;
    fn known_hosts(&self) -> Vec<ssh_key::PublicKey>;

    fn deserialize_dir<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
    where
        D: Deserializer<'de>,
        Self: Sized,
    {
        let dir = PathBuf::deserialize(deserializer)?;

        Ok(if dir.is_absolute() {
            dir
        } else {
            let home = PathBuf::from(std::env::var("HOME").unwrap_or_default());
            home.join(dir)
        })
    }
}

impl Deref for ForgeConfig {
    type Target = dyn ForgeConfigTrait;

    fn deref(&self) -> &Self::Target {
        match self {
            ForgeConfig::Gitlab(conf) => conf,
            ForgeConfig::Github(conf) => conf,
        }
    }
}
