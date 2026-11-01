# Black-box e2e UI automation for iOS (XCUITest/WebDriverAgent, Appium, Maestro)

Vet date:
 2026-06-13.
Scope:
 the external,
 out-of-process "Playwright for iOS" layer,
 the iOS analog of
`../kotlin-android-kopia-pcloud-vet-report/vet-android-e2e.md`.

The analogy path spelled `../kotlin-android-kopia-pcloud-vet-reports/` when this vet was recorded on 2026-06-13.
The folder became `kotlin-android-kopia-pcloud-vet-report/` on 2026-07-15 (`bd19d57e20`),
so the `../kotlin-android-kopia-pcloud-vet-reports/` spelling no longer resolves.

A driver that launches an installed app on a real device or simulator and drives it (taps,
 reads UI
state,
 asserts) from outside the app process.
In-process UI testing (`XCTest`/`XCUIApplication` compiled into the app,
 `flutter_test`,
`runComposeUiTest`) is a separate category,
 named where it changes the recommendation.

This vet matches the depth of the Android e2e vet:
 the same Appium major (3.5.0) and a Maestro within a
patch (2.6.1 vs the Android run's 2.6.0),
 driving a real counter app to a final asserted state,
 plus the
addressability evidence that is the actual finding.
 The one difference the owner asked for:
 real-device
(iPhone X) validation is deferred to the end and is recorded in its own section below;
 everything else here
ran on the iOS 26.5 simulator.

## Verdict up front

On the iOS simulator,
 both Appium (XCUITest driver) and Maestro actually work end to end,
 driving a real
Flutter counter app from "0" to "2",
 the exact iOS analog of the Android "Count:
 2" proof.
 The substrate both
of them wrap is WebDriverAgent,
 itself an XCUITest app,
 so WebDriverAgent/XCUITest is the iOS primitive in the
same role UiAutomator played on Android:
 not a peer driver but the engine the peers are built on.

The decision-relevant finding is not "the tools run" but addressability:
 what each UI framework's rendering
model projects into the iOS accessibility tree,
 which is the only thing a black-box driver (and VoiceOver) can
see.
 That projection splits the survivors into three classes,
 and the split is captured from real element-tree
dumps,
 not assumed.
 Those same dumps are the headless half of the owner-owed VoiceOver sweep
(`../ios-iphone-x-music-player-kopia-stack.md`,
 task 7):
 they record what VoiceOver would read,
 minus audible
speech.

## 1. Ecosystem audit and versions

### The three tools and how each drives iOS (verified from the run, plus the Android source audits)

- XCUITest with WebDriverAgent,
   the primitive.
  WebDriverAgent (appium/WebDriverAgent,
   Apache-2.0) is an XCUITest application:
   it links Apple's XCTest
  `XCUIApplication`/`XCUIElement` API and exposes it over an on-device HTTP server.
   XCUITest is the
  accessibility-driven UI test framework Apple ships;
   it reads the same `XCUIElement` tree that
  `xcrun simctl`/`instruments` expose,
   which is a projection of the app's `UIAccessibility` tree.
   WebDriverAgent
  is the engine,
   not an external orchestrator:
   it is what both peers below install and talk to.
   This is the
  structural twin of UiAutomator on Android (the on-device instrumentation primitive that Maestro and Appium
  both wrap).

- Appium server 3.5.0 + appium-xcuitest-driver 11.11.2,
   both Apache-2.0,
   TypeScript/JS.
  The driver implements the W3C WebDriver protocol on the host and,
   at session creation,
   builds (or installs) a
  WebDriverAgent,
   launches it on the target,
   forwards its port,
   and proxies W3C commands to WDA's HTTP API.
   On
  the simulator the WDA build needs no code signing;
   on a device WDA must be code-signed (the device section
  below).
   Same Appium major as the Android `vet-android-e2e.md` run,
   so the comparison is apples to apples:
  appium-xcuitest-driver here is the structural peer of appium-uiautomator2-driver there.

- Maestro CLI 2.6.1 (mobile-dev-inc/maestro),
   Apache-2.0,
   Kotlin,
   JDK-hosted.
  Host-side JVM CLI.
   For iOS it drives the target through an XCTest runner over `xcrun simctl` (simulator) or a
  signed runner plus a usbmux tunnel (device),
   the iOS counterpart of its Android adb path.
   Declarative YAML
  flows,
   the closest feel to a high-level Playwright script.
   Maestro's real-device iOS path is historically
  thinner than its simulator and Android paths;
   the simulator run here is clean,
   and the device path is part of
  the deferred device section.

### Licenses

- WebDriverAgent:
   Apache-2.0.
- Appium + appium-xcuitest-driver:
   Apache-2.0.
- Maestro:
   Apache-2.0.
All three are clean open source;
   no open-source-default exception needed.
   These are the same licenses,
   and in
two of three the same projects,
   cleared in the Android e2e vet.

## 2. Addressability by rendering model (the crux for any black-box tool, and for VoiceOver)

iOS black-box drivers and VoiceOver see only what a framework projects into `UIAccessibility`,
 surfaced as the
`XCUIElement` tree.
 The 18 survivors span three rendering backends,
 and the projection differs by backend.
 One
representative per backend was installed on the booted iOS 26.5 simulator and its full element tree dumped via
Appium;
 the trees are saved at `/Volumes/MacData/ios-vet/uiauto-trees/*.sim.xml` on the Mac.
 Node-type counts
and labels below are from those dumps.

### WebView-backed: projects an `XCUIElementTypeWebView` subtree

- Capacitor (`capgate`):
   the WKWebView DOM projects as `XCUIElementTypeWebView` (3 nodes) under
  `XCUIElementTypeOther` containers,
   with `XCUIElementTypeStaticText` for text;
   labels "Capacitor vet" and
  "WKWebView OK" are exposed,
   22 hittable elements,
   and a tap changed the tree.
- Cordova (`cordovagate`,
   Quasar `www`):
   same shape,
   `XCUIElementTypeWebView` (3) plus a real
  `XCUIElementTypeButton` ("QUASAR BUTTON") alongside StaticText "Quasar in WKWebView" and "JS:
   720";
   tapping
  the button changed the tree.
   So a web framework's semantic controls (a `<button>`) reach the native tree as
  `XCUIElementTypeButton`,
   not just as text.

This is the class that also covers Dioxus:
 its lockfile resolves `wry` and `objc2-web-kit`
(`/Volumes/MacData/ios-vet/dioxusgate/Cargo.lock`),
 so dioxus-mobile renders through a WKWebView and shares the
WebView projection,
 not a self-drawn one.
 The WKWebView a11y bridge is the most mature on iOS (VoiceOver reads
the web ARIA tree),
 so this class is the lowest a11y risk.

### Real UIKit views: native `StaticText` labels, no WebView

- React Native (`rngate`):
   `XCUIElementTypeStaticText` with labels "RN Gate" and "Hermes:
   ON".
   RN renders real
  `UIView`s,
   so each gets native accessibility for free.
- NativeScript (`nsgate`):
   StaticText "NS Gate",
   "V8 JS:
   720".
   Same:
   real UIKit views.
- UIKit baseline (`uikitgate`):
   StaticText "Pure UIKit,
   native a11y",
   "RENDER OK".
   The reference point.

These project the same way native UIKit does,
 so the black-box tool and VoiceOver see real labelled controls
with no WebView indirection.
 Low a11y risk by construction.

### Self-drawn (Skia canvas): a custom `UIAccessibility` bridge, and the bridges differ

This is the class where a render PASS can still hide an a11y gap,
 so the dumps matter most here.

- Flutter (`flgate`,
   the default counter):
   a rich bridge.
   The title projects with `traits="Header"`,
   the action
  is a real `XCUIElementTypeButton name="Increment" traits="Button"`,
   and the live counter is a StaticText "0".
  Flutter's SemanticsTree maps to `UIAccessibility` with correct roles,
   not just text.
- Compose Multiplatform (`composegate`):
   its two text elements both project as `XCUIElementTypeStaticText` with
  `accessible="true"` and correct labels ("Compose Gate",
   "Compose Multiplatform on iOS"),
   inside an
  `XCUIElementTypeScrollView`.
   So Compose MP's text does bridge to `UIAccessibility`.
   The honest limit:
   this gate
  has no interactive control,
   so Compose MP's role projection (does a Compose `Button` reach the tree as a
  Button with the right traits and state,
   the way Flutter's does) is untested here and stays the load-bearing
  open question for the owner VoiceOver sweep.

Lynx (`lynxgate`) is a fourth,
 narrower case:
 its own engine renders a `lynxview` container whose text children
project as `StaticText` ("Lynx Gate",
 "JS:
 720",
 "Rust:
 720",
 "CROSSING OK"),
 so its content is addressable,
but,
 like Compose,
 the gate carries no interactive control to test role projection.

### Why this is the finding

On Android the crux was Compose's `testTagsAsResourceId` projection into the accessibility tree;
 on iOS the
crux is the rendering backend,
 because the survivors span WebView,
 native UIKit,
 and self-drawn Skia,
 and only
the first two get a11y "for free.
" The dumps confirm:
 every class exposes text,
 the WebView and UIKit classes
expose real control roles automatically,
 and among the self-drawn class Flutter's bridge is demonstrably rich
(Header,
 Button traits) while Compose MP's is confirmed only for text.
 That maps exactly onto the a11y posture
in the synthesis doc and gives it on-device-tree evidence instead of a priori reasoning.

## 3. Full verification on the simulator (commands + output)

Device under test:
 booted iPhone 17 Pro,
 iOS 26.5,
 `09D9EB9B-8036-4D23-929D-F75ADE9987FA`.
Appium server held open on the Mac (`appium server -p 4723 --relaxed-security`).
 A raw-fetch W3C client
(`/Users/user/ios-vet/drive-sim.mjs`,
 the iOS analog of the Android `drive-counter.mjs`) created the session,
dumped `/source`,
 found elements,
 tapped,
 and re-dumped.

### Appium drove the Flutter counter 0 to 2

The gate apps all share one bundle id (`dev.monochromatic.iosvet.hellodevice`,
 the codesign runbook's
reuse-one-id strategy),
 so install was decoupled from drive:
 `xcrun simctl uninstall` then `install` the
specific `.app`,
 then Appium attached by bundle id with `forceAppLaunch`.
 (With `noReset:true` and no explicit
install,
 Appium silently relaunched whichever app already held the shared id,
 which is the first thing this run
caught and corrected.
)

```txt
$ node count.mjs dev.monochromatic.iosvet.hellodevice   # Flutter installed
counter before: 0
counter after 2 taps: 2
PASS: drove Flutter counter to 2
```

The client found the `Increment` button by accessibility id,
 tapped twice,
 and read the counter StaticText back
as "2".
 This is the iOS analog of the Android "Count:
 2" assertion.

### Maestro drove the same counter 0 to 2 (declarative)

Flow `/Volumes/MacData/ios-vet/flutter-counter.flow.yaml`:

```yaml
appId: dev.monochromatic.iosvet.hellodevice
---
- launchApp
- assertVisible: "You have pushed the button this many times:"
- assertVisible: "0"
- tapOn: "Increment"
- tapOn: "Increment"
- assertVisible: "2"
```

```txt
$ ./maestro/bin/maestro --device 09D9EB9B-... test flutter-counter.flow.yaml
Running on iPhone 17 Pro - iOS 26.5 - 09D9EB9B-...
 > Flow flutter-counter.flow
Launch app "dev.monochromatic.iosvet.hellodevice"... COMPLETED
Assert that "You have pushed the button this many times:" is visible... COMPLETED
Assert that "0" is visible... COMPLETED
Tap on "Increment"... COMPLETED
Tap on "Increment"... COMPLETED
Assert that "2" is visible... COMPLETED
```

Maestro matched the Flutter `Increment` button and the counter text by visible label with no locator
gymnastics,
 the same ergonomics it showed against Compose on Android.

### Addressability dumps captured per representative

Eight element trees were captured (`*.sim.xml`):
 capgate (Capacitor),
 cordovagate (Cordova/Quasar),
 lynxgate
(Lynx),
 rngate (React Native),
 nsgate (NativeScript),
 uikitgate (UIKit),
 composegate (Compose MP),
 flgate
(Flutter).
 These are the per-framework accessibility-tree evidence for the VoiceOver sweep.

## 4. Real-device (iPhone X) validation

Status:
 the entire device pipeline up to the XCTest session is proven on the real iPhone X;
 the final black-box
drive is blocked by a toolchain-versus-OS-version gap that is uniform across every framework,
 not a per-framework
limitation,
 characterized precisely below.
 Pursuit was stopped by owner decision after the close-out attempt
below also hit the gap.

Which frameworks cannot do on-device UI automation:
 all of them equally,
 and none because of the framework.
 On
the iOS 26.5 simulator every rendering model drives cleanly (section 3).
 On this iPhone X (iOS 16.7) no framework
can be black-box driven right now,
 because the block is the host toolchain's inability to stand up an XCTest
session against iOS 16.7,
 which is identical for Capacitor,
 Compose,
 Flutter,
 React Native,
 and every other
survivor.
 So this is recorded as an environment limitation of the iPhone-X-plus-Xcode-26 setup,
 not as a vet
result that separates frameworks.

What is proven on the device (each over SSH,
 no owner GUI action):

- WebDriverAgent app-id provisioned headlessly.
   A fresh bundle id `dev.monochromatic.iosvet.wda` was minted
  with `-allowProvisioningUpdates` over SSH,
   the same autonomous path that minted the `anchor` app-id earlier
  in the vet.
   So a new provisioned app-id is not an owner step.
- WDA built and signed for the device.
   `xcodebuild ... -scheme WebDriverAgentRunner -destination
  'generic/platform=iOS' -allowProvisioningUpdates OTHER_CODE_SIGN_FLAGS=--keychain <vet.keychain>
  build-for-testing` ended `** TEST BUILD SUCCEEDED **`,
   producing `WebDriverAgentRunner-Runner.app` signed by
  the vet identity (`Apple Development: ...`,
   team HWLVAKDV4F),
   bundle id `dev.monochromatic.iosvet.wda.xctrunner`.
- WDA installed on the iPhone X.
   `ideviceinstaller -n install` over wifi completed,
   and the runner shows in the
  device app list.
- WDA process launched on the device.
   Both `go-ios runwda` and `tidevice xctest` launched the runner (correct
  vet `SignIdentity`,
   a real pid),
   over wifi and again over USB.
   go-ios sees the wireless device over usbmux
  (`deviceList:[...]`),
   so even the wireless transport reaches it.

The one ungated step:
 the XCTest test-host (testmanagerd) session never engages,
 so WDA's HTTP server never
binds 8100.
 The runner process launches then exits within seconds with no "ServerURLHere" line and an empty
`/status`,
 identically under both go-ios and tidevice (two independent launchers failing the same way points
away from the launcher).
 Appium's own paths confirm the cause from the other side:

- `usePreinstalledWDA` is rejected outright:
   "only supported on iOS/tvOS 17.0 and newer ... WebDriverAgent v13
  no longer uses the legacy XCTest launch path that was required on iOS 16 and below.
  " The iPhone X is 16.7.
- With the device wired (owner connected USB,
   which made it visible to `xcodebuild`/`xctrace` for the first
  time,
   it is invisible to them over wifi),
   `xcodebuild test-without-building` failed with
  `DVTDeviceOperation: Encountered a build number "" that is incompatible with DVTBuildVersion` and
  `Cannot test target "WebDriverAgentRunner" on "iPhone X": Logic Testing Unavailable`.

Root cause (a toolchain-versus-OS gap,
 not a missing image):
 the installed Xcode is 26,
 whose XCTest and
testmanagerd support targets current iOS and cannot stand up a test session against the A11 device's iOS 16.7
(the empty build number and "Logic Testing Unavailable").
 It is not the device-support image:
 16.4 is the last
per-version DeveloperDiskImage Apple ships (both Xcode 26 and Xcode 15.2 carry device support only to 16.4,
 and
all 16.
x devices reuse the 16.4 image),
 and that image is enough for plain app debug,
 which is why every render
gate and the WDA process-launch succeed.
 Appium's own `usePreinstalledWDA` confirms the other half:
 its WDA v13
"no longer uses the legacy XCTest launch path that was required on iOS 16 and below",
 so the current Appium
toolchain has dropped iOS 16 entirely.

Close-out attempted and stopped:
 the standard fix is to drive the device from an Xcode that natively supported
iOS 16.7 (Xcode 14.3.1 to 15.2).
 Xcode 15.2 was downloaded and expanded,
 but using it needs its iOS platform
components installed through a first-launch step that requires sudo and a GUI (`xcodebuild` against it reports
`iOS 17.2 is not installed. To use with Xcode, first download and install the platform`),
 and it would also need
an iOS-16-compatible WebDriverAgent (the bundled v13 dropped iOS 16).
 At that point the owner judged the
remaining yak-shave (sudo first-launch,
 an older WDA build,
 the daemon/`xcode-select` switch) not worth it and
stopped.
 The signing,
 provisioning,
 build,
 install,
 and launch are all proven on the real device above;
 what
remains is purely host-toolchain plumbing for a four-major-version-old OS.

So the on-device black-box drive is recorded as an environment limitation,
 not closed.
 The simulator leg fully
covers the addressability matrix,
 both external drivers,
 and the WDA substrate for every rendering model,
 and
the device's rendering of all 18 frameworks is already proven separately in `device-gate-results.md`;
 the only
thing not demonstrated on the physical device is the black-box automation HTTP session,
 blocked uniformly by the
Xcode-26-cannot-test-iOS-16.7 gap.

## 5. Comparison and when to prefer each

- XCUITest/WebDriverAgent (the primitive):
   use directly when you want the Apple-native engine with no extra
  driver,
   or when you are writing in-process `XCUIApplication` tests compiled into the app (the unit/component
  layer).
   It is what Appium and Maestro install and talk to,
   so proving it on a target proves the substrate for
  both.
- Maestro (the declarative peer,
   recommended default for flows):
   one binary,
   YAML flows,
   implicit waits,
   matches
  visible labels and accessibility ids directly,
   closest to the repo's Playwright-for-web ergonomics.
   Needs a
  JDK.
   The caution is iOS real-device maturity,
   lighter than its simulator and Android paths.
- Appium + xcuitest-driver (the programmable peer):
   W3C/WebDriver protocol,
   any client language,
   full
  programmatic control,
   and the route into existing Selenium/Appium grids and device clouds.
   Heavier (server
  plus client,
   a WDA build per device),
   and on a free-team device it inherits the WDA code-signing problem.
  Prefer it when you need the protocol,
   a non-JVM client,
   or a grid.

Black-box (Maestro/Appium) versus in-process (`XCUIApplication`,
 `flutter_test`,
 `runComposeUiTest`):
 prefer
in-process for asserting on framework semantics directly with millisecond feedback and time control;
 prefer
black-box for true system e2e (permission dialogs,
 deep links,
 the actually-installed release artifact,
cross-framework screens).
 They are complementary layers,
 as on Android.

## 6. Reproducibility notes (iOS-specific friction found during the run)

- WebDriverAgent build on first session:
   the first Appium simulator session triggers an `xcodebuild` of WDA
  (a few minutes),
   then it is cached.
   `appium:usePrebuiltWDA` must not be set on that first run,
   or Appium tries
  to connect to a WDA that does not exist yet (ECONNREFUSED 127.0.0.1:8100).
- Shared bundle id:
   because the gate apps reuse one bundle id,
   drive each one by `simctl uninstall`/`install`
  then attach by bundle id with `forceAppLaunch`;
   do not rely on Appium's `app`-path install with `noReset`.
- External-volume TCC prompt:
   when a tool under `xcrun simctl spawn`/`xcodebuild` reads a binary on
  `/Volumes/MacData`,
   macOS raises a one-time GUI "wants to access files on an external volume" prompt that
  blocks the run until approved.
   It is a GUI approval,
   not a headless one.
   Keep automation binaries that
  `simctl` launches on the internal disk where practical,
   or approve once.

## 7. Cleanup and residue

- Appium installs/uninstalls WDA on the simulator per session;
   nothing framework-specific is left installed
  beyond the last driven gate app under the shared id.
- Maestro left no driver residue on the simulator.
- The Appium server process and the WDA simulator build are kept for the device pass;
   the captured trees and the
  client scripts live under `/Volumes/MacData/ios-vet/` and `~/ios-vet/`.
   No repo state was mutated by the run.

## Bottom line

The iOS black-box automation layer is real and matches the Android depth on the simulator:
 WebDriverAgent/XCUITest
is the primitive,
 Appium 3.5.0 and Maestro 2.6.1 both wrap it and both drove a Flutter counter to "2".
 The
finding that decides framework fit is addressability:
 WebView and native-UIKit survivors project real control
roles automatically,
 the self-drawn class needs its own bridge,
 and the dumps show Flutter's bridge is rich
(Header/Button roles) while Compose MP's is confirmed for text and untested for control roles.
 Real-device WDA
signing is the one piece the simulator does not exercise and is recorded in the device section.
