# Browser Selector

A Slint picker for links and local web pages on Linux. It opens one window per request.
Click a browser or profile to open the target. Use the arrow keys and Enter to choose with the
keyboard, or Escape to close.

## Install and set the defaults

Builds use the system GIO 2.66 or newer and SQLite development libraries, Rust, and `pkg-config`.
Installation uses `update-desktop-database` from `desktop-file-utils`.

```sh
# From the repository root.
mise run //package/browser-selector/desktop-app:install
```

In KDE System Settings, select **Browser Selector** as the default web browser to route HTTP and
HTTPS. In **File Associations**, select **Browser Selector** for `text/html` and
`application/xhtml+xml` to route ordinary `.html` and `.htm` files as well as XHTML.

The install task copies the executable to `~/.local/bin/monochromatic-browser-selector` and
registers `monochromatic.browser-selector.desktop` under the XDG applications directory.
The executable accepts one HTTP URL, HTTPS URL, native file URI, or local path.

## Browser profiles

Plain browser entries use the launchers registered with the desktop. Automatic profile discovery
covers the native Firefox, Firefox ESR, Firefox Developer Edition, and Helium desktop launchers.
Profile choices use standard native desktop commands with a single URL field. Firefox profiles come
from `profiles.ini` and read-only SQLite group databases, including groups outside the legacy index.
When several Firefox channels are installed, a profile's compatibility metadata or its group's
last-used installation determines which browser can open it. Helium profiles come from `Local State`.
For Helium wrappers, the picker keeps the launcher's fixed arguments before the profile flags and target.

## Uninstall

Choose replacement defaults in KDE System Settings, then remove the installed files.

```sh
# From the repository root.
mise run //package/browser-selector/desktop-app:uninstall
```
