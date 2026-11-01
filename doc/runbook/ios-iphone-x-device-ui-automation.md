# Driving on-device black-box UI automation on the iPhone X via Xcode 15.2

## What this proves

This runbook is the manual close-out for the one iOS vetting step that could not be automated over SSH:
running black-box UI automation (XCUITest / WebDriverAgent) against the physical iPhone X (iOS 16.7.16).
 The
simulator leg is already complete (`../decision/ios-iphone-x-vet-report/vet-ui-automation.md`,
 section 3).
 The
device leg is blocked by a host-toolchain gap,
 not a framework or signing limit:
 the installed Xcode 26 cannot
stand up an XCTest test-host session against iOS 16.7 (four major versions older),
 and Appium's WebDriverAgent
v13 dropped the iOS-16 launch path.
 The fix is to drive the device from Xcode 15.2,
 which natively supported iOS
16.7,
 but using Xcode 15.2 needs an admin (sudo) first launch plus a GUI device-preparation that an SSH session
cannot perform.

Honesty note:
 the steps below could not be run end to end from the vetting session,
 because every blocking step
needs the admin password this agent does not hold;
 the expected outputs come from observed behaviour this
session (the `xctrace` device line,
 the WDA build success) and from documented WebDriverAgent and Xcode behaviour
(the `ServerURLHere` line,
 the `/status` JSON).
 Treat the WDA-runs-on-iOS-16 outcome as expected-but-unverified;
the Fallback section covers the one way it can still fail.

Bridges already tried and confirmed not to work (so this handoff is a real obstacle,
 not an unconsidered one):
`go-ios runwda` and `tidevice xctest` both launch the WDA process on the device (correct vet identity,
 real pid)
but its XCTest session never engages and it exits without binding port 8100;
 `tidevice wdaproxy` additionally
