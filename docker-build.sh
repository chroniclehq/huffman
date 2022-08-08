#!/usr/bin/env bash

# TODO: Get these from commit hash
VERSION="0.2"
REGISTRY="ghcr.io/chroniclehq"

docker build -f Dockerfile.build --platform=linux/amd64 -t $REGISTRY/huffman-build:$VERSION . && \
docker build -f Dockerfile.runtime --platform=linux/amd64 --build-arg VERSION=$VERSION --build-arg REGISTRY=$REGISTRY -t $REGISTRY/huffman-runtime:$VERSION .

docker tag $REGISTRY/huffman-build:$VERSION $REGISTRY/huffman-build:latest
docker tag $REGISTRY/huffman-runtime:$VERSION $REGISTRY/huffman-runtime:latest