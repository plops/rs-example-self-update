This document serves as the technical specification for the implementation of a self-updating Rust command-line interface (CLI) application.

## 1. Project Overview
**Goal:** Create a cross-platform Rust executable (Windows, macOS, Linux) that automatically detects, downloads, verifies, and installs updates from a public GitHub repository without user intervention, ensuring integrity via cryptographic signatures.

**Key Constraint:** The update check must occur on every application startup but **must not block** the main application execution flow.

---

## 2. GitHub Configuration (Server-Side)

To facilitate the update mechanism, the hosting repository must adhere to strict conventions.

### 2.1 Repository Settings
*   **Visibility:** Public.
*   **Releases:** Must utilize GitHub Releases features.
*   **Tagging Strategy:** Semantic Versioning (SemVer) must be used for tags (e.g., `v1.0.1`, `v2.1.0`). The `v` prefix is mandatory.

### 2.2 Release Assets (Artifacts)
Every release must contain the following assets for each supported platform. The naming convention is critical for the updater to correctly identify the target.

| Platform    | Target Triple              | Required Asset Name                      | Signature Method                         |
| :---        | :---                       | :---                                     | :---                                     |
| **Linux**   | `x86_64-unknown-linux-gnu` | `rs-example-self-update-vX.Y.Z-x86_64-unknown-linux-gnu.tar.gz` | Embedded (ZipSign tar)                   |
| **macOS**   | `x86_64-apple-darwin`      | `rs-example-self-update-vX.Y.Z-x86_64-apple-darwin.tar.gz`      | Embedded (ZipSign tar)                   |
| **macOS**   | `aarch64-apple-darwin`     | `rs-example-self-update-vX.Y.Z-aarch64-apple-darwin.tar.gz`     | Embedded (ZipSign tar)                   |
| **Windows** | `x86_64-pc-windows-msvc`   | `rs-example-self-update-vX.Y.Z-x86_64-pc-windows-msvc.zip`      | Embedded (ZipSign zip)                   |

*Note: `rs-example-self-update` and `vX.Y.Z` are placeholders for the actual application name and version.*

### 2.3 Secrets Management
*   **Encrypted Secrets:** The repository must store the `ZIPSIGN_PRIV_KEY` and `ZIPSIGN_PASSWORD` in GitHub Actions Secrets to allow the CI pipeline to sign releases automatically.

---

## 3. Application Specifications (Client-Side)

### 3.1 Version Discovery & Comparison
*   **Self-Awareness:** The application must compile with the version defined in `Cargo.toml` embedded (accessible via `env!("CARGO_PKG_VERSION")`).
*   **Discovery:** The app queries the GitHub Releases API (`GET /repos/{owner}/{repo}/releases/latest`) over **HTTPS**.
*   **Comparison Logic:**
    1.  Parse the remote tag (remove `v` prefix).
    2.  Compare remote SemVer vs. local SemVer.
    3.  If `Remote > Local`, initiate the update sequence.
    4.  If `Remote <= Local`, terminate the update thread silently.

### 3.2 Non-Blocking Execution Strategy
To satisfy the requirement that the update check does not slow down startup:
1.  **Main Thread:** Starts the application logic immediately.
2.  **Update Thread:** A separate, asynchronous background thread is spawned immediately upon startup to perform the network check.
    *   *Scenario A (Short-lived execution):* If the main program finishes before the network check, the program exits. The update is skipped (fail-safe).
    *   *Scenario B (Long-lived execution):* If the main program is still running, the update thread proceeds to download and prompt/install.

### 3.3 Verification Strategy (Zipsign)
Before any file modification occurs, the application must verify the authenticity of the downloaded asset.
*   **Embedded Key:** The `zipsign` **Public Key** must be hardcoded into the application binary at compile time.
*   **Verification Process:**
    1.  Download the binary archive (e.g., `.tar.gz` or `.zip`).
    2.  The application uses `self_update` with the `signatures` feature to extract and verify the embedded signature.
    3.  Compute the Ed25519 signature verification using the embedded Public Key.
*   **Failure Condition:** If verification fails (invalid signature, wrong key, or file corruption), the update **must** be aborted immediately, and the downloaded files deleted.

### 3.4 Update Application & Fallback
The update process must be atomic and reversible.
1.  **Backup:** The current running executable is copied to a backup path (e.g., `rs-example-self-update.bak`).
2.  **Swap:** The new binary replaces the current binary.
    *   *Windows Specifics:* The running executable is renamed (e.g., to `app-name.old`) before the new one is moved into place, to avoid file-locking issues.
3.  **Health Check (Validation):**
    *   The updater runs the *new* binary in a subprocess with a specific flag (e.g., `--health-check`).
    *   If the subprocess returns exit code `0`, the update is finalized.
    *   If the subprocess crashes or returns non-zero, the **Rollback** is triggered.
4.  **Rollback:**
    *   The new (broken) binary is deleted.
    *   The backup (`app-name.bak`) is restored to the original path.
    *   The user is notified of the failed update attempt.

---

## 4. Error Handling & User Feedback

The application must be robust against network and environment failures.

| Error Scenario                    | Required Action                       | User Notification                                                                     |
| :---                              | :---                                  | :---                                                                                  |
| **No Internet Connection**        | Abort update thread silently.         | None (Logs only if verbose mode is on).                                               |
| **GitHub API Rate Limit**         | Abort update thread silently.         | None.                                                                                 |
| **Signature Verification Failed** | **CRITICAL ABORT.** Delete downloads. | **STDERR:** "Security Warning: Update signature verification failed. Update aborted." |
| **Download Interrupted**          | Retry once, then abort.               | **STDERR:** "Update download failed."                                                 |
| **Health Check Failed**           | Trigger Rollback.                     | **STDERR:** "Update failed validation. Restoring previous version."                   |
| **Permission Denied (OS)**        | Abort.                                | **STDERR:** "Insufficient permissions to update."                                     |

---

## 5. Build System & CI/CD Requirements

Automating the release process is required to ensure signatures match the binaries.

### 5.1 Build Environment
*   **Toolchain:** Rust (latest stable).
*   **Dependencies:** `self_update` (with `archive-zip`, `compression-zip-deflate`, `signatures` features).
*   **External Tools:** `zipsign` must be installed in the CI environment.

### 5.2 CI Workflow (GitHub Actions)
A workflow file (e.g., `release.yml`) must be configured to trigger on `push tags: v*`.
1.  **Matrix Build:** Spin up runners for Ubuntu, macOS, and Windows.
2.  **Compile:** Build with `--release`.
3.  **Compress:** Zip (Windows) or Tarball (Unix) the binary.
4.  **Sign:**
    *   Retrieve `ZIPSIGN_PRIV_KEY` from Secrets.
    *   Decrypt key using `ZIPSIGN_PASSWORD`.
    *   Embed the signature into the archive using `zipsign sign [zip|tar]`.
5.  **Upload:** Publish the signed archive to the GitHub Release. (No separate `.sig` file is uploaded).

---

## 6. Security Summary
*   **Transport:** TLS 1.2/1.3 (via HTTPS) protects against passive eavesdropping.
*   **Integrity:** Ed25519 Signatures (Zipsign) protect against Man-in-the-Middle (MitM) attacks, DNS spoofing, and GitHub compromise (assuming the private key in Secrets remains secure).
*   **Availability:** Fallback/Rollback mechanisms protect against delivering broken builds to users.
