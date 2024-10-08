# |-------<[ Build ]>-------|

FROM rust:1.31-slim AS build

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl-dev \
    pkg-config

RUN mkdir -p /build/out
WORKDIR /build

COPY ./ ./
RUN cargo build --release \
 && cp target/release/javelin ./out


# |-------<[ App ]>-------|

FROM rust:1.31-slim

LABEL maintainer="dev.patrick.auernig@gmail.com"

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
    ca-certificates

RUN mkdir -p /var/data /app/config
WORKDIR /app

COPY --from=build /build/out/javelin ./javelin

EXPOSE 1935 8080
ENTRYPOINT ["/app/javelin", "--hls-root=/var/data"]
