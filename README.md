# Huffman

Image optimization using Rust and Libvips.

## Requirements
You must have the following packages installed before getting started
- Rust
- Vips
- pkg-config
- cargo watch (Optional)
- Docker (Optional. Required only for testing deploy phase)

## Configuration
You must have the following env variables set before getting started. For local development create a `.env` file with these values:

- AWS_ACCESS_KEY_ID: Access key id for AWS user
- AWS_SECRET_ACCESS_KEY: Access key secret for AWS user
- AWS_REGION: AWS region where buckets reside
- SOURCE_BUCKET: The source bucket to read images from
- CACHE_BUCKET: The bucket to store variants in
- SQS_URL: URL for the SQS queue
- SQS_POLL_INTERVAL= Polling interval for the SQS queue

Alternatively you can use doppler.io

## Running the server

```
$ RUSTFLAGS="$(pkg-config vips --libs)" cargo run
```

With cargo-watch for live reload during development

```
$ RUSTFLAGS="$(pkg-config vips --libs)" cargo watch -x run
```

## Docker Config

Docker is used for building huffman into an image with all it's required dependencies. We use [multistage builds](https://docs.docker.com/develop/develop-images/multistage-build/) for keeping the final container size low. Most of the Vips and Rust setup is borrowed from [olxgroup-oss/dali](https://github.com/olxgroup-oss/dali/blob/master/Dockerfile.vips).

- **Build layer** (Dockerfile.build): Uses the rust-alpine image and builds Vips from source followed by building huffman into a binary.
- **Runtime layer** (Dockerfile.runtime): Uses the alpine image and builds Vips from source. Install doppler-cli for pulling in the env variables. Copies the built huffman binary and the Rocket.toml config file from the build layer and runs the huffman binary.

To build the docker images run
```
$ sh docker-build.sh
```

> Make sure the version is updated before pushing images into the registry

To push the docker images into the registry (Github), run
```
$ sh docker-deploy.sh
```