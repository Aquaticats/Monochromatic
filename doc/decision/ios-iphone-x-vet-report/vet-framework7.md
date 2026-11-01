# iOS source-audit: Framework7

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
   WKWebView-JIT (out-of-process).
   Framework7 ships zero native code and no JS engine of its own:
   it is a JS/HTML/CSS library ("Full Featured Mobile HTML Framework For Building iOS & Android Apps",
   /tmp/agent/framework7-audit-20260612/`README.md` line 11).
   On an iOS device its app code (the F7 core + Vue/React/Svelte bindings + your UI JS) runs as JavaScript inside the
   host shell's WKWebView,
   executed by WebKit's out-of-process JavaScriptCore.
   F7's only platform contact is feature-detection of the shell:
   window.
  Capacitor / window.
  cordova probing in src/core/shared/get-device.
  js (lines 25-27) and delegation to shell plugins in src/core/components/statusbar/statusbar.
  js (window.
  Capacitor.
  Plugins.
  StatusBar).
   The substrate is Capacitor (or Cordova):
   "Capacitor uses WKWebView,
   not the deprecated UIWebView" (capacitorjs.
  com/docs/ios).
- Minimum iOS deployment:
   Not pinned by Framework7 (it is pure JS/CSS,
   no native deployment target).
   The floor is set by the Capacitor substrate;
   current Capacitor major versions require iOS 14+ (verify against the exact Capacitor version chosen).
   Framework7's own CSS/browserslist target is "IOS >= 18" (`package.json` line 63) for styling,
   but that is a CSS-prefixing target,
   not a runtime deployment floor.
- Gate expectation:
   expected-pass
- Confidence:
   high
- Key finding:
   Framework7 is inert on all three iOS walls:
   it ships zero native code and no JS engine,
   contributing only the WKWebView UI layer.
   Its app code runs as JavaScript in the JIT-blessed,
   out-of-process WKWebView (a clean wall-2 pass,
   the inverse of the NativeScript in-process-V8 failure),
   and the entire iOS decision -- static-lib linking,
   in-app HTTP server,
   background URLSession,
   and the symphonia+cpal audio core -- rests on the Capacitor substrate via its CAPPlugin bridge,
   with the Rust audio core reusable through FFI rather than rewritten on AVAudioEngine.

## Wall 2: JIT / executable memory

Verdict:
 wkwebview-jit

PASS,
 not a problem.
 Framework7 carries no managed/scripted runtime that would need executable memory in the app process:
 it is a JS library with no native or bytecode engine (no Swift/ObjC/.
swift/.
m files anywhere;
 native-file search under /tmp/agent/framework7-audit-20260612 returns none;
 deps are pure-JS dom7/swiper/ssr-window per `package.json`).
 The app's JavaScript executes in the host WKWebView,
 the one iOS process that holds the dynamic-codesigning (JIT) entitlement.
 This is the exact inverse of the in-process-V8/NativeScript DENY_EXECMEM failure:
 F7 has no in-process engine to trip W^X.
 No AOT step and no slow-interpreter fallback are required because the JIT-blessed WebView process owns code execution.

Source:
 /tmp/agent/framework7-audit-20260612/src/core/shared/get-device.
js (probes window.
Capacitor/window.
cordova;
 F7 is a library,
 owns no engine);
 native-file find under repo returns zero .
swift/.
m/.
mm;
 capacitorjs.
