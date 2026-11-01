# iOS UI/app-shell framework vet for the iPhone X

Status as of 2026-06-13 (fifth handover):
 the testing-infrastructure axis is now run to Android depth,
 not
just enumerated,
 in two new reports under `../decisions/ios-iphone-x-vet-report/` (moved on 2026-07-15 to
`../decision/ios-iphone-x-vet-report/`).
 `vet-ui-automation.md`:
black-box e2e is XCUITest/WebDriverAgent (the iOS primitive,
 the UiAutomator analog) wrapped by Appium 3.5.0
and Maestro 2.6.1;
 both drove a Flutter counter 0 to 2 on the iOS 26.5 simulator,
 and per-rendering-model
accessibility-tree dumps were captured (WebView projects `XCUIElementTypeWebView`;
 native UIKit projects
native labels;
 self-drawn Skia differs,
 Flutter exposes roles like Header/Button,
 Compose MP exposes text).
`vet-test-frameworks.md`:
 Rust unit plus proptest pass on the `aarch64-apple-ios-sim` target (run in the sim
via `simctl spawn`),
 cargo-fuzz ran 34M executions clean,
 cargo-mutants caught 22/24 in a bounded Mac Podman
container (2 equivalent mutants),
 kotlin.
test plus kotest-property pass on `iosSimulatorArm64`,
 and fast-check
property plus shrinking pass for the JS/TS layer;
 the Kotlin/Native mutation gap (PITest is JVM-only) is
recorded,
 Stryker is cited (owner-vetted,
 not re-run).
 On the real iPhone X,
 WebDriverAgent was provisioned
(new app-id,
 headless,
 like the anchor),
 built,
 signed (vet identity),
 installed,
 and launched (go-ios and
tidevice,
 wireless and wired);
 the only ungated step is the XCTest test-host session,
 blocked because the
installed Xcode 26 cannot stand up an XCTest session against iOS 16.7 (a host-toolchain-versus-OS-version gap,
not a missing device-support image:
 16.4 is the last DDI Apple ships and is reused for all 16.
x;
 Appium's WDA
v13 also dropped the iOS-16 launch path).
 This is uniform across every framework,
 so on-device black-box UI
automation is unavailable for all 18 equally on this iPhone-X-plus-Xcode-26 setup,
 not a per-framework result;
the simulator leg covers them all.
 Close-out (driving the device from Xcode 14.3.1 to 15.2,
 which natively
supported 16.7) was attempted:
 Xcode 15.2 was downloaded and expanded,
 but it needs a sudo-plus-GUI first launch
(`iOS 17.2 is not installed`) and an iOS-16-compatible WDA,
 and the owner stopped the pursuit there.
 Work ran
serially on the one Mac (single device,
 keychain,
 simulator);
 mutation ran in the Mac's own Podman per owner
directive.
 Runbook updated with the WebDriverAgent device procedure and this limitation.

Status as of 2026-06-12 (fourth handover):
 signing path DONE;
 source-audit fan-out DONE (16 reports +
synthesis);
 device gating COMPLETE (18 frameworks render-verified both legs,
 Slint disqualified and Qt
culled,
 20 examined:
 the 16-framework source-audit set minus Slint and Qt,
 plus the 4 owner-appended gates);
 the stage-2 capabilities the synthesis flagged as uncertain are now device-proven framework-independently in
one linked Rust staticlib on the iPhone X (an in-app HTTP/S3 loopback server,
 cpal CoreAudio output with
silence only,
 and ring's TLS crypto via X25519 + AES-GCM for the HTTPS-to-pCloud path),
 one render reads
"Rust FFI:
 720 / S3 SERVER OK / AUDIO OK / CRYPTO OK",
 see `device-gate-results.md` "Stage 2
supporting-stack probes";
 the
second-pass language-minimization ranking (directive 4) and the accessibility-posture consolidation are
written into the synthesis doc;
 the on-device VoiceOver fidelity sweep is the one vetting dimension that
needs the owner plus the device GUI and stays owed (crash-survival a11y is already closed for every survivor
on-device).
 Remaining stage-2 work is integration on proven foundations (kopia as a Go gomobile c-archive,
a live TLS handshake and pCloud transfer once the device has network,
 background `URLSession`/audio
restructuring),
 none blocking the framework choice.
 Render-verified so far:
 Capacitor PASS,
 Flutter PASS,
 and the
entire .
NET trio (the shared Microsoft.
iOS substrate plus MAUI,
 Avalonia,
 and Uno,
 each gated as an
actual framework,
 not just on the substrate).
 Slint DISQUALIFIED (decisive finding,
 below).
 The whole
gate-and-render-verify chain also works over wireless,
 so no USB cable is needed.
 Compose Multiplatform
also PASS (Kotlin/Native AOT,
 Skiko/Metal),
 and React Native FULL PASS (render + Hermes + dual-target +
Rust crossing;
 CocoaPods proven;
 a legacy Obj-C `RCTBridgeModule` surfaced the Rust value even under the
New Architecture,
 so no C++/JSI TurboModule was needed).
 NativeScript also FULL PASS on both legs:
 jitless
V8 10.3.22 runs on the iPhone X with no AMFI/execmem kill (the iOS inverse of its Android DENY_EXECMEM
death),
 and the Rust crossing renders "Rust:
 720 / CROSSING OK" with zero hand-written native code (a
C-ABI header + modulemap in `App_Resources/iOS/src` + a `-u` linker flag,
 the cleanest crossing yet).
Lynx also FULL PASS on both legs:
 native UIKit `LynxView` (header-verified `: UIView`,
 no WKWebView),
 a
jitless engine (PrimJS default / JSC fallback,
 both jitless on a non-entitled device) runs the bundle on
the iPhone X with no AMFI/execmem kill,
 and the Rust crossing renders "Rust:
 720 / CROSSING OK" on both
