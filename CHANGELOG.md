# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).


## [Unreleased]

*No changes yet.*

---

## [0.3.6]

### Added
- Permitted stream keys can now be set via configuration file.

---

## [0.3.5]

### Fixed
- Composition time is now included in PES presentation timestamp.
- PES now also has a decoding timestamp set.

---

## [0.3.4]

### Added
- Configuration option for integrated web server address.
- Compile time and command line option to disable HLS support.
- Multiple methods to handle re-publishing to same application.
- Stream statistics API endpoint.

### Changed
- Moved codec related code into sub-crate.

---

## [0.3.3]

### Fixed
- HLS directory will now be completely cleared only on startup.
- The "web" feature now includes the "hls" feature set.

---

## [0.3.2]

### Added
- Internal HTTP file server with simple JSON API (currently just active streams).

### Changed
- File cleanup is no longer done in batches.
- TLS support is disabled by default.
- HLS directory is now cleared every time on stream publish start.

---

## [0.3.1]

### Changed
- Just some minor tweaks.

---

## [0.3.0]

### Added
- Optional support for HLS streaming output.

---

## [0.2.3]

### Added
- Allow limitation of permitted stream keys with `--permit-stream-key=<key>` flag.

### Fixed
- Help text should now be always up-to-date with `Cargo.toml`.

---

## [0.2.2]

### Fixed
- Just minor manifest formatting fixes, nothing important.

---

## [0.2.1]

### Fixed
- No longer requires a password if running with `--no-tls` flag.

---

## [0.2.0]

### Added
- Optional TLS support is now available.

---

## [0.1.1]

### Fixed
- Publishing clients should no longer linger forever.

---

## [0.1.0]

### Added
- Required event handlers to make the protocol work.
- Dockerfile and image for easier setup.


<!-- links -->

[Unreleased]: https://gitlab.com/valeth/javelin/tree/develop
[0.3.6]: https://gitlab.com/valeth/javelin/tree/0.3.6
[0.3.5]: https://gitlab.com/valeth/javelin/tree/0.3.5
[0.3.4]: https://gitlab.com/valeth/javelin/tree/0.3.4
[0.3.3]: https://gitlab.com/valeth/javelin/tree/0.3.3
[0.3.2]: https://gitlab.com/valeth/javelin/tree/0.3.2
[0.3.1]: https://gitlab.com/valeth/javelin/tree/0.3.1
[0.3.0]: https://gitlab.com/valeth/javelin/tree/0.3.0
[0.2.3]: https://gitlab.com/valeth/javelin/tree/0.2.3
[0.2.2]: https://gitlab.com/valeth/javelin/tree/0.2.2
[0.2.1]: https://gitlab.com/valeth/javelin/tree/0.2.1
[0.2.0]: https://gitlab.com/valeth/javelin/tree/0.2.0
[0.1.1]: https://gitlab.com/valeth/javelin/tree/0.1.1
[0.1.0]: https://gitlab.com/valeth/javelin/tree/0.1.0
