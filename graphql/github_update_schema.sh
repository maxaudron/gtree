#!/bin/sh

set -e

graphql-client introspect-schema --header 'User-Agent: my man i am tired' https://api.github.com/graphql > ./graphql/github_schema.json
