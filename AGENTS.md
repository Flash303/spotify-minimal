# AGENTS.md — Minimal Spotify (Rust)

Reference for any agent working on this repository.
Manual tests (webview, DRM, audio) are performed by the user:
ask them explicitly once compilation and automated tests pass.

## Mission

PoC validating two hypotheses: audio playback with DRM in the OS native
webview, and ad-request interception/blocking from Rust.
Primary target: Debian/Wayland (WebKitGTK). Daily driver: Windows 11.

## Commands

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo run -p spotify-shell
RUST_LOG=info cargo run -p spotify-shell
cargo run -p spotify-shell -- --help
```

Produced binary: `spotify-minimal`. Window title: `spotify-minimal`
(used for audio routing under PipeWire/PulseAudio).

Windows installer (NSIS, per-user, no UAC):
`packaging/windows/spotify-minimal.nsi`. Rebuild:
`cargo build --release -p spotify-shell` then `makensis` on the `.nsi`
(`makensis /DVERSION=x.y.z` overrides the version).
Linux installer (.deb, any arch): built by CI from `packaging/linux/`.
Keep the version in sync between `Cargo.toml`, the `.nsi` and the `.deb`.

Environment variables: `RUST_LOG` (logs).

## Layout

```text
Cargo.toml                    workspace (resolver 2, shared deps and lints)
crates/adblock-engine/        lib, zero GUI dependency
  src/lib.rs                  Blocker, BlockError, FILTER_RULES, patterns_from_rules + tests
crates/spotify-shell/         spotify-minimal binary
  src/main.rs                 logs, CLI (--help), startup
  src/config.rs               constants (URL, title, size)
  src/blocker.rs              Rust navigation + JS hook + tests
  src/webview.rs              tao + wry, cfg linux (build_gtk) / others (build)
  src/mixer.rs                Windows: mixer name/icon via CoreAudio
  build.rs                    Windows: exe resource (icon + VERSIONINFO)
  assets/spotify-minimal.rc   Windows resource manifest
assets/filters/spotify-ads.txt  single rule source (Rust engine + JS patterns)
assets/icon.ico / icon.png    app icon (exe + window)
docs/ARCHITECTURE.md          request flow, wry limits, structural choices
docs/MANUAL_TESTS.md          4 acceptance criteria protocol
docs/DEBIAN_WAYLAND.md        apt deps, DRM/Widevine, PipeWire
packaging/windows/spotify-minimal.nsi  NSIS installer (per-user, no UAC)
packaging/linux/              .desktop entry + .deb build files
TODO.md                       task list
.github/workflows/ci.yml     fmt + clippy (-D warnings) + test + build + installers, ubuntu + windows (x64 + arm64)
```

## Conventions

- No comments in code except proven complex problems.
- Edition 2024, `rust-version = "1.85"`, `unsafe_code = "deny"`.
  Sole exception: `spotify-shell/src/mixer.rs` (`#![allow(unsafe_code)]`),
  CoreAudio COM calls impossible in safe Rust (see ARCHITECTURE.md).
  Temporary workaround for the upstream `WebView2Feedback#2236` bug: remove it
  as soon as Microsoft/wry offer native naming (see ARCHITECTURE.md).
- Lib (`adblock-engine`): typed errors via `thiserror`. Binary: `Box<dyn Error>`.
- Logs in English via `tracing`, driven by `RUST_LOG`. No `println!` except `--help`.
- Injected JS in ES5, no comments, single `__SPOTIFY_BLOCK_PATTERNS__` marker.
- `Cargo.lock` versioned (`--locked` in CI). Inherited workspace lints (`[lints] workspace = true`).
- Commits in English. Docs, code, log messages and comments in English.

## Verified API facts (do not reinvent)

- `adblock` 0.13: `Engine::new_with_filter_set(FilterSet)`,
  `FilterSet::new(bool)` + `add_filter_list(String, ParseOptions::default())`,
  4-argument `Request::new(url, source, type, method)`,
  `check_network_request(&Request).should_block()`. `Engine` is `!Send`/`!Sync`:
  instances owned by the main thread only.
- `wry` 0.57: no remote-HTTPS subrequest interception.
  Used: `with_url`, `with_autoplay(true)`, `with_devtools(true)`,
  `with_initialization_script`, `with_navigation_handler`, `with_ipc_handler`.
  No exposed DRM flag: autoplay + the native UA are the levers.
- `tao` 0.37: `EventLoop::new()` without `Result`, `run(self, handler) -> !` (diverges),
  hence `webview::run() -> Result<Infallible, ...>`.
  `WindowBuilder::build` returns `Result<Window, tao::error::OsError>`.
  On Linux: `WindowExtUnix::gtk_window() -> &ApplicationWindow` and
  `WebViewBuilderExtUnix::build_gtk` (official wry pattern for Wayland).
- `tao::dpi` is re-exported by `tao` (no direct `dpi` dependency).
- Windows (`windows` 0.62): `IAudioSessionControl::SetDisplayName` /
  `SetIconPath` on the `msedgewebview2.exe` sessions descended from our
  process (verified via ToolHelp), `PWSTR::as_wide`, `PCWSTR::from_raw`.
  `wry` offers nothing for this; upstream limitation tracked in
  `WebView2Feedback#2236` (no Microsoft fix to date).

## Known pitfalls

- JS review: patterns include the path (`spclient.wg.spotify.com/ads`), never
  the bare host when the host also serves legitimate traffic (else overblocking).
- `is_blocked` is fail-open on invalid URLs (documented, deliberate).
- `@@` (exception) rules are ignored for JS patterns: the Rust engine
  remains the reference for navigations.
- Custom protocols (`with_custom_protocol`) are useless here: they don't capture remote HTTPS.
