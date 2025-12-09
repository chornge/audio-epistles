# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.3.0] - 2025-12-09

### Added

- Comprehensive test suite for `episode.rs` (timestamp parsing, chapter extraction)
- Structured logging using `tracing` crate with configurable log levels via `RUST_LOG`
- Type-safe newtypes: `VideoId`, `AudioPath`, `VideoPath`, `Seconds`
- Database audit trail with upload history tracking
- Migration support for database schema changes
- Doc comments for all public functions and modules
- `SECURITY.md` with security guidelines and audit findings
- `CONTRIBUTING.md` with contribution guidelines

### Changed

- Replaced `println!`/`eprintln!` with structured `tracing` macros throughout
- Improved error handling: replaced `.expect()` and `.unwrap()` with proper `?` propagation
- Database schema now tracks full upload history instead of single ID
- WebDriver process management now uses RAII guard pattern for cleanup

### Fixed

- Zombie chromedriver processes no longer occur on crash/error
- Proper cleanup of WebDriver even when upload fails partway through

### Security

- Conducted comprehensive security audit
- Added security best practices documentation

## [2.2.6] - 2025-12-09

### Changed

- Bump version to 2.2.6
- Update CI workflow (release pipeline improvements)

## [2.2.5] - 2025-10-10

### Changed

- Update CI workflow for release automation
- Improved release pipeline stability

## [2.2.4] - 2025-10-10

### Changed

- Bump version to 2.2.4
- Minor maintenance updates

## [2.2.3] - 2025-10-10

### Changed

- Bump version to 2.2.3
- CI/CD workflow refinements

## [2.2.2] - 2025-10-10

### Changed

- Tweak contents in zipped directory for releases
- Improved release packaging

## [2.2.1] - Previous releases

### Added

- Initial automated YouTube to Spotify podcast publishing
- Video fetching from YouTube playlists
- Audio extraction and trimming using ffmpeg
- Spotify/Anchor.fm upload via WebDriver automation
- SQLite database for tracking published videos
- Docker support
- GitHub Actions CI/CD

[Unreleased]: https://github.com/chornge/audio-epistles/compare/v2.3.0...HEAD
[2.3.0]: https://github.com/chornge/audio-epistles/compare/v2.2.6...v2.3.0
[2.2.6]: https://github.com/chornge/audio-epistles/compare/v2.2.5...v2.2.6
[2.2.5]: https://github.com/chornge/audio-epistles/compare/v2.2.4...v2.2.5
[2.2.4]: https://github.com/chornge/audio-epistles/compare/v2.2.3...v2.2.4
[2.2.3]: https://github.com/chornge/audio-epistles/compare/v2.2.2...v2.2.3
[2.2.2]: https://github.com/chornge/audio-epistles/compare/v2.2.1...v2.2.2
[2.2.1]: https://github.com/chornge/audio-epistles/releases/tag/v2.2.1
