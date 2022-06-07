pub fn git_credentials_callback(
    _user: &str,
    user_from_url: Option<&str>,
    _cred: git2::CredentialType,
) -> Result<git2::Cred, git2::Error> {
    if let Some(user) = user_from_url {
        git2::Cred::ssh_key_from_agent(user)
    } else {
        Err(git2::Error::from_str("no url username found"))
    }
}

pub fn callbacks<'g>() -> git2::RemoteCallbacks<'g> {
    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(git_credentials_callback);

    callbacks
}

#[tracing::instrument(level = "trace")]
pub fn fetch_options<'g>() -> git2::FetchOptions<'g> {
    let mut opts = git2::FetchOptions::new();
    opts.remote_callbacks(callbacks());
    opts.download_tags(git2::AutotagOption::All);

    opts
}
