#!/usr/bin/env bash

# TODO: Get these from commit hash
VERSION="0.1"
REGISTRY="ghcr.io/chroniclehq"

docker build -f Dockerfile.build -t $REGISTRY/huffman-build:$VERSION . && \
docker build -f Dockerfile.runtime --build-arg VERSION=$VERSION --build-arg REGISTRY=$REGISTRY -t $REGISTRY/huffman-runtime:$VERSION .