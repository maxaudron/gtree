use graphql_client::GraphQLQuery;

mod config;
pub use config::*;
use serde::Deserialize;
use tracing::instrument;

use crate::forge::ForgeError;

use super::ForgeTrait;

#[derive(Clone, Debug)]
pub struct Github {
    client: reqwest::Client,
    base_url: reqwest::Url,
}

impl Github {
    #[tracing::instrument(level = "trace")]
    pub async fn new(host: &str, token: &str, tls: bool) -> Result<Github, ForgeError> {
        let (client, base_url) = Self::new_client(host, token, tls)?;

        Ok(Github { client, base_url })
    }

    #[tracing::instrument(level = "trace")]
    pub async fn from_config(forge: &config::Config) -> Result<Github, ForgeError> {
        Github::new(&forge.host, &forge.token, forge.tls).await
    }
}

impl Github {
    #[instrument(level = "debug", ret, err)]
    async fn graphql(
        &self,
        scope: &str,
        after: Option<String>,
    ) -> Result<projects::ProjectsViewerRepositories, ForgeError> {
        let query = Projects::build_query(projects::Variables { after });

        Ok(self
            .client
            .post(self.base_url.join("graphql")?)
            .json(&query)
            .send()
            .await?
            .json::<ProjectsResponse>()
            .await?
            .data
            .viewer
            .repositories)
    }
}

#[async_trait::async_trait]
impl ForgeTrait for Github {
    #[tracing::instrument(level = "trace")]
    async fn projects(&self, scope: &str) -> Result<Vec<super::Project>, ForgeError> {
        let response = self.graphql(scope, None).await?;

        let mut nodes = response.nodes.unwrap_or_default();

        if nodes.is_empty() {
            return Ok(Vec::new());
        }

        let mut page = response.page_info.end_cursor.unwrap_or_default();
        let mut has_next_page = response.page_info.has_next_page;

        while has_next_page {
            let response = self.graphql(scope, Some(page)).await?;

            page = response.page_info.end_cursor.unwrap_or_default();
            has_next_page = response.page_info.has_next_page;

            nodes.extend(response.nodes.unwrap_or_default());
        }

        return Ok(nodes
            .into_iter()
            .filter_map(|n| n.map(|n| n.into()))
            .collect());
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectsResponse {
    data: projects::ResponseData,
}

#[allow(clippy::upper_case_acronyms)]
type URI = String;
type GitSSHRemote = String;

#[derive(GraphQLQuery)]
#[graphql(
    query_path = "graphql/github_projects_query.graphql",
    schema_path = "graphql/github_schema.json",
    response_derives = "Clone,Debug",
    variables_derives = "Clone,Debug"
)]
pub struct Projects;

impl From<projects::ProjectsViewerRepositoriesNodes> for super::Project {
    fn from(project: projects::ProjectsViewerRepositoriesNodes) -> Self {
        super::Project {
            id: project.id,
            name: project.name,
            path: project.name_with_owner,
            ssh_clone_url: Some(project.ssh_url),
            http_clone_url: Some(project.url),
            default_branch: project.default_branch_ref.map(|d| d.name),
        }
    }
}
