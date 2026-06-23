#!/bin/sh

set -e

get-graphql-schema https://gitlab.com/api/graphql > ./graphql/gitlab_schema.graphql
