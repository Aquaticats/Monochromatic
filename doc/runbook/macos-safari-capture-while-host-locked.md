# Capture macOS Safari and iPhone Simulator Safari while `m1` is locked

Visual checks of web pages in Apple browsers run on `m1` (Apple M1, 16 GiB RAM, macOS 27).
Keep-awake tools on the host (Vorssaint, `caffeinate`, pointer nudging) have proven unreliable,
so every capture path here must keep working while the host's login session is locked.
Last verified 2026-10-18.
To rebuild the whole rig from a wiped `m1`, follow [`m1-apple-browser-rig-rebuild.md`](m1-apple-browser-rig-rebuild.md);
it embeds every file and command that otherwise lives only on `m1`.

## What fails while the host is locked

On `m1`, `ioreg -n Root -d1 -a | grep -A1 CGSSessionScreenIsLocked` prints `<true/>` while locked;
the key is absent while unlocked.

- Safari (the app) stops painting: the page reports `document.visibilityState` as `hidden`.
- Safari's WebDriver (`/usr/bin/safaridriver`) still creates sessions, navigates and runs scripts,
  but `GET /session/{id}/screenshot` returns a blank dark frame of the right size.
- The Safari Technology Preview MCP (`safari-mcp-stp`) behaves the same: scripts run, screenshots are black.
- Published experience agrees:
  Safari has no headless mode,
  and its WebDriver needs an unlocked window server
  (for example [caramel issue 61][caramel-61]).

## iPhone Simulator: works while locked

`xcrun simctl io <udid> screenshot <file>` reads the simulated device's own frame buffer,
so it captures correctly with the host locked.
Drive navigation with `xcrun simctl openurl <udid> <url>`.

## macOS Safari: real Safari inside a UTM macOS VM

A guest macOS session has its own window server that does not lock when the host locks,
so real Safari inside the guest keeps painting.

Setup on `m1` as of 2026-10-18:

- VM bundle: `/Volumes/MacData/utm/macOS.utm` (UTM, Apple Virtualization backend, 8 GiB RAM),
  guest macOS 27.0 with Safari 27.0, the same versions as the host.
- Guest display: 1920x1200 at 80 PPI, which is a 1x (non-Retina) display;
  this matches a desktop screen at device pixel ratio 1.
- Guest address: `192.168.64.2` on UTM's shared network (`bridge100`); host side is `192.168.64.1`.
- Login: user `user`; SSH key `~/.ssh/id_ed25519_vm` on `m1` is in the guest's `authorized_keys`,
  so reach it as `ssh m1` then `ssh -i ~/.ssh/id_ed25519_vm user@192.168.64.2`.
  No entry on the Linux side is needed.
- `utmctl` does not work over SSH (`OSStatus error -1743`, "does not work from SSH sessions");
  start or stop the VM from the UTM window on the host.
  UTM Server (UTM 5.0.5 on `m1`) cannot help:
  [its docs][utm-server] say "Currently, only QEMU backend virtual machines are supported",
  and this VM uses the Apple backend.
  The `utm://start` URL command was removed in UTM 4.5.1 ([remote control docs][utm-remote]).
  `utmctl` does work from a LaunchAgent in the GUI session, with no permission prompt:
  `~/Library/LaunchAgents/dev.monochromatic.utm-autostart.plist` on `m1` runs
  `/Applications/UTM.app/Contents/MacOS/utmctl start macOS` at login (`RunAtLoad`, no `KeepAlive`).
  Loading it while the VM ran returned `OSStatus error -2700` / `Operation not available`
  (UTM declining to start a running VM, not a permission error),
  and a probe agent running `utmctl list` printed the VMs (`macOS` `started`).
  To run any `utmctl` command from SSH, wrap it in a one-shot agent the same way
  (`launchctl bootstrap gui/$(id -u) <plist>`, read its log, then `launchctl bootout`).

Guest settings that keep it rendering, applied with `sudo` in the guest:

