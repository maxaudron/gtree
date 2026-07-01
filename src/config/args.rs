use clap::{Parser, Subcommand};

#[derive(Parser, Clone, Debug)]
// #[clap(override_usage("gtree <SUBCOMMAND> [SCOPE]"))]
#[command(name = "gtree")]
#[command(bin_name = "gtree")]
/// Sync Gitlab Trees
pub struct Args {
    #[clap(subcommand)]
    pub command: Commands,

    /// Number of jobs to run in parallel, 0 is automatic
    #[clap(short = 'j', long = "jobs", default_value = "0", global = true)]
    pub jobs: usize,
}

#[derive(Parser, Clone, Debug, PartialEq)]
pub struct ScopeArg {
    /// Only operate on this subtree
    #[clap(global = true)]
    pub scope: Option<String>,
}

#[derive(PartialEq, Clone, Debug, Subcommand)]
pub enum Commands {
    /// Download new repositories and delete old ones, also update
    Sync(ScopeArg),
    /// Pull and Push new commits to and from the cloned repos
    Update(ScopeArg),
    /// List Directories
    List(ScopeArg),
    /// Clone a repository
    Clone(ScopeArg),
}
