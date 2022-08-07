#!/usr/bin/env bash

VERSION="0.1"
REGISTRY="ghcr.io/chroniclehq"

docker build -f Dockerfile.runtime -t $REGISTRY/huffman-runtime:$VERSION .
docker push $REGISTRY/huffman-runtime:$VERSION

docker build -f Dockerfile.app --build-arg VERSION=$VERSION --build-arg REGISTRY=$REGISTRY -t $REGISTRY/huffman-app:$VERSION .
docker push $REGISTRY/huffman-app:$VERSION