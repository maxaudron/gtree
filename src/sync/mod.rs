use gtree::local::Repos;

impl crate::GTree {
    pub async fn sync(&self, repos: Repos) {
        for mut repo in repos {
            match repo.sync() {
                Ok(u) => println!("{}", u),
                Err(u) => println!("{}", u),
            };
        }
    }
}
