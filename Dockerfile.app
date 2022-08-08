ARG VERSION=latest
ARG REGISTRY=ghcr.io/chroniclehq

FROM ${REGISTRY}/huffman-builder:${VERSION} as builder

# Copy source files into the image
WORKDIR /usr/src/huffman
COPY . .

# Build huffman into a binary
RUN RUSTFLAGS="-C target-feature=-crt-static $(pkg-config --libs vips)" cargo install --path .

# Use the runtime image as base
FROM ${REGISTRY}/huffman-runtime:${VERSION}

WORKDIR /app

# Copy built files from build image
COPY --from=builder /usr/local/cargo/bin/huffman /app/huffman
COPY --from=builder /usr/src/huffman/Rocket.toml /app/Rocket.toml

EXPOSE 8000

CMD ["doppler", "run", "--", "/app/huffman"]