Here is the complete Requirements Specification Document. You can save this as `REQUIREMENTS.md` in your project repository to guide the development and CI setup.

***

# Requirements Specification: Self-Updating Rust Application

## 1. Project Overview
The objective is to develop a cross-platform Rust command-line application capable of updating itself automatically by fetching the latest compatible binary from GitHub Releases. The update process must be secure, verifying the authenticity and integrity of the downloaded binary using `zipsign` (Minisign) signatures before applying any changes.

## 2. Technology Stack
*   **Language**: Rust (Latest Stable).
*   **Core Update Library**: [`self_update`](https://crates.io/crates/self_update).
*   **Cryptography/Signing**: [`zipsign`](https://crates.io/crates/zipsign) (Ed25519 signatures).
*   **Hosting**: GitHub Releases.
*   **CI/CD**: GitHub Actions.
*   **Supported Platforms**:
    *   Windows (`x86_64-pc-windows-msvc`)
    *   macOS (`x86_64-apple-darwin` / `aarch64-apple-darwin`)
    *   Linux (`x86_64-unknown-linux-gnu`)

---

## 3. GitHub & Infrastructure Configuration

### 3.1 Repository Settings
*   The repository must be public (or a token must be provided if private, though this spec assumes public).
*   **Tags**: Releases must use Semantic Versioning tags (e.g., `v1.0.0`, `v1.0.1`).

### 3.2 Secrets Management
The following secrets must be stored in **GitHub Repository Secrets**:
1.  `ZIPSIGN_PRIV_KEY`: The contents of the `zipsign.priv` file generated locally.
2.  `ZIPSIGN_PASSWORD`: The password used to encrypt the private key (if applicable).

### 3.3 Release Artifact Naming Convention
To ensure the `self_update` library can identify the correct binary for the OS, release assets must follow this strict naming pattern:
*   **Archive**: `<bin_name>-<version>-<target>.<extension>`
*   **Signature**: `<bin_name>-<version>-<target>.<extension>.sig`

**Examples:**
*   `my-app-1.2.0-x86_64-unknown-linux-gnu.tar.gz`
*   `my-app-1.2.0-x86_64-unknown-linux-gnu.tar.gz.sig`
*   `my-app-1.2.0-x86_64-pc-windows-msvc.zip`
*   `my-app-1.2.0-x86_64-pc-windows-msvc.zip.sig`

---

## 4. Application Requirements

### 4.1 Version Identification
*   **Source of Truth**: The application version must be defined in `Cargo.toml`.
*   **Build Integration**: The build system must inject the version into the binary using `env!("CARGO_PKG_VERSION")`.
*   **Runtime Check**: The app must be able to output its current version (e.g., via `app --version`).

### 4.2 Update Logic
1.  **Discovery**: The app checks the specific GitHub Repository for the "Latest" release.
2.  **Comparison**: The app parses the remote tag (e.g., `v1.0.1`) and compares it against the local version.
    *   If `Remote > Local`: Proceed to update.
    *   If `Remote <= Local`: Exit update process (notify user "Up to date").
3.  **Target Selection**: The app must identify the current OS and Architecture to download the matching asset.

### 4.3 Security & Verification
*   **Public Key Embedding**: The `zipsign.pub` key content must be hardcoded (embedded) into the application source code.
*   **Signature Check**:
    *   The app must download both the archive (`.zip`/`.tar.gz`) and the signature (`.sig`).
    *   The app must verify the signature against the embedded public key **before** extracting or executing the binary.
    *   **Failure Condition**: If verification fails (invalid signature, corrupted file, wrong key), the update **must abort** immediately without modifying the local binary.

### 4.4 Health Check & Rollback
*   **Backup**: Before replacing the binary, the current executable must be copied to a backup path (e.g., `app.bak`).
*   **Health Flag**: The application must implement a hidden flag: `--health-check`. When run with this flag, it should perform a minimal self-test (e.g., load config, print "OK") and exit with code `0`.
*   **Post-Update Verification**:
    1.  The updater launches the *new* binary with `--health-check`.
    2.  **Success**: If exit code is `0`, delete backup.
    3.  **Failure**: If exit code is non-zero or process crashes, restore `app.bak` to the original filename and alert the user.

---

## 5. Error Handling & User Feedback

The application must provide clear CLI output for the following scenarios:

| Scenario | System Action | User Message Requirement |
| :--- | :--- | :--- |
| **Network Failure** | Abort. | "Could not connect to update server. Check internet connection." |
| **No New Version** | Exit. | "Already up to date (Version X.Y.Z)." |
| **Signature Mismatch** | **CRITICAL ABORT.** Delete downloaded temp files. | "Update validation failed! The downloaded file may be corrupted or tampered with. Update cancelled." |
| **Permission Denied** | Abort. | "Insufficient permissions to replace binary. Try running as Administrator/Root." |
| **Health Check Fail** | Rollback to backup. | "New version failed to start. Restoring previous version..." |

---

## 6. Build & Release Pipeline (CI Specifications)

A GitHub Actions workflow (`release.yml`) is required to automate the distribution:

1.  **Trigger**: On push of tag `v*`.
2.  **Matrix Build**: Build for Ubuntu, macOS, and Windows.
3.  **Packaging**:
    *   Unix: `tar.gz`
    *   Windows: `.zip`
4.  **Signing Step**:
    *   Install `zipsign`.
    *   Load private key from Secrets.
    *   Generate `.sig` file for the archive.
5.  **Publishing**: Upload archive + signature to the GitHub Release.

---

## 7. Implementation Roadmap

1.  **Local Setup**: Install `zipsign`, generate keys (`zipsign.pub`, `zipsign.priv`).
2.  **Manifest Setup**: Add `self_update` dependencies to `Cargo.toml`.
3.  **Code - Verification**: Implement `update_from_github` with `.verify_with_zipsign()`.
4.  **Code - Safety**: Implement `safe_update` wrapper (Backup -> Update -> Health Check -> Rollback).
5.  **CI Setup**: Create `.github/workflows/release.yml` with signing logic.
6.  **Test**: Push tag `v0.1.0`, verify release creation. Build `v0.0.9` locally and run it to test the update path.