# Electron counter

A minimal Electron app proving this repo can ship a no-Vite,
 fully ESM and TypeScript desktop package.

## What it demonstrates

- Electron main process built as native ESM.
- Browser renderer built from TypeScript and loaded as `<script type="module">`.
- No preload script,
   no `nodeIntegration`,
   and renderer sandboxing stays enabled.
- Linux launch forces Chromium's Ozone Wayland backend and the automated test unsets `DISPLAY`,
  so an XWayland fallback cannot make the test pass.
- Distribution bundles are generated with shared
   `@monochromatic-dev/electron-infra-core` helpers around `@electron/packager`,
   not Vite or Forge.

## Tasks

- `mise run //package/electron-infra/app-counter:build` builds the staged Electron app.
- `mise run //package/electron-infra/app-counter:test` runs unit tests plus the nested Wayland boundary test.
- `mise run //package/electron-infra/app-counter:distribute` builds Linux,
   Windows,
   and macOS bundles for x64 and arm64.
- `mise run //package/electron-infra/app-counter:distribute:dry-run`
   prints the exact distribution target matrix without downloading Electron zips.

## Pure Wayland verification

The Wayland boundary test uses shared `@monochromatic-dev/electron-infra-core` helpers to run the app inside
`package/nested-wayland-session/cli` instead of an external compositor such as cage or niri.
The hosted Electron command is launched through `/usr/bin/env --unset=DISPLAY`,
 with `XDG_SESSION_TYPE=wayland`.
The app must draw into the nested Wayland compositor,
 accept synthetic keyboard input,
 update the counter,
 and write the observed state file.

## Distribution target matrix

- Linux x64
- Linux arm64
- Windows x64
- Windows arm64
- macOS x64
- macOS arm64

These are unpacked platform-native Electron bundles.
 Signing and notarization are intentionally not done in this sample package.
