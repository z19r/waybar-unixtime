# Releasing waybar-unixtime

The release process mirrors [whetstone](https://github.com/z19r/whetstone):
`VERSION` is the single source of truth, and pushing a change to it on
`master` is what triggers the release workflow.

## One command

```bash
just release patch   # or: minor | major
```

That runs `release-check` first (it is a recipe dependency, so a failing
gate aborts before anything is mutated), then:

1. Refuses to run on a dirty tree or off `master`.
2. `cargo set-version --bump LEVEL`, then `cargo check`.
3. Writes the new version to `VERSION`.
4. Promotes `## [Unreleased]` in `CHANGELOG.md` to `## [X.Y.Z] - DATE`,
   grouping conventional commits since the last tag into
   Added (`feat:`) / Fixed (`fix:`) / Changed (everything else).
5. Commits to `release/vX.Y.Z`, pushes, opens a PR.
6. Waits for CI checks to appear, then `gh pr checks --watch --fail-fast`.
7. Squash-merges and deletes the branch.
8. Returns to `master`, pulls, and `gh run watch`es the release workflow.

To see what would happen without touching anything:

```bash
just release-dry-run minor
```

## What the workflow does

`.github/workflows/release.yml` fires on pushes to `master` that touch
`VERSION`:

| Job | What it does |
| --- | --- |
| `check-tag` | Reads `VERSION`, derives `vX.Y.Z`, flags SemVer prereleases. Logs but does **not** fail if the tag exists — that is recovery mode. |
| `verify` | `just release-check` on the release commit. |
| `build` | `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu` (the latter via `cross`), tarred per target. |
| `assets` | `waybar-unixtime-assets.tar.gz` from `assets/`, `examples/`, `packaging/`. |
| `release` | Creates the tag if missing, generates `SHA256SUMS.txt`, creates or updates the GitHub release. |
| `verify-release` | Re-reads the published release and fails if it is a draft, has the wrong prerelease state, or is missing any expected asset or checksum. |
| `publish-crate` | Skipped for prereleases. Queries crates.io (200 → skip, 404 → publish, anything else → fail), then `cargo publish --allow-dirty`. |

Every job is idempotent, so re-running a partially failed release is safe.

## Prereleases

A version containing `-` (e.g. `1.0.0-rc.1`) is marked prerelease on
GitHub and is **not** published to crates.io. Cut the stable version to
publish.

## Required secret

`publish-crate` needs `CARGO_REGISTRY_TOKEN` visible to this repository —
either a repo secret or an org secret granted to it:

```bash
gh secret set CARGO_REGISTRY_TOKEN --repo z19r/waybar-unixtime
```

Get the token from <https://crates.io/settings/tokens> with the
`publish-new` and `publish-update` scopes.

## The site

`site/` deploys to Netlify, not GitHub Pages. `site/netlify.toml` sets
`publish = "."` with no build command — it is static HTML, so Netlify
serves the directory as-is and applies the security and cache headers.
Deploys are driven by Netlify's git integration, so there is no site
workflow in `.github/workflows/`.