```bash
# in the guest, over SSH
sudo pmset -a sleep 0 displaysleep 0 disksleep 0 powernap 0
sysadminctl -screenLock off -password '<password>'
defaults -currentHost write com.apple.screensaver idleTime -int 0
sudo /usr/bin/safaridriver --enable
```

Auto-login:
`sudo sysadminctl -autologin set -userName user -password '<password>'` failed with
`SACSetAutoLoginPassword error:22` even with FileVault off.
Writing `/etc/kcpassword` directly works
(the password XORed with the fixed key `7d 89 52 23 d2 bc dd ea a3 b9 1f`, cycled,
after zero-padding the password to a multiple of 12 bytes; mode 600, owner `root:wheel`)
plus `sudo defaults write /Library/Preferences/com.apple.loginwindow autoLoginUser user`.
Verified 2026-10-18 by `sudo shutdown -r now`: the guest came back logged in at the console, unlocked.

Keep a reboot from reopening a normal Safari window, which blocks WebDriver
(`POST /session` then times out with `The session timed out while connecting to a Safari instance`):

```bash
# in the guest, over SSH
defaults write com.apple.loginwindow TALLogoutSavesState -bool false
defaults write com.apple.loginwindow LoginwindowLaunchesRelaunchApps -bool false
defaults write com.apple.Safari NSQuitAlwaysKeepsWindows -bool false
```

Keep updates from restarting the guest (`softwareupdate --schedule off` still reports checking on in macOS 27,
so set the preferences):

```bash
# in the guest, over SSH
sudo defaults write /Library/Preferences/com.apple.SoftwareUpdate AutomaticallyInstallMacOSUpdates -bool false
sudo defaults write /Library/Preferences/com.apple.SoftwareUpdate AutomaticDownload -bool false
sudo defaults write /Library/Preferences/com.apple.SoftwareUpdate CriticalUpdateInstall -bool false
sudo defaults write /Library/Preferences/com.apple.SoftwareUpdate AutomaticCheckEnabled -bool false
sudo defaults write /Library/Preferences/com.apple.commerce AutoUpdate -bool false
```

Over SSH without a TTY,
`sudo -n` does not reuse an earlier `sudo -S` in a separate command;
pipe the password to every `sudo -S -p ""` call.

### WebDriver in the guest: a login LaunchAgent

`~/Library/LaunchAgents/dev.monochromatic.safaridriver.plist` in the guest runs
`/usr/bin/safaridriver --port 4445` with `RunAtLoad` and `KeepAlive`,
in the logged-in GUI session; after the reboot test it was up and `ready` with no manual step.

If you start the driver by hand instead,
use `nohup ... </dev/null &`;
a bare `cmd &` inside `ssh host '...'` dies when the SSH command returns,
and the next request fails with curl exit 7 (connection refused).

Never run `/Applications/Safari.app/Contents/MacOS/Safari` directly (for example with `--version`)
to read the version: it starts a Safari process that never exits and blocks every later `POST /session`.
Read the version with `defaults read /Applications/Safari.app/Contents/Info CFBundleShortVersionString`.
If a session times out, quit any normal Safari first (`osascript -e 'quit app "Safari"'`).
After `Safari instance is already paired with another WebDriver session`,
`DELETE /session/<id>` the stale session.

### Tunnels: m1 launchd agent plus a local systemd unit

`~/Library/LaunchAgents/dev.monochromatic.vm-tunnel.plist` on `m1` keeps one SSH connection to the guest
(`KeepAlive`, `ThrottleInterval` 10, `ServerAliveInterval` 15) carrying both forwards:

