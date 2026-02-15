# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.7] - 2026-02-15

### Added
- Background update checks without blocking the main application flow.
- Persistent state management in a system-dependent cache folder to track failed update versions and prevent retrying them.

## [0.2.6] - 2026-02-15

### Added
- Multi-platform support in release workflow: Linux, macOS (AMD64/ARM64), and Windows.
- Simplified asset naming convention (`linux-amd64`, `macos-arm64`, etc.) for easier discovery.
- Runtime platform detection in `main.rs` to fetch matching architecture assets.
- Rust caching in CI/CD pipeline for faster builds.

### Changed
- Updated technical specification to reflect new simplified asset naming rules.
- Updated GitHub Actions to `checkout@v4` and `action-gh-release@v2`.

## [0.2.0] - 2026-02-15

### Added
- Embedded signature verification using `self_update` v0.42.0.
- `zipsign-api` integration for cryptographic types.
- Documentation index in `doc/README.md`.

### Changed
- Updated `zipsign` to v1.85.0 (Kijewski version).
- Refactored GitHub Actions release workflow to use embedded signatures (removed separate `.sig` files).
- Simplified `zipsign sign` CLI usage in CI/CD pipeline.
- Updated technical specifications and guides to reflect the move to embedded signatures.

### Fixed
- GitHub build action failure due to outdated `zipsign` CLI arguments.
- Unused import warnings in `src/main.rs`.

## [0.1.0] - Prior versions

- Initial setup with basic self-update mechanism.
