<p align="center">
  <img src="assets/logo.svg" width="120" alt="waybar-unixtime logo">
</p>

<h1 align="center">waybar-unixtime</h1>

<p align="center">
  Live unix timestamps in your Waybar. Themed by omarchy.
  <br>
  <a href="https://zackkitzmiller.github.io/waybar-unixtime/">
    Website</a> ·
  <a href="#install">Install</a> ·
  <a href="#theming">Theming</a>
</p>

<p align="center">
  <a
    href="https://github.com/zackkitzmiller/waybar-unixtime/actions"
    ><img
    src="https://github.com/zackkitzmiller/waybar-unixtime/actions/workflows/ci.yml/badge.svg"
    alt="CI"></a>
  <a href="https://crates.io/crates/waybar-unixtime"><img
    src="https://img.shields.io/crates/v/waybar-unixtime.svg"
    alt="crates.io"></a>
  <a href="LICENSE"><img
    src="https://img.shields.io/badge/license-MIT-c9a554.svg"
    alt="MIT"></a>
</p>

---

The current unix timestamp, ticking every second, right in your bar.
Left-click copies it. Right-click flips to milliseconds. The colors
come from whatever [omarchy](https://omarchy.org) theme you're
running — switch themes and the module follows.

```
… 🕓 1786096903 …
```

## Features

- **Live epoch** — streams waybar JSON once a second (continuous
  `exec` mode, one tiny process, no polling scripts)
- **Seconds ⇄ milliseconds** — toggle at runtime with `SIGUSR1`
- **Click to copy** — `waybar-unixtime copy | wl-copy`
- **Rich tooltip** — epoch, UTC, local time, and ISO 8601 at a glance
- **omarchy-native theming** — `waybar-unixtime css` reads the active
  theme's `colors.toml` and emits namespaced `@define-color` CSS
- **Single static-ish binary** — Rust, no runtime deps

## Install

### Arch (AUR-style)

```sh
git clone https://github.com/zackkitzmiller/waybar-unixtime
cd waybar-unixtime/packaging/aur && makepkg -si
```

### Cargo

```sh
cargo install waybar-unixtime
```

### Prebuilt binaries

Grab a tarball for `x86_64` or `aarch64` Linux from the
[releases page](https://github.com/zackkitzmiller/waybar-unixtime/releases),
verify against `SHA256SUMS.txt`, drop the binary in your `$PATH`.

### From source

```sh
just install   # cargo install + generates themed CSS
```

## Waybar setup

Add the module (full example in
[`examples/waybar-config.jsonc`](examples/waybar-config.jsonc)):

```jsonc
"custom/unixtime": {
  "exec": "waybar-unixtime",
  "return-type": "json",
  "restart-interval": 1,
  "on-click": "waybar-unixtime copy | wl-copy",
  "on-click-right": "pkill -USR1 -x waybar-unixtime"
}
```

Put `"custom/unixtime"` in `modules-right` (or wherever), then:

```sh
waybar-unixtime css --install   # writes ~/.config/waybar/unixtime.css
```

and add to the top of your waybar `style.css`:

```css
@import "unixtime.css";
```

## Theming

Themes inherit from omarchy. The `css` subcommand resolves
`~/.config/omarchy/current/theme/colors.toml` and generates:

```css
@define-color unixtime-accent #c9a554;  /* theme accent   */
@define-color unixtime-fg     #c2c2b0;  /* foreground     */
@define-color unixtime-bg     #222222;  /* background     */
```

Everything is namespaced `unixtime-*`, so it can't collide with your
existing waybar styles. Override any rule after the `@import` to
customize. To re-theme automatically on omarchy theme switches, add
this line to `~/.config/omarchy/hooks/theme-set` (or any theme-change
hook you use):

```sh
waybar-unixtime css --install && pkill -SIGUSR2 waybar
```

Not on omarchy? Point the generator anywhere:

```sh
OMARCHY_THEME_DIR=~/my/theme waybar-unixtime css
```

Missing or unreadable themes fall back to a built-in palette, so the
module always renders.

## CLI

```
waybar-unixtime            # stream JSON lines forever (default)
waybar-unixtime once       # single JSON line
waybar-unixtime copy       # bare timestamp (pipe to wl-copy)
waybar-unixtime css        # themed CSS to stdout
waybar-unixtime css --install
       --millis            # start in milliseconds mode
       --interval <ms>     # tick rate, 50..=3600000 (default 1000)
```

## Development

```sh
just check   # fmt-check + clippy -D warnings + tests
just test
just site    # serve the marketing site locally
```

See [CONTRIBUTING.md](CONTRIBUTING.md). Released under the
[MIT license](LICENSE).
