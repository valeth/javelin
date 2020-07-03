# Javelin

A simple video live streaming server.

Supported sources:
- RTMP (H.264 + AAC)

Supported outputs:
- RTMP
- HLS (H.264 + AAC)


## Install

```sh
cargo install javelin
# Make sure your $CARGO_HOME/bin is in your $PATH

# add an user with an associated stream key to the database
javelin permit-stream username mysecretstreamkey

# start the server
javelin run
```

Check out the [Wiki][wiki_installation] for more info about other possible installation methods.


## Feature flags

The following feature flags can be supplied to `cargo`:

| Flag          | Default | Description
| ------------- |:-------:| ----------------------------------------
| `rtmp`        | yes     | Enables the RTMP source
| `rtmps`       | no      | Enables additional TLS support for RTMP
| `hls`         | yes     | Enables the HLS server
| `db-sqlite`   | yes     | Use SQLite as database backend
| `db-mongo`    | no      | Use MongoDB as database backend

> Note: One database backend is required.


## Contributions

See [CONTRIBUTING.md]


## Versioning

The project follows [semantic versioning][semver].

We are not on a stable version yet, which means the following additional rules apply:

- Breaking changes cause a minor version bump (`0.1.0` -> `0.2.0`)
- Non-breaking changes cause a patch version bump (`0.1.1` -> `0.1.2`)


<!-- links -->

[wiki_installation]: https://gitlab.com/valeth/javelin/wikis/installation
[CONTRIBUTING.md]: CONTRIBUTING.md
[semver]: https://semver.org