legs via a pure-Objective-C `LynxModule` `.m` shim (not the `.mm` originally assumed).
 Three Lynx-specific
build snags recorded in `device-gate-results.md`:
 per-file `-Werror` vs Xcode 26's `-Wc99-designator`
(Podfile `post_install` appends `-Wno-error`);
 xcodegen's `framework:` dependency does not link a
static-lib xcframework through CocoaPods (link the Rust lib via the `rust-gate` pod instead);
 and the
Xcode 16+ debug-dylib split means an `nm` check on a Debug build must target `*.debug.dylib`,
 not the thin
launcher.
 Qt CULLED 2026-06-12:
 no prebuilt arm64-iphonesimulator slice in any Qt version (device arm64
plus simulator x86_64 only;
 `lipo`/`otool` on 6.5.3 and 6.12.0,
 corroborated by QTBUG-101276 Open and the
Qt iOS docs),
 so it cannot render on the M1's native arm64 iOS 26.5 simulator;
 the only arm64-sim path is
a from-source Qt build that independently breaks the no-hand-written-C++ rule (sole proven iOS CXX-Qt path
uses a developer-authored C++ `main.cpp` on unmerged fork patches).
 Fails two owner hard rules;
 details in
`device-gate-results.md`.
 Dioxus also FULL PASS both legs (first appended gate,
 2026-06-12:
 Dioxus 0.7.9 via
the `dx` CLI,
 Rust UI rendering through wry/WKWebView,
 dual-target structurally clean because it is pure
Rust per-target,
 the anti-Qt;
 a Rust-computed value renders with no FFI shim;
 WebKit-native a11y).
 SnapKit
also FULL PASS both legs (second appended gate,
 2026-06-12:
 pure-Swift UIKit Auto Layout DSL via SPM,
 the
first SPM dependency gate;
 centered layout renders,
 native UIKit a11y).
 UIKit and SwiftUI also FULL PASS
both legs (third and fourth appended gates,
 2026-06-12:
 pure-Swift native baselines,
 both render-verified;
SwiftUI is the HelloDevice canary's framework formally re-confirmed).
 The deferred WKWebView block is now
also DONE (2026-06-12):
 Cordova FULL PASS both legs (cordova-ios 8,
 SPM,
 the second WKWebView substrate),
and the four UI layers Ionic (`@ionic/core` 8.8.10,
 ESM web components upgraded under the `app://localhost`
scheme),
 Framework7 (9.0.5,
 iOS theme),
 Onsen UI (2.12.8),
 and Quasar (2.20.0 + Vue 3.5.38,
 Material via the
UMD build) all FULL PASS both legs,
 each rendering its own real components (not just text) in the proven
Cordova/WKWebView shell with its npm distributable bundled in-app and 6!
=720 computed in page JS.
 So the
ENTIRE initial framework vet is now complete:
 18 frameworks render-verified both legs (the 16-framework
source-audit set minus Slint and Qt,
 plus the 4 owner-appended),
 Slint disqualified,
 Qt culled.
 The pure-Swift gates (UIKit,
 SwiftUI,
 SnapKit) are baseline-only under the allowed-language rule.
What remains is not framework gating but the survivors' work:
 the second-pass language ranking (directive
4),
 the stage-2 supporting-stack vets,
 and the on-device VoiceOver a11y sweep.
 Owner constraints (2026-06-12):
 (a) no C or C++
anywhere,
 even experiments;
 Rust-crossing checks use Rust binding crates or a C-ABI/Swift boundary,
 and
Qt is driven from Rust bindings (CXX-Qt/qmetaobject-rs);
 (b) every framework must render on BOTH the
device and the latest iOS simulator (iOS 26.5) from one codebase.
 Retroactive sweep complete:
 Compose,
Capacitor,
 Flutter,
 and the full .
NET trio all cleared both legs.
 Rust-linking gates must build the Rust
core for both `aarch64-apple-ios` and `aarch64-apple-ios-sim` and link by RID (sim-link finding).

## Goal and hard constraints

Vet iOS UI/app-shell frameworks on the owner's real iPhone X to the depth of the Android series
(`doc/decision/kotlin-android-kopia-pcloud-stack.md`):
 clone,
 source-audit,
 build,
 and run on the
device,
 not judged from metadata.
 Two apps define the requirements:
 the kopia-to-pCloud backup app
(in-app S3/HTTP server,
 streaming HTTPS to pCloud,
 background transfer,
 kopia as a linked static lib)
and `package/music-player/desktop-app` (Rust + Slint,
 to port to iOS).

Owner-stated hard constraints (these decide outcomes):

- Accessibility (a11y) is mandatory.
   A framework whose a11y needs an iOS-17 API is out.
- Implementation languages are constrained to Kotlin,
   TypeScript,
   and Rust (owner,
   2026-06-12).
   After the
  vet completes,
   the chosen framework and the eventual iOS port must minimize Swift and every other
  language outside that allowed set (Objective-C `.m` shims,
   C/C++,
   and so on).
   This is a ranking axis on
  the final recommendation,
   not a vet gate:
   all frameworks are still vetted for completeness,
   but a
  framework whose app code is authored in an allowed language is preferred over one that forces a
  non-allowed language.
   Concretely:
   Dioxus (app code is Rust) and Compose Multiplatform (Kotlin) and the
  WebView/JS shells (Capacitor,
   React Native,
   NativeScript,
   Lynx in TypeScript) align;
   the pure-Swift
  frameworks (UIKit,
   SwiftUI,
   SnapKit) do not,
   so they are baseline-only,
   not implementation candidates.
  The thin Objective-C `.m` shims used for the React Native,
   NativeScript,
   and Lynx Rust crossings are also
  non-allowed-language deviations to minimize;
   Dioxus needs none (pure Rust),
   which is a point in its
  favour for the music-player port (whose Slint UI would be replaced by a Rust UI keeping the Rust core).
- The target is the iPhone X specifically:
   `iPhone10,3`,
   iOS 16.7.16,
   UDID
  `9057e2a8c2e70162e35b9ea8bf006f736670877b`.
   It is A11 silicon and never receives iOS 17,
   so anything
  that depends on an iOS-17 API cannot run here even if it builds.
- The Mac's internal SSD is brittle and easily filled (about 50 GiB free in a shared APFS container).
   Do
  every heavy install and build on the `MacData` volume (`/Volumes/MacData`,
   roughly 330 GiB free),
   NOT
  the internal disk.
   Only files and directories at or under about 1 MB belong on the internal disk.
   See
  "Mac disk layout (MacData)" under DONE for the rule,
   the current symlink layout,
   and the env vars to
  set so each new toolchain (Gradle/Konan,
   Qt,
   Node/CocoaPods,
   cargo) caches onto MacData.

## DONE

### Signing and device path

Full procedure:
 `doc/runbook/ios-iphone-x-codesign-setup.md`.
 Reusable:
 build over `ssh m1` with the
bundle id forced to `dev.monochromatic.iosvet.hellodevice`,
 `DEVELOPMENT_TEAM=HWLVAKDV4F
CODE_SIGN_STYLE=Automatic`,
 and the vet keychain;
 no `-allowProvisioningUpdates`.
 The exact pattern and
the signing details are in `doc/decision/ios-iphone-x-vet-report/device-gate-results.md`.

Trust anchor (do not uninstall):
 all gates reuse the one `hellodevice` bundle id,
 so swapping a gate by
uninstall-then-install leaves the cert with zero apps for a moment,
 which drops the device-wide developer
trust and makes the next launch fail as "not explicitly trusted" (looks like a framework failure,
 is
not).
 A permanent app `dev.monochromatic.iosvet.anchor` ("Vet Anchor",
 built from `~/ios-vet/Anchor`,
same cert `1690CF17`) now stays installed so the cert never reaches zero apps.
 Rules:
 never uninstall the
anchor,
 and swap gate apps with `ideviceinstaller -n upgrade` (in place),
 not uninstall-then-install.
Its first build used `-allowProvisioningUpdates` (over SSH this still reached Apple and minted a new
7-day profile);
 the codesign step required the vet keychain because login.
keychain is unreachable from
SSH.
 Behavior across the 7-day expiry is untested.
 See the runbook's "Keep the developer trust" section.

### Toolchains on the Mac

Installed:
 full Xcode 26.5;
 `ios-deploy`,
 `libimobiledevice` (idevicedebug,
 idevicescreenshot,
idevicecrashreport),
 `ideviceinstaller`,
 `xcodegen` (brew);
 CocoaPods 1.16.2;
 Flutter 3.44.2 plus its
iOS engine artifacts (`flutter precache --ios` done);
 Rust iOS targets (`aarch64-apple-ios`,
 `-sim`).
.
NET fully ready (dotnet 10.0.300):
 the `ios` workload's runtime/AOT packs (the prior session had only
written the manifest;
 `dotnet workload restore` pulled the `osx-arm64.Cross.ios-arm64` AOT cross-compiler
and `Mono.ios-arm64` device runtime),
 the `maui-ios` workload,
 and the `Microsoft.iOS`,
 Avalonia,
 and Uno
