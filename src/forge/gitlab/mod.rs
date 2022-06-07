use anyhow::{bail, Context, Result};
use gitlab::AsyncGitlab;

use graphql_client::GraphQLQuery;
use tracing::{debug, trace};

pub mod config;

#[derive(Clone, Debug)]
pub struct Gitlab {
    client: AsyncGitlab,
}

impl Gitlab {
    pub async fn new(host: &str, token: &str, tls: bool) -> Result<Gitlab> {
        let mut gitlab = gitlab::GitlabBuilder::new(host, token);

        if !tls {
            gitlab.insecure();
        }

        let gitlab = gitlab.build_async().await?;

        Ok(Gitlab { client: gitlab })
    }

    pub async fn from_config(forge: &config::Gitlab) -> Result<Gitlab> {
        Gitlab::new(&forge.host, &forge.token, forge.tls).await
    }
}

#[async_trait::async_trait]
impl super::ForgeTrait for Gitlab {
    async fn projects(&self, scope: &str) -> Result<Vec<super::Project>> {
        let query = Projects::build_query(projects::Variables {
            scope: scope.to_owned(),
        });
        debug!("query: {:#?}", query);
        let res = self.client.graphql::<Projects>(&query).await?;

        let res = res
            .projects
            .unwrap()
            .nodes
            .unwrap()
            .into_iter()
            .filter(|x| x.is_some())
            .map(|x| x.unwrap().into())
            .collect();

        Ok(res)
    }
}

#[derive(GraphQLQuery)]
#[graphql(
    query_path = "graphql/projects_query.graphql",
    schema_path = "graphql/schema.graphql",
    response_derives = "Clone, Debug"
)]
pub struct Projects;

impl Into<super::Project> for projects::ProjectsProjectsNodes {
    fn into(self) -> super::Project {
        super::Project {
            id: self.id,
            name: self.name,
            path: self.full_path,
            ssh_clone_url: self.ssh_url_to_repo,
            http_clone_url: self.http_url_to_repo,
        }
    }
}
