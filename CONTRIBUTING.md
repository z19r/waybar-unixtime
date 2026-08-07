# Contributing

Thanks for helping out. The whole project is driven by `just`:

```sh
just            # list recipes
just check      # fmt-check + clippy (deny warnings) + tests
just run once   # smoke-test the module locally
just css        # preview generated CSS for your omarchy theme
just site       # serve the marketing site locally
```

## Ground rules

- Max line length is 80 characters, in every language.
- `just check` must pass before you open a PR.
- Tests first (TDD). New behavior needs a failing test before the
  implementation, and coverage stays at 80%+ (`just coverage`).
- Conventional commits: `feat:`, `fix:`, `chore:`, `docs:`, `test:`.
- Keep files small and focused; prefer new modules over long ones.

## Releases

Maintainers run `just release <patch|minor|major>`. Merging the
release PR tags, builds binaries, creates the GitHub release, and
publishes to crates.io automatically.
