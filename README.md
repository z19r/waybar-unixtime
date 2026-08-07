<p align="center">
  <img src="assets/logo.svg" width="120" alt="waybar-unixtime logo">
</p>

<h1 align="center">waybar-unixtime</h1>

<p align="center">
  Live unix timestamps in your Waybar. Themed by omarchy.
  <br>
  <a href="https://waybar-unixtime.z19r.com">
    Website</a> ·
  <a href="#install">Install</a> ·
  <a href="#theming">Theming</a>
</p>

<p align="center">
  <a
    href="https://github.com/z19r/waybar-unixtime/actions"
    ><img
    src="https://github.com/z19r/waybar-unixtime/actions/workflows/ci.yml/badge.svg"
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
Left-click opens a dropdown of every format — seconds, millis,
micros, nanos, ISO 8601, European, US, British, Japanese, RFC 2822 —
and clicking one copies it. The colors come from whatever
[omarchy](https://omarchy.org) theme you're running — switch themes
and the module follows.

```
… 1786096903 …
```

## Features

- **Live epoch** — ticking every second in the bar
- **Format dropdown** — left-click opens a menu of 15 formats;
  each entry copies its current value to the clipboard:

  | Timestamp | Date formats |
  |-----------|--------------|
  | Seconds | ISO 8601 UTC / local / date |
  | Milliseconds | European (+ short) |
  | Microseconds | US (+ short), British |
  | Nanoseconds | Japanese, RFC 2822, Unix readable |

- **Panel tooltip** — hover shows every format with live values,
  grouped like the [UnixTime](https://unixtime.labor77.de/en) panel
- **Converter** — `waybar-unixtime convert 1786098721` (auto-detects
  s/ms/µs/ns) or `convert "30.07.2026 12:34"` or
  `convert "now +2h30m"` prints every format for that instant
- **Interactive picker** — right-click opens a walker/fuzzel/wofi
  dmenu with all formats, a free-text converter, and history
- **History** — every copy is remembered (capped, configurable);
  re-copy from the picker or `waybar-unixtime history`
- **Custom formats** — name your own strftime patterns in
  `~/.config/waybar-unixtime/config.toml`; they join the dropdown,
  tooltip, and picker
- **Switchable display** — scroll on the module (two-finger swipe
  on touchpads) to cycle through every format; middle-click, if you
  have one, toggles seconds ⇄ millis; `waybar-unixtime set iso-utc`
  jumps straight to any key
- **omarchy-native theming** — `waybar-unixtime css` reads the
  active theme's `colors.toml` and emits namespaced CSS
- **Single static-ish binary** — Rust, no runtime deps

## Install

### Arch (AUR-style)

```sh
git clone https://github.com/z19r/waybar-unixtime
cd waybar-unixtime/packaging/aur && makepkg -si
```

### Cargo

```sh
cargo install waybar-unixtime
```

### Prebuilt binaries

Grab a tarball for `x86_64` or `aarch64` Linux from the
[releases page](https://github.com/z19r/waybar-unixtime/releases),
verify against `SHA256SUMS.txt`, drop the binary in your `$PATH`.

### From source

```sh
just install   # cargo install + generates themed CSS
```

## Waybar setup

Print a ready-to-paste module block (with dropdown menu wired up)
and add it to your waybar config:

```sh
waybar-unixtime snippet
```

Full example in
[`examples/waybar-config.jsonc`](examples/waybar-config.jsonc).
Note: use polling (`once` + `"interval": 1`) — waybar 0.15.0 does
not render bars whose custom modules stream continuously.

Put `"custom/unixtime"` in `modules-right` (or wherever), then:

```sh
waybar-unixtime css --install    # themed unixtime.css
waybar-unixtime menu --install   # dropdown unixtime-menu.xml
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
waybar-unixtime once          # one waybar JSON line (default)
waybar-unixtime copy [FMT]    # print a timestamp in any format
waybar-unixtime convert IN    # epoch/date/-> every format
waybar-unixtime picker        # interactive dmenu panel
waybar-unixtime history       # recently copied values
waybar-unixtime formats       # list all keys with live examples
waybar-unixtime set FMT       # change the bar display format
waybar-unixtime toggle        # flip seconds <-> milliseconds
waybar-unixtime cycle [--back]# step through all formats (scroll)
waybar-unixtime menu          # dropdown XML (--install to write)
waybar-unixtime css           # themed CSS   (--install to write)
waybar-unixtime snippet       # ready-to-paste waybar config
waybar-unixtime run           # streaming mode (waybar != 0.15.0)
```

`FMT` is any key from `formats`, or `custom:<strftime>` — e.g.
`waybar-unixtime copy "custom:%d.%m.%y %H:%M"`. `IN` accepts an
epoch in any precision, `now`, date strings (ISO / European / US),
and an optional trailing offset: `convert "1786098721 -1d"`.

## Settings

`~/.config/waybar-unixtime/config.toml`:

```toml
default_format = "seconds"   # bar text before any set/toggle
history_size = 50            # copied-value history (0 = off)

[[custom]]                   # named custom formats
name = "Deploy tag"
format = "%Y%m%d-%H%M"
```

After adding custom formats, re-run `waybar-unixtime menu --install`
and refresh the `menu-actions` block from `waybar-unixtime snippet`
so the dropdown picks them up.

## Development

```sh
just check   # fmt-check + clippy -D warnings + tests
just test
just site    # serve the marketing site locally
```

See [CONTRIBUTING.md](CONTRIBUTING.md). Released under the
[MIT license](LICENSE).
