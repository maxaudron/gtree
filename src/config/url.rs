#![allow(unused)]
//! Implementation to parse a few different types of urls/references to
//! git repositories
//!
//! - github:maxaudron/gtree
//! - git@github.com:maxaudron/gtree.git
//! - ssh://git@github.com/maxaudron/gtree.git
//! - https://github.com/maxaudron/gtree.git

use nom::{
    IResult, Parser,
    bytes::complete::{tag, take_until1},
    combinator::{opt, rest},
    sequence::terminated,
};
use tracing::{debug, instrument};

use crate::{config::Config, forge::ForgeType};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GitUrl {
    pub forge: ForgeType,
    pub domain: String,
    pub owner: Option<String>,
    pub path: Option<String>,
    pub user: Option<String>,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum GitUrlError {
    #[error("not forge could be found for the input: {0}")]
    NoForgeFound(String),
    #[error("could not parse git url: {0}")]
    ParseError(nom::Err<nom::error::Error<String>>),
    #[error("no path found in git url")]
    NoPath,
}

impl From<nom::Err<nom::error::Error<&str>>> for GitUrlError {
    fn from(err: nom::Err<nom::error::Error<&str>>) -> Self {
        Self::ParseError(err.map_input(|input| input.into()))
    }
}

fn parse_scheme(input: &str) -> IResult<&str, Option<&str>> {
    opt(terminated(take_until1("://"), tag("://"))).parse(input)
}

fn parse_user(input: &str) -> IResult<&str, Option<&str>> {
    opt(terminated(take_until1("@"), tag("@"))).parse(input)
}

fn parse_forge(input: &str, http: bool) -> IResult<&str, Option<&str>> {
    let sep = if http { "/" } else { ":" };
    opt(terminated(take_until1(sep), tag(sep))).parse(input)
}

fn parse_owner(input: &str) -> IResult<&str, Option<&str>> {
    opt(terminated(take_until1("/"), tag("/")).or(rest)).parse(input)
}

impl Config {
    #[instrument(level = "debug", ret, err)]
    pub fn make_git_url(&self, mut url: &str) -> Result<GitUrl, GitUrlError> {
        let (rest, schema) = parse_scheme(url)?;
        let (rest, user) = parse_user(rest)?;
        let http = schema.map(|s| s.contains("http")).unwrap_or(false);
        let (rest, forge) =
            parse_forge(rest, http).map_err(|_| GitUrlError::NoForgeFound(rest.to_string()))?;

        debug!(
            "parse_forge: {:?}, default_forge: {:?}",
            forge, self.settings.default_forge
        );

        let forge = forge
            .or(self.settings.default_forge.as_deref())
            .ok_or(GitUrlError::NoForgeFound(rest.to_string()))?;
        let (rest, mut owner) = parse_owner(rest)?;
        owner = owner.and_then(|s| if s.is_empty() { None } else { Some(s) });
        let path = if !rest.is_empty() {
            Some(rest.trim_end_matches(".git"))
        } else {
            None
        };

        let forge = self
            .resolve_forge(forge)
            .ok_or(GitUrlError::NoForgeFound(forge.to_string()))?;

        let git_url = GitUrl::from_parts(
            ForgeType::Github,
            forge.to_string(),
            owner.map(|x| x.to_string()),
            path.map(|x| x.to_string()),
            user.map(|x| x.to_string()),
        );

        Ok(git_url)
    }
}

impl GitUrl {
    pub fn from_parts(
        forge: ForgeType,
        domain: String,
        owner: Option<String>,
        path: Option<String>,
        user: Option<String>,
    ) -> Self {
        Self {
            forge,
            domain,
            owner,
            path,
            user,
        }
    }

    pub fn full_path(&self) -> Result<String, GitUrlError> {
        if self.owner.is_none() && self.path.is_none() {
            return Err(GitUrlError::NoPath);
        }

        let mut res = String::new();
        if let Some(owner) = self.owner.as_ref() {
            res.push_str(owner);
        }

        if let Some(path) = self.path.as_ref() {
            res.push('/');
            res.push_str(path);
        }

        Ok(res)
    }

    pub fn ssh_url(&self) -> Result<String, GitUrlError> {
        Ok(format!(
            "{}@{}:{}",
            self.user.as_deref().unwrap_or("git"),
            self.domain,
            self.full_path()?
        ))
    }

    pub fn https_url(&self) -> Result<String, GitUrlError> {
        Ok(format!(
            "https://{}{}/{}",
            self.user.as_deref().unwrap_or(""),
            self.domain,
            self.full_path()?
        ))
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use crate::{
        config::{Config, Settings, url::GitUrl},
        forge::ForgeType,
    };

    fn config() -> Config {
        use std::collections::BTreeMap;

        use crate::{config::ForgeConfig, forge::github};

        let mut forge = BTreeMap::new();
        forge.insert(
            "github.com".to_string(),
            ForgeConfig::Github(github::Config {
                host: "github.com".to_string(),
                token: "testtoken".to_string(),
                directory: "repo/github.com".into(),
                ..Default::default()
            }),
        );

        let mut alias = BTreeMap::new();
        alias.insert("github".to_string(), "github.com".to_string());

        let settings = Settings::default();

        Config {
            forge,
            alias,
            settings,
        }
    }

    fn git_url() -> GitUrl {
        GitUrl::from_parts(
            ForgeType::Github,
            "github.com".to_string(),
            Some("maxaudron".to_string()),
            Some("gtree".to_string()),
            None,
        )
    }

    fn git_url_user() -> GitUrl {
        GitUrl::from_parts(
            ForgeType::Github,
            "github.com".to_string(),
            Some("maxaudron".to_string()),
            Some("gtree".to_string()),
            Some("git".to_string()),
        )
    }

    #[test]
    pub fn parse_forge() {
        assert_eq!(
            config().make_git_url("github:").unwrap(),
            GitUrl::from_parts(
                ForgeType::Github,
                "github.com".to_string(),
                None,
                None,
                None
            )
        )
    }

    #[test]
    pub fn parse_short() {
        assert_eq!(
            config().make_git_url("github:maxaudron/gtree").unwrap(),
            git_url()
        )
    }

    #[test]
    pub fn parse_git() {
        assert_eq!(
            config()
                .make_git_url("git@github.com:maxaudron/gtree.git")
                .unwrap(),
            git_url_user()
        )
    }

    #[test]
    pub fn parse_ssh() {
        assert_eq!(
            config()
                .make_git_url("ssh://git@github.com:maxaudron/gtree.git")
                .unwrap(),
            git_url_user()
        )
    }

    #[test]
    pub fn parse_http() {
        assert_eq!(
            config()
                .make_git_url("https://git@github.com/maxaudron/gtree.git")
                .unwrap(),
            git_url_user()
        );
    }

    #[test]
    #[should_panic(expected = "NoForgeFound")]
    pub fn parse_no_forge_panic() {
        assert_eq!(config().make_git_url("maxaudron/gtree").unwrap(), git_url())
    }

    #[test]
    #[traced_test]
    pub fn parse_only_path() {
        let mut config = config();
        config.settings.default_forge = Some("github.com".to_string());
        assert_eq!(config.make_git_url("maxaudron/gtree").unwrap(), git_url())
    }

    #[test]
    #[traced_test]
    pub fn parse_only_forge() {
        assert_eq!(
            config().make_git_url("github:").unwrap(),
            GitUrl::from_parts(
                ForgeType::Github,
                "github.com".to_string(),
                None,
                None,
                None
            )
        );
    }
}
