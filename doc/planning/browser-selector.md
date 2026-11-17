# Browser selector

Proposal awaiting design confirmation.

## First release scope 2026-11-17T16:35:12-05:00

Build a minimal Slint browser and profile picker for Linux and verify it on KDE.
Use the repository's Rust and Slint build pattern and system GIO for desktop application integration.
The user selects the installed chooser in KDE's default application and file association settings.
Show the picker for every open request, including when there is only one eligible browser.

The desktop entry should advertise `x-scheme-handler/http`, `x-scheme-handler/https`, `text/html`,
and `application/xhtml+xml`. The last association includes `.xhtml` and `.xht` files.

## Desktop evidence 2026-11-17T16:35:12-05:00

On the KDE Wayland test machine, `gio mime` reported Firefox ESR as the HTTP and HTTPS default
and Firefox ESR Safe Mode as the HTML default. The file association list also included editors.
The chooser needs browser discovery even when the input is a local file.

Codex probed ordinary HTML and XHTML content under both `.html` and `.htm` in a disposable fixture.
`xdg-mime query filetype`, `gio info`, and `file --mime-type` returned `text/html` for ordinary HTML
and `application/xhtml+xml` for XHTML. KDE verification will exercise both kinds of file associations
and both URL schemes in a disposable profile.

The [terminal's Slint build](../../package/terminal/app/build.rs) provides the markup compilation pattern.
The [file manager's Cargo lockfile](../../package/file-manager/app/Cargo.lock) records the `gio` dependency.
`pkg-config --modversion gio-2.0 gio-unix-2.0` reported `2.90.1` for both on the test machine.

## Open choices 2026-11-17T16:35:12-05:00

Choose how profiles enter the picker and which browser installations to verify.
Use the resulting requirements to define the selection and launch behavior.
