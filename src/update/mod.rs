use gtree::local::Repos;

impl crate::GTree {
    pub async fn update(&self, repos: Repos) {
        for mut repo in repos {
            if let Some(_) = repo.repo {
                match repo.update() {
                    Ok(u) => println!("{}", u),
                    Err(u) => println!("{}", u),
                };
            }
        }
    }
}
