# Firefox ESR desktop icon

## Installed artwork at 2026-11-03T02:59:20-05:00

The user requested the supplied Firefox 2017 SVG everywhere Firefox ESR's application icon appears.
The downloaded artwork is installed at
`~/.local/share/icons/hicolor/scalable/apps/firefox-esr.svg`.
Its SHA-256 is `6795eab92a70e063f3d8a2d4d463b6cb4fae860857e54055a3c8bc15d01f447f`.
The source is the [user-supplied Wikimedia SVG][artwork].

Both local desktop entries now use the absolute path to that SVG:

- `~/.local/share/applications/firefox-esr.desktop`,
  used by KDE's launcher and pinned taskbar entry.
- `~/.local/share/applications/userapp-Firefox-42E8S3.desktop`,
  an existing hidden entry whose executable also points to ESR.

The PNG files in `~/.local/opt/firefox-esr/browser/chrome/icons/default/`
were rendered from the same SVG at their existing sizes:
`16`,
`32`,
`48`,
`64`,
and `128` pixels.
Each PNG preserves the artwork's aspect ratio with transparent padding to a square.
The SVG remains outside the browser installation;
a Firefox update may replace the PNG files inside its installation.

This is application-icon customization.
Firefox's embedded page branding and generic updater symbol were not edited.
The separate Nightly installation and the browser profiles were not edited.

## Verification

`desktop-file-validate` accepted both launchers.
It reported the pre-existing warning
`key "Encoding" in group "Desktop Entry" is deprecated`
for `userapp-Firefox-42E8S3.desktop`.

`kbuildsycoca6 --noincremental` rebuilt KDE's application cache.
The session received `org.kde.KIconLoader.iconChanged` with group `0` at `/KIconLoader`.
KRunner's rendered search result for `Firefox ESR`
was inspected and displayed the supplied orange fox and blue globe.
The search popup was closed after inspection.

`pgrep --list-full firefox` found no Firefox process during verification.
An existing ESR window therefore was not available for taskbar or switcher verification.
The temporary KWin inspection script was unloaded.

The user's subsequent launcher screenshot also shows the requested artwork beside `Firefox ESR`.
A fresh download from the exact supplied URL matched the installed SVG's SHA-256.
Visual comparison with the backed-up PNG confirms the change:
the original icon has a purple center,
while the replacement has a blue globe and the fox's extended snout.
The handover was rendered through Pandoc's CommonMark reader and its HTML output inspected.

## Backups and continuation

Original launcher and PNG files,
their installed paths,
and replacement hashes are recorded under
`~/.local/state/firefox-esr-icon/2026-11-03T07-54-27.307Z/`.
`manifest.json` identifies each original file;
the SVG was newly created and has no previous copy.

The scratch installer and rendering evidence live under `~/temp/agent/firefox-esr-icon/`.
Full desktop captures contain unrelated private screen content and must not be published.
There is no pending install or refresh operation.

[artwork]: https://upload.wikimedia.org/wikipedia/commons/2/28/Firefox_logo%2C_2017.svg
