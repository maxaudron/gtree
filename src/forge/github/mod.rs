use anyhow::Result;

mod config;
pub use config::*;

#[derive(Clone, Debug)]
pub struct Github {}

impl Github {
    #[tracing::instrument(level = "trace")]
    pub async fn new(host: &str, token: &str, tls: bool) -> Result<Github> {
        Ok(Github {})
    }

    #[tracing::instrument(level = "trace")]
    pub async fn from_config(forge: &config::Config) -> Result<Github> {
        Github::new(&forge.host, &forge.token, forge.tls).await
    }
}

#[async_trait::async_trait]
impl super::ForgeTrait for Github {
    #[tracing::instrument(level = "trace")]
    async fn projects(&self, scope: &str) -> Result<Vec<super::Project>> {
        let res = Vec::new();

        Ok(res)
    }
}
