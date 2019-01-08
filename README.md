# Javelin RTMP Server

Streaming server written in Rust.

Supported sources:
- RTMP

Supported outputs:
- RTMP
- HLS (H.264 + AAC)


## How to install and run

### Via Cargo

```sh
cargo install javelin
# Make sure your $CARGO_HOME/bin is in your $PATH
javelin --no-tls --permit-stream-key="mysecretstreamkey"
```

### Via Docker

```sh
docker pull registry.gitlab.com/valeth/javelin:latest
docker run --tty -p 1935:1935 \
    registry.gitlab.com/valeth/javelin:latest \
    --hls-root=/tmp/streamout \
    --permit-stream-key=123456
```

> Try `javelin --help` for more command line options.
