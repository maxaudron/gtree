use std::sync::{Arc, atomic::AtomicBool};

use super::{Repo, RepoError};

use git2::{FetchOptions, RemoteCallbacks, build::RepoBuilder};
use tracing::debug;

impl Repo {
    #[tracing::instrument(level = "trace")]
    pub fn clone(&mut self, url: &str) -> Result<(), RepoError> {
        debug!("cloning repo {url} to {:?}", self.path);
        std::fs::create_dir_all(&self.path).unwrap();
        self.repo = Some({
            let mut builder = RepoBuilder::new();
            let (fo, _updated) = self.fetch_options();
            builder.fetch_options(fo);
            builder.clone(url, &self.path)?
        });

        Ok(())
    }

    #[tracing::instrument(level = "debug", ret, err)]
    pub fn fetch<'a>(&mut self) -> Result<bool, RepoError> {
        let mut remote = self.default_remote()?;
        let (mut fo, updated) = self.fetch_options();
        remote.fetch(&[&self.default_branch], Some(&mut fo), Some("gtree fetch"))?;

        Ok(updated.load(std::sync::atomic::Ordering::Relaxed))
    }

    pub fn fetch_options(&self) -> (FetchOptions<'_>, Arc<AtomicBool>) {
        let mut fo = FetchOptions::new();
        let (cb, updated) = self.remote_callbacks();
        fo.remote_callbacks(cb);

        (fo, updated)
    }

    pub fn remote_callbacks(&self) -> (RemoteCallbacks<'_>, Arc<AtomicBool>) {
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, _allowed_types| {
            git2::Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
        });
        callbacks.certificate_check(|cert, _| {
            if self.known_hosts.is_empty() {
                return Ok(git2::CertificateCheckStatus::CertificatePassthrough);
            }

            let hostkey = cert.as_hostkey().unwrap().hash_sha256().unwrap();
            if self.known_hosts.contains(hostkey) {
                Ok(git2::CertificateCheckStatus::CertificateOk)
            } else {
                Ok(git2::CertificateCheckStatus::CertificatePassthrough)
            }
        });

        let updated = Arc::new(AtomicBool::new(false));
        let updated_inner = updated.clone();

        // This callback gets called for each remote-tracking branch that gets
        // updated. The message we output depends on whether it's a new one or an
        // update.
        callbacks.update_tips(move |refname, a, b| {
            if a.is_zero() {
                debug!("[new]     {:20} {}", b, refname);
            } else {
                debug!("[updated] {:10}..{:10} {}", a, b, refname);
            }

            updated_inner.store(true, std::sync::atomic::Ordering::Relaxed);
            true
        });

        (callbacks, updated)
    }
}
