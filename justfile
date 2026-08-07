# waybar-unixtime — task runner

set shell := ["bash", "-euo", "pipefail", "-c"]

crate := "waybar-unixtime"

# List available recipes
default:
    @just --list

# Build debug binary
build:
    cargo build

# Build optimized release binary
release-build:
    cargo build --release

# Run the module locally (ctrl-c to stop)
run *ARGS:
    cargo run -- {{ARGS}}

# Run all tests
test:
    cargo test

# Coverage report (requires cargo-llvm-cov)
coverage:
    cargo llvm-cov --fail-under-lines 80

# Format all Rust code
fmt:
    cargo fmt

# Check formatting without writing
fmt-check:
    cargo fmt --check

# Clippy with warnings as errors
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# fmt-check + lint + test (CI parity)
check: fmt-check lint test

# Install to ~/.cargo/bin and generate themed CSS
install:
    cargo install --path . --locked
    waybar-unixtime css --install

# Regenerate themed CSS from the active omarchy theme
css:
    cargo run --quiet -- css

# Serve the marketing site locally
site port="8737":
    @echo "http://localhost:{{port}}"
    python3 -m http.server {{port}} --directory site

# Remove build artifacts
clean:
    cargo clean

# Verify the tree is releasable (fmt, lint, tests, dry-run publish)
release-check:
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test
    cargo publish --dry-run --allow-dirty

# Show what a release would do without doing it
release-dry-run LEVEL:
    #!/usr/bin/env bash
    set -euo pipefail
    CURRENT=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
    echo "current: v${CURRENT}"
    echo "would bump: {{LEVEL}}"
    just release-check
    echo "dry run OK — run: just release {{LEVEL}}"

# Cut a release: bump version, open PR, merge triggers tag+publish
release LEVEL:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -n "$(git status --porcelain)" ]; then
        echo "working tree dirty; commit or stash first" >&2
        exit 1
    fi
    BRANCH=$(git rev-parse --abbrev-ref HEAD)
    if [ "$BRANCH" != "master" ] && [ "$BRANCH" != "main" ]; then
        echo "run from master/main (on $BRANCH)" >&2
        exit 1
    fi
    just release-check
    git pull --ff-only
    cargo set-version --bump {{LEVEL}}
    VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
    git checkout -b "release/v${VERSION}"
    git add Cargo.toml Cargo.lock
    git commit -m "chore: release v${VERSION}"
    git push -u origin "release/v${VERSION}"
    gh pr create \
        --title "chore: release v${VERSION}" \
        --body "Release v${VERSION} ({{LEVEL}} bump). Merge to publish." \
        --base "$BRANCH"
    echo "PR opened — merge it and CI tags + publishes v${VERSION}"
