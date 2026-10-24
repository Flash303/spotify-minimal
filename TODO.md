# Tasks

## To do

- [ ] Debian/Wayland validation: build, DRM/Widevine, `pavucontrol` audio
  routing (protocols in `docs/DEBIAN_WAYLAND.md` and `docs/MANUAL_TESTS.md`).

## Done

- [x] PoC: wry webview + Rust adblock + `--plan-b` fallback.
- [x] Basic manual Windows tests (rendering, playback).
- [x] Filters expanded: ~25 third-party ad-tech domains (TradeDesk, Criteo,
  Outbrain/Taboola, Comscore, Moat, IAS, SSPs/DMPs) as `||domain^$third-party`;
  path-scoped Spotify rules (`/ads`, `/ad-logic`) unchanged.
- [x] Windows mixer renamed via `mixer.rs`
  (`SetDisplayName`/`SetIconPath` on descendant `msedgewebview2.exe` sessions)
  + embedded icon (`assets/icon.ico`, `icon.png`, `.rc`).
  Wave Link excluded: it routes by process (`msedgewebview2`), not by session —
  verified on-machine, no app-side lever.
- [x] Windows NSIS per-user installer (`packaging/windows/spotify-minimal.nsi`,
  shortcuts + uninstaller, no UAC). Linux `.deb` built by CI.
