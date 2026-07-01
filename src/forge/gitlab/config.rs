use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::config::ForgeConfigTrait;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct Config {
    pub host: String,
    pub token: String,
    pub directory: PathBuf,
    #[serde(default = "default_tls")]
    pub tls: bool,
    #[serde(default)]
    pub auto_create_branches: bool,

    #[serde(default)]
    pub known_hosts: Vec<ssh_key::PublicKey>,
}

const fn default_tls() -> bool {
    true
}

impl ForgeConfigTrait for Config {
    fn root(&self) -> &Path {
        &self.directory
    }

    fn known_hosts(&self) -> Vec<ssh_key::PublicKey> {
        self.known_hosts.clone()
    }
}
