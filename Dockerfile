# |-------<[ Build ]>-------|

FROM rust:1.30-slim AS build

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

FROM rust:1.30-slim

LABEL maintainer="dev.patrick.auernig@gmail.com"

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
    ca-certificates

RUN mkdir -p /app
WORKDIR /app

COPY --from=build /build/out/javelin ./javelin

EXPOSE 1935
CMD ["/app/javelin", "--no-tls"]
