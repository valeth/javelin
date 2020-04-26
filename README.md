# Javelin

A simple video live streaming server.

> This software is still under development.
>
> For all versions under 1.0.0, breaking changes should only occur
> on minor release increments, patch releases are backwards compatible.

Supported sources:
- RTMP (H.264 + AAC)

Supported outputs:
- RTMP
- HLS (H.264 + AAC)


## How to install and run

```sh
cargo install javelin
# Make sure your $CARGO_HOME/bin is in your $PATH
javelin --rtmp-permit-stream="username:mysecretstreamkey"
```

Check out the [Wiki][wiki_installation] for more info about other possible installation methods.


## Contributions

See [CONTRIBUTING.md]


<!-- links -->

[wiki_installation]: https://gitlab.com/valeth/javelin/wikis/installation
[CONTRIBUTING.md]: CONTRIBUTING.md
