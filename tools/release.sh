#!/bin/bash

# Rationale:
# In a typical Rust project workflow, releasing a new version involves updating the 'version' 
# field in Cargo.toml and then creating a git tag that matches that version.
# This script automates the tagging and pushing process to ensure consistency and 
# prevent accidental overwriting or duplication of release tags.

# Function:
# 1. Extracts the current version from Cargo.toml.
# 2. Checks if a git tag for this version (e.g., v0.2.3) already exists locally or remotely.
# 3. If the tag exists, it warns the user and provides a one-liner command to increment 
#    the patch version in Cargo.toml.
# 4. If the tag does not exist, it creates a local tag and pushes it to 'origin'.

set -e

# Path to Cargo.toml
CARGO_TOML="Cargo.toml"

if [ ! -f "$CARGO_TOML" ]; then
    echo "Error: $CARGO_TOML not found in the current directory."
    exit 1
fi

# 1. Extract version from Cargo.toml
# We look for the line starting with 'version =' and capture the value inside quotes.
VERSION=$(grep '^version =' "$CARGO_TOML" | head -n 1 | awk -F '"' '{print $2}')

if [ -z "$VERSION" ]; then
    echo "Error: Could not identify version in $CARGO_TOML."
    exit 1
fi

TAG="v$VERSION"

# 2. Verify if the tag already exists
# git rev-parse checks if the tag is known to the local repository.
if git rev-parse "$TAG" >/dev/null 2>&1; then
    echo "--------------------------------------------------------"
    echo "WARNING: Tag '$TAG' already exists!"
    echo "It looks like you haven't incremented the version in $CARGO_TOML."
    echo "--------------------------------------------------------"
    
    # Calculate suggested next version (patch increment)
    # This assumes a standard X.Y.Z format.
    IFS='.' read -r major minor patch <<< "$VERSION"
    if [[ "$patch" =~ ^[0-9]+$ ]]; then
        NEXT_PATCH=$((patch + 1))
        NEXT_VERSION="$major.$minor.$NEXT_PATCH"
        
        echo "Don't forget to update CHANGELOG.md"
        echo "Run this command to increment the version to $NEXT_VERSION and commit:"
        echo ""
        echo "  sed -i 's/^version = \"$VERSION\"/version = \"$NEXT_VERSION\"/' $CARGO_TOML && git commit -am \"bump version to $NEXT_VERSION\""
        echo ""
    else
        echo "Could not automatically determine the next version. Please update $CARGO_TOML manually."
    fi
    exit 1
fi

# 3. Execute the release commands
echo "Preparing to release $TAG..."
echo "Running: git tag $TAG && git push origin $TAG"

git tag "$TAG"
git push origin "$TAG"

echo ""
echo "Successfully released and pushed $TAG!"
