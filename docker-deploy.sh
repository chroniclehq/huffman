#!/usr/bin/env bash

# TODO: Get these from commit hash
VERSION="0.1"
REGISTRY="ghcr.io/chroniclehq"

docker push $REGISTRY/huffman-build:$VERSION && \
docker push $REGISTRY/huffman-runtime:$VERSION