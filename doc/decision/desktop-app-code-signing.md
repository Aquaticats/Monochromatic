# Desktop app code signing: build natively, sign on Linux, organization identity

## Decision

`package/music-player/desktop-app` is distributed as a signed macOS `.app` and a signed Windows `.exe`.
 Each binary
is built on its own operating system,
 and the bundling,
 signing,
 and notarization are centralized on the Linux
primary box:

- macOS:
   the Mach-O is built on the Apple Silicon Mac (`ssh m1`),
   then bundled into `Music Player.app`,
   signed with
  a Developer ID under the hardened runtime,
   submitted to Apple's notary service,
   and stapled,
   all from Linux using
  `rcodesign` (the `apple-codesign` crate).
- Windows:
   the `.exe` is built on the Windows box (`ssh x13-win`),
   then Authenticode-signed with an RFC3161
  timestamp from Linux using `osslsigncode` inside `Containerfile.sign`.

The signing identity is a Canadian-registered organization,
 not the maintainer as an individual,
 so the maintainer's
legal name stays out of the public signature.
 Real credentials are acquired later (see the two
`HANDOVER.*-signing-credentials.md` runbooks);
 until then a throwaway self-signed identity proves the pipeline
mechanics end to end.
 Scope is an app bundle plus a signed exe;
 installers (`.dmg`,
 MSI) are deferred.

## Why build natively but sign centrally

Cross-compiling a Slint + cpal GUI from Linux (osxcross or cargo-zigbuild plus the Apple SDK,
 linking CoreAudio and
AppKit,
 or the Windows SDK) is fragile and out of scope;
 both native builds already work on their own machines.
Signing,
 by contrast,
 is the cross-platform-friendly half:
 `rcodesign` rewrites a code signature into an
already-built Mach-O and talks to Apple's notary service from Linux,
 and `osslsigncode` signs a PE from Linux.
Centralizing signing on the Linux box keeps one signing host,
 one place for the credentials,
 and matches how the
maintainer already drives the macOS and Windows work over SSH.

## macOS specifics

- The `.app` is assembled by `bundle:macos` from `macos/Info.plist` (a template whose version is filled from
  `Cargo.toml`) and the committed `assets/music-player.icns`.
- `sign:macos` runs `rcodesign sign --for-notarization` (hardened runtime plus a secure timestamp) with
  `macos/entitlements.plist` (intentionally empty:
   the app only plays audio).
- Notarization needs an Apple Developer Program Organization membership (99 USD/year,
   requires a D-U-N-S Number for
  the entity) and an App Store Connect API key.
   For Developer ID distribution the bundle id need not be
  pre-registered with Apple.

## Windows specifics

- `sign:windows` runs `osslsigncode sign` with an RFC3161 timestamp so the signature outlives the certificate.
- Since 2023-06-01,
   OV and EV code-signing keys must live in certified hardware or a cloud HSM and are
  non-exportable,
   so a plain `.pfx` is no longer issued for real certificates.
   The chosen path is a cloud key driven
  from Linux over PKCS#11.
   See
  [windows-code-signing-hardware-key-requirement](../troubleshooting/windows-code-signing-hardware-key-requirement.md).
- The certificate is organization-validated,
   so its subject is the org's legal name.
- `build.rs` embeds `assets/music-player.ico` into the `.exe` via `winresource`,
   gated to `cfg(windows)` so Linux
  and macOS builds are untouched.

## Organization identity for name privacy

A code signature publishes its subject.
 An individual Apple Developer ID makes Gatekeeper show
`Developer ID Application: <person>`,
 and an individual Authenticode certificate shows that person in the Windows
publisher prompt and the file's Digital Signatures tab.
 Signing under a Canadian organization puts the org's name
there instead.
 This is why the cheaper individual options are rejected:
 Apple Individual enrollment,
 and Certum's
individual Open Source Code Signing certificate (about 58 USD/year),
 both of which would expose the maintainer's
legal name.
 The organization route costs more;
 the budget was relaxed for the privacy benefit,
 and payment is
deferred.

## Tooling

- `rcodesign` is a repo-wide mise tool (`github:indygreg/apple-platform-rs`);
   the Linux build needs no Mac.
- `osslsigncode`,
   `opensc`,
   and `openssl` live in `Containerfile.sign` (CachyOS; `osslsigncode` is in no Arch or CachyOS repository,
   so the image compiles the latest upstream release from source);
   the proprietary cloud-key PKCS#11 module is
  added per the Windows runbook.
- Icons are generated once from `assets/icon.svg` by `gen:icons` in `Containerfile.icons` (`rsvg-convert`,
  `png2icns`,
   `icotool`);
   the `.icns` and `.ico` are committed.
- mise tasks:
   `gen:icons`,
   `bundle:macos`,
   `sign:macos`,
   `notarize:macos`,
   `sign:windows`,
   and the `verify:signing`
  self-signed umbrella.
   Each task reads real credentials from env vars and falls back to a self-signed identity (with
  a neutral placeholder subject) when they are unset.

## Alternatives rejected

- Native `codesign` plus `notarytool` on the Mac and `signtool` on Windows:
   the vendor-official tools,
   but they
  spread the pipeline across three machines and require Xcode and a keychain on the Mac and the Windows SDK on the
  PC.
   `rcodesign` and `osslsigncode` centralize everything on the Linux box.
- Azure Trusted Signing:
   its tooling is Windows-only (not drivable from plain Linux),
   and onboarding is restricted
  (organizations need a multi-year history,
   individuals are limited to the US and Canada).
- Individual certificates of any kind:
   they expose the maintainer's legal name.
- Cross-compiling the macOS or Windows binary from Linux:
   fragile SDK and framework linking,
   out of scope.
- Installers (`.dmg`,
   MSI,
   NSIS):
   deferred until the signing chain is proven.
   Update 2026-10-14: unsigned `.dmg` images and standalone `.exe` files are now released by CI without this chain;
   see `doc/decision/music-player-releases.md`.

## Sources

- apple-codesign documentation,
   signing and notarizing from non-Apple operating systems:
  <https://gregoryszorc.com/docs/apple-codesign/stable/>
- Apple Developer Program membership comparison and enrollment:
  <https://developer.apple.com/support/compare-memberships/> and <https://developer.apple.com/programs/enroll/>
- osslsigncode:
   <https://github.com/mtrojnar/osslsigncode>
- The 2023-06-01 code-signing key-storage requirement is traced in
  [windows-code-signing-hardware-key-requirement](../troubleshooting/windows-code-signing-hardware-key-requirement.md).
