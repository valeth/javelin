FROM rust:1.30-slim AS build

RUN mkdir -p /build/out
WORKDIR /build
COPY ./ ./
RUN cargo build --release \
 && cp target/release/javelin ./out


FROM rust:1.30-slim

RUN mkdir -p /app
WORKDIR /app
COPY --from=build /build/out/javelin ./javelin

EXPOSE 1935
CMD ["/app/javelin"]