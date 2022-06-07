use structopt::StructOpt;

#[derive(StructOpt, Clone, Debug)]
/// Sync Gitlab Trees
pub struct Args {
    #[structopt(subcommand)]
    pub command: Commands,

    /// Only operate on this subtree
    pub scope: Option<String>,
}

#[derive(PartialEq, Clone, Debug)]
#[derive(StructOpt)]
#[structopt(about = "the stupid content tracker")]
pub enum Commands {
    /// Download new repositories and delete old ones, also update
    Sync,
    /// Pull and Push new commits to and from the cloned repos
    Update,
    /// List Directories
    List,
}