project templates (`dotnet new install`).
 Still to install for remaining gates:
 JDK 17+/Kotlin/KMP
(Compose),
 React Native community CLI + Watchman/Metro,
 NativeScript CLI (`ns`) + Homebrew CMake +
xcodeproj gem,
 Lynx (rspeedy/pnpm),
 Qt 6.5 LTS for iOS (qt-unified) + CMake;
 for the appended set,
 the
`dx` (dioxus-cli) CLI (SnapKit/UIKit/SwiftUI need only the already-installed xcodegen + SPM).
 Install all
of these onto MacData,
 not the internal SSD (see next).

### Mac disk layout (MacData)

The internal SSD is brittle and easily filled;
 MacData (`/Volumes/MacData`,
 APFS,
 read-write,
 about
330 GiB free) is the work disk.
 Rule (owner):
 anything larger than about 1 MB lives on MacData;
 only
small files and directories stay on the internal disk.
 Done 2026-06-12 and validated by a clean rebuild
through the symlinks:

- Gate app dirs moved to `/Volumes/MacData/ios-vet/<name>` with a symlink left at `~/ios-vet/<name>`,
   so
  every path in these docs still resolves:
   `capgate`,
   `flgate`,
   `mauigate`,
   `mauiui`,
   `avx`,
   `unogate`,
  `slint-gate`.
   New gate projects:
   create them directly under `/Volumes/MacData/ios-vet/` and symlink
  into `~/ios-vet/` if a stable `~` path is wanted.
- The .
  NET/NuGet caches moved to `/Volumes/MacData/dotnet-cache/` with symlinks left behind:
  `~/.nuget/packages`,
   `~/.dotnet/packs`,
   `~/.dotnet/library-packs`,
   `~/.dotnet/template-packs`.
   A clean
  `dotnet build` restores and AOT-compiles through these symlinks with no change.
- Stayed on the internal disk (all at or under 1 MB,
   or renewal-critical):
   the vet keychain
  (`~/ios-vet/vet.keychain-db`,
   28 KB,
   the only signing secret,
   keep it on the reliable internal disk),
  `accesskit_ios-patched` (128 KB),
   `renew.log`,
   and the `HelloDevice` canary (~75 MB after cleaning its
  stale `build/`).
   HelloDevice is the one deliberate exception to the 1 MB rule:
   the daily
  profile-renewal LaunchAgent builds it,
   and keeping it self-contained on the internal disk means the
  renewal does not break if MacData is unmounted.
   Its renewal derived data (`build-renew/`) regenerates
  there.

