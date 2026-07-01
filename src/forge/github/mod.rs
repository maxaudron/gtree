mod config;
pub use config::*;

use crate::forge::ForgeError;

#[derive(Clone, Debug)]
pub struct Github {}

impl Github {
    #[tracing::instrument(level = "trace")]
    pub async fn new(host: &str, token: &str, tls: bool) -> Result<Github, ForgeError> {
        Ok(Github {})
    }

    #[tracing::instrument(level = "trace")]
    pub async fn from_config(forge: &config::Config) -> Result<Github, ForgeError> {
        Github::new(&forge.host, &forge.token, forge.tls).await
    }
}

#[async_trait::async_trait]
impl super::ForgeTrait for Github {
    #[tracing::instrument(level = "trace")]
    async fn projects(&self, scope: &str) -> Result<Vec<super::Project>, ForgeError> {
        let res = Vec::new();

        Ok(res)
    }
}
