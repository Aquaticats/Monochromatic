# Rebuild the m1 Apple browser test rig from nothing

Purpose: if `m1` (Apple M1 Mac, 16 GiB RAM) is wiped, rebuild every piece that lets agents on the Linux
workstation see web pages in real Apple browsers, top to bottom, from this file alone.
Why each piece exists, and the pitfalls found while building it:
[`macos-safari-capture-while-host-locked.md`](macos-safari-capture-while-host-locked.md).
Recorded 2026-10-18 from the working rig.

The rig has three capture paths, each usable while `m1`'s screen is locked:

- iPhone 15 Simulator Safari: screenshots through `simctl`, real touch input through `idb`.
- Real macOS Safari inside a UTM macOS VM, driven through WebDriver.
- `webkit-capture`, an off-screen `WKWebView` on the host, as a fallback when the VM is down.

The VM also serves a second, concurrent WebDriver endpoint for Safari Technology Preview (Linux port 4450).

Never commit passwords here (this repository is public).
The VM guest's login password is the one the user chose when creating the guest.

## What survives an internal-disk wipe

`MacData` is an external 512 GB USB SSD mounted at `/Volumes/MacData`.
It holds the VM bundle (`/Volumes/MacData/utm/macOS.utm`, about 35 GB) and the simulator device-set image
(`/Volumes/MacData/CoreSimulator/xinyanchen-sim-devices.sparsebundle`), so after a wipe of `m1` alone,
skip "Create the macOS VM" and "Create the simulator device set" and only re-attach them.
Write-heavy work belongs on `MacData`: `m1`'s internal SSD is fragile.

## 1. Host basics on m1

1.  Install macOS 27 (the rig was verified on macOS 27.0, Safari 27.0), create user `user`,
    and enable System Settings, General, Sharing, Remote Login.
2.  From the Linux workstation, install its key (the Linux side's `~/.ssh/config` has `Host m1`,
    `HostName user-m1.local`, `User user`, `IdentityFile ~/.ssh/user-m1`):

    ```bash
    # Linux workstation
    ssh-copy-id -i ~/.ssh/user-m1.pub m1
    ```

3.  Install Homebrew, Xcode 27 (verified with 27.0, build 27A266a), select it, and download the iOS runtime:

    ```bash
    # on m1
    sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer
    xcodebuild -downloadPlatform iOS
    xcrun simctl runtime list
    ```

    The working rig had `iOS 27.0 (24A434)`.
4.  Install UTM (5.0.5 was in use), and Vorssaint plus `caffeinate` if you want the host awake;
    nothing here depends on the host staying unlocked.
5.  Enable Safari's WebDriver on the host (needed only for the host's own Safari, which goes blank when locked):
    `sudo safaridriver --enable`.

Note that `~/Library/LaunchAgents/cat.aquati.screen-autolock.plist` on `m1` runs
`~/.local/bin/screen-autolock` every 60 seconds with `IDLE_SECONDS=1800`,
which locks the host after 30 minutes idle; the capture paths here do not care.

## 2. iPhone 15 Simulator in its own device set on MacData

### Create the simulator device set (only if MacData was lost)

```bash
# on m1
mkdir -p /Volumes/MacData/CoreSimulator
hdiutil create -type SPARSEBUNDLE -fs APFS -size 20g -volname XinyanSimDevices \
  /Volumes/MacData/CoreSimulator/xinyanchen-sim-devices.sparsebundle
hdiutil attach /Volumes/MacData/CoreSimulator/xinyanchen-sim-devices.sparsebundle
mkdir -p /Volumes/XinyanSimDevices/devices
xcrun simctl --set /Volumes/XinyanSimDevices/devices create "iPhone 15 client" \
  com.apple.CoreSimulator.SimDeviceType.iPhone-15 com.apple.CoreSimulator.SimRuntime.iOS-27-0
```

`create` prints the new UDID; the working rig's was `4E2C74A5-7001-4F13-82CD-CD7AA9AA2CA9`.
Update the UDID in the scripts in "Linux-side scripts" if it changes.

### Attach and boot (after every m1 reboot)

```bash
# on m1
hdiutil attach /Volumes/MacData/CoreSimulator/xinyanchen-sim-devices.sparsebundle
xcrun simctl --set /Volumes/XinyanSimDevices/devices boot 4E2C74A5-7001-4F13-82CD-CD7AA9AA2CA9
```

`simctl io <udid> screenshot` works with the host locked.
Pages reach the Linux preview server because a Linux-side unit forwards `m1`'s `localhost:4173` to it
("Linux-side units").