Going forward,
 point each new toolchain's cache at MacData before installing,
 so nothing large lands on
internal:
 Gradle `GRADLE_USER_HOME=/Volumes/MacData/gradle`;
 Kotlin/Native `KONAN_DATA_DIR=/Volumes/
MacData/konan`;
 CocoaPods `CP_CACHE_DIR` / `CP_HOME_DIR` under MacData;
 npm/pnpm store and `node_modules`
under MacData;
 cargo `CARGO_HOME=/Volumes/MacData/cargo` for new gate clones;
 Qt installed under
`/Volumes/MacData/Qt`.
 Homebrew itself stays at `/opt/homebrew` (do not relocate brew),
 but prefer
brew formulae that are small;
 large SDK/toolchain payloads go to MacData by the env vars above.

### Source-audit fan-out

A Workflow fan-out (33 agents) source-audited all 16 frameworks against the three iOS walls plus the
functional requirements,
 adversarially cite-checked,
 and synthesized a gate plan.
 Output:
 16
`doc/decision/ios-iphone-x-vet-report/vet-<fw>.md` (verbatim,
 not lint-conformed) and the synthesis
`doc/decision/ios-iphone-x-music-player-kopia-stack.md`.
 Key synthesis results:
 16 frameworks collapse
to 9 distinct gates (WKWebView substrate covers Capacitor/Cordova/Ionic/Framework7/Onsen/Quasar;
 the
.
NET trio MAUI/Avalonia/Uno shares one workload);
 nothing is JIT-fatal on iOS (every in-app JS engine
runs jitless,
 the inverse of Android);
 about 52 deduplicated reports total;
 the shared Rust/Go core is
the reusable spine;
 the in-app HTTP/S3 server is the one genuinely uncertain capability (de-risk:
 embed
it inside the linked staticlib).

### Device gates (render-verified)

Gate criterion is RENDER,
 not launch.
 `ios-deploy ... run` prints `success` once the process is created
under lldb,
 before any UI draws,
 and `--justlaunch` then kills the app on detach.
 To verify a gate:
relaunch with `idevicedebug -d run dev.monochromatic.iosvet.hellodevice` (holds the app alive),
`idevicescreenshot` after about 15 s,
 and scan the app stdout plus `idevicecrashreport` for a dyld
`Symbol not found` or a Rust panic.
 Pull screenshots with `scp m1:/tmp/<f>.png /tmp/` and read them.
This step is what caught the false Slint pass;
 do not skip it for any remaining gate.
 When the device is
attached wirelessly (Xcode wireless debugging),
 add `-n`/`--network` to the libimobiledevice tools
(`idevicedebug -n run`,
 `idevicescreenshot -n`,
 `idevicecrashreport -n`);
 `ios-deploy` uses wifi by
default.
 The whole chain (build,
 install,
 run,
 screenshot,
 crash logs) is confirmed working over wifi.

Dual-target criterion (owner,
 2026-06-12):
 a gate is not a PASS until it render-verifies on BOTH the
iPhone X device (iosArm64,
 iOS 16.7) and the latest iOS simulator (iosSimulatorArm64,
 iPhone 17 Pro /
iOS 26.5 on Xcode 26.5),
 from one codebase with no device-only or simulator-only fork.
 The simulator leg
needs no signing:
 `xcrun simctl boot <sim-udid>` (headless renders offscreen),
 `xcodebuild ... -sdk
iphonesimulator -destination 'platform=iOS Simulator,id=<sim-udid>' CODE_SIGNING_ALLOWED=NO build`,
 then
`xcrun simctl install/launch/io <sim-udid> ... screenshot`.
 CoreSimulator cannot write the screenshot to
the external MacData volume (TCC,
 code 1),
 so write to the Mac's internal `/tmp` and `scp` it off.
 Sim
udid `09D9EB9B-8036-4D23-929D-F75ADE9987FA`.
 The retroactive sweep is complete:
 Capacitor,
 Flutter,
 and
the full .
NET trio have all cleared the simulator leg;
 every remaining gate clears both legs up front.
Two harness findings:
 (1) an Impeller/Metal display-link renderer (Flutter is the proven case) can
screenshot black on a GUI-less `simctl` sim though the process is alive,
 so force the Skia path to capture
(Flutter:
 `FLTEnableImpeller=false`);
 Compose/Skiko-Metal,
 Avalonia-Skia,
 and WKWebView capture fine
headlessly.
 (2) Rust-linking apps must build the Rust core for both `aarch64-apple-ios` and
`aarch64-apple-ios-sim` and link by RID,
 or the sim build fails `ld: building for 'iOS-simulator'`.

- Capacitor (rank 2,
   covers the six WKWebView members):
   PASS on both legs (device + sim),
   renders
  (`Capacitor vet / WKWebView OK`).
   Capacitor 7 uses Swift Package Manager,
   not CocoaPods.
   WebKit gives
  native a11y.
   App at `~/ios-vet/capgate`.
   Simulator:
   same `App.xcodeproj`,
   `-sdk iphonesimulator
  CODE_SIGNING_ALLOWED=NO`,
   `simctl` install/launch on iPhone 17 Pro / iOS 26.5.
- Flutter (rank 4):
   PASS on both legs (device + sim),
   renders the Dart-AOT Release counter UI on device
  and the same counter UI on the simulator.
   Native UIKit a11y.
   App at `~/ios-vet/flgate`
  (`flutter build ios --release --config-only` then xcodebuild on `ios/Runner.xcworkspace` for device;
  `flutter build ios --simulator --debug` for the sim,
   since Flutter has no release/AOT on the simulator).
  Headless gotcha:
   Flutter 3.44 is Impeller-only on iOS,
   and Impeller on a `simctl`-booted sim with no
  GUI presents no frame (black screenshot,
   process alive);
   capture needed `FLTEnableImpeller=false`
  (Skia),
   removed after,
   the app ships Impeller.
   Not a Flutter limit (GUI sim and device render fine).
  Note:
   a plugin-less Flutter app has no Podfile,
   so the CocoaPods path is still unproven (prove it via
  the React Native gate).
