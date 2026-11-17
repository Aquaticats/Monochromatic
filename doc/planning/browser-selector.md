# Browser selector

The implementation is in `package/browser-selector/desktop-app`. The package builds and passes its tests.

## First release scope 2026-11-17T16:35:12-05:00

Build a minimal Slint browser and profile picker for Linux and verify it on KDE.
Use the repository's Rust and Slint build pattern and system GIO for desktop application integration.
The user selects the installed chooser as the default web browser for HTTP and HTTPS in KDE.
Set `text/html` and `application/xhtml+xml` for the chooser in KDE's File Associations settings.
Show the picker for every open request, including when there is only one eligible browser.

The desktop entry should advertise `x-scheme-handler/http`, `x-scheme-handler/https`, `text/html`,
and `application/xhtml+xml`. The last association includes `.xhtml` and `.xht` files.

## Desktop evidence 2026-11-17T16:35:12-05:00

On the KDE Wayland test machine, `gio mime` reported Firefox ESR as the HTTP and HTTPS default
and Firefox ESR Safe Mode as the HTML default. The file association list also included editors.
The chooser needs browser discovery even when the input is a local file.
The browser setting in Plasma 6.8.3 writes the two URL scheme associations.
[Plasma default browser component][plasma]

Codex probed ordinary HTML and XHTML content under both `.html` and `.htm` in a disposable fixture.
`xdg-mime query filetype`, `gio info`, and `file --mime-type` returned `text/html` for ordinary HTML
and `application/xhtml+xml` for XHTML. KDE verification will exercise both kinds of file associations
and both URL schemes in a disposable profile.

The [terminal's Slint build](../../package/terminal/app/build.rs) provides the markup compilation pattern.
The [file manager's Cargo lockfile](../../package/file-manager/app/Cargo.lock) records the `gio` dependency.
`pkg-config --modversion gio-2.0 gio-unix-2.0` reported `2.90.1` for both on the test machine.

## Browser and profile choices 2026-11-17T16:40:02-05:00

Discover installed browsers through GIO. Show a plain choice for each browser and separate choices
for named profiles discovered in Firefox, Firefox ESR, Firefox Developer Edition, and Helium.
The plain choice uses the browser's registered desktop entry. A named choice opens the same target
with explicit profile arguments.

Firefox provides legacy profile names and paths in `profiles.ini`. Its current profile manager
also stores named profiles in SQLite group databases. Include both kinds of profiles in the picker.
Use system SQLite in read-only mode for profile names and paths. `pkg-config --modversion sqlite3`
reported `3.54.1` on the test machine. The installed Firefox variants have distinct remoting names
and can share a profile inventory. Use evidence from Firefox's per-install defaults and profile
compatibility metadata to determine which Firefox can open a profile.
[Mozilla profile architecture](https://firefox-source-docs.mozilla.org/toolkit/profile/index.html)

All three installed Firefox variants accept `--profile PATH --new-tab URI` according to
`--help` in a disposable home. Codex checked each installed revision's startup code for
path-based forwarding to an already running instance.

Helium uses Chromium's profile storage. The installed wrapper forwards arguments to Helium.
In a disposable headless run, Helium opened an HTML file whose name contained spaces and `#`.
It stored the requested profile directory in `Local State` under `profile.info_cache`.
[Chromium user data directory](https://chromium.googlesource.com/chromium/src/+/HEAD/docs/user_data_dir.md)

## Picker behavior 2026-11-17T16:40:02-05:00

Use one window with the incoming target and a list of browser and profile choices.
Open the selected target on click or Enter. Support the arrow keys for selection and Escape to close.
Show an error when there are no available browsers, or when the selected launch fails.
Use the desktop entry's `%u` argument for a single URL or file per invocation.

The package will live at `package/browser-selector/desktop-app` and follow the existing desktop packages'
`mise` tasks. Install its binary and desktop entry under the user's local install directories.
The desktop environment owns default application changes. Test registration and removal in a
disposable KDE profile.

## Verification before delivery 2026-11-17T16:44:59-05:00

Check that each selected browser receives the exact target in the selected profile.
Use disposable profiles to verify both Firefox storage formats and Helium with cold and running
browser instances. Run the Slint picker in a nested desktop to exercise keyboard and pointer input.
Use separate fixtures for the default applications so verification preserves the desktop's active
associations.

[plasma]: https://github.com/KDE/plasma-workspace/blob/v6.8.3/kcms/componentchooser/components/componentchooserbrowser.cpp

## Cargo trust approval 2026-11-17T17:33:49-05:00

The user approved exact-version trust exceptions after `cargo-shadow` refused the new lockfile.
All five versions were already present in `package/terminal/app/Cargo.lock`.
The published source files and original manifests matched the Git blobs at the commits recorded by Cargo.
The package manifests generated by Cargo and generated lockfiles were outside the source comparison.

- `dlib@0.5.3` changed publisher. Its source matches the upstream version bump. [Release commit][dlib]
- `icu_locale_fallback@2.4.0` and `icu_locale_fallback_data@2.4.0` changed publisher.
  Both match a verified commit in the Unicode ICU4X repository. [Release source][icu]
- `tiny-skia@0.12.0` and `tiny-skia-path@0.12.0` changed repository URLs. GitHub redirects the prior
  URL to the Linebender repository, and both packages match the version tag. [Release source][tiny-skia]

[dlib]: https://github.com/elinorbgr/dlib/commit/ab1a79fd8b92a331a5f123be9e1dcbbcbd48dbf8
[icu]: https://github.com/unicode-org/icu4x/commit/031d9e89f6a3fd26a14491e090729d46094328fd
[tiny-skia]: https://github.com/linebender/tiny-skia/tree/v0.12.0

## Verification results 2026-11-17T17:43:00-05:00

Codex ran the package's `lint`, `test`, `build`, `install`, and `uninstall` tasks under the repository toolchain.
All four nextest cases passed. File Enforcer found no generated-file drift.
On a nested Wayland session hosted by KDE, the installed release showed the chooser in light and dark,
launched the exact escaped URL with the selected Firefox profile, handed a local HTML path through GIO,
and closed on Escape without dispatch. `gio mime` set and restored all four defaults in disposable XDG data.
Activation in KDE System Settings remains a user action.

Firefox, Firefox ESR, Firefox Developer Edition, and Helium opened the cold and warm loopback HTTP
targets in the selected disposable profiles. Developer Edition's default fixture failed with
`Failed to launch tab subprocess @FSC::SFNS::Recv (Error:0)`. The passing run set
`dom.ipc.forkserver.enable` to `false` in the fixture profile and waited for startup before choosing the
second target. Some content subprocesses still exited on signal 11 during the passing run.
The selector does not change browser preferences.
