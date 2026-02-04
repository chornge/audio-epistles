# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.4.0] - 2026-02-04

### Added

* Cargo dependency monitoring via Dependabot (weekly updates with grouped PRs)
* Code formatting check (`cargo fmt --check`) in CI pipeline
* Clippy linting check in CI pipeline
* `RUST_LOG` environment variable support in Docker (default: info)
* Docker resource limits (CPU: 2 cores, Memory: 4GB)

### Changed

* Updated `bytes` crate to 1.11.1 to fix integer overflow vulnerability (RUSTSEC-2026-0007)
* Improved `.dockerignore` to explicitly exclude secrets and sensitive files
* Makefile `clippy` and `ci` targets now mirror CI workflow
* Release notes now pull from CHANGELOG.md

### Security

* Fixed RUSTSEC-2026-0007: Integer overflow in `BytesMut::reserve` (bytes 1.10.1 → 1.11.1)
* Added `.env` to `.dockerignore` to prevent credential leakage in Docker builds
* Added `no-new-privileges` security option to Docker container

## [2.3.0] - 2025-12-16

### Added

* Comprehensive test suite for `episode.rs` (timestamp parsing, chapter extraction)
* Structured logging using `tracing` crate with configurable log levels via `RUST_LOG`
* Type-safe newtypes: `VideoId`, `AudioPath`, `VideoPath`, `Seconds`
* Database audit trail with upload history tracking
* Migration support for database schema changes
* Doc comments for all public functions and modules
* `CONTRIBUTING.md` with contribution guidelines

### Changed

* Replaced `println!`/`eprintln!` with structured `tracing` macros throughout
* Improved error handling: replaced `.expect()` and `.unwrap()` with proper `?` propagation
* Database schema now tracks full upload history instead of single ID
* WebDriver process management now uses RAII guard pattern for cleanup
* Zombie chromedriver processes no longer occur on crash/error
* Proper cleanup of WebDriver even when upload fails partway through

## [2.2.6] - 2025-10-10

### Changed

* Update CI workflow (release pipeline improvements)
* Update CI workflow for release automation
* Improved release pipeline stability
* Minor maintenance updates
* CI/CD workflow refinements
* Tweak contents in zipped directory for releases
* Improved release packaging
* Initial automated YouTube to Spotify podcast publishing
* Video fetching from YouTube playlists
* Audio extraction and trimming using ffmpeg
* Spotify/Anchor.fm upload via WebDriver automation
* SQLite database for tracking published videos
* Docker support
* GitHub Actions CI/CD