- Slint (rank 1):
   DISQUALIFIED (iOS-17 API death).
   See next section.
- Qt (rank 9):
   CULLED (no prebuilt arm64-iphonesimulator slice;
   fails dual-target and the no-C++ rule).
  See the "Remaining gates" list and `device-gate-results.md`.
- .
  NET trio (rank 3,
   MAUI/Avalonia/Uno):
   PASS on both legs (device + sim),
   all four parts gated on both.
  Simulator:
   `dotnet build -c Debug -f net10.0-ios -p:RuntimeIdentifier=iossimulator-arm64`,
   run on
  iPhone 17 Pro / iOS 26.5;
   substrate rendered "Rust FFI returns:
   720",
   MAUI/Avalonia/Uno drew their UIs.
  The substrate exposed the Rust dual-triple rule:
   the device-only `librustgate.a` failed the sim link
  (`ld: building for 'iOS-simulator' ... built for 'iOS'`);
   fixed by building the Rust core for
  `aarch64-apple-ios-sim` too and selecting by RID (a `RustTriple` csproj property),
   device build
  unchanged.
   This is mandatory for every Rust-linking gate and the music-player/kopia cores:
   ship the
  Rust `.a` for both `aarch64-apple-ios` and `aarch64-apple-ios-sim` (XCFramework packages both).
   The shared Microsoft.
  iOS/Mono
  substrate renders in both Release (full AOT) and Debug (`MtouchInterpreter=all`) and P/Invokes a linked
  Rust `.a` (a value computed in Rust,
   6!
  =720 via a heap `Vec`,
   crosses `[DllImport("__Internal")]`),
   no
  JIT/`EXC_BAD_ACCESS`/execmem kill (`~/ios-vet/mauigate`,
   with a `rust/` staticlib).
   Then each framework
  rendered for real:
   MAUI (`~/ios-vet/mauiui`,
   native UIKit handlers),
   Avalonia (`~/ios-vet/avx`,
   its own
  SkiaSharp/Metal renderer),
   Uno (`~/ios-vet/unogate`,
   Uno 6 Skia renderer).
   The substrate gate alone
  would have falsely passed (it runs none of the frameworks' own UI/a11y code,
   exactly where Slint died),
  so each framework was rendered,
   not inferred.
   a11y posture (matters under a11y-must):
   MAUI strongest
  (native UIKit a11y);
   Avalonia and Uno-Skia self-draw via their own a11y bridges,
   fidelity is a stage-2
  check,
   with Uno's native-UIKit renderer as its a11y-safe fallback.
   Full evidence in
  `device-gate-results.md`.
- Compose Multiplatform (rank 5):
   PASS on both legs (device + latest simulator),
   renders.
   Kotlin/Native
  LLVM-AOT static framework,
   Skiko/Metal self-renderer;
   a solid blue UI with white "Compose Gate" text
  drew on the iPhone X (iosArm64,
   held alive,
   no crash) and on the iPhone 17 Pro / iOS 26.5 simulator
  (iosSimulatorArm64,
   signing-free `simctl` build) from the same `iosApp.xcodeproj` + `:shared` sources.
   The working version matrix is Kotlin 2.4.0 / Compose Multiplatform 1.11.1 / Gradle 8.14 on
  Temurin JDK 21 (the template's 1.9.21 / Gradle 8.2.1 fails twice:
   Gradle 8.2.1 will not run on JDK 21,
  and Kotlin/Native gained Xcode 26 support only in 2.2.21).
   App at `/Volumes/MacData/ios-vet/composegate`
  (iOS-only trim of the JetBrains template,
   Android module removed so no Android SDK is needed;
   Konan and
  Gradle caches on MacData).
   a11y is a native `UIAccessibility` bridge (CMP 1.6+,
   iOS 14+),
   Avalonia-class,
  VoiceOver fidelity owed at stage 2.
   In-app server:
   `ktor-server-cio:3.5.0` publishes an iosArm64 variant
  (so `embeddedServer(CIO)` compiles for the device;
   plain HTTP only,
   no native HTTPS);
   building it and a
  Rust `.a` cinterop are stage-2 checks.
   A gate-methodology gotcha surfaced here:
   the first build
  screenshotted all-black because bare Material text on Compose's dark default canvas is invisible;
   force
  an explicit background for screenshot gates (recorded in `device-gate-results.md`).
