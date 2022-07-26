FROM rust:1.62.1-alpine3.15

ENV VIPS_VERSION=8.10.6

# Install dependencies for building vips & huffman
RUN apk add --update --no-cache --repository=http://dl-cdn.alpinelinux.org/alpine/edge/main \
    build-base clang pkgconfig libgsf glib expat tiff libjpeg-turbo libexif giflib librsvg lcms2 \
    libpng orc libwebp openssl

# Dev dependencies for building vips
RUN apk add --update --no-cache --repository=http://dl-cdn.alpinelinux.org/alpine/edge/main --virtual .build-deps \
    expat-dev giflib-dev lcms2-dev libexif-dev libheif-dev libimagequant-dev libjpeg-turbo-dev \
    libpng-dev librsvg-dev libwebp-dev openssl-dev orc-dev tiff-dev glib-dev

# Download and build VIPS
RUN wget https://github.com/libvips/libvips/releases/download/v${VIPS_VERSION}/vips-${VIPS_VERSION}.tar.gz

RUN mkdir /vips && \
    tar xvzf vips-${VIPS_VERSION}.tar.gz -C /vips --strip-components 1 && \
    cd /vips && \
    ./configure --enable-debug=no && \
    make && \
    make install && \
    ldconfig /etc/ld.so.conf.d && \
    rm -rf vips vips-${VIPS_VERSION}.tar.gz

# Install dependencies for vips to run
RUN apk add --update --no-cache libimagequant --repository=http://dl-cdn.alpinelinux.org/alpine/edge/main && \
    apk add --update --no-cache libimagequant --repository=http://dl-cdn.alpinelinux.org/alpine/edge/community libheif=1.12.0-r2 && \
    apk add --update --no-cache libimagequant --repository=http://dl-cdn.alpinelinux.org/alpine/edge/main libde265=1.0.8-r2 && \
    export GI_TYPELIB_PATH=/usr/local/lib/girepository-1.0