# Huffman

Image optimization using Rust and Libvips.

## Requirements
You must have the following packages installed before getting started
- Libvips
- Rust
- pkg-config
- cargo watch (Optional)

## Configuration
You must have the following env variables set before getting started. For local development create a `.env` file.

- AWS_ACCESS_KEY_ID: Access key id for AWS user
- AWS_SECRET_ACCESS_KEY: Access key secret for AWS user
- AWS_REGION: AWS region where buckets reside
- SOURCE_BUCKET: The source bucket to read images from
- CACHE_BUCKET: The bucket to store variants in

## Running the server

```
$ RUSTFLAGS="$(pkg-config vips --libs)" cargo run
```

With cargo-watch for live reload during development

```
$ RUSTFLAGS="$(pkg-config vips --libs)" cargo watch -x run
```