- React Native (rank 6):
   FULL PASS on both legs (render + Hermes + dual-target + Rust crossing).
   RN 0.86.0
  app at `/Volumes/MacData/ios-vet/RnGate`.
   The render+Hermes half renders blue "RN Gate / Hermes:
   ON" on
  the iPhone X (Release,
   signed,
   upgrade-installed) and on iPhone 17 Pro / iOS 26.5
  (`-sdk iphonesimulator CODE_SIGNING_ALLOWED=NO`) from one `.xcworkspace` + `App.tsx`.
   Proves CocoaPods
  (Pods/hermes-engine/React.
  framework,
   the path Capacitor-SPM and plugin-less Flutter left unproven).
  Hermes bytecode VM live in Release (no Metro at launch,
   iOS-legal);
   `ios/.xcode.env.local` pins
  `NODE_BINARY` to the real mise node so the build phase resolves it over SSH.
   RN renders native UIViews,
  so a11y is native UIKit (VoiceOver owed).
   Rust crossing (PASS):
   `rust_gate_answer() = 720` (the same
  `.a` the .
  NET gate used) crosses Rust -> Obj-C -> JS and the screen reads "Rust answer:
   720 / CROSSING OK"
  on both legs.
   How:
   the Rust core built for `aarch64-apple-ios` + `aarch64-apple-ios-sim`,
   packed with
  `xcodebuild -create-xcframework` into `RustGate.xcframework`;
   a local pod (`modules/rust-gate`,
  `rust-gate.podspec` with `vendored_frameworks`) autolinked via an app-root `react-native.config.js`
  `dependencies.rust-gate.root`,
   so no pbxproj hand-edit;
   the shim `modules/rust-gate/ios/RustGate.m` is a
  legacy `RCTBridgeModule` returning the C-ABI value via `constantsToExport`.
   Decisive finding:
   RN 0.86
  defaults to the New Architecture (bridgeless),
   yet this legacy Obj-C module still surfaces through the
  interop layer (`NativeModules.RustGate.answer` resolved on both legs) with no `newArchEnabled=false` and
  no C++/JSI TurboModule.
   RN-specific dual-triple caveat:
   a Release sim build links all sim archs,
   so the
  arm64-only XCFramework fails the x86_64 link (`ld: library 'rustgate' not found`);
   fix is
  `ARCHS=arm64 ONLY_ACTIVE_ARCH=YES` on this Apple-silicon Mac (an Intel-Mac sim would need an added
  `x86_64-apple-ios` slice,
   a triple-triple).

## The Slint disqualification (decisive)

Slint builds and the process launches,
 but it crashes before rendering on iOS 16.7.
 Its iOS support is
iOS 17+ in two independent places,
 both verified on the device:

1.  a11y:
     `accesskit_ios` (latest 0.1.1) references four iOS-17 symbols unconditionally,
     no availability
    guard or weak link:
     `UIAccessibilityPriorityHigh`/`Low` and
    `UIAccessibilitySpeechAttributeAnnouncementPriority` (`accesskit_ios/src/event.rs`) and
    `UIAccessibilityTraitToggleButton` (`accesskit_ios/src/node.rs`).
     dyld `Symbol not found:
    _UIAccessibilityPriorityHigh`,
     SIGKILL before UI.
2.  Slint's own winit iOS backend reads dark/light via the iOS-17 `UITrait` API:
    `internal/backends/winit/ios/color_scheme.rs:37` calls `UITraitUserInterfaceStyle::class()` /
    `install_trait_change_observer`.
     Runtime panic:
     `class UITraitUserInterfaceStyle could not be found`.

A local accesskit fork patched past all four a11y symbols (on the Mac at `~/ios-vet/accesskit_ios-patched`,
wired via `[patch.crates-io]` in `~/ios-vet/slint-gate/Cargo.toml`) cleared the dyld wall;
 then wall 2
