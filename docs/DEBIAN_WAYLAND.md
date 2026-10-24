# Debian / Wayland

## Build dependencies

```sh
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

Then `cargo build -p spotify-shell`.

## Wayland

The code uses `build_gtk` (`WebViewBuilderExtUnix`) on `window.gtk_window()`
(`WindowExtUnix`) on Linux, compatible with X11 and Wayland (plain `wry`
`build` only covers X11). No specific environment variable is required;
`tao` initializes GTK.

## DRM / Widevine (hypothesis to validate)

- WebKitGTK plays media via GStreamer: install the base/good/bad/ugly plugins
  and `libav` per your distribution.
- The Widevine CDM does not ship with WebKitGTK. Leads: Chromium package with
  Widevine + libraries expected by the encrypted backend, or comparative
  Firefox/Chrome test to isolate an account or network issue.
- A DRM failure here does not invalidate the adblock engine: it is exactly the
  identified spec risk, covered by plan B
  (`spotify-minimal --plan-b`).

## Audio (PipeWire / WirePlumber)

The single process shows up in `pavucontrol` as `spotify-minimal`.
Routing: `pavucontrol` Playback tab, per-app switch, persistent via
WirePlumber rules if needed.

## Useful variables

```sh
RUST_LOG=info ./target/debug/spotify-minimal
SPOTIFY_PLAN_B_BIN=/usr/bin/chromium ./target/debug/spotify-minimal --plan-b
```