com/docs/ios ('Capacitor uses WKWebView,
 not the deprecated UIWebView') for the JIT-entitled WebView process

## Wall 1: link and call a Go/Rust static library

Feasible:
 yes

Mechanism:
 Inherited from the Capacitor substrate,
 not Framework7 itself.
 kopia ships as a Go gomobile c-archive .
xcframework (or Rust staticlib);
 it is linked into the Capacitor-generated Xcode project by editing the plugin .
podspec (CocoaPods) or Package.
swift (SPM).
 A Capacitor plugin (Swift class extending CAPPlugin/CAPBridgedPlugin,
 methods marked @objc,
 registered via pluginMethods/CAPPluginMethod and registerPlugin()) calls the static lib's C/FFI symbols
 and returns results to the WebView through CAPPluginCall.
resolve().
 Framework7's JS UI invokes that plugin like any JS module.
 F7 contributes nothing to the FFI path beyond rendering the UI that triggers it.

Source:
 capacitorjs.
com/docs/plugins/ios ('Capacitor iOS plugins are both CocoaPods and Swift Package Manager libraries... edit the .
podspec for CocoaPods and the Package.
swift for SPM' to add static libraries/xcframeworks;
 CAPPluginCall,
 call.
resolve());
 capacitorjs.
com/docs/ios/custom-code (CAPPlugin + @objc + registerPlugin bridge);
 F7 delegation pattern at /tmp/agent/framework7-audit-20260612/src/core/components/statusbar/statusbar.
js

## Wall 3: background execution

Inherited from the Capacitor substrate;
 Framework7 has no background story of its own (it is paused JS in a backgrounded WebView).
 A multi-hour kopia snapshot cannot be a long-running foreground process on iOS;
 it must be restructured around background URLSession
 (chunked uploads to pCloud that the system continues while suspended) and BGProcessingTask (short,
 idle/charging windows),
 both written as native Swift in the Capacitor Xcode project and surfaced to the F7 UI via a custom plugin.
 This is identical to every other WKWebView candidate:
 the platform ceiling is iOS's,
 the implementation lives in the native shell,
 and F7 only displays progress.

Source:
 capacitorjs.
com/docs/ios/custom-code (full Xcode project access to add native Swift using URLSession/BGProcessingTask,
 exposed via a CAPPlugin);
 F7 contributes only JS UI (/tmp/agent/framework7-audit-20260612/src/core/shared/get-device.
js shows F7's platform surface is JS feature-detection only)

## In-app HTTP server (kopia S3 target)

Feasible:
 yes

Mechanism:
 Substrate capability,
 not Framework7's.
 An in-app local HTTP/S3 endpoint that kopia targets is implemented as native Swift
 (e.g. an embedded NSURLSession/Network.
framework listener,
 or by linking kopia's own server as part of the static lib) inside the Capacitor Xcode project
 and controlled through a CAPPlugin.
 F7's WebView JS cannot itself bind a listening socket,
 but it does not need to:
 the loopback server runs at the native layer and the F7 UI starts/stops it via the plugin bridge.

Source:
 capacitorjs.
com/docs/ios/custom-code (open the Xcode project,
 add custom native Swift,
 expose via CAPPlugin/registerPlugin);
 capacitorjs.
com/docs/plugins/ios (link native/static libraries via .
podspec / Package.
swift)

## HTTPS streaming client (pCloud)

Feasible:
 yes

Mechanism:
 Two inherited paths from the substrate.
 (1) From F7's JS in the WKWebView,
 standard web fetch/streams reach pCloud over HTTPS (WKWebView is a modern WebKit engine).
 (2) For true streaming/background upload that survives app suspension,
 a native Swift URLSession (background configuration) is written in the Capacitor Xcode project and driven
 from the F7 UI through a CAPPlugin.
 Framework7 supplies only the JS calling surface;
 the streaming transport is the shell's.

Source:
 capacitorjs.
com/docs/ios ('Capacitor uses WKWebView' -> modern fetch/streams in the WebView);
 capacitorjs.
com/docs/ios/custom-code (native URLSession via custom Swift + CAPPlugin)

## Audio (music-player port)

Decode:
 symphonia 0.6 (all features) + libopus decode runs in the existing Rust core at the native layer,
 unchanged;
 it never touches the WebView.
 Framework7 contributes none of the decode path.

Output:
 cpal 0.18 drives CoreAudio on the iOS target at the native layer for low-latency output;
 the WKWebView/F7 is not in the audio path.
 Web Audio API in the WebView is the JS-only alternative you do NOT need here because a Rust core already exists.

Rust core reuse:
 REUSE via FFI -- not a rewrite on AVAudioEngine.
 The Rust core (symphonia decode + true-peak normalization + on-disk peak cache + cpal CoreAudio output) compiles to a
 staticlib/xcframework and links into the Capacitor Xcode project exactly like the kopia lib (wall 1).
 cpal already has an iOS/CoreAudio backend,
 so the audio engine stays in Rust.
 A thin Capacitor CAPPlugin bridges transport/queue/session control between the Rust core and Framework7's JS UI;
 F7 renders only the player chrome (transport buttons,
 two-axis paginated queue,
 now-playing).
 AVAudioEngine is only needed if you abandon cpal,
 which is unnecessary.

Source:
 capacitorjs.
com/docs/plugins/ios (.
podspec / Package.
swift to link the Rust staticlib/xcframework);
 capacitorjs.
com/docs/ios/custom-code (CAPPlugin bridge from native Rust-backed Swift to the F7 WebView UI);
 F7 owns only JS UI per /tmp/agent/framework7-audit-20260612/`README.md` and src/core/shared/get-device.
js

## Gate probe and toolchain

Build a Capacitor iOS app whose WebView loads a trivial Framework7 UI (one F7 page with one F7 button),
 plus one custom Capacitor plugin (Swift CAPPlugin)
 that calls a linked Rust/Go staticlib (the FFI round-trip standing in for kopia/symphonia).
 On-device signal:
 tap the Framework7 button on the iPhone X
 and see the value returned across dart-free Swift-to-JS bridge (CAPPluginCall.
resolve) render back inside the WebView.
 Passing proves (a) F7 JS executes in the JIT-blessed WKWebView,
 and (b) the substrate links and calls a native static lib from app code.
 Because F7 itself is inert on every wall,
 this gate is really a Capacitor-substrate gate.

Toolchain:
 macOS with Xcode (+ command-line tools,
 CocoaPods or SPM),
 Node.
js + npm for Capacitor CLI (@capacitor/core,
 @capacitor/cli,
 @capacitor/ios),
 Framework7 npm package,
 and the cross-compilers for the static lib:
 Go with gomobile (for kopia c-archive) and/or the Rust aarch64-apple-ios toolchain
 (rustup target add aarch64-apple-ios) for the music-player core.
 A code-signing identity / provisioning profile for on-device install (already established by the canary).

## Supporting-stack vets this framework drags in

- Capacitor iOS substrate vet (this is the real subject;
   F7 inherits all of it):
   WKWebView host,
   CAPPlugin bridge,
   Xcode project generation
- Native static-lib link + FFI bridge:
   Go gomobile c-archive .
  xcframework for kopia,
   Rust aarch64-apple-ios staticlib for the music-player core,
   surfaced via a Swift CAPPlugin
- In-app loopback HTTP/S3 server in native Swift for kopia to target
- Background transfer:
   native background-URLSession + BGProcessingTask plugin (multi-hour snapshot restructured into chunked,
   suspendable uploads)
- HTTPS streaming client to pCloud (native URLSession streaming bridged to the F7 UI)
- Audio:
   reuse symphonia+cpal Rust core via FFI behind a CAPPlugin (cpal CoreAudio backend),
   F7 renders transport/queue UI only
- QA stack:
   in-process WebView UI test (WKWebView automation / WebDriver-style),
   e2e on device (Appium/XCUITest driving the F7 WebView),
   JS-side property/fuzz tests for the UI logic,
   plus Rust core's own cargo test/fuzz/property tests at the native layer

## Cited sources

- Framework7 is a JS/HTML/CSS UI library for building iOS/Android apps;
   ships no native code:
   /tmp/agent/framework7-audit-20260612/`README.md` line 11
   ('Full Featured Mobile HTML Framework For Building iOS & Android Apps');
   `package.json` dependencies are pure-JS (dom7,
   swiper,
   ssr-window,
   path-to-regexp,
   htm,
   skeleton-elements);
   native-file find returns zero .
  swift/.
  m/.
  mm/.
  xcodeproj/Podfile
- Framework7's only platform contact is JS feature-detection of the shell (Capacitor/Cordova):
   /tmp/agent/framework7-audit-20260612/src/core/shared/get-device.
  js lines 25-27 (cordova:
   !
  !
  window.
  cordova,
   capacitor:
   !
  !
  window.
  Capacitor)
- Framework7 delegates native actions to the shell's JS plugins rather than implementing them:
   /tmp/agent/framework7-audit-20260612/src/core/components/statusbar/statusbar.
  js (window.
  Capacitor.
  Plugins.
  StatusBar.
  hide/show/setStyle;
   device.
  cordova && window.
  StatusBar)
- The substrate is WKWebView on iOS (wall-2 engine):
   capacitorjs.
  com/docs/ios ('Capacitor uses WKWebView,
   not the deprecated UIWebView')
- Custom native Swift/ObjC code is added via CAPPlugin + @objc methods + registerPlugin bridge in the generated Xcode
   project:
   capacitorjs.
  com/docs/ios/custom-code
- Static libraries / xcframeworks (Go c-archive,
   Rust staticlib) link via .
  podspec (CocoaPods) or Package.
  swift (SPM);
   results return to JS via CAPPluginCall.
  resolve():
   capacitorjs.
  com/docs/plugins/ios
- Native iOS APIs (URLSession background transfer,
   AVAudioEngine,
   CoreAudio,
   embedded HTTP server) are reachable by writing native Swift in the Xcode project and exposing via a plugin:
   capacitorjs.
  com/docs/ios/custom-code;
   capacitorjs.
  com/docs/basics/workflow
  ('Opening the native project can give you full control over the native runtime... create plugins,
   add custom native code')

## Adversarial cite-check

- Wall 2 confirmed:
   confirmed
- Wall 1 confirmed:
   confirmed
- Corrections:
   All load-bearing claims check out;
   one precision nuance worth flagging (does NOT flip either verdict).

VERIFIED REPO CLAIMS (all exact):
- `README.md` line 11 contains verbatim "Full Featured Mobile HTML Framework For Building iOS & Android Apps"
   (cited correctly for iosRuntimeModel).
- src/core/shared/get-device.
  js lines 25-27 are exactly:
   line 25 `cordova: !!window.cordova,`,
   line 27 `capacitor: !!window.Capacitor,` (the window.
  Capacitor/window.
  cordova feature-detection probes;
   cited line range 25-27 accurate).
- src/core/components/statusbar/statusbar.
  js delegates to window.
  Capacitor.
  Plugins.
  StatusBar (lines 24/34/86/88/99/110/124) gated by an isCapacitor() check;
   F7-delegation-to-shell-plugin claim supported.
- Native-file search under /tmp/agent/framework7-audit-20260612 returns ZERO .
  swift/.
  m/.
  mm files (confirmed;
   also zero .
  node/.
  wasm/.
  a/.
  dylib/.
  so binaries).
   src/ is JS-only (core + react/vue/svelte bindings + material-color-utilities).
- `package.json` deps are pure-JS:
   dom7,
   swiper,
   ssr-window (all three audit-named ones present and correct),
   plus htm,
   path-to-regexp,
   skeleton-elements.
   None is a native/bytecode engine.
   Supports "no in-process engine.
  "

VERIFIED DOC CLAIMS (all exact quotes confirmed on live pages):
- capacitorjs.
  com/docs/ios contains verbatim "Capacitor uses WKWebView,
   not the deprecated UIWebView.
  "
- capacitorjs.
  com/docs/plugins/ios confirms "Capacitor iOS plugins are both CocoaPods and Swift Package Manager libraries...
  edit the .
  podspec for CocoaPods and the Package.
  swift for SPM",
   plus CAPPluginCall / call.
  resolve() and CAPPlugin/CAPBridgedPlugin/@objc.
- capacitorjs.
  com/docs/ios/custom-code confirms CAPPlugin,
   CAPBridgedPlugin,
   @objc,
   and registerPlugin() (the wall-1 audit cites registerPlugin to this custom-code page,
   so the citation is correctly placed;
   note plugins/ios itself uses pluginMethods rather than registerPlugin,
   but the audit does not rely on plugins/ios for registerPlugin).

PRECISION NUANCE (not a flip,
 not blocking):
 The wall-2 verdict attributes the "out-of-process JavaScriptCore"
 and "WebView process holds the dynamic-codesigning (JIT) entitlement" characterization in a context
 that cites capacitorjs.
com/docs/ios.
 That page only supports "Capacitor uses WKWebView,
 not the deprecated UIWebView" -- it does NOT state the multi-process or JIT-entitlement detail.
 However,
 the architectural fact itself is correct,
 not fabricated:
 WKWebView runs web content in a separate WebKit WebContent process
 which holds the dynamic-codesigning entitlement enabling JavaScriptCore's JIT,
 which is precisely why an in-process engine (V8/NativeScript) trips W^X/DENY_EXECMEM while the WebView does not.
 The audit's "exact inverse" framing is accurate.
 Critically,
 the PASS verdict rests on "F7 ships no in-process engine" (fully verified above),
 not on the JIT-entitlement detail,
 so the citation gap is cosmetic and the verdict stands.
 Team is safe to skip the in-process-engine toolchain for Framework7.
- Sources checked:
   /tmp/agent/framework7-audit-20260612/`README.md` (line 11);
   /tmp/agent/framework7-audit-20260612/src/core/shared/get-device.
  js (lines 25-27);
   /tmp/agent/framework7-audit-20260612/src/core/components/statusbar/statusbar.
  js (window.
  Capacitor.
  Plugins.
  StatusBar delegation);
   /tmp/agent/framework7-audit-20260612/`package.json` (dependencies);
   native-file find under /tmp/agent/framework7-audit-20260612 (zero .
  swift/.
  m/.
  mm + zero native binaries);
   <https://capacitorjs.com/docs/ios> (WKWebView-not-UIWebView quote);
   <https://capacitorjs.com/docs/plugins/ios> (.
  podspec/Package.
  swift,
   CAPPluginCall.
  resolve,
   CAPPlugin/CAPBridgedPlugin/@objc);
   <https://capacitorjs.com/docs/ios/custom-code> (CAPPlugin,
   @objc,
   registerPlugin)