(Slint's own UITrait code) fired.
 So even forking accesskit is not enough;
 the second wall is in Slint.
Making Slint run needs a maintained downport fork of both accesskit and Slint's winit iOS backend,
 with
an iOS-16 a11y fidelity loss,
 re-verified every Slint bump;
 five distinct iOS-17 API uses surfaced,
 so
more may lurk.
 This is the iOS analog of Slint's Android disqualification.
 Every other candidate uses
native iOS a11y (UIKit/WebKit) and does not hit this wall.
 Full evidence in `device-gate-results.md`.

## Scope and the music-player decision

The owner confirmed (2026-06-12) that the Slint disqualification does NOT narrow the vet scope:
 it just
removes Slint from contention.
 Continue the full funnel with Slint marked out;
 do not settle on one
framework or skip gates.
 Two further owner directives the same day:
 (1) defer the six WKWebView
frameworks to the very end,
 after every native and managed framework;
 (2) append Dioxus,
 SnapKit,
 UIKit,
and SwiftUI to the queue,
 positioned just before that deferred web block.
 The current remaining order is
in "Remaining gates" below.

Separately,
 the music-player UI choice is an open owner decision informed by the gate results:
 maintain
a downported Slint fork (keep the UI,
 accept the fork burden),
 or rewrite the UI in a device-verified
native-a11y framework keeping the Rust audio core via FFI (symphonia/opus/cpal port unchanged;
 cpal has
an iOS CoreAudio backend).
 Device-verified native-a11y options now include both Flutter and MAUI (MAUI
keeps the Rust core via `[DllImport("__Internal")]`,
 proven above),
 and now Dioxus,
 the Rust-native UI
option (FULL PASS both legs,
 below):
 the whole UI is Rust,
 so the music-player audio core would link with no
FFI boundary,
 though Dioxus a11y is WebKit-webview-class (VoiceOver via the web tree),
 not self-drawn
native.
 Not blocking;
 the funnel continues regardless.

## Remaining gates (synthesis order)

Each must be render-verified (not just launched),
 and each adds only its own SDK/CLI on the shared base
(Xcode + signing + `rustup target add aarch64-apple-ios`).

.
NET trio (rank 3),
 Compose Multiplatform (rank 5),
 React Native (rank 6,
 including its Rust crossing),
 and
NativeScript (rank 7,
 including its Rust crossing),
 and Lynx (rank 8,
 including its Rust crossing) are all
DONE (FULL PASS,
 above).
 Qt (rank 9) is CULLED (no prebuilt arm64-iphonesimulator slice in any version,
 so
it cannot render on the M1's native arm64 iOS 26.5 simulator,
 and the only arm64-sim path breaks the no-C++
rule;
 see below and `device-gate-results.md`).
 Dioxus (first appended gate) is DONE (FULL PASS both legs,
below).
 SnapKit,
 UIKit,
 and SwiftUI (second through fourth appended gates) are all DONE (FULL PASS both
legs,
 below);
 the pure-Swift trio is baseline-only under the allowed-language rule.
 The deferred WKWebView
block (Cordova plus Ionic/Framework7/Onsen/Quasar) is also DONE (FULL PASS both legs,
 2026-06-12),
 so the
entire initial framework vet is now complete and no framework gating remains.
 The post-gate survivor work
(second-pass language ranking,
 stage-2 supporting-stack vets,
 VoiceOver a11y sweep) is tracked below.
 The
historical gate order,
 for the record:

Owner constraint (2026-06-12):
 no C or C++ anywhere,
 including throwaway experiments.
 The Rust-crossing
checks below that were written as C++/`.mm` glue must instead use Rust binding crates,
 a C-ABI boundary
(Rust `extern "C"` declared in a Swift bridging header,
 an ABI declaration not hand-written C/C++),
 or
Swift glue.
 The gate question is unchanged (a Rust value crosses in and the UI renders),
 only the FFI
mechanism is constrained.
 ObjC clarification (owner,
 2026-06-12):
 "no C/C++" is the language-name ban,
 so
a minimal Objective-C `.m` shim is allowed purely to register/call Rust over a C ABI where there is no
pure-Swift path (RN old-arch modules,
 likely NativeScript/Lynx);
 thin bridge only,
 no app logic,
 recorded
as a noted deviation;
 `.c`/`.cpp`/`.mm` stay banned and RN New-Arch C++/JSI TurboModules are not used.
Dual-target (2026-06-12):
 each gate below must render on both the device and the latest sim,
 so build the
Rust core for both `aarch64-apple-ios` and `aarch64-apple-ios-sim` and link by RID/SDK (proven via the
.
NET substrate's `RustTriple` fix);
 a single device archive fails the sim link.

- React Native (rank 6):
   FULL PASS,
   DONE (above;
   render + Hermes + dual-target + Rust crossing).
   The Rust
  staticlib reached JS through a thin Obj-C `.m` `RCTBridgeModule` over a C ABI (owner-approved deviation,
  no New-Arch C++/JSI),
   packaged as a dual-triple XCFramework and autolinked via a local pod.
   The legacy
  Obj-C module surfaced under RN's New Architecture through the interop layer,
   so no `newArchEnabled=false`
  and no TurboModule were needed.
   This is the reusable Rust-crossing template for the gates below.
- NativeScript (rank 7,
   needs-device):
   FULL PASS,
   DONE (above).
   Jitless V8 10.3.22 initialized in ~60ms on
  the iPhone X with no AMFI/execmem kill (iOS inverse of its Android DENY_EXECMEM death),
   and the Rust
  crossing renders "V8 JS:
   720 / Rust:
   720 / CROSSING OK" on both legs.
   App at
  `/Volumes/MacData/ios-vet/nsgate` (NS 9.0.6,
   `@nativescript/ios` 9.0.3).
   Build path:
   `ns prepare ios`
  then build the generated `platforms/ios/nsgate.xcworkspace` with the vet-keychain wrapper (not `ns build`,
  whose signing won't thread the keychain);
   `gem install xcodeproj` is required first or prepare aborts
  (exit 127).
   The Rust crossing needed zero hand-written native code,
   but two non-obvious snags (both in
  `device-gate-results.md`):
   (1) NativeScript's metadata generator only exposes a C function that is in a
  clang module,
   so a default static-lib pod header (include-path only) is invisible,
   the fix is a
  `module.modulemap` + header in `App_Resources/iOS/src/`;
   (2) `-dead_strip` drops the static-lib symbol
  that only JS references,
   so the runtime resolves null and asserts (Helpers.
  mm),
   the fix is
  `OTHER_LDFLAGS = -u _rust_gate_answer` in `App_Resources/iOS/build.xcconfig` (a local pod still links the
  dual-triple XCFramework).
   This C-ABI-header-in-a-modulemap pattern is the template for Lynx/Qt.
- Lynx (rank 8,
   expected-pass):
   FULL PASS,
   DONE (above).
   Native UIKit (`LynxView : UIView`,
  header-verified,
   no WKWebView);
   a jitless engine (PrimJS default / JSC fallback,
   both jitless on a
  non-entitled device) runs the bundle on the iPhone X with no AMFI/execmem kill,
   render both legs;
   the
  Rust crossing renders "Rust:
   720 / CROSSING OK" on both legs.
   The native surface is a pure-Objective-C
  `RustGateModule` (`LynxModule` protocol:
   `+name`,
   `+methodLookup`,
   registered via
  `[LynxConfig registerModule:]`,
   called from JS as `NativeModules.RustGateModule.answer()`),
   a `.m` shim
  (not the `.mm` originally assumed),
   linking the Rust staticlib through the `rust-gate` local pod.
   App at
  `/Volumes/MacData/ios-vet/lynxgate` (rspeedy `react-ts` bundle + xcodegen/CocoaPods native app;
   build
  the bundle with `npm run build`,
   then xcodegen `generate` + `pod install`,
   then build the `.xcworkspace`
  with the vet-keychain wrapper).
   Three build snags,
   all in `device-gate-results.md`:
   per-file `-Werror`
  vs Xcode 26 `-Wc99-designator` (Podfile `post_install` appends `-Wno-error` to each per-file
  `COMPILER_FLAGS`);
   xcodegen's `framework:` dep does not link a static-lib xcframework through CocoaPods
  (use the `rust-gate` pod + `-u _rust_gate_answer`);
   and the Xcode 16+ `ENABLE_DEBUG_DYLIB` split means a
  Debug-build `nm` check must target `LynxGate.debug.dylib`,
   not the thin launcher.
- Qt (rank 9,
   needs-device):
   CULLED 2026-06-12.
   No prebuilt Qt kit ships an arm64-iphonesimulator slice
  (device arm64 plus simulator x86_64 only,
   confirmed by `lipo`/`otool` on 6.5.3 and 6.12.0;
   QTBUG-101276
  Open;
   the Qt iOS docs say the simulator libs are x86_64 and must run under Rosetta on Apple Silicon),
   so
  Qt cannot render on the M1's native arm64 iOS 26.5 simulator and fails the dual-target prerequisite.
   The
  only arm64-sim path is a from-source Qt cross-compile,
   which independently breaks the no-hand-written-C++
  rule (the sole proven iOS CXX-Qt path uses a developer-authored C++ `main.cpp` on unmerged fork patches).
  Fails two owner hard rules;
   no gate build attempted.
   Full reasoning in `device-gate-results.md`.
- Owner-appended set (gate after the above,
   before the web block):
   Dioxus DONE (FULL PASS both legs,
  2026-06-12:
   Dioxus 0.7.9 `dx` CLI,
   Rust UI through wry/WKWebView,
   dual-target structurally clean because
  pure Rust per-target,
   Rust-computed value renders with no FFI shim;
   built `dx bundle --ios --target
  <triple>`,
   device leg signed by hand with the vet keychain since `dx`'s deploy assumes USB;
   see
  `device-gate-results.md`).
   SnapKit DONE (FULL PASS both legs,
   2026-06-12:
   pure-Swift UIKit Auto Layout
  DSL via SPM,
   the first SPM dependency gate;
   xcodegen UIKit app,
   device leg signed inline by the runbook
  wrapper;
   see `device-gate-results.md`).
   UIKit DONE (FULL PASS both legs,
   2026-06-12:
   pure-UIKit Swift app,
  native `NSLayoutConstraint`,
   the native baseline).
   SwiftUI DONE (FULL PASS both legs,
   2026-06-12:
   SwiftUI
  App-lifecycle app,
   the HelloDevice canary's framework formally re-confirmed).
   All four appended gates
  done;
   the pure-Swift trio (SnapKit/UIKit/SwiftUI) is baseline-only under the allowed-language rule.
- Deferred web block:
   DONE (2026-06-12,
   FULL PASS both legs).
   Cordova (cordova-ios 8 SPM,
   the second
  WKWebView substrate) plus the Ionic,
   Framework7,
   Onsen,
   and Quasar UI layers,
   each rendered with its real
  npm distributable bundled in the Cordova/WKWebView shell on the iPhone X and the iPhone 17 Pro / iOS 26.5
  sim.
   Details in `device-gate-results.md`.
   This closes the initial framework vet.

After gates:
 stage 2 deep supporting-stack vets (the ~52-report roadmap,
 enumerated inside each
`vet-*.md`).
 Progress so far (2026-06-12,
 in `device-gate-results.md` "Stage 2 supporting-stack probes" and
the synthesis doc's "Stage 2 status"):
 the in-app HTTP/S3 server,
 cpal CoreAudio output,
 and ring's TLS
crypto (X25519 + AES-GCM) are all device-proven in one linked Rust staticlib on the iPhone X (the capabilities
that were genuinely uncertain),
 so the kopia and music-player cores are de-risked on every track.
 The kopia Go
payload was also tested:
 a Go `c-archive` cross-compiles to both iOS triples and its runtime runs on the
iPhone X in a pure-Swift host ("Go:
 720 RUNTIME OK"),
 but co-hosting it inside the .
NET/Mono runtime SIGKILLs
on the device (two managed runtimes collide;
 the runtime-less Rust `.a` did not),
 so kopia's Go payload needs
a non-managed host,
 which rules the .
NET trio out as the kopia host specifically (it stays valid as a
music-player UI).
 Still owed,
 none blocking the framework choice:
 building actual kopia as the c-archive
(integration on the proven mechanism),
 a live TLS handshake and pCloud transfer (need network;
 ring's crypto
core is already proven offline),
 background `URLSession`/`BGProcessingTask` and backgrounded-audio
restructuring,
 the per-framework FFI-marshaling and
UI-test/e2e harnesses,
 and the owner-owed on-device VoiceOver fidelity sweep (the one dimension that needs
the owner plus the GUI;
 crash-survival a11y is already closed for every survivor).
 Keep the synthesis doc
current as these land.

## Mac artifacts (not in the repo)

`~/ios-vet/` holds the layout,
 but most entries are now symlinks to `/Volumes/MacData/ios-vet/` (see
"Mac disk layout" above).
 On the internal disk:
 `vet.keychain-db` (signing,
 28 KB),
 `HelloDevice/`
(canary,
 renewal-critical),
 `accesskit_ios-patched/` (the a11y fork),
 `renew.log`,
 and the renewal
LaunchAgent (`~/Library/LaunchAgents/dev.monochromatic.iosvet.profilerenew.plist`).
 Symlinked to MacData:
`capgate/` (Capacitor),
 `flgate/` (Flutter),
 `slint-gate/` (Slint clone + the `[patch]`'d `Cargo.toml`),
`mauigate/` (.
NET/Microsoft.
iOS substrate + the `rust/` FFI staticlib),
 `mauiui/` (MAUI),
 `avx/`
(Avalonia xplat,
 iOS head `avx.iOS`),
 `unogate/` (Uno).
 The NuGet/.
NET packs are symlinked from
`~/.nuget` and `~/.dotnet` to `/Volumes/MacData/dotnet-cache/`.
 Screenshots and logs land in `/tmp`
(ephemeral).
 The throwaway keychain password is only in the repo `.env.local` as
`XCODE_IDENTITY_SSH_USABLE`.

## Notes / gotchas

- The Mac login shell is zsh:
   it does not word-split unquoted variable expansions,
   so inline xcodebuild
  flags (do not build them in a `$WS` variable).
- `set -e` plus `ios-deploy ... | tail` or a non-zero ios-deploy exit can abort a script before the
  verification step;
   capture full logs and avoid `set -e` around the render check.
- ultracode is ON this session (Workflow orchestration used for the source-audit fan-out).
