use anyhow::Result;

use graphql_client::GraphQLQuery;

pub mod config;

#[derive(Clone, Debug)]
pub struct Gitlab {
    client: gitlab::AsyncGitlab,
}

impl Gitlab {
    #[tracing::instrument(level = "trace")]
    pub async fn new(host: &str, token: &str, tls: bool) -> Result<Gitlab> {
        let mut gitlab = gitlab::GitlabBuilder::new(host, token);

        if !tls {
            gitlab.insecure();
        }

        let gitlab = gitlab.build_async().await?;

        Ok(Gitlab { client: gitlab })
    }

    #[tracing::instrument(level = "trace")]
    pub async fn from_config(forge: &config::Gitlab) -> Result<Gitlab> {
        Gitlab::new(&forge.host, &forge.token, forge.tls).await
    }
}

#[async_trait::async_trait]
impl super::ForgeTrait for Gitlab {
    #[tracing::instrument(level = "trace")]
    async fn projects(&self, scope: &str) -> Result<Vec<super::Project>> {
        let query = Projects::build_query(projects::Variables {
            scope: scope.to_owned(),
            after: "".to_owned(),
        });

        let res = self.client.graphql::<Projects>(&query).await?;

        let projects = res.projects.unwrap();

        let mut page = projects.page_info.end_cursor.unwrap();
        let mut has_next_page = projects.page_info.has_next_page;

        let mut nodes = projects.nodes.unwrap().clone();

        while has_next_page {
            let query = Projects::build_query(projects::Variables {
                scope: scope.to_owned(),
                after: page,
            });

            let res = self.client.graphql::<Projects>(&query).await?;

            let projects = res.projects.unwrap();

            page = projects.page_info.end_cursor.unwrap();
            has_next_page = projects.page_info.has_next_page;

            nodes.append(&mut projects.nodes.unwrap());
        }

        let res = nodes
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
    response_derives = "Clone,Debug",
    variables_derives = "Clone,Debug"
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
        }
    }
}
