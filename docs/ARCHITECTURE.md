# Architecture

## Overview

Two-crate Cargo workspace separating the filterable-testable part from the GUI pipeline:

```text
                    +-------------------+
                    | open.spotify.com  |
                    +---------+---------+
                               |
                     +---------v---------+
                     |   spotify-shell   |  binary (tao + wry)
                     |  webview.rs       |  borderless window, autoplay, devtools
                     |  blocker.rs  -----+--+----------------------+
                     +---------+---------+  |                      |
                               |            |                      |
                  top-level     |   init-script JS (fetch/XHR/     |  IPC postMessage
                  navigation    |   beacon) generated from Rust    |  -> tracing logs
                               |                                  |
                     +---------v----------------------------------v---------+
                     |                    adblock-engine                    |
                     |  Blocker (adblock::Engine) + patterns_from_rules     |
                     |  source: assets/filters/spotify-ads.txt              |
                     +------------------------------------------------------+
```

## Blocking layers

Stable `wry` exposes no interception of remote HTTPS subrequests
(no `webRequest` equivalent; `with_custom_protocol` only covers custom
schemes). The PoC therefore stacks:

1. Navigations: `with_navigation_handler` asks `Blocker::is_blocked(url,
   SOURCE_URL, "document")` and rejects ad navigations.
2. Page subrequests: bootstrap script injected before any site script,
   wrapping `fetch`, `XMLHttpRequest` and `sendBeacon`. Patterns derive from
   `patterns_from_rules(FILTER_RULES)`: host + path, never the bare host when
   the host also serves legitimate traffic. Each block is reported via
   `window.ipc` and logged on the Rust side.
3. Future extension: `with_proxy_config` (local MITM) or, on Linux, hooking
   the WebKitGTK resource-load signal.

## Structural choices

- Single rule source: `assets/filters/spotify-ads.txt`, embedded via
  `include_str!` in `adblock-engine`. EasyList/uBlock exports can be appended
  without code changes.
- `adblock::Engine` is `!Send`/`!Sync`: `Blocker` instances live on the
  main thread (navigation handler), the JS snippet being precomputed as a `String`.
- `is_blocked` is fail-open on invalid URLs: never break playback over a
  malformed URL (the strict `check` method exists for demanding callers).
- No custom user-agent: the native UA maximizes Widevine compatibility,
  which is the point of the DRM test.
- The `spotify-minimal` window title identifies the process in
  `pavucontrol` for PipeWire routing (no code required).

## Windows name and icon

On Windows, audio comes out of the `msedgewebview2.exe` child processes, not
our binary: without intervention, the mixer shows "Microsoft Edge
WebView2" (upstream limitation `WebView2Feedback#2236`, no Microsoft fix).
`src/mixer.rs` (compiled on Windows only) spawns a thread that, immediately
then every 2 s, enumerates the CoreAudio render sessions and applies
`SetDisplayName("spotify-minimal")` + `SetIconPath(<our exe>)` to sessions
whose process is an `msedgewebview2.exe` descended from ours (parent chain
verified via ToolHelp: other WebView2 apps are untouched). Fully fail-open:
silent failure, never a crash.
Machine-verified limit: Elgato Wave Link identifies routed apps by their
process (`ProcessName: msedgewebview2`, name resolved from the Edge exe on
disk, see `AudioAppConfigurations` in its `Settings.json`) and does not read
the CoreAudio session name — our rename has no effect there, even after a
Wave Link restart. No app-side lever (same finding for Teams); workaround:
rename the channel in Wave Link.
Sole module allowed to contain `unsafe` (COM calls with no safe equivalent:
`wasapi` does not expose the setters).
Icon: `assets/icon.ico` (exe, via `build.rs` + `embed-resource` and
`assets/spotify-minimal.rc` with `VERSIONINFO`) and `assets/icon.png`
(window, `tao::Icon::from_rgba` decoded by the `png` crate).

### Workaround end of life (remove on upstream fix)

`mixer.rs` is acknowledged debt, not a feature: it only exists because
WebView2 does not name its audio sessions (`WebView2Feedback#2236`).
Once naming becomes native (WebView2 option or `wry`), delete:
`src/mixer.rs`, its call in `main.rs`, the `windows` dependency and the
`#![allow(unsafe_code)]` (flip `unsafe_code` back to `"forbid"` if no `unsafe`
remains). Keep: exe/window icon (`build.rs`, `.rc`, `icon.ico`, `icon.png`,
`png`), which are permanent.

## Documented risk

Native HTTPS interception may be limited by WebKitGTK and Widevine DRM often
fails outside mainstream browsers.
