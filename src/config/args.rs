use clap::{Parser, Subcommand};

#[derive(Parser, Clone, Debug)]
#[clap(override_usage("gtree <SUBCOMMAND> [SCOPE]"))]
/// Sync Gitlab Trees
pub struct Args {
    #[clap(subcommand)]
    pub command: Commands,

    /// Only operate on this subtree
    #[clap(global = true)]
    pub scope: Option<String>,
}

#[derive(PartialEq, Clone, Debug, Subcommand)]
pub enum Commands {
    /// Download new repositories and delete old ones, also update
    Sync,
    /// Pull and Push new commits to and from the cloned repos
    Update,
    /// List Directories
    List,
}