### Real touch input: idb

`safaridriver` cannot drive this simulator: it only searches the default device set
("Could not find any session hosts that match the requested capabilities").
Meta's `idb` can, because `idb_companion` takes `--device-set-path`:

```bash
# on m1
brew tap facebook/fb
HOMEBREW_NO_AUTO_UPDATE=1 brew install facebook/fb/idb-companion
/opt/homebrew/bin/python3 -m venv /Volumes/MacData/idb-venv
/Volumes/MacData/idb-venv/bin/pip install fb-idb
nohup /opt/homebrew/bin/idb_companion --udid 4E2C74A5-7001-4F13-82CD-CD7AA9AA2CA9 \
  --device-set-path /Volumes/XinyanSimDevices/devices --grpc-port 10882 </dev/null >/tmp/idb-companion.log 2>&1 &
/Volumes/MacData/idb-venv/bin/idb --companion localhost:10882 ui tap 38 155
```

`ui tap` takes screen points (393 by 852 on iPhone 15): page coordinates plus the status bar,
about 58 points in Safari with the bottom address bar.
`uv tool install fb-idb` failed on `m1` because `uv` there is a mise shim that errored; the venv avoids it.
Verified 2026-10-18: a tap at (38, 155) opened the portfolio's phone menu and a second tap closed it.

## 3. macOS VM with real Safari

### Create the macOS VM (only if MacData was lost)

