[![git mirror](https://img.shields.io/badge/git-repo-cba6f7?logo=git&link=https%3A%2F%2Fgit.vapor.systems%2Fgtree.git)](https://git.vapor.systems/gtree.git)
[![github mirror](https://img.shields.io/badge/github-repo-blue?logo=github&link=https%3A%2F%2Fgithub.com%2Fmaxaudron%2Fgtree)](https://github.com/maxaudron/gtree)
[![gitlab mirror](https://img.shields.io/badge/gitlab-repo-orange?logo=github&link=https%3A%2F%2Fgitlab.com%2Fcocainefarm%2Fgtree)](https://gitlab.com/cocainefarm/gtree)

# Table of Contents

A tool to clone and pull whole group trees from a git forge, properly organized on disk.

# Usage

    gtree
    Sync Gitlab Trees
    
    USAGE:
        gtree <SUBCOMMAND> [SCOPE]
    
    ARGS:
        <SCOPE>    Only operate on this subtree
    
    OPTIONS:
        -h, --help           Print help information
        -j, --jobs <JOBS>    Number of jobs to run in parallel, 0 is automatic [default: 0]
    
    SUBCOMMANDS:
        help      Print this message or the help of the given subcommand(s)
        list      List Directories
        sync      Download new repositories and delete old ones, also update
        update    Pull and Push new commits to and from the cloned repos

# Config

Default location for the config file is `$HOME/.config/gtree/config.toml`, in the toml format, yaml is also supported.

    # Give the forge a easily identifiable name
    ["gitlab.com"]
    # Configure which kind of forge this is
    # Currently only gitlab is supported
    type = "gitlab"
    
    # Set the domain name to reach the forge at
    host = "gitlab.com"
    
    # API Token for the forge
    # for gitlab this is a Personal Access Token
    # https://gitlab.com/-/profile/personal_access_tokens
    token = "HgDAfJ9tfD5xUw2L6SUm"
    
    # Directory to clone the repos into
    directory = "/home/audron/repo/gitlab.com"