- `-R 4173:127.0.0.1:4173`: the guest's `localhost:4173` is m1's preview server, so pages keep the same origin.
- `-L 4447:127.0.0.1:4445`: m1's port 4447 is the guest's WebDriver
  (not 4445, which m1's own `safaridriver` uses).

It reconnects by itself after a guest reboot.
On the Linux side:

```bash
# /var/home/user/Monochromatic
systemd-run --user --unit=xinyanchen-vm-webdriver --collect --property=MemoryMax=64M \
  --property=Restart=always --property=RestartSec=10 \
  ssh -o BatchMode=yes -o ExitOnForwardFailure=yes -o ServerAliveInterval=15 -N -L 4446:127.0.0.1:4447 m1
```

Then plain WebDriver HTTP against `http://127.0.0.1:4446` drives real Safari in the guest.
The guest has no `python3` (calling it pops an Xcode command line tools install dialog),
so decode screenshots on the Linux side.

### Verified while the host was locked

On 2026-10-18 `m1` was locked with `pmset displaysleepnow`
(lock flag `<true/>` confirmed before and after).
Guest Safari reported `document.visibilityState` `visible`,
and its WebDriver screenshot of a page was pixel-identical (ImageMagick AE 0 at 2% fuzz)
to the same capture with the host unlocked;
after the guest reboot, all six pages of a site captured correctly with the host still locked.

### Display scale

The guest display is 1920x1200 with UTM's `DynamicResolution` on,
which resizes the guest mode to follow the UTM window
(it was found at 1043x652 scaled, with a 1920-wide Safari window hanging off-screen).
Pin the mode before each capture with `displayplacer` 1.4.0
(arm64 release binary from `jakehilborn/displayplacer`, installed at `/usr/local/bin/displayplacer` in the guest):

```bash
# in the guest, over SSH
/usr/local/bin/displayplacer "id:35B36D67-6BAC-47B7-98B5-07877849011C mode:24 origin:(0,0) degree:0"
```

- Mode 24, 1920x1200 unscaled: `devicePixelRatio` 1, like a 1080p screen at 1x.
- Mode 34, 1920x1200 HiDPI: `devicePixelRatio` 2, like a 4K screen at 2x.

Either way a 1920x1080 Safari window gives a 1920x1028 viewport;
screenshots come out 1920x1028 and 3840x2056 device pixels.
`displayplacer list` shows the display id and modes.

### Capture script

A Node script that pins the mode,
opens a WebDriver session through local port 4446,
sizes the window,
and saves one PNG per page and scroll offset:
`/home/user/temp/agent/xinyanchen-build/vm-safari-shot.mjs`
(`node vm-safari-shot.mjs <out-dir> 1920x1080@1 index.html@0 about.html@1500`).
It uses only WebDriver's standard endpoints
(`POST /session`, `POST /session/{id}/window/rect`, `POST /session/{id}/url`,
`POST /session/{id}/execute/sync`, `GET /session/{id}/screenshot`, `DELETE /session/{id}`),
so a copy works for any site served on the forwarded port.

## macOS WebKit fallback: `webkit-capture` on the host

`/Volumes/MacData/xinyanchen-capture/webkit-capture` on `m1` is a small Swift tool:
an off-screen `WKWebView` (the system WebKit that Safari uses) with occlusion detection disabled
through the private `_setWindowOcclusionDetectionEnabled:`,
Safari's user agent,
and `takeSnapshot` output at the host's 2x scale.
It renders while the host is locked (page reports `visible`).

```bash
# on m1
webkit-capture <url> <width> <height> <out.png> [script] [settle-ms] [full]
```

Positive control on 2026-10-18:
`filter`,
gradient and SVG `mask-image`,
`transform: scaleX(-1)` and `box-shadow` rendered as in Chromium;
`backdrop-filter` did not render in its snapshot,
and a 3D `rotateY` came out slightly different.
It is Safari's engine,
not the Safari app:
no toolbar,
Safari preference defaults,
or content blockers.
Prefer the VM;
use this when the VM is down.

[caramel-61]: https://github.com/caramelizedev/caramel/issues/61
[utm-server]: https://docs.getutm.app/remote/server/
[utm-remote]: https://docs.getutm.app/advanced/remote-control/
