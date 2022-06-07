#!/bin/sh

set -e

get-graphql-schema https://gitlab.com/api/graphql > ./graphql/schema.graphql