In UTM: Create, Virtualize, macOS, let UTM download the current IPSW (the rig ran guest macOS 27.0),
8192 MB memory, default CPU count, a 64 GB or larger disk, and save it as `macOS` under `/Volumes/MacData/utm/`.
Display: 1920 by 1200 at 80 PPI (UTM's default "dynamic resolution" stays on).
In the guest, create user `user` and enable Remote Login.

The guest gets `192.168.64.2` on UTM's shared network; the host side is `192.168.64.1`.

### Host key into the guest

```bash
# on m1
ssh-keygen -q -t ed25519 -N "" -f ~/.ssh/id_ed25519_vm
ssh-copy-id -i ~/.ssh/id_ed25519_vm.pub user@192.168.64.2
```

### Guest settings that keep it rendering

Run in the guest over SSH, entering the guest password for each `sudo`:

```bash
# in the guest
sudo pmset -a sleep 0 displaysleep 0 disksleep 0 powernap 0
sysadminctl -screenLock off -password -
defaults -currentHost write com.apple.screensaver idleTime -int 0
sudo /usr/bin/safaridriver --enable
defaults write com.apple.loginwindow TALLogoutSavesState -bool false
defaults write com.apple.loginwindow LoginwindowLaunchesRelaunchApps -bool false
defaults write com.apple.Safari NSQuitAlwaysKeepsWindows -bool false
sudo defaults write /Library/Preferences/com.apple.SoftwareUpdate AutomaticallyInstallMacOSUpdates -bool false
sudo defaults write /Library/Preferences/com.apple.SoftwareUpdate AutomaticDownload -bool false
sudo defaults write /Library/Preferences/com.apple.SoftwareUpdate CriticalUpdateInstall -bool false
sudo defaults write /Library/Preferences/com.apple.SoftwareUpdate AutomaticCheckEnabled -bool false
sudo defaults write /Library/Preferences/com.apple.commerce AutoUpdate -bool false
```

Over SSH without a TTY, each `sudo` needs the password again (`sudo -n` does not reuse an earlier one).

### Guest auto-login

`sudo sysadminctl -autologin set ...` failed with `SACSetAutoLoginPassword error:22` even with FileVault off.
Write `/etc/kcpassword` instead: the password's bytes, zero-padded to a multiple of 12,
XORed with the cycled key `7d 89 52 23 d2 bc dd ea a3 b9 1f`.
This Node snippet writes the file; it reads the password from the terminal, so it never lands in shell history:

```js
// kcpassword.mjs: node kcpassword.mjs > kcpassword, then type the guest password and press Enter
import { createInterface } from 'node:readline/promises';

const rl = createInterface({ input: process.stdin, output: process.stderr, terminal: true });
const password = await rl.question('guest password: ');
rl.close();
const key = [0x7d, 0x89, 0x52, 0x23, 0xd2, 0xbc, 0xdd, 0xea, 0xa3, 0xb9, 0x1f];
const bytes = [...Buffer.from(password)];
while (bytes.length % 12) bytes.push(0);
process.stdout.write(Buffer.from(bytes.map((byte, index) => byte ^ key[index % key.length])));
```

```bash
# copy kcpassword into the guest's /tmp, then in the guest
sudo install -m 600 -o root -g wheel /tmp/kcpassword /etc/kcpassword && rm /tmp/kcpassword
sudo defaults write /Library/Preferences/com.apple.loginwindow autoLoginUser user
sudo shutdown -r now
```

After the reboot, `who` in the guest must show `user console` and `ioreg -n Root -d1 -a | grep -A1 CGSSessionScreenIsLocked`
must print nothing.

### Guest display scale tool

```bash
# Linux workstation: fetch the arm64 release and copy it to the guest through m1
gh release download v1.4.0 --repo jakehilborn/displayplacer --pattern displayplacer-apple-v140 --dir ~/temp/agent/dp
# in the guest
sudo install -d -m 755 /usr/local/bin && sudo install -m 755 /tmp/displayplacer /usr/local/bin/displayplacer
/usr/local/bin/displayplacer list
```

`displayplacer list` prints the display id; the rig's was `35B36D67-6BAC-47B7-98B5-07877849011C`,
with mode 24 (1920 by 1200 unscaled, 1x) and mode 34 (1920 by 1200 HiDPI, 2x).
Update `GUEST_DISPLAY` and `MODE_FOR_SCALE` in the scripts if they differ.

### Guest LaunchAgent: WebDriver at login

`~/Library/LaunchAgents/dev.monochromatic.safaridriver.plist` in the guest (create the folder first):

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>dev.monochromatic.safaridriver</string>
  <key>ProgramArguments</key>
  <array>
    <string>/usr/bin/safaridriver</string>
    <string>--port</string>
    <string>4445</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>StandardErrorPath</key>
  <string>/tmp/safari-driver.log</string>
</dict>
</plist>
```

### Host LaunchAgents: tunnel to the guest, and start the VM at login

`~/Library/LaunchAgents/dev.monochromatic.vm-tunnel.plist` on `m1` carries the guest's WebDriver to `m1`'s
port 4447 and `m1`'s preview port 4173 into the guest:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>dev.monochromatic.vm-tunnel</string>
  <key>ProgramArguments</key>
  <array>
    <string>/usr/bin/ssh</string>
    <string>-N</string>
    <string>-o</string><string>BatchMode=yes</string>
    <string>-o</string><string>ExitOnForwardFailure=yes</string>
    <string>-o</string><string>ServerAliveInterval=15</string>
    <string>-o</string><string>ServerAliveCountMax=3</string>
    <string>-i</string><string>/Users/user/.ssh/id_ed25519_vm</string>
    <string>-L</string><string>4447:127.0.0.1:4445</string>
    <string>-R</string><string>4173:127.0.0.1:4173</string>
    <string>user@192.168.64.2</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>ThrottleInterval</key>
  <integer>10</integer>
  <key>StandardErrorPath</key>
  <string>/tmp/vm-tunnel.log</string>
</dict>
</plist>
```

`~/Library/LaunchAgents/dev.monochromatic.utm-autostart.plist` on `m1` starts the VM at login
(`utmctl` fails from SSH with `OSStatus error -1743` but works from a GUI-session agent):

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>dev.monochromatic.utm-autostart</string>
  <key>ProgramArguments</key>
  <array>
    <string>/Applications/UTM.app/Contents/MacOS/utmctl</string>
    <string>start</string>
    <string>macOS</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>StandardOutPath</key>
  <string>/tmp/utm-autostart.log</string>
  <key>StandardErrorPath</key>
  <string>/tmp/utm-autostart.log</string>
</dict>
</plist>
```

Load each new agent once with `launchctl bootstrap gui/$(id -u) <plist>`; afterwards login loads them.
The UTM VM must be registered in UTM (open `macOS.utm` once) for `utmctl start macOS` to find it.

### Second WebDriver endpoint: Safari Technology Preview in the guest

Safari allows one WebDriver session at a time, so two automation runs on port 4446 queue.
Apple documents the limit in [About WebDriver for Safari][wd-about]: "Only one Safari browser instance can be active
at any given time, and only one WebDriver session at a time can be attached to the browser instance."
Apple documents the way around it in [Testing with WebDriver in Safari][wd-testing]:
"Safari and Safari Technology Preview each provide their own `safaridriver` executable",
"Each `safaridriver` is capable of launching only the Safari version it's associated with, and the two can run
simultaneously."
So Safari Technology Preview (STP) in the guest gets its own driver, port chain and Linux unit.
It is a different WebKit than Safari 27.0 (the guest's STP was Release 254, driver `22627.1.2.1`),
so its pixels can differ from the Safari endpoint's.
Port chain: guest 4448, `m1` 4449, Linux 4450.
Nothing here edits the Safari chain (4445, 4447, 4446).

#### Install STP into the guest

Download the macOS 27 build from the [Safari download page][safari-download]
(the link labelled "Safari Technology Preview for macOS 27"; this run used `SafariTechPreview254.dmg`)
to `m1`'s external disk, then unpack the signed installer package and copy only the app.
The guest's `user` is in the `admin` group and `/Applications` is group-writable, so no `sudo` is needed
(`installer` would need the guest password):

```bash
# on m1
mkdir -p /Volumes/MacData/stp && cd /Volumes/MacData/stp
curl --fail --location --output SafariTechPreview254.dmg \
  https://secure-appldnld.apple.com/STP/142-36179-20261008-3c90402e-c681-4700-bb14-4265744f1038/SafariTechPreview254.dmg
hdiutil attach -nobrowse -readonly SafariTechPreview254.dmg
pkgutil --check-signature "/Volumes/Safari Technology Preview/SafariTechPreview254.pkg"
pkgutil --expand-full "/Volumes/Safari Technology Preview/SafariTechPreview254.pkg" expanded
cd expanded/SafariTechPreview254.pkg/Payload/Applications
COPYFILE_DISABLE=1 tar -cf - "Safari Technology Preview.app" \
  | ssh -o BatchMode=yes -i ~/.ssh/id_ed25519_vm user@192.168.64.2 'tar -C /Applications -xf -'
ssh -o BatchMode=yes -i ~/.ssh/id_ed25519_vm user@192.168.64.2 \
  'codesign --verify --deep --strict "/Applications/Safari Technology Preview.app" && echo codesign-ok'
```

The download link changes with each STP release; take the current one from the download page.
`hdiutil attach -mountpoint` onto `/Volumes/MacData` failed with `Permission denied`, so use the default mount point.
The package also installs three LaunchAgents under `/Library` (push, sync, authentication services);
the copy skips them, and whether session creation needs them is unverified until `--enable` is done.

#### Enable STP's driver (needs the guest administrator password, once)

Apple: "Run `safaridriver --enable` once" ([Testing with WebDriver in Safari][wd-testing]),
which for STP means its own executable.
Run it in the guest, from Terminal in the UTM window or over SSH with `-t` (so `sudo` can prompt),
and type the guest password when `sudo` asks:

```bash
# Linux workstation, then m1: allocate a terminal at each hop
ssh -t m1
ssh -t -i ~/.ssh/id_ed25519_vm user@192.168.64.2
# in the guest
sudo "/Applications/Safari Technology Preview.app/Contents/MacOS/safaridriver" --enable
```

If `POST /session` still fails afterwards, restart the guest driver so it rereads the setting:
`launchctl kickstart -k gui/$(id -u)/dev.monochromatic.safaridriver-stp`.

Without it, `POST /session` on the STP driver fails with
`You must enable 'Allow remote automation' in the Developer section of Safari Settings to control Safari via WebDriver.`
The same switch is in the app: open STP in the guest, Settings, Developer (older macOS: Develop menu, Allow Remote
Automation), and tick "Allow remote automation".
Without a password the non-`sudo` run of `--enable` prints `Password is not valid, please try again.` and exits 1.

#### Guest LaunchAgent: STP's WebDriver at login

`~/Library/LaunchAgents/dev.monochromatic.safaridriver-stp.plist` in the guest:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>dev.monochromatic.safaridriver-stp</string>
  <key>ProgramArguments</key>
  <array>
    <string>/Applications/Safari Technology Preview.app/Contents/MacOS/safaridriver</string>
    <string>--port</string>
    <string>4448</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>StandardErrorPath</key>
  <string>/tmp/safari-driver-stp.log</string>
</dict>
</plist>
```

#### Host LaunchAgent: forward STP's port to m1

A separate agent, so the one serving 4447 stays untouched.
`~/Library/LaunchAgents/dev.monochromatic.vm-tunnel-stp.plist` on `m1`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>dev.monochromatic.vm-tunnel-stp</string>
  <key>ProgramArguments</key>
  <array>
    <string>/usr/bin/ssh</string>
    <string>-N</string>
    <string>-o</string><string>BatchMode=yes</string>
    <string>-o</string><string>ExitOnForwardFailure=yes</string>
    <string>-o</string><string>ServerAliveInterval=15</string>
    <string>-o</string><string>ServerAliveCountMax=3</string>
    <string>-i</string><string>/Users/user/.ssh/id_ed25519_vm</string>
    <string>-L</string><string>4449:127.0.0.1:4448</string>
    <string>user@192.168.64.2</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>ThrottleInterval</key>
  <integer>10</integer>
  <key>StandardErrorPath</key>
  <string>/tmp/vm-tunnel-stp.log</string>
</dict>
</plist>
```

Load each once: `launchctl bootstrap gui/$(id -u) <plist>` in the guest (for the driver) and on `m1` (for the forward).
This agent carries no `-R 4173` forward; the guest's `localhost:4173` already comes from the Safari chain's agent,
so STP in the guest reaches the same preview origin only while that agent is up.

#### Using the endpoint

The STP capability is `{"alwaysMatch": {"browserName": "Safari Technology Preview"}}`
([Selenium's Safari page][selenium-safari] shows the same name as `browser_name` for STP;
`safari` selects the plain Safari driver).
Both browsers share the guest's one window server and one display, and the display mode set by `displayplacer`
applies to both.
Apple's documentation says the two drivers can run simultaneously; the first concurrent run with the Safari endpoint
(2026-10-18) had not happened when this was written, because the `--enable` step still needed the guest password,
so treat window-rect and focus-sensitive steps run in both at once as unverified.

## 4. Host fallback: webkit-capture

Compile on `m1` into `/Volumes/MacData/xinyanchen-capture/webkit-capture`
(`xcrun swiftc -O -o webkit-capture WebKitCapture.swift`):

```swift
// WebKitCapture.swift: capture a page with the system WebKit without depending on the Mac's screen.
// Usage: webkit-capture <url> <viewport-width> <viewport-height> <out.png> [script] [settle-ms] [full]
import AppKit
import WebKit

let args = CommandLine.arguments
guard args.count >= 5, let url = URL(string: args[1]), let width = Double(args[2]), let height = Double(args[3]) else {
  FileHandle.standardError.write("usage: webkit-capture <url> <width> <height> <out.png> [script] [settle-ms] [full]\n".data(using: .utf8)!)
  exit(2)
}
let output = args[4]
let script = args.count > 5 ? args[5] : ""
let settle = args.count > 6 ? (Double(args[6]) ?? 1500) / 1000 : 1.5
let fullPage = args.count > 7 && args[7] == "full"

final class Capture: NSObject, WKNavigationDelegate {
  let window: NSWindow
  let webView: WKWebView

  init(width: Double, height: Double) {
    let frame = NSRect(x: 0, y: 0, width: width, height: height)
    window = NSWindow(contentRect: frame, styleMask: [.borderless], backing: .buffered, defer: false)
    let configuration = WKWebViewConfiguration()
    configuration.websiteDataStore = .nonPersistent()
    webView = WKWebView(frame: frame, configuration: configuration)
    super.init()
    webView.customUserAgent = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/27.0 Safari/605.1.15"
    // Keep painting while the screen is locked or the window is covered.
    let occlusion = NSSelectorFromString("_setWindowOcclusionDetectionEnabled:")
    if webView.responds(to: occlusion) {
      webView.perform(occlusion, with: false)
    } else {
      FileHandle.standardError.write("warning: occlusion detection could not be disabled\n".data(using: .utf8)!)
    }
    webView.navigationDelegate = self
    window.contentView = webView
    window.setFrameOrigin(NSPoint(x: -20000, y: -20000))
    window.orderFrontRegardless()
  }

  func webView(_ webView: WKWebView, didFinish navigation: WKNavigation!) {
    webView.evaluateJavaScript(script.isEmpty ? "true" : script) { _, error in
      if let error { FileHandle.standardError.write("script error: \(error)\n".data(using: .utf8)!) }
      DispatchQueue.main.asyncAfter(deadline: .now() + settle) { self.measureAndSnap() }
    }
  }

  func webView(_ webView: WKWebView, didFail navigation: WKNavigation!, withError error: Error) { fail(error) }
  func webView(_ webView: WKWebView, didFailProvisionalNavigation navigation: WKNavigation!, withError error: Error) { fail(error) }

  func fail(_ error: Error) {
    FileHandle.standardError.write("load failed: \(error)\n".data(using: .utf8)!)
    exit(1)
  }

  func measureAndSnap() {
    guard fullPage else { return snap() }
    webView.evaluateJavaScript("document.documentElement.scrollHeight") { value, _ in
      let pageHeight = (value as? Double) ?? Double(self.webView.frame.height)
      let frame = NSRect(x: 0, y: 0, width: self.webView.frame.width, height: pageHeight)
      self.window.setContentSize(frame.size)
      self.webView.frame = frame
      DispatchQueue.main.asyncAfter(deadline: .now() + settle) { self.snap() }
    }
  }

  func snap() {
    let configuration = WKSnapshotConfiguration()
    configuration.afterScreenUpdates = true
    webView.evaluateJavaScript("[document.visibilityState, devicePixelRatio, innerWidth, innerHeight]") { state, _ in
      FileHandle.standardError.write("page state: \(String(describing: state))\n".data(using: .utf8)!)
      self.webView.takeSnapshot(with: configuration) { image, error in
        guard let image, let tiff = image.tiffRepresentation, let bitmap = NSBitmapImageRep(data: tiff),
              let png = bitmap.representation(using: .png, properties: [:]) else {
          FileHandle.standardError.write("snapshot failed: \(String(describing: error))\n".data(using: .utf8)!)
          exit(1)
        }
        do {
          try png.write(to: URL(fileURLWithPath: output))
          print("wrote \(output) \(bitmap.pixelsWide)x\(bitmap.pixelsHigh)")
          exit(0)
        } catch {
          FileHandle.standardError.write("write failed: \(error)\n".data(using: .utf8)!)
          exit(1)
        }
      }
    }
  }
}

let app = NSApplication.shared
app.setActivationPolicy(.accessory)
let capture = Capture(width: width, height: height)
capture.webView.load(URLRequest(url: url))
DispatchQueue.main.asyncAfter(deadline: .now() + 120) {
  FileHandle.standardError.write("timed out\n".data(using: .utf8)!)
  exit(1)
}
app.run()
```

## 5. Linux-side units

The site under test is served on the Linux workstation; these units connect it to `m1`.
`systemd-run` units vanish at the Linux machine's reboot, so re-run them after one.

```bash
# Linux workstation: a no-store static server (Simulator Safari caches aggressively), on 4174.
# The multiplexing variant also serves /home/user/temp/agent/wt-<name> at /@wt-<name>/, which the style-diff tools
# and their iframe harness (/@wt-harness/blank.html) need; restarted this way after the 2026-10-19 reboot.
systemd-run --user --unit=xinyanchen-no-store --collect --property=MemoryMax=256M \
  "$(command -v node)" /var/home/user/temp/agent/xinyanchen-build/no-store-server-mux.mjs
# m1's localhost:4173 becomes the Linux 4174 server, for the Simulator and (through the vm-tunnel agent) the VM
systemd-run --user --unit=xinyanchen-m1-tunnel --collect --property=MemoryMax=64M \
  ssh -N -o BatchMode=yes -o ExitOnForwardFailure=yes -o ServerAliveInterval=30 -R 127.0.0.1:4173:127.0.0.1:4174 m1
# the guest's WebDriver, through m1's 4447, as local 4446
systemd-run --user --unit=xinyanchen-vm-webdriver --collect --property=MemoryMax=64M \
  --property=Restart=always --property=RestartSec=10 \
  ssh -o BatchMode=yes -o ExitOnForwardFailure=yes -o ServerAliveInterval=15 -N -L 4446:127.0.0.1:4447 m1
# Safari Technology Preview's WebDriver (guest 4448, m1 4449) as local 4450, a second concurrent endpoint
systemd-run --user --unit=xinyanchen-vm-webdriver-stp --collect --property=MemoryMax=64M \
  --property=Restart=always --property=RestartSec=10 \
  /usr/bin/ssh -o BatchMode=yes -o ExitOnForwardFailure=yes -o ServerAliveInterval=15 -N -L 4450:127.0.0.1:4449 m1
```

`no-store-server.mjs` serves one directory with `Cache-Control: no-store`:

```js
// no-store-server.mjs: static file server for device testing; ROOT and PORT are the only settings.
import { createReadStream, statSync } from 'node:fs';
import { createServer } from 'node:http';
import { extname, join, normalize } from 'node:path';

const ROOT = '/var/home/user/xinyanchen-portfolio/portfolio-xinyanchen0124';
const PORT = 4174;
const TYPES = {
  '.html': 'text/html; charset=utf-8', '.css': 'text/css; charset=utf-8', '.mjs': 'text/javascript; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8', '.svg': 'image/svg+xml', '.png': 'image/png', '.jpg': 'image/jpeg',
  '.ico': 'image/x-icon', '.ttf': 'font/ttf', '.pdf': 'application/pdf', '.webp': 'image/webp', '.woff2': 'font/woff2',
};

createServer((req, res) => {
  const path = decodeURIComponent(new URL(req.url, 'http://x').pathname);
  const file = normalize(join(ROOT, path.endsWith('/') ? `${path}index.html` : path));
  if (!file.startsWith(ROOT)) {
    res.writeHead(403).end();
    return;
  }
  try {
    if (!statSync(file).isFile()) throw new Error('not a file');
  } catch {
    res.writeHead(404).end();
    return;
  }
  res.writeHead(200, { 'Content-Type': TYPES[extname(file)] ?? 'application/octet-stream', 'Cache-Control': 'no-store' });
  createReadStream(file).pipe(res);
}).listen(PORT, '127.0.0.1');
```

## 6. Linux-side scripts

### Simulator screenshots: sim-shots.mjs

```js
// Open URLs in the iPhone 15 Simulator's Safari on m1 and copy each screenshot back.
// Usage: node sim-shots.mjs <label> <path-with-query-or-fragment>...
// Text fragments reach any element: page.html#:~:text=Potential%20for%20Impact
import { execFileSync } from 'node:child_process';

const [label, ...paths] = process.argv.slice(2);
const SET = '/Volumes/XinyanSimDevices/devices';
const UDID = '4E2C74A5-7001-4F13-82CD-CD7AA9AA2CA9';
const ssh = (cmd) => execFileSync('ssh', ['-o', 'BatchMode=yes', 'm1', cmd], { encoding: 'utf8', timeout: 120_000 });
paths.forEach((path, index) => {
  const name = `sim-${label}-${String(index).padStart(2, '0')}.png`;
  // A unique query per shot defeats Safari's cache; the fragment scrolls to the section.
  const [base, fragment] = path.split('#');
  const url = `http://localhost:4173/${base}${base.includes('?') ? '&' : '?'}sim=${Date.now()}${fragment ? `#${fragment}` : ''}`;
  ssh(`xcrun simctl --set ${SET} openurl ${UDID} '${url}' && sleep 7 && xcrun simctl --set ${SET} io ${UDID} screenshot /Volumes/XinyanSimDevices/${name} >/dev/null 2>&1`);
  execFileSync('scp', ['-q', `m1:/Volumes/XinyanSimDevices/${name}`, `/home/user/temp/agent/xinyanchen-build/sim/${name}`]);
  console.log(name, url);
});
```

### VM Safari screenshots: vm-safari-shot.mjs

```js
// Capture real Safari inside the UTM macOS guest through WebDriver; it keeps painting while m1 is locked.
// Needs the tunnel chain local 4446 -> m1 4447 -> guest 4445.
// Usage: node vm-safari-shot.mjs <out-dir> <width>x<height>@<1|2> <page>@<scrollY> [<page>@<scrollY> ...]
//   @1 is a 1080p screen at 1x, @2 a 4K screen at 2x; both are 1920 CSS pixels wide.
//   page is a path under http://localhost:4173/ (the guest sees m1's preview on that origin).
//   Step scroll offsets by the viewport height (1028 for a 1920x1080 window) or frames repeat content.
import { execFileSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { setTimeout as wait } from 'node:timers/promises';

const DRIVER = 'http://127.0.0.1:4446';
const GUEST_DISPLAY = 'id:35B36D67-6BAC-47B7-98B5-07877849011C';
// displayplacer modes on the guest's 1920x1200 display: 24 is unscaled (1x), 34 is HiDPI (2x).
const MODE_FOR_SCALE = { 1: 24, 2: 34 };
const [outDir, size, ...shots] = process.argv.slice(2);
const [dimensions, scale = '2'] = size.split('@');
const [width, height] = dimensions.split('x').map(Number);
mkdirSync(outDir, { recursive: true });

// UTM's dynamic resolution resets the guest mode when its window changes, so set it every run.
execFileSync('ssh', ['-o', 'BatchMode=yes', 'm1',
  `ssh -o BatchMode=yes -i ~/.ssh/id_ed25519_vm user@192.168.64.2 '/usr/local/bin/displayplacer "${GUEST_DISPLAY} mode:${MODE_FOR_SCALE[scale]} origin:(0,0) degree:0"'`]);
await wait(2000);

async function call(method, path, body) {
  const response = await fetch(`${DRIVER}${path}`, {
    method,
    headers: { 'Content-Type': 'application/json' },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  const json = await response.json();
  if (json.value?.error) throw new Error(`${method} ${path}: ${json.value.error}: ${json.value.message}`);
  return json.value;
}

console.log('driver status', JSON.stringify(await call('GET', '/status')));
const { sessionId } = await call('POST', '/session', { capabilities: { alwaysMatch: { browserName: 'safari' } } });
const session = `/session/${sessionId}`;
try {
  // The window rect includes Safari's toolbar: a 1920x1080 window gives a 1920x1028 viewport.
  await call('POST', `${session}/window/rect`, { x: 0, y: 0, width, height });
  for (const shot of shots) {
    const [page, scrollY = '0'] = shot.split('@');
    await call('POST', `${session}/url`, { url: `http://localhost:4173/${page}?vm=${Date.now()}` });
    await wait(4000);
    const state = await call('POST', `${session}/execute/sync`, {
      script: 'scrollTo(0, arguments[0]); return [innerWidth, innerHeight, devicePixelRatio, document.visibilityState];',
      args: [Number(scrollY)],
    });
    await wait(1000);
    // Clamping at the page end can leave the page elsewhere; record where it is.
    state.push(...await call('POST', `${session}/execute/sync`, {
      script: 'return [scrollY, document.documentElement.scrollHeight];',
      args: [],
    }));
    const file = `${outDir}/${page.replace(/\.html$/, '')}-${width}x${scale}-${scrollY}.png`;
    writeFileSync(file, Buffer.from(await call('GET', `${session}/screenshot`), 'base64'));
    console.log(JSON.stringify({ file, viewport: state }));
  }
} finally {
  await call('DELETE', session);
}
```

A script that evaluates JavaScript instead of capturing (`vm-safari-eval.mjs`) is the same file with the shot loop
replaced by one `POST /session/{id}/url` and one `POST /session/{id}/execute/sync` whose result it prints.
Keyboard input uses `POST /session/{id}/actions` with key code points (`` Space, `` Tab).

If `POST /session` answers "already paired with another WebDriver session", `DELETE /session/<id>` the stale one.
`GET /status` still answers `ready: true` while paired, so it proves nothing about a free session.
When no live client knows the id (the script died before its `finally` ran), restart that driver's launch agent in
the guest (`launchctl kickstart -k gui/$(id -u)/dev.monochromatic.safaridriver-stp`, or
`dev.monochromatic.safaridriver` for regular Safari) and quit the leftover automation window; this ends whatever
that driver was running, so check that no other job holds its lock first.
A common way to kill a script before cleanup is piping it into `head`: `head` exits after its lines and the script
dies on SIGPIPE at its next write, so redirect WebDriver scripts to a file instead.
If it times out "connecting to a Safari instance", quit any normal Safari window in the guest
(`osascript -e 'quit app "Safari"'`) and never start Safari's binary directly.

## Verify the rebuilt rig

1.  `xcrun simctl --set /Volumes/XinyanSimDevices/devices list devices` shows `iPhone 15 client` `Booted`.
2.  `node sim-shots.mjs check index.html` returns a screenshot of the page.
3.  `/Volumes/MacData/idb-venv/bin/idb --companion localhost:10882 describe` reports 393 by 852 points.
4.  `curl --silent http://127.0.0.1:4446/status` on Linux prints `"ready":true`.
5.  `node vm-safari-shot.mjs out 1920x1080@1 index.html@0` reports `[1920,1028,1,"visible",...]`,
    and `@2` reports device pixel ratio 2.
6.  Lock `m1` (`pmset displaysleepnow`) and repeat 2 and 5: both must still produce real pages.
7.  `curl --silent http://127.0.0.1:4450/status` on Linux prints `"ready":true` (the STP chain).
    A `POST /session` to port 4450 with
    `{"capabilities": {"alwaysMatch": {"browserName": "Safari Technology Preview"}}}`
    must return a session whose `navigator.userAgent` and `document.title` read from `http://localhost:4173/index.html`;
    delete it afterwards.
    It needs the one-time `--enable` above, and it works with no session open on 4446.

[wd-about]: https://developer.apple.com/documentation/webkit/about-webdriver-for-safari
[wd-testing]: https://developer.apple.com/documentation/webkit/testing-with-webdriver-in-safari
[safari-download]: https://developer.apple.com/safari/download/
[selenium-safari]: https://www.selenium.dev/documentation/webdriver/browsers/safari/