hits a Python 3.14 proxy bug;
 Appium `usePreinstalledWDA` is rejected ("only supported on iOS/tvOS 17.0 and
newer");
 building WDA with Xcode 26 and launching via `xcodebuild test-without-building` fails with
`build number "" incompatible with DVTBuildVersion` and `Logic Testing Unavailable`;
 rebuilding WDA with Xcode
15.2 fails because its platform is not installed (`iOS 17.2 is not installed`),
 which is exactly what step 2
below fixes.
 `iproxy` forwarding works;
 there is just nothing serving behind it until WDA's session starts.

All commands run in a terminal on the Mac itself (the M1 MacBook Air),
 because the sudo steps need an interactive
admin password.
 The agent drives everything else over SSH once the admin steps are done.

## Setup

Status:
 TODO

Bring the machine to this starting state before the steps.

- Hardware:
   the iPhone X (`iPhone10,3`,
   iOS 16.7.16,
   build 20H392) connected to the Mac with a USB cable and
  unlocked.
   Wifi-only is not enough here:
   `xcodebuild` and `xctrace` see the device only over USB.
- The device has Developer Mode on and trusts this Mac (set up in `ios-iphone-x-codesign-setup.md`).
   Confirm with
  `idevice_id -l`,
   which must print `9057e2a8c2e70162e35b9ea8bf006f736670877b` (your device's UDID;
   read yours
  from `idevice_id -l` if different,
   and substitute it everywhere below as `$UDID`).
- Xcode 15.2 expanded at `/Volumes/MacData/Xcodes/Xcode.app`.
   If it is gone,
   re-create it:
   download
  `Xcode_15.2.xip` from **<https://developer.apple.com/download/all/>** signed in with the developer Apple ID
  (`little.plan2433@fastmail.com`),
   then `xip --expand Xcode_15.2.xip` in `/Volumes/MacData/Xcodes`.
- Xcode 26 remains installed at `/Applications/Xcode.app` (the default toolchain;
   the Restore section switches
  back to it).
- The vet signing keychain `~/ios-vet/vet.keychain-db` exists and its unlock password is in the repo `.env.local`
  as `XCODE_IDENTITY_SSH_USABLE` (from `ios-iphone-x-codesign-setup.md`).
- Tools installed this session and reused here:
   `libimobiledevice` (for `idevice_id`,
   `iproxy`,
  `ideviceinstaller`),
   `appium` 3.5 with the `xcuitest` driver,
   and `node`.
   All are on the Mac's PATH via
  Homebrew and mise.
- Admin (sudo) access on the Mac,
   and a few minutes of the owner's attention.

## Steps

Status:
 TODO

1. Open **Terminal** on the Mac (**Spotlight** with **Cmd+Space**,
    type `Terminal`,
    press **Return**).
   Expected outcome:
    a shell prompt.
2. Accept the Xcode 15.2 license:
   ```sh
   sudo /Volumes/MacData/Xcodes/Xcode.app/Contents/Developer/usr/bin/xcodebuild -license accept
   ```
   Enter your admin password when prompted.
    Expected outcome:
    the command returns to the prompt with no output
   (a license-already-accepted run is also silent).
3. Install Xcode 15.2's bundled platform and components:
   ```sh
   sudo /Volumes/MacData/Xcodes/Xcode.app/Contents/Developer/usr/bin/xcodebuild -runFirstLaunch
   ```
   Expected outcome:
    it prints package-install progress and ends with `** INSTALL SUCCEEDED **` or returns
   silently;
    a second run prints `No actions to perform` or returns silently.
4. Point the active developer toolchain at Xcode 15.2 (so the on-device CoreDevice and instruments daemons use
   the 15.2 versions,
    the ones that speak iOS 16.7):
   ```sh
   sudo xcode-select -s /Volumes/MacData/Xcodes/Xcode.app/Contents/Developer
   ```
   Expected outcome:
    no output.
    Verify with `xcode-select -p`,
    which must print
   `/Volumes/MacData/Xcodes/Xcode.app/Contents/Developer`.
5. Confirm Xcode 15.2 now sees the wired device:
   ```sh
   xcrun xctrace list devices
   ```
   Expected outcome:
    under `== Devices ==` (not `== Simulators ==`) a line
   `iPhone X (16.7.16) (9057e2a8c2e70162e35b9ea8bf006f736670877b)`.
6. If that line is missing or the device later reports unavailable,
    prepare it once in the GUI:
    open **Xcode**
   (the 15.2 copy at `/Volumes/MacData/Xcodes/Xcode.app`),
    open **Window > Devices and Simulators**
   (**Shift+Cmd+2**),
    select the **iPhone X** in the left list,
    and wait until the status reads **Connected** with
   no **Preparing device for development** spinner.
   Expected outcome:
    the device shows **Connected** and a green dot;
    re-running the step 5 command now lists it.
7. Tell the agent to rebuild WebDriverAgent with Xcode 15.2 and run it on the device,
    or do it yourself.
    The build
   (signed by the vet identity,
    `DEVELOPER_DIR` pointed at Xcode 15.2) ends `** TEST BUILD SUCCEEDED **` and
   writes a `*.xctestrun` under `/Volumes/MacData/ios-vet/wda-dd152/Build/Products/`.
    Then launch it on the
   device:
   ```sh
   DEVELOPER_DIR=/Volumes/MacData/Xcodes/Xcode.app/Contents/Developer \
   xcodebuild test-without-building \
     -xctestrun /Volumes/MacData/ios-vet/wda-dd152/Build/Products/WebDriverAgentRunner_iphoneos17.2-arm64.xctestrun \
     -destination "platform=iOS,id=9057e2a8c2e70162e35b9ea8bf006f736670877b"
   ```
   Expected outcome:
    it streams test output and prints a line containing
   `ServerURLHere->http://192.168.x.x:8100<-ServerURLHere`,
    then appears to hang.
    Leave it running;
    the hang is
   WDA serving.
8. In a second **Terminal** tab (**Cmd+T**),
    forward the port and confirm WDA answers:
   ```sh
   iproxy 8100:8100 -u 9057e2a8c2e70162e35b9ea8bf006f736670877b &
   curl -s http://127.0.0.1:8100/status
   ```
   Expected outcome:
    JSON containing `"state" : "success"` and the device's `"name"`.
    WDA is now live on the
   iPhone X.
9. Hand back to the agent to drive the device:
    it points Appium at `appium:webDriverAgentUrl=http://127.0.0.1:8100`
   and re-runs the simulator drive (install a gate app,
    dump `/source`,
    tap,
    re-dump) against the real device,
   writing the per-app trees as `*.device.xml`.
    Expected outcome:
    the agent reports a captured element tree from
   the iPhone X and,
    for the Flutter gate,
    `counter after 2 taps: 2`.

## What to check

Status:
 TODO

- `xcode-select -p` prints `/Volumes/MacData/Xcodes/Xcode.app/Contents/Developer` (Xcode 15.2 is active).
- `xcrun xctrace list devices` includes the exact line
  `iPhone X (16.7.16) (9057e2a8c2e70162e35b9ea8bf006f736670877b)`.
- The step 7 launch prints `ServerURLHere->http://` and does not exit.
- `curl -s http://127.0.0.1:8100/status` returns JSON containing `"state" : "success"` (not a connection
  refused,
   not an empty body).
- The device element tree the agent saves
  (`/Volumes/MacData/ios-vet/uiauto-trees/<framework>.device.xml`) opens with
  `<XCUIElementTypeApplication ... bundleId="dev.monochromatic.iosvet.hellodevice"` and the gate's real labels,
  not the springboard.

## Restore

Status:
 TODO

- Switch the active toolchain back to Xcode 26,
   so the daily provisioning-renewal LaunchAgent and any other build
  keep using the current Xcode:
  ```sh
  sudo xcode-select -s /Applications/Xcode.app/Contents/Developer
  ```
  Verify `xcode-select -p` prints `/Applications/Xcode.app/Contents/Developer`.
- Stop the leftover processes:
   in the step 7 terminal press **Ctrl+C** (stops the WDA test run),
   and
  `pkill iproxy` for the port forward.
- Optional,
   remove the WDA runner from the device (it is a distinct bundle id and harmless to leave,
   and removing
  it does not drop developer trust because the `anchor` app stays installed):
  ```sh
  ideviceinstaller -n uninstall dev.monochromatic.iosvet.wda.xctrunner
  ```
- Optional,
   reclaim about 12 GB plus the downloads:
  ```sh
  rm -rf /Volumes/MacData/Xcodes
  rm ~/Downloads/Xcode_15.2.xip ~/Downloads/Xcode_15.4.xip
  ```

## Fallback: if WDA still exits without serving under Xcode 15.2

If step 7 launches the runner but never prints `ServerURLHere` and the process exits (the v13 WDA refusing iOS
16 outright rather than the toolchain mismatch),
 install an iOS-16-era WebDriverAgent and rebuild it:

1. Swap the Appium xcuitest driver for a v5 line (its bundled WDA 5.
   x still supports iOS 16):
   ```sh
   appium driver uninstall xcuitest
   appium driver install xcuitest@5.14.2
   ```
   Expected outcome:
    `Driver xcuitest@5.14.2 successfully installed`.
2. Rebuild WDA from the new bundle at
   `~/.appium/node_modules/appium-xcuitest-driver/node_modules/appium-webdriveragent` with Xcode 15.2 (the same
   build command the agent used,
    `DEVELOPER_DIR` pointed at Xcode 15.2),
    then repeat from step 7.
   Expected outcome:
    `** TEST BUILD SUCCEEDED **`,
    then `ServerURLHere->http://` on launch.
