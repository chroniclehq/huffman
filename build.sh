#!/usr/bin/env bash

# TODO: Get values from cli arguments
VERSION="latest"
REGISTRY="ghcr.io/chroniclehq"
PLATFORM="linux/amd64"

# Building the builder image
docker build -f Dockerfile.builder --platform=$PLATFORM -t $REGISTRY/huffman-builder:$VERSION .
docker tag $REGISTRY/huffman-build:latest

# Building the runtime image
docker build -f Dockerfile.runtime --platform=$PLATFORM -t $REGISTRY/huffman-runtime:$VERSION .
docker tag $REGISTRY/huffman-runtime:latest

# Building the app image
docker build -f Dockerfile.app --platform=$PLATFORM --build-arg VERSION=$VERSION --build-arg REGISTRY=$REGISTRY -t $REGISTRY/huffman-app:$VERSION .
docker tag $REGISTRY/huffman-app:$VERSION