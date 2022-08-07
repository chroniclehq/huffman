ARG VERSION=0.1 
ARG REGISTRY=ghcr.io/chroniclehq

FROM ${REGISTRY}/huffman-runtime:${VERSION}

# Copy source files into image
WORKDIR /usr/src/huffman
COPY . .

# Build app and run it
RUN RUSTFLAGS="-C target-feature=-crt-static $(pkg-config --libs vips)" cargo install --path .

CMD ["doppler", "run", "--", "huffman"]