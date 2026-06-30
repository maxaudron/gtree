use std::ops::Deref;

use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};

use crate::config::ForgeConfig;

pub mod gitlab;
pub mod github;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ForgeType {
    Gitlab,
    Github,
    Forgejo,
    SSH,
}

#[derive(Clone, Debug)]
pub enum Forge {
    Gitlab(self::gitlab::Gitlab),
}

impl Forge {
    pub async fn new(config: &ForgeConfig) -> Result<Forge> {
        match config {
            ForgeConfig::Gitlab(config) => {
                Ok(Forge::Gitlab(gitlab::Gitlab::from_config(config).await?))
            }
            #[allow(unreachable_patterns)]
            _ => bail!("wrong forge type found"),
        }
    }
}

#[async_trait::async_trait]
pub trait ForgeTrait {
    async fn projects(&self, scope: &str) -> Result<Vec<Project>>;
}

impl Deref for Forge {
    type Target = dyn ForgeTrait;

    fn deref(&self) -> &Self::Target {
        match self {
            Forge::Gitlab(forge) => forge,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    pub ssh_clone_url: Option<String>,
    pub http_clone_url: Option<String>,
}
