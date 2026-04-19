# plx

Fast, powerline-styled terminal segments written in Rust. Renders path, git status, and tmux window titles using [libgit2](https://libgit2.org/) — no git subprocess calls.

## Usage

```
plx <path|git|tmux-title|weather|...>
```

- **`path`** — Powerline path segment with truncation and home directory collapsing
- **`git`** — Git status segment showing branch, staged/modified/untracked counts, ahead/behind, stash, and repo state (rebase, merge, etc.)
- **`tmux-title`** — Compact tmux window title with repo name, branch, and dirty indicator
- **`weather`** — One-line current conditions for tmux `status-right` (see [Weather](#weather))

## Building

### With Nix

```bash
nix build
# or run directly
nix run . -- path
```

### With Cargo

```bash
cargo build --release
```

## Integration

### Starship custom module

```toml
[custom.path_segment]
command = "plx path"
when = "true"
format = "$output"
shell = ["bash", "--nologin"]

[custom.git_segment]
command = "plx git"
when = "true"
format = "$output"
shell = ["bash", "--nologin"]
```

### Tmux status bar

```tmux
set -g automatic-rename-format '#{plx tmux-title}'
# or via a shell wrapper in status-right
```

## Weather

`plx weather` prints a one-line current-conditions summary like `Tacoma, US ⛅ 58°F`. It is designed to be called from `tmux status-right` as often as `status-interval 1`: every failure path (network timeout, parse error, geocode failure, cache miss with no network) exits 0 and prints an empty string. Nothing is ever written to stdout that tmux can't render.

### Minimal example

```tmux
set -g status-right '#(plx weather --units imperial) | %H:%M'
```

Zero-config (no API key, no lat/lon): uses [Open-Meteo](https://open-meteo.com/) and [ifconfig.co](https://ifconfig.co) for IP geolocation. Both are free and require no signup.

### Flags

| Flag | Description | Default |
|---|---|---|
| `--lat FLOAT` | Latitude | (IP geolocation) |
| `--lon FLOAT` | Longitude | (IP geolocation) |
| `--location-cmd CMD` | Shell command whose stdout is `"lat\|lon"` | — |
| `--provider NAME` | `openmeteo` or `openweather` | `openmeteo` |
| `--api-key KEY` | Required for `openweather` | — |
| `--units UNITS` | `metric` or `imperial` | `metric` |
| `--cache-ttl MIN` | Cache TTL in minutes | `15` |
| `--no-show-city` | Hide `City, CC` prefix | (shown) |
| `--no-show-icon` | Hide weather icon | (shown) |
| `--use-nerd-font` | Use Nerd Font glyphs instead of Unicode | off |
| `-h`, `--help` | Show help | — |

### Environment variables

All optional, and lower precedence than CLI flags:

- `PLX_WEATHER_LAT`, `PLX_WEATHER_LON` — fixed coordinates
- `PLX_WEATHER_LOCATION_CMD` — location command (same contract as `--location-cmd`)
- `PLX_WEATHER_PROVIDER`, `PLX_WEATHER_API_KEY`
- `PLX_WEATHER_UNITS`, `PLX_WEATHER_CACHE_TTL`
- `PLX_WEATHER_DEBUG=1` — log errors to stderr (otherwise fully silent)

### TOML configuration

Add a `[weather]` block to `~/.config/plx/config.toml`:

```toml
[weather]
provider = "openmeteo"
units = "imperial"
cache_ttl = 15
show_city = true
show_icon = true
use_nerd_font = true
# api_key = "..."           # for openweather
# lat = 47.13                # optional pinned location
# lon = -122.16
# location_cmd = "my-loc"   # shell command returning "lat|lon"
```

Precedence: **CLI flag > `PLX_WEATHER_*` env > `[weather]` TOML > built-in default**.

### Caching

Rendered output is cached at `$XDG_CACHE_HOME/plx/weather.json` (falling back to `~/.cache/plx/weather.json`). A cache hit is a single file read and print — typically single-digit milliseconds, safe for `status-interval 1`. Entries are keyed on `(provider, lat rounded to 2 decimal places, lon rounded to 2 decimal places, units)` so switching units or providers does not thrash the cache.

### Robustness contract

- Every HTTP call is capped at 3 seconds.
- On network failure, falls back to the previously cached value if present, else empty string.
- `plx weather` always exits 0.
- Never writes anything to stdout other than the rendered line or an empty string.

### Opting out

The weather subcommand is feature-gated and enabled by default. To build without it (and without the HTTP dependency):

```bash
cargo build --release --no-default-features --features banner
```
