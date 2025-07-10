#!/bin/bash

set -e

DIR=$(mktemp -d)

# Clone the risc0 repository
git clone https://github.com/risc0/risc0.git "$DIR"
cd "$DIR"

# Build the images
echo "Building bento images..."
docker compose -f ./compose.yml --env-file ./bento/dockerfiles/sample.env build

# Tag the services
docker tag "agent" "ere-risczero/agent:latest"
docker tag "bento-rest_api" "ere-risczero/rest_api:latest"

# Clean up
rm -rf "$DIR"
