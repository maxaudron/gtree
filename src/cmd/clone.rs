use crate::{GTreeError, config::url::GitUrl, repo::Repo};

impl crate::GTree {
    pub fn git_clone(&self, url: GitUrl) -> Result<(), GTreeError> {
        let forge = self.config.forge.get(&url.domain).unwrap();

        let mut repo = Repo {
            name: url.full_path()?,
            path: forge.root().join(url.full_path()?),
            known_hosts: forge
                .known_hosts()
                .iter()
                .map(|k| k.fingerprint(ssh_key::HashAlg::Sha256).sha256().unwrap())
                .collect(),
            ..Default::default()
        };

        Ok(repo.clone(&url.ssh_url()?)?)
    }
}
