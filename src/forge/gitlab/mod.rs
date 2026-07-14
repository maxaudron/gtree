use graphql_client::GraphQLQuery;

mod config;
pub use config::*;
use serde::Deserialize;
use tracing::instrument;

use crate::forge::ForgeError;

use super::ForgeTrait;

#[derive(Clone, Debug)]
pub struct Gitlab {
    client: reqwest::Client,
    base_url: reqwest::Url,
}

impl Gitlab {
    #[tracing::instrument(level = "trace")]
    pub async fn new(host: &str, token: &str, tls: bool) -> Result<Gitlab, ForgeError> {
        let (client, base_url) = Self::new_client(host, token, tls)?;

        Ok(Gitlab { client, base_url })
    }

    #[tracing::instrument(level = "trace")]
    pub async fn from_config(forge: &config::Config) -> Result<Gitlab, ForgeError> {
        Gitlab::new(&forge.host, &forge.token, forge.tls).await
    }
}

impl Gitlab {
    #[instrument(level = "debug", ret, err)]
    async fn graphql(
        &self,
        scope: String,
        after: String,
    ) -> Result<projects::ProjectsProjects, ForgeError> {
        let query = Projects::build_query(projects::Variables { scope, after });

        Ok(self
            .client
            .post(self.base_url.join("/api/graphql")?)
            .json(&query)
            .send()
            .await?
            .json::<ProjectsResponse>()
            .await?
            .data
            .projects
            .unwrap_or_default())
    }
}

#[async_trait::async_trait]
impl ForgeTrait for Gitlab {
    #[tracing::instrument(level = "trace")]
    async fn projects(&self, scope: &str) -> Result<Vec<super::Project>, ForgeError> {
        let projects = self.graphql(scope.to_string(), "".to_string()).await?;
        tracing::debug!("projects: {:#?}", projects);

        let mut nodes = projects.nodes.unwrap().clone();
        if nodes.is_empty() {
            return Ok(Vec::new());
        };

        let mut page = projects.page_info.end_cursor.unwrap();
        let mut has_next_page = projects.page_info.has_next_page;

        while has_next_page {
            let projects = self.graphql(scope.to_string(), page).await?;

            page = projects.page_info.end_cursor.unwrap();
            has_next_page = projects.page_info.has_next_page;

            nodes.append(&mut projects.nodes.unwrap());
        }

        let res = nodes
            .into_iter()
            .flatten()
            .filter(|x| {
                x.repository
                    .as_ref()
                    .and_then(|x| x.root_ref.as_ref())
                    .is_some()
            })
            .map(|x| x.into())
            .collect();

        Ok(res)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProjectsResponse {
    data: projects::ResponseData,
}

#[derive(GraphQLQuery)]
#[graphql(
    query_path = "graphql/gitlab_projects_query.graphql",
    schema_path = "graphql/gitlab_schema.graphql",
    response_derives = "Clone,Debug,Default",
    variables_derives = "Clone,Debug,Default"
)]
pub struct Projects;

impl From<projects::ProjectsProjectsNodes> for super::Project {
    fn from(project: projects::ProjectsProjectsNodes) -> Self {
        super::Project {
            id: project.id,
            name: project.name,
            path: project.full_path,
            ssh_clone_url: project.ssh_url_to_repo,
            http_clone_url: project.http_url_to_repo,
            default_branch: project.repository.map(|r| r.root_ref.unwrap()),
        }
    }
}
