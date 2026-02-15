# Release Process

This project uses GitHub Actions to automatically build and publish releases for Linux, macOS, and Windows.

## Automated Releases

Releases are triggered by pushing a git tag that starts with `v` (e.g., `v1.0.0`).

### Step-by-Step Instructions

1.  **Prepare your changes**:
    Ensure `Cargo.toml` has the correct version number.
    ```bash
    # Example: update version in Cargo.toml
    # [package]
    # name = "my-app"
    # version = "1.0.0"
    ```

2.  **Commit and push**:
    Commit all final changes to the `main` branch.
    ```bash
    git add .
    git commit -m "Prepare release v1.0.0"
    git push origin main
    ```

3.  **Tag the release**:
    Create a local tag and push it to GitHub.
    ```bash
    git tag v1.0.0
    git push origin v1.0.0
    ```

4.  **Verify the build**:
    - Go to the **Actions** tab on GitHub.
    - You will see a workflow named **Release** running.
    - It will build the binary for:
        - Linux (x86_64)
        - macOS (x86_64 and Apple Silicon)
        - Windows (x86_64)