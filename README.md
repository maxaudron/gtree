
# Table of Contents

1.  [Usage](#org86116bf)
2.  [Config](#org4e7be59)

A tool to clone and pull whole group trees from a git forge, properly organized on disk.


<a id="org86116bf"></a>

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


<a id="org4e7be59"></a>

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

