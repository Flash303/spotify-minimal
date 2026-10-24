# Minimal Spotify (Rust)

A vibe-coded proof of concept built with AI: an ad-free Spotify app that wraps
the official web player (`https://open.spotify.com`) in a native window
driven by Rust, with built-in ad blocking.

The web player was chosen deliberately: it has far fewer limitations than the
alternatives — no API approval or SDK playback restrictions, full catalog and
search, and DRM handled by the system webview instead of custom CDM plumbing.

## Stack

| Brick      | Crate  | Version |
|------------|--------|---------|
| Windowing  | `tao` | 0.37 |
| Webview    | `wry` | 0.57 |
| Adblock    | `adblock` (Brave engine) | 0.13 |

Native webview per OS: WebView2 (Windows), WebKitGTK (Linux), WKWebView (macOS).

## Prerequisites

Windows: WebView2 Runtime (preinstalled on recent Windows 10/11).

Debian/Ubuntu:

```sh
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

Linux/Wayland and DRM details: see `docs/DEBIAN_WAYLAND.md`.

## Commands

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo run -p spotify-shell
RUST_LOG=info cargo run -p spotify-shell
cargo run -p spotify-shell -- --help
```

Produced binary: `spotify-minimal`. Identical window title (audio routing
under PipeWire/PulseAudio, see `docs/DEBIAN_WAYLAND.md`).

> Status: only tested on Windows 11 so far. Linux and macOS builds compile
> in CI but have never run on a real machine.

## Installing

- Windows: run `spotify-minimal-setup-<version>.exe` (NSIS, per-user, no UAC,
  built from `packaging/windows/spotify-minimal.nsi`). Installs to
  `%LOCALAPPDATA%\Programs\spotify-minimal` with Start Menu + Desktop
  shortcuts and an uninstaller entry. Rebuild: `cargo build --release
  -p spotify-shell` then `makensis packaging/windows/spotify-minimal.nsi`.
- Linux: install the `.deb` from CI (`packaging/linux/`) with `apt install
  ./spotify-minimal_<version>_<arch>.deb`. Installs to `/usr/bin` with a
  desktop entry and icon.

## Structure

```text
crates/adblock-engine   filtering engine (adblock), no GUI dependency
crates/spotify-shell    spotify-minimal binary: tao window + wry webview, JS/Rust bridge
assets/filters/spotify-ads.txt   single rule source
docs/                   architecture, manual tests, Debian/Wayland notes
packaging/              Windows (NSIS) and Linux (.deb) installers
```

## How blocking works

`wry` exposes no interception of remote HTTPS subrequests. The PoC
therefore stacks three layers:

1. `with_navigation_handler`: Rust decision (`adblock-engine`) on navigations.
2. Bootstrapped JS hook: `fetch` / `XHR` / `sendBeacon` neutralized for the
   patterns derived from `assets/filters/spotify-ads.txt`, reported back via IPC
   (`page-level ad request blocked` logs).
3. Extension point: proxy (`with_proxy_config`) or the WebKitGTK
   `resource-load` signal on Linux.

Details: `docs/ARCHITECTURE.md`.

## Manual validation

The 4 acceptance criteria and their protocol live in `docs/MANUAL_TESTS.md`.

## License

MIT. The `adblock` engine is an MPL-2.0 dependency, used unmodified.
