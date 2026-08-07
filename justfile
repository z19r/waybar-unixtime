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

# Install to ~/.cargo/bin, generate themed CSS + dropdown menu
install:
    cargo install --path . --locked
    waybar-unixtime css --install
    waybar-unixtime menu --install

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

# Release quality gate (fmt + clippy + test)
release-check:
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings
    cargo test

# Preview what a release would do without changing anything
release-dry-run LEVEL:
    #!/usr/bin/env bash
    set -euo pipefail
    if [[ ! "{{ LEVEL }}" =~ ^(patch|minor|major)$ ]]; then
        echo "Usage: just release-dry-run patch|minor|major"; exit 1
    fi
    CURRENT=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
    echo "Current version: $CURRENT"
    echo "Bump level: {{ LEVEL }}"
    just release-check
    echo ""
    echo "All checks passed. Run: just release {{ LEVEL }}"

# Bump version, create release branch + PR (requires: cargo-set-version, gh)
release LEVEL: release-check
    #!/usr/bin/env bash
    set -euo pipefail
    if [[ ! "{{ LEVEL }}" =~ ^(patch|minor|major)$ ]]; then
        echo "Usage: just release patch|minor|major"; exit 1
    fi
    if [[ -n "$(git status --porcelain)" ]]; then
        echo "Error: dirty working tree"; exit 1
    fi
    BRANCH=$(git rev-parse --abbrev-ref HEAD)
    if [[ "$BRANCH" != "master" ]]; then
        read -r -p "Not on master (currently on $BRANCH). Switch to master? [y/N] " REPLY || REPLY=""
        if [[ "$REPLY" =~ ^[Yy]$ ]]; then
            git checkout master
        else
            echo "Aborted: release must run from master"; exit 1
        fi
    fi
    git pull --ff-only origin master
    cargo set-version --bump {{ LEVEL }}
    cargo check --quiet
    VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
    # VERSION is the file the release workflow triggers on and reads.
    echo "$VERSION" > VERSION
    # Promote [Unreleased] to [VERSION] with auto-generated entries from
    # conventional commits since the last tag.
    TODAY=$(date -u +%Y-%m-%d)
    PREV_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
    if [[ -n "$PREV_TAG" ]]; then
        COMMIT_LOG=$(git log "${PREV_TAG}..HEAD" --pretty=format:"%s" --no-merges | grep -v '^release:' || true)
    else
        COMMIT_LOG=$(git log --pretty=format:"%s" --no-merges | grep -v '^release:' || true)
    fi
    ADDED=()
    FIXED=()
    CHANGED=()
    RE_FEAT='^feat(\([^)]*\))?!?:[[:space:]](.+)'
    RE_FIX='^fix(\([^)]*\))?!?:[[:space:]](.+)'
    RE_OTHER='^(refactor|perf|chore|ci|docs|test|build|style)(\([^)]*\))?!?:[[:space:]](.+)'
    while IFS= read -r msg; do
        [[ -z "$msg" ]] && continue
        if [[ "$msg" =~ $RE_FEAT ]]; then
            ADDED+=("${BASH_REMATCH[2]}")
        elif [[ "$msg" =~ $RE_FIX ]]; then
            FIXED+=("${BASH_REMATCH[2]}")
        elif [[ "$msg" =~ $RE_OTHER ]]; then
            CHANGED+=("${BASH_REMATCH[3]}")
        fi
    done <<< "$COMMIT_LOG"
    {
        echo "## [Unreleased]"
        echo ""
        echo "## [${VERSION}] - ${TODAY}"
        if (( ${#ADDED[@]} )); then
            echo ""
            echo "### Added"
            echo ""
            for b in "${ADDED[@]}"; do echo "- $b"; done
        fi
        if (( ${#FIXED[@]} )); then
            echo ""
            echo "### Fixed"
            echo ""
            for b in "${FIXED[@]}"; do echo "- $b"; done
        fi
        if (( ${#CHANGED[@]} )); then
            echo ""
            echo "### Changed"
            echo ""
            for b in "${CHANGED[@]}"; do echo "- $b"; done
        fi
    } > /tmp/waybar_unixtime_cl_section
    awk '
        /^## \[Unreleased\]/ {
            while ((getline line < "/tmp/waybar_unixtime_cl_section") > 0) print line
            next
        }
        { print }
    ' CHANGELOG.md > CHANGELOG.md.tmp
    mv CHANGELOG.md.tmp CHANGELOG.md
    rm -f /tmp/waybar_unixtime_cl_section
    git checkout -b "release/v${VERSION}"
    git add Cargo.toml Cargo.lock VERSION CHANGELOG.md
    git commit -m "release: v${VERSION}"
    git push -u origin "release/v${VERSION}"
    gh pr create \
        --title "release: v${VERSION}" \
        --body "Bump to v${VERSION} ({{ LEVEL }} release)" \
        --base master

    echo "Waiting for CI checks to appear..."
    for i in $(seq 1 30); do
        if gh pr checks --json name 2>/dev/null | grep -q name; then break; fi
        sleep 2
    done
    echo "Watching CI checks..."
    gh pr checks --watch --fail-fast

    echo "CI passed. Merging..."
    gh pr merge --squash --delete-branch

    git checkout master
    git pull --ff-only origin master

    echo "Watching release workflow..."
    gh run watch

    echo ""
    echo "Release v${VERSION} complete."
