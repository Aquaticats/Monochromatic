# iOS source-audit: Quasar

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Verbatim structured output from the 2026-06-12 `ios-framework-desk-audit` fan-out (16 parallel
source audits against the three iOS platform walls plus the kopia/music-player functional
requirements),
 with an adversarial cite-check.
 Not lint-conformed.
 On-device gate results,
 where
run,
 live in `device-gate-results.md`;
 the cross-cutting decision is in
`../ios-iphone-x-music-player-kopia-stack.md`.

## Verdict

- iOS runtime model:
   Split model.
   The Quasar app's UI is a Vue 3 single-page app
   that runs as JavaScript inside an out-of-process WKWebView
   (Apple JavaScriptCore + JIT in the WebKit "Web Content Process"),
   not in the app process.
   Quasar ships no native iOS runtime of its own:
   its iOS build mode scaffolds a standard Capacitor (`@capacitor/ios`,
   preferred) or Cordova (`cordova-plugin-wkwebview-engine`,
   legacy) Xcode project and delegates to `cap` + `xcrun xcodebuild` (app-vite/lib/modes/capacitor/capacitor-builder.
  js #buildIos).
   The native app-process code is AOT-compiled Swift/ObjC (Capacitor's CAPBridge host plus any native plugins).
   So app code executes in two tiers:
   AOT-native Swift/ObjC in the app process,
   and JIT-compiled JS in the separate WKWebView content process.
- Minimum iOS deployment:
   iOS 15.0 (Capacitor's stated minimum for current major:
   capacitorjs.
  com/docs/ios requires iOS 15+ and Xcode 14+).
   The iPhone X gate device runs iOS up to 16.
  x,
   so it satisfies this.
   cpal's iOS RemoteIO/AVAudioSession backend imposes no higher floor.
- Gate expectation:
   expected-pass
- Confidence:
   high
- Key finding:
   Quasar adds no iOS runtime:
   it is a Vue-3-in-WKWebView UI layer that scaffolds a standard Capacitor/Cordova Xcode shell,
   so every wall is decided by the Capacitor substrate,
   not by Quasar.
   Wall 2 is structurally satisfied (UI JS JITs in the out-of-process WKWebView content process,
   app-process code is AOT Swift/ObjC),
   and wall 1 plus all four functional requirements (kopia staticlib,
   in-app HTTP server,
   HTTPS/background transfer,
   and the music-player's symphonia+cpal Rust core,
   which cpal 0.18 outputs to iOS via a real RemoteIO/AVAudioSession CoreAudio backend) collapse to a single mechanism:
   a native Capacitor Swift plugin linking a C/Rust/Go static lib via CocoaPods/SPM and bridging to JS.
   The cost is a double bridge (JS<->Swift<->Rust) and a full Slint-to-Vue UI rewrite for the player,
   but no audio rewrite onto AVAudioEngine.

## Wall 2: JIT / executable memory

Verdict:
 wkwebview-jit

Quasar's UI logic is JavaScript,
 and it relies on JIT,
 but that JIT runs in the out-of-process WKWebView Web Content Process,
 which is the one process Apple grants the dynamic-codesigning (JIT) entitlement.
 No JIT or W^X violation occurs in the app process:
 Capacitor's host and any native plugins are AOT Swift/ObjC.
 This is the iOS-favorable mirror image of the NativeScript-on-Android DENY_EXECMEM problem;
 here the scripted UI is structurally quarantined in the allowed process.
 WKWebView's multi-process architecture (separate Web Content Process running the JS engine + JIT,
 IPC to the app process) is documented by Apple,
 and Capacitor explicitly uses WKWebView (not UIWebView).

Source:
 Apple WKWebView docs (developer.
apple.
com/documentation/webkit/wkwebview:
 out-of-process Web Content Process runs JS JIT);
 capacitorjs.
com/docs/ios ('Capacitor uses WKWebView,
 not the deprecated UIWebView');
 /tmp/agent/quasar-audit-20260612/app-vite/lib/modes/cordova/config-file.
js:
177 (cordova-plugin-wkwebview-engine / CDVWKWebViewEngine.
m)

## Wall 1: link and call a Go/Rust static library

Feasible:
 yes

Mechanism:
 Not provided by Quasar;
 inherited from the Capacitor substrate.
 A Capacitor iOS plugin is a Swift class extending CAPPlugin + CAPBridgedPlugin
 that registers methods (pluginMethods array of CAPPluginMethod) exposed to JS over the CAPBridge (CAPPluginCall.
resolve/reject).
 That Swift plugin links a Rust/Go/C static library
 or XCFramework as a normal Xcode dependency declared via CocoaPods (.
podspec s.
dependency) or Swift Package Manager (Package.
swift),
 then calls it through the Swift/ObjC -> C FFI.
 kopia ships as a gomobile c-archive .
xcframework (or a Rust staticlib);
 the plugin calls it and surfaces results to the Vue UI over the JS bridge.
 Note this is a two-hop path:
 JS <-> Swift (Capacitor bridge) and Swift <-> Rust/Go (C FFI).

Source:
 capacitorjs.
com/docs/plugins/ios (CAPPlugin/CAPBridgedPlugin,
 CAPPluginMethod,
 CocoaPods .
podspec / SPM Package.
swift dependency linking of C/Rust/Go static libs and XCFrameworks);
 /tmp/agent/quasar-audit-20260612/app-vite/lib/modes/capacitor/capacitor-builder.
js (xcodebuild of the generated App.
xcworkspace)

## Wall 3: background execution

Inherited from the Capacitor/iOS substrate;
 no long-running foreground service exists.
 A multi-hour kopia snapshot cannot run as a continuous task
 and must be restructured around background URLSession
 (out-of-process uploads/downloads to pCloud that survive suspension) plus BGProcessingTask (short,
 idle/charging windows) driven from a native Capacitor plugin.
 The WKWebView/JS layer cannot keep work alive in the background;
 the transfer must live in the Swift plugin calling URLSession background sessions and BGTaskScheduler.
 Quasar contributes nothing here and adds nothing that changes it.

Source:
 Apple iOS background execution model (URLSession background configuration + BGTaskScheduler/BGProcessingTask);
 Capacitor native-plugin mechanism per capacitorjs.
com/docs/plugins/ios.
 No Quasar source touches background execution (grep of app-vite found only WKWebView/build delegation).

## In-app HTTP server (kopia S3 target)

Feasible:
 yes

Mechanism:
 Not in the WebView;
 in a native Capacitor plugin.
 The in-app S3/HTTP endpoint kopia targets is run by Swift/ObjC native code
 (e.g. an embedded HTTP server library or the Go/Rust static lib's own listener bound to 127.0.0.1) inside the app
 process,
 exposed to JS only for control.
 The Vue UI cannot host a socket server itself;
 it orchestrates the native plugin over the CAPBridge.

Source:
 capacitorjs.
com/docs/plugins/ios (native plugin runs arbitrary Swift/ObjC + linked C/Rust/Go code in-process);
 substrate capability,
 not a Quasar feature

## HTTPS streaming client (pCloud)

Feasible:
 yes

Mechanism:
 Two valid paths,
 both via the substrate.
 (1) Native:
 a Capacitor plugin uses URLSession (incl.
 background URLSession for the streaming/resumable transfer to pCloud),
 which is the correct path for multi-GB streamed uploads.
 (2) WebView:
 fetch/XHR/WebSocket from the Vue layer works for foreground requests but cannot stream in the background
 and is subject to WKWebView networking.
 For kopia-to-pCloud the native URLSession path is required.
 Note Quasar's Cordova mode injects a CDVWKWebViewEngine TLS-trust override (accepts server trust unconditionally),
 which is a security smell to avoid in the production HTTPS-to-pCloud path.

Source:
 /tmp/agent/quasar-audit-20260612/app-vite/lib/modes/cordova/config-file.
js:
160-210 (#prepareWkWebEngine injects didReceiveAuthenticationChallenge accepting serverTrust);
 capacitorjs.
com/docs/plugins/ios (native plugin -> URLSession)

## Audio (music-player port)

Decode:
 symphonia 0.6 (all features) + libopus decode runs unchanged as Rust inside a native plugin's linked static lib;
 pure CPU/no platform API,
 reused via FFI.

Output:
 cpal 0.18 has a real iOS CoreAudio backend that uses RemoteIO Audio Units + AVAudioSession for output.
 The output stream is driven by RemoteIO AudioUnit render callbacks;
 the host module is cfg-gated for non-macOS Apple targets (iOS/tvOS).
 So cpal output is reused on iOS via FFI;
 no AVAudioEngine rewrite is required for audio output.
 Low-latency output is achievable through RemoteIO.
 Always-on true-peak normalization + on-disk peak cache are pure Rust and reuse unchanged.

Rust core reuse:
 Reused via FFI,
 NOT rewritten on AVAudioEngine.
 The entire Rust core (symphonia decode + libopus + true-peak normalization + peak cache + cpal RemoteIO output) is
 kept behind a Capacitor native plugin and called through Swift<->Rust C FFI.
 What does NOT carry over is the Slint UI:
 the player's UI is a from-scratch rewrite in Vue/Quasar,
 and the Rust core is reached through two bridges (JS<->Swift Capacitor bridge,
 then Swift<->Rust C FFI).
 So 'reuse' means the audio/business core reuses;
 the app itself is a UI rewrite,
 not a port of the Slint app.
 The Slint master-rev pin with winit/femtovg/software renderers is irrelevant on this stack
 since rendering becomes WKWebView/Vue.

Source:
 /tmp/agent/cpal-audit-20260612/src/host/coreaudio/ios/`mod.rs`:
1 ('CoreAudio implementation for iOS using AVAudioSession and RemoteIO Audio Units';
 objc2_avf_audio:
:
AVAudioSession;
 kAudioOutputUnitProperty_EnableIO);
 /tmp/agent/cpal-audit-20260612/src/host/coreaudio/`mod.rs`:
12-26 (#[cfg(not(target_os = "macos"))] mod ios);
 /tmp/agent/cpal-audit-20260612/`Cargo.toml`:
3 (version = 0.18.1)

## Gate probe and toolchain

Minimal app:
 scaffold a Quasar Capacitor app (quasar dev/build -m capacitor -T ios),
 add one native Capacitor plugin (Swift CAPPlugin)
 that links a trivial Rust staticlib/XCFramework exporting a C function
 (e.g. returns a number or echoes a string over FFI),
 register one pluginMethod,
 and call it from a Vue page.
 Build the generated App.
xcworkspace with xcodebuild,
 sign,
 install on the iPhone X.
 Exact on-device signal:
 the Vue WebView renders the value returned by the Rust FFI round-trip
 (JS -> CAPBridge -> Swift -> Rust staticlib -> back),
 proving wall 1 (link + call a native static lib from app code) on a real device.
 Wall 2 needs no probe:
 the UI JS JITs in the WKWebView content process by construction.
 Optional second probe:
 have the plugin start a 127.0.0.1 HTTP listener and a background URLSession to confirm the kopia-app substrate paths.

Toolchain:
 macOS with Xcode (>= 14,
 iOS 15+ SDK per Capacitor) + Command Line Tools;
 CocoaPods (and/or SPM,
 built into Xcode);
 Node.
js + a package manager (pnpm/npm) with @quasar/cli and @quasar/app-vite;
 @capacitor/cli + @capacitor/ios;
 Rust toolchain with the aarch64-apple-ios target (rustup target add aarch64-apple-ios) plus cargo-lipo
 or cbindgen / `cargo build --target aarch64-apple-ios` to produce the staticlib/XCFramework;
 for kopia,
 the Go toolchain with gomobile to emit the c-archive .
xcframework.
 A paid/free Apple developer signing identity for on-device install.

## Supporting-stack vets this framework drags in

- Capacitor iOS substrate vet (CAPBridge host,
   plugin registration,
   app lifecycle) since Quasar contributes no native runtime
- Native plugin authoring + C FFI binding vet:
   Swift CAPPlugin <-> Rust/Go staticlib (cbindgen/gomobile,
   memory ownership across the JS<->Swift<->Rust double bridge,
   threading off the JS thread)
- In-app local HTTP/S3 server inside a native plugin (kopia target endpoint)
- HTTPS streaming + background URLSession + BGProcessingTask transfer to pCloud (multi-hour snapshot restructuring;
   avoid the Cordova-mode TLS-trust override)
- Audio vet:
   symphonia/libopus decode + true-peak normalization Rust core behind FFI,
   cpal RemoteIO output latency/glitch behavior on device,
   AVAudioSession category/interruption handling
- QA vets:
   in-process WebView UI test (Vue Test Utils / Vitest at unit,
   Cypress/Playwright or Capacitor's WebDriver for e2e on device),
   native-plugin Swift unit tests (XCTest),
   Rust core property/fuzz/mutation tests at the FFI boundary,
   and adversarial boundary tests for the JS<->native marshaling (CAPPluginCall getString/getObject)
   and the in-app HTTP endpoint

## Cited sources

- Quasar's iOS build delegates to Capacitor/Cordova and xcodebuild;
   ships no native iOS runtime:
   /tmp/agent/quasar-audit-20260612/app-vite/lib/modes/capacitor/capacitor-builder.
  js (#buildIos runs xcrun xcodebuild on the generated App.
  xcworkspace) and app-vite/lib/modes/capacitor/capacitor-installation.
  js (cap add ios)
- Quasar is a Vue 3 component/UI framework:
   /tmp/agent/quasar-audit-20260612/ui/`package.json`:
  14,92 (vue ^3.5.38)
- iOS web layer is WKWebView:
   capacitorjs.
  com/docs/ios ('Capacitor uses WKWebView,
   not the deprecated UIWebView');
   /tmp/agent/quasar-audit-20260612/app-vite/lib/modes/cordova/config-file.
  js:
  175-205 (cordova-plugin-wkwebview-engine,
   CDVWKWebViewEngine.
  m)
- WKWebView runs JS + JIT in a separate out-of-process Web Content Process holding the dynamic-codesigning entitlement:
   developer.
  apple.
  com/documentation/webkit/wkwebview (multi-process architecture;
   JS engine + JIT in the Web Content Process,
   IPC to app process)
- Native plugin = Swift CAPPlugin/CAPBridgedPlugin bridged to JS;
   links C/Rust/Go static libs/XCFrameworks via CocoaPods .
  podspec or SPM Package.
  swift:
   capacitorjs.
  com/docs/plugins/ios
- cpal 0.18 has an iOS CoreAudio output backend using RemoteIO AudioUnit + AVAudioSession,
   cfg-gated for non-macOS Apple targets:
   /tmp/agent/cpal-audit-20260612/src/host/coreaudio/ios/`mod.rs`:
  1;
   src/host/coreaudio/`mod.rs`:
  12-26;
   `Cargo.toml`:
  3 (0.18.1)
- Cordova mode injects an unconditional TLS server-trust acceptance into CDVWKWebViewEngine.
  m (security smell for the pCloud HTTPS path):
   /tmp/agent/quasar-audit-20260612/app-vite/lib/modes/cordova/config-file.
  js:
  160-210 (#prepareWkWebEngine,
   didReceiveAuthenticationChallenge using serverTrust)

## Adversarial cite-check

- Wall 2 confirmed:
   not-confirmed
- Wall 1 confirmed:
   confirmed
- Corrections:
   WALL-2 (load-bearing citation gap):
   The pillar of the "wkwebview-jit" verdict is the parenthetical attributed to Apple's WKWebView class-reference page
   (developer.
  apple.
  com/documentation/webkit/wkwebview),
   namely that the "out-of-process Web Content Process runs JS JIT".
   I rendered the full page (WebFetch returned only the title because the page is a JS-rendered SPA,
   so I captured the rendered DOM text via agent-browser).
   The page contains ZERO occurrences of "out-of-process",
   "Web Content Process",
   "content process",
   "JIT",
   "just-in-time",
   or "separate process".
   The single "entitlement" hit is unrelated (the com.
  apple.
  developer.
  web-browser entitlement for the share-sheet "Add to Home Screen" feature).
   The page only describes WKWebView as "a platform-native view... [that] presents HTML,
   CSS,
   and JavaScript content alongside your app's native views";
   it says nothing about process isolation or where JIT executes.
   Classification:
   NOT-CONFIRMED,
   not contradicted (the page does not assert the reverse;
   it is simply silent on the multi-process/JIT architecture).
   Because the verdict's central mechanism (JIT structurally quarantined in the allowed Web Content Process,
   no JIT/W^X in the app process) rests entirely on this one citation,
   and the other two wall-2 citations only establish peripheral facts (Capacitor uses WKWebView-not-UIWebView;
   the Cordova engine plugin name),
   the wall-2 verdict is not supported by its cited sources as written.
   Locating a valid Apple source that actually documents the WKWebView out-of-process/JIT architecture
   (e.g. a WWDC session or WebKit security doc) was outside the scope of this cite-check;
   I do not assert such a source here,
   only that the CITED page does not support the claim.
   WALL-1 (minor over-specification,
   verdict stands):
   capacitorjs.
  com/docs/plugins/ios supports the core mechanism (a Swift class extending CAPPlugin + CAPBridgedPlugin,
   a pluginMethods array of CAPPluginMethod,
   CAPPluginCall resolve/reject,
   and .
  podspec s.
  dependency / Package.
  swift dependency declaration).
   The page does NOT specifically describe linking C/Rust/Go static libraries or XCFrameworks;
   that is a true,
   standard Xcode/CocoaPods/SPM capability extrapolated by the audit,
   not quoted from the doc.
   This is extrapolation,
   not fabrication,
   and does not flip the wall-1 verdict.
   Repo-path citations both verified accurate:
   config-file.
  js:
  177 is inside #prepareWkWebEngine and references cordova-plugin-wkwebview-engine / CDVWKWebViewEngine.
  m (though the file's actual purpose is dev-server SSL-cert tampering,
   it does confirm Quasar's Cordova mode uses the WKWebView engine plugin);
   capacitor-builder.
  js #buildIos runs xcrun xcodebuild -workspace App.
  xcworkspace -scheme App.
   The iosRuntimeModel claim that the iOS mode scaffolds a @capacitor/ios project is further grounded by
   capacitor-installation.
  js:
  131 installing @capacitor/${target}.
- Sources checked:
   /tmp/agent/quasar-audit-20260612/app-vite/lib/modes/cordova/config-file.
  js (line 177 / #prepareWkWebEngine:
   confirmed references cordova-plugin-wkwebview-engine and CDVWKWebViewEngine.
  m);
   /tmp/agent/quasar-audit-20260612/app-vite/lib/modes/capacitor/capacitor-builder.
  js (#buildIos:
   confirmed xcrun xcodebuild -workspace App.
  xcworkspace -scheme App);
   /tmp/agent/quasar-audit-20260612/app-vite/lib/modes/capacitor/capacitor-installation.
  js (line 131:
   installs @capacitor/${target},
   grounds @capacitor/ios scaffolding);
   <https://capacitorjs.com/docs/ios> (confirmed exact quote:
   'Capacitor uses WKWebView,
   not the deprecated UIWebView');
   <https://capacitorjs.com/docs/plugins/ios> (confirmed CAPPlugin/CAPBridgedPlugin,
   pluginMethods array of CAPPluginMethod,
   resolve/reject,
   .
  podspec s.
  dependency / Package.
  swift;
   does NOT mention C/Rust/Go static libs or XCFrameworks);
   <https://developer.apple.com/documentation/webkit/wkwebview> (rendered full page via agent-browser:
   NO mention of out-of-process / Web Content Process / JIT / just-in-time / separate process;
   load-bearing wall-2 citation NOT supported)
