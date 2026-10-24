# Manual tests (PoC acceptance criteria)

Run with logs: `RUST_LOG=info cargo run -p spotify-shell`.

## 1. Rendering and login

- The `spotify-minimal` window opens at 1280x800 on `https://open.spotify.com`.
- Login succeeds and the session persists across restarts (webview profile cookies).
- Fail on lasting blank page or login loop.

## 2. DRM (critical point)

- Play a protected track: sound comes out, no "web player is not working" banner.
- On Debian: WebKitGTK + GStreamer + Widevine CDM prerequisites
  (see `docs/DEBIAN_WAYLAND.md`).

## 3. Adblock

- Watch the logs: `page-level ad request blocked: ...` during navigation and playback.
- Open devtools (enabled by default): requests to hosts in
  `assets/filters/spotify-ads.txt` (e.g. `google-analytics.com`,
  `spclient.wg.spotify.com/ads...`) are missing or fail simulated
  (`net::ERR_BLOCKED_BY_CLIENT`).
- Playback is not interrupted: no legitimate endpoint
  (`/metadata`, audio playback) must be blocked.
- Known edge case: `@@` rules only apply to Rust navigations,
  the JS hook may overblock — report it with the exact URL.

## 4. Audio routing

- Open `pavucontrol` during playback: the app shows up as
  `spotify-minimal`.
- Switch its output to another device: sound follows, independently of the
  usual browser.
- On Windows (dev only): the mixer shows `spotify-minimal` with the green
  icon. If the mixer is already open at launch, close and reopen it to see
  the new icon; the name updates immediately.
