use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::config::ForgeConfigTrait;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Deserialize, Serialize)]
pub struct Gitlab {
    // pub url: url::Url,
    pub host: String,
    pub token: String,
    pub directory: PathBuf,
    #[serde(default = "default_tls")]
    pub tls: bool,
    #[serde(default)]
    pub auto_create_branches: bool,
}

const fn default_tls() -> bool {
    true
}

impl ForgeConfigTrait for Gitlab {
    fn root(&self) -> &str {
        self.directory.to_str().unwrap()
    }
}
