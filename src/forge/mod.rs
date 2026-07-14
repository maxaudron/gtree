use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::config::ForgeConfig;

pub mod github;
pub mod gitlab;

static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, thiserror::Error)]
pub enum ForgeError {
    #[error("unknown forge type found")]
    UnknownForge,

    #[error("request error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("failed to parse url: {0}")]
    Url(#[from] url::ParseError),

    #[error("internal error")]
    InternalError,
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
    Github(self::github::Github),
}

impl Forge {
    pub async fn new(config: &ForgeConfig) -> Result<Forge, ForgeError> {
        match config {
            ForgeConfig::Gitlab(config) => {
                Ok(Forge::Gitlab(gitlab::Gitlab::from_config(config).await?))
            }
            ForgeConfig::Github(config) => {
                Ok(Forge::Github(github::Github::from_config(config).await?))
            }
            #[allow(unreachable_patterns)]
            _ => Err(ForgeError::UnknownForge),
        }
    }
}

#[async_trait::async_trait]
pub trait ForgeTrait {
    async fn projects(&self, scope: &str) -> Result<Vec<Project>, ForgeError>;

    fn new_client(
        host: &str,
        token: &str,
        tls: bool,
    ) -> Result<(reqwest::Client, reqwest::Url), ForgeError>
    where
        Self: Sized,
    {
        let mut headers = reqwest::header::HeaderMap::new();
        let auth = format!("Bearer {token}");
        let mut auth_value = reqwest::header::HeaderValue::from_str(&auth).unwrap();
        auth_value.set_sensitive(true);
        headers.insert(reqwest::header::AUTHORIZATION, auth_value);

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .user_agent(USER_AGENT)
            .build()?;

        let schema = if tls { "https" } else { "http" };
        let base_url = reqwest::Url::parse(&format!("{schema}://{host}"))?;

        Ok((client, base_url))
    }
}

impl Deref for Forge {
    type Target = dyn ForgeTrait;

    fn deref(&self) -> &Self::Target {
        match self {
            Forge::Gitlab(forge) => forge,
            Forge::Github(forge) => forge,
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
    pub default_branch: Option<String>,
}
