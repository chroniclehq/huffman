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

Alternatively you can use doppler.io for the secrets

## Running the server for development

```
$ RUSTFLAGS="$(pkg-config vips --libs)" cargo run
```

With cargo-watch for live reload during development

```
$ RUSTFLAGS="$(pkg-config vips --libs)" cargo watch -x run
```

## Building & Publishing via Docker

Docker is used for building huffman into an image with all it's required dependencies. We use [multistage builds](https://docs.docker.com/develop/develop-images/multistage-build/) for keeping the final container size low. Most of the Vips and Rust setup is borrowed from [olxgroup-oss/dali](https://github.com/olxgroup-oss/dali/blob/master/Dockerfile.vips).

- **Builder layer** (Dockerfile.build): Uses the rust-alpine image and builds Vips from source.
- **Runtime layer** (Dockerfile.runtime): Uses the alpine image and builds Vips from source. Install doppler-cli for pulling in the env variables. 
- **App Layer** (Dockerfile.app): Import builder and compile huffman into a binary read for execution. Import runtime and copies the built huffman binary into it. Copy the Rocket.toml config file from the build layer and start the huffman binary.

> Since we copy this project into the docker image, any changes to the files that aren't included in .dockerignore will invalidate the cache for that layer. Make sure frequently changing files that aren't related to code are always added to the `.dockerignore` file to avoid this and keep builds fast.

During a push to master, the builder and runtime images are created if there is a change in the respective dockerfiles. This is because we don't include the project source in these images and so they won't change often. The app image is created for every commit and tagged with the commit version.

To build the docker images locally you can run
```
$ sh build.sh
```
> Building docker images with a different target is incredibly slow on the M1 (>1 hour). If you are building the images to test locally, change the platform to arm64 in the build.sh file.