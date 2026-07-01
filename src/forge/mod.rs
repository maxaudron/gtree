use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::config::ForgeConfig;

pub mod gitlab;
pub mod github;

#[derive(Debug, thiserror::Error)]
pub enum ForgeError {
    #[error("unknown forge type found")]
    UnknownForge,

    #[error("gitlab error: {0}")]
    Gitlab(#[from] ::gitlab::GitlabError),
}

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
    pub async fn new(config: &ForgeConfig) -> Result<Forge, ForgeError> {
        match config {
            ForgeConfig::Gitlab(config) => {
                Ok(Forge::Gitlab(gitlab::Gitlab::from_config(config).await?))
            }
            #[allow(unreachable_patterns)]
            _ => Err(ForgeError::UnknownForge)
        }
    }
}

#[async_trait::async_trait]
pub trait ForgeTrait {
    async fn projects(&self, scope: &str) -> Result<Vec<Project>, ForgeError>;
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
