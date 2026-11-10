use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::config::{ForgeConfigTrait, GenericForgeConfig};

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "default_host")]
    pub host: String,
    pub token: String,
    #[serde(deserialize_with = "Config::deserialize_dir")]
    pub directory: PathBuf,
    #[serde(default = "default_tls")]
    pub tls: bool,
    #[serde(default)]
    pub auto_create_branches: bool,

    #[serde(flatten)]
    pub generic: GenericForgeConfig,
}

fn default_host() -> String {
    String::from("gitlab.com")
}

const fn default_tls() -> bool {
    true
}

impl ForgeConfigTrait for Config {
    fn root(&self) -> &Path {
        &self.directory
    }

    fn generic(&self) -> &GenericForgeConfig {
        &self.generic
    }
}
