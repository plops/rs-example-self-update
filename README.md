# rs-example-self-update

Official Repository: [https://github.com/plops/rs-example-self-update](https://github.com/plops/rs-example-self-update)

A template project for building self-updating Rust CLI applications using GitHub Releases.

## 🚀 Overview

This project demonstrates a robust self-update mechanism that includes:
- **Non-blocking checks**: Periodic update checks that don't block the startup flow.
- **Embedded Signatures**: Cryptographic verification using `zipsign` (v1.85.0+) embedded directly into archives.
- **Atomic Operations**: Backup and rollback capabilities to ensure the application remains functional even if an update fails validation.
- **Cross-Platform**: Support for Linux, macOS, and Windows.

## 📖 Documentation

Detailed documentation can be found in the `doc/` directory:
- **[Documentation Overview](file:///home/kiel/stage/rs-example-self-update/doc/README.md)**
- **[Technical Specification](file:///home/kiel/stage/rs-example-self-update/spec/01_requirements.md)**

## 🚦 Getting Started

1. **Configure GitHub**: Ensure your repository is public and you have `ZIPSIGN_PRIV_KEY` and `ZIPSIGN_PASSWORD` in your GitHub Secrets.
2. **Key Generation**: Use `zipsign gen-key` to create your signing keys.
3. **Embed Public Key**: Update `src/main.rs` with your Verifying (Public) Key bytes.
4. **Tag & Release**: Push a tag starting with `v` to trigger the automated release workflow.

## ⚖️ License

This project is licensed under the MIT License.