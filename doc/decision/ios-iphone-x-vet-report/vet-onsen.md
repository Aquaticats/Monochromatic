# iOS source-audit: Onsen UI

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
   App UI code executes as JavaScript on top of Web Components (Custom Elements) inside the host WebView's
   JavaScriptCore engine.
   On iOS the host is Capacitor/Cordova's out-of-process WKWebView WebContent process,
   which holds the dynamic-codesigning (JIT) entitlement;
   the app's own process runs only AOT-native Swift/ObjC (the Cordova/Capacitor plugins).
   Onsen UI ships NO runtime and NO native code of its own;
   it is purely a DOM-manipulation library.
   Source:
   the entire repo is pure JS/CSS built to UMD+ESM via rollup (onsenui/`package.json` main=js/onsenui.
  js,
   module=esm/index.
  js);
   platform.
  js operates only on window/navigator/document/window.
  screen/HTMLElement (onsenui/esm/ons/platform.
  js:
  19,83,98,117) and detects the shell via window.
  cordova (platform.
  js:
  83 isWebView()).
   Engine = JavaScriptCore inside WKWebView.
- Minimum iOS deployment:
   Inherited from substrate,
   not declared by Onsen UI.
   Current Capacitor 6/7 targets iOS 13.0+ (Capacitor's minimum);
   Cordova similarly.
   Onsen UI's own code is plain DOM/Custom Elements with no iOS-version floor of its own.
   Effective minimum = whatever the chosen Capacitor/Cordova version requires (iOS 13+ for current Capacitor).
- Gate expectation:
   expected-pass
- Confidence:
   high
- Key finding:
   Onsen UI is a pure-JavaScript Web-Components DOM UI library with no runtime,
   no native code,
   and no FFI of its own (repo-wide grep for native/FFI/audio tokens is empty;
   `package.json` builds only UMD/ESM JS;
   platform.
  js only touches window/document and detects window.
  cordova).
   It is a WKWebView UI layer that inherits ALL three iOS walls
   and ALL functional requirements (kopia static-lib linking,
   in-app HTTP server,
   streaming HTTPS,
   background transfer,
   audio) from the Capacitor/Cordova substrate.
   It sidesteps wall 2 cleanly (app code is JS JITed in the out-of-process WKWebView WebContent process,
   the one process iOS grants JIT).
   The kopia/music-player Rust+Go cores are reusable via native plugins,
   not via Onsen;
   the music-player's Slint UI must be fully rewritten as Onsen DOM.
   The Onsen gate is a near-certain UI-render pass;
   every hard wall is deferred to the substrate vet.

## Wall 2: JIT / executable memory

Verdict:
 wkwebview-jit

Onsen UI sidesteps wall 2 because it is a DOM/Web-Components UI layer:
 its app code is JavaScript executed by JavaScriptCore inside the out-of-process WKWebView WebContent process,
 which is the one process iOS grants the JIT/dynamic-codesigning entitlement to.
 The third-party app process itself never needs executable memory;
 it runs AOT-native plugins only.
 There is no managed/scripted runtime in the app process to AOT-compile or interpret.
 This is the inverse of the NativeScript-on-Android DENY_EXECMEM failure:
 Onsen never asks the app process for execmem.

Source:
 App-code-is-JS-in-WebView:
 onsenui/esm/ons/platform.
js (DOM-only APIs,
 window.
cordova detection at line 83) + onsenui/`package.json` (rollup UMD/ESM,
 no native artifact).
 iOS JIT-entitlement-restricted-to-WKWebView-WebContent:
 Apple docs <https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.cs.allow-jit>
 (on iOS the JIT entitlement is functional only for WebKit's out-of-process content renderer;
 third-party app processes cannot allocate executable memory)
 and WebKit JavaScriptCore JIT tiers <https://webkit.org/blog/10300/speculation-in-javascriptcore/>

## Wall 1: link and call a Go/Rust static library

Feasible:
 partial

Mechanism:
 Indirect,
 substrate-provided.
 Onsen UI itself has ZERO FFI capability (repo-wide grep for .
framework/xcframework/staticlib/dart:
ffi/gomobile/JSI/TurboModule/cinterop returns nothing).
 kopia as a Go gomobile c-archive .
xcframework or a Rust staticlib must be linked into a Capacitor/Cordova NATIVE plugin (Swift/ObjC),
 which links the static lib at the Xcode/CocoaPods/SwiftPM level
 and is exposed to the JS UI via the WKWebView message bridge (window.
cordova / Capacitor.
Plugins).
 JS app code calls the plugin by serialized messages;
 it cannot do C ABI FFI directly.
 Bulk data (e.g. the S3/HTTP bytes kopia needs) crosses the bridge as JSON/base64,
 which is the performance wrinkle.
 So linking+calling a Go/Rust static lib is feasible via the substrate plugin layer,
 not via Onsen.

Source:
 Zero-FFI in Onsen:
 repo-wide rg over /tmp/agent/onsen-audit-20260612 (excluding node_modules) for native/FFI tokens returned empty.
 Substrate dependency:
 onsenui/esm/ons/platform.
js:
83 (window.
cordova),
 official guide <https://onsen.io/v2/guide/hybrid/cordova.html> (Onsen provides no native code;
 all native via Cordova plugins).

## Wall 3: background execution

Onsen UI contributes nothing to background execution;
 it is foreground DOM only.
 The background-transfer story is entirely the substrate's:
 a Capacitor/Cordova native plugin wrapping background URLSession for uploads/downloads and BGProcessingTask (short,
 idle/charging) for deferred work.
 A multi-hour kopia snapshot is NOT expressible as a long-running foreground service on iOS
 and must be restructured around background URLSession chunked transfer driven from the native plugin,
 with the WebView UI showing progress when foregrounded.
 No Onsen API touches this.

Source:
 Onsen core has no background/service code (rg over onsenui/esm found only foreground DOM/event APIs;
 platform.
js is detection-only).
 Background capability is a native-plugin/substrate concern per <https://onsen.io/v2/guide/hybrid/cordova.html>.

## In-app HTTP server (kopia S3 target)

Feasible:
 yes

Mechanism:
 Not via Onsen (UI layer has no networking/server code).
 An in-app local HTTP/S3 endpoint for kopia to target is provided by a Capacitor/Cordova native plugin wrapping a
 native server (e.g. a GCDWebServer-style Swift/ObjC HTTP server) bound to 127.0.0.1.
 The Onsen DOM UI only triggers start/stop and shows status over the WKWebView bridge.

Source:
 Onsen has no server code (rg over onsenui/esm:
 only DOM/component modules).
 Substrate/plugin responsibility per <https://onsen.io/v2/guide/hybrid/cordova.html>

## HTTPS streaming client (pCloud)

Feasible:
 yes

Mechanism:
 From the WebView,
 fetch/XHR/WebSocket reach pCloud over HTTPS,
 but in-process WKWebView streaming/upload of large bodies is awkward and memory-bound.
 The robust path is a native plugin using URLSession (foreground + background) for streaming HTTPS to pCloud;
 Onsen UI only orchestrates via the bridge.
 Onsen provides no HTTP client of its own.

Source:
 Onsen core has no HTTP-client module (rg over onsenui/esm).
 URLSession/background transfer is substrate/native-plugin territory per <https://onsen.io/v2/guide/hybrid/cordova.html>

## Audio (music-player port)

Decode:
 symphonia 0.6 (all features) + libopus decode is pure Rust
 and runs unchanged as an AOT-compiled native static lib inside a Capacitor/Cordova plugin;
 no DOM/Onsen involvement.
 Web Audio decode in the WebView is NOT needed and would not cover symphonia's format range.

Output:
 cpal 0.18 has an iOS backend over CoreAudio/AudioUnit,
 so low-latency output is driven natively from the Rust core;
 AVAudioEngine rewrite is NOT required.
 The audio path never passes through Onsen/WKWebView;
 only play/pause/seek UI events cross the bridge.

Rust core reuse:
 REUSABLE via FFI,
 no AVAudioEngine rewrite.
 The existing Rust symphonia+cpal core (decode,
 true-peak normalization with on-disk peak cache,
 queue/pagination,
 session persistence) compiles to a Rust staticlib,
 links into a native iOS plugin,
 and outputs through cpal's CoreAudio backend.
 What CANNOT be reused:
 the Slint UI (winit + femtovg/software renderer) has no place in a WebView app
 and must be fully rewritten as Onsen DOM components;
 Onsen drives only the player chrome and sends control messages to the Rust core over the bridge.

Source:
 Onsen has zero audio/AVAudioEngine/CoreAudio references (repo-wide rg empty).
 cpal iOS/CoreAudio backend and symphonia pure-Rust decode are properties of the Rust crates,
 reused behind the substrate plugin,
 not provided by Onsen.

## Gate probe and toolchain

Minimal app:
 a Capacitor (or Cordova) iOS shell whose WKWebView loads a single page importing onsenui (js/onsenui.
min.
js + css/onsenui.
`min.css`) with a couple of components,
 e.g. ons-navigator with two ons-page screens,
 an ons-button,
 and an ons-list.
 Build,
 codesign,
 install on the iPhone X.
 On-device signal that confirms viability:
 the Onsen DOM renders with the iOS (flat) auto-styling,
 tapping the button performs the ons-navigator pushPage transition smoothly at 60fps,
 and touch/scroll respond inside the WKWebView
 with no console errors (inspect via Safari Web Inspector attached to the device).
 This proves the UI layer works;
 it does NOT exercise walls 1/2/3,
 which belong to the substrate vet.

Toolchain:
 macOS with Xcode (+ Command Line Tools) and an iOS Simulator/device toolchain;
 Node.
js + npm to fetch the onsenui package;
 the substrate CLI (Capacitor:
 @capacitor/core + @capacitor/cli + @capacitor/ios,
 OR Cordova CLI) to generate and build the iOS WKWebView shell;
 CocoaPods (Capacitor/Cordova plugin deps).
 For the later supporting-stack vets:
 Go toolchain + gomobile (kopia c-archive) or the Rust toolchain
 with aarch64-apple-ios target (Rust staticlib for kopia and for the music-player core),
 plus cargo-lipo/xcframework tooling.

## Supporting-stack vets this framework drags in

- Capacitor/Cordova substrate vet:
   WKWebView config,
   bridge perf,
   plugin authoring model
- kopia static-lib plugin:
   link Go gomobile c-archive .
  xcframework or Rust staticlib into a Swift/ObjC Capacitor plugin and call it
- In-app HTTP/S3 server plugin (GCDWebServer-style) bound to 127.0.0.1 for kopia to target
- Streaming HTTPS + background transfer plugin over URLSession (foreground + background) and BGProcessingTask scheduling
- Rust audio plugin:
   symphonia 0.6 + libopus decode and cpal 0.18 CoreAudio output as an AOT staticlib behind the bridge
- Bulk-data-over-bridge perf vet:
   JSON/base64 cost for kopia byte streams crossing WKWebView
- QA:
   e2e through WKWebView (Appium/WebDriverAgent or Capacitor test harness),
   DOM unit tests for Onsen components,
   plus the substrate plugins' native unit/property/fuzz tests

## Cited sources

- Onsen UI is pure JavaScript on top of Web Components,
   framework-agnostic,
   no native code:
   /tmp/agent/onsen-audit-20260612/onsenui/`README.md`
   ("core library is written in pure Javascript on top of Web Components") + onsenui/`package.json` (main=js/onsenui.
  js,
   module=esm/index.
  js,
   rollup build)
- App code is DOM JS that detects and runs inside a Cordova/WKWebView shell;
   no runtime of its own:
   /tmp/agent/onsen-audit-20260612/onsenui/esm/ons/platform.
  js:
  83 isWebView() returns !
  !
  (window.
  cordova||window.
  phonegap||window.
  PhoneGap);
   lines 19,98,117 use window.
  HTMLElement/navigator.
  userAgent/window.
  screen only
- Onsen UI has zero native iOS source and zero FFI/audio/server code:
   find over /tmp/agent/onsen-audit-20260612 for *.
  swift/*.
  m/*.
  mm/*.
  h/*.
  pbxproj/Podfile returned nothing;
   repo-wide rg for .
  framework|xcframework|staticlib|dart:
  ffi|gomobile|JSI|TurboModule|cinterop|AVAudioEngine|CoreAudio returned nothing
- Onsen provides no native code;
   all native functionality comes from Cordova/Capacitor plugins:
   <https://onsen.io/v2/guide/hybrid/cordova.html> (official guide:
   include cordova.
  js for native features;
   Onsen is the UI layer only)
- On iOS the JIT/execmem entitlement is functional only for WebKit's out-of-process WebContent process;
   third-party app processes cannot allocate executable memory:
   <https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.cs.allow-jit>
- JavaScriptCore JITs JavaScript through Baseline/DFG/FTL tiers:
   <https://webkit.org/blog/10300/speculation-in-javascriptcore/>

## Adversarial cite-check

- Wall 2 confirmed:
   confirmed
- Wall 1 confirmed:
   confirmed
- Corrections:
   CITATION DEFECT (wall-2,
   substance still correct):
   The audit attributes the iOS restriction "the JIT entitlement is functional only for WebKit's out-of-process content
   renderer;
   third-party app processes cannot allocate executable memory" to Apple's page com.
  apple.
  security.
  cs.
  allow-jit.
   That page is a macOS HARDENED RUNTIME entitlement;
   it does not govern iOS and does not state the WKWebView/iOS restriction attributed to it.
   (The Mozilla platform-tilt issue #3 explicitly frames allow-jit as the *macOS* model that iOS does NOT extend
to third parties.
  ) The actual iOS mechanism is the private dynamic-codesigning entitlement (plus com.
  apple.
  private.
  verified-jit / com.
  apple.
  security.
  cs.
  single-jit) carried by the WebContent process.
   Verified in primary source WebKit/Source/JavaScriptCore/`entitlements.plist`
   (<https://github.com/WebKit/WebKit/blob/main/Source/JavaScriptCore/entitlements.plist>),
   which lists dynamic-codesigning=true and verified-jit=true and where com.
  apple.
  security.
  cs.
  allow-jit is ABSENT.
   Cross-confirmed by HN thread 18429757 (dynamic-codesigning gates MAP_JIT,
   used by MobileSafari),
   saagarjha "Jailed JIT on iOS",
   and the newosxbook iOS entitlement DB for com.
  apple.
  WebKit.
  WebContent.
  xpc.
   NET:
   the verdict's underlying FACT (WebContent process is the one process granted JIT;
   third-party app process gets no execmem;
   Onsen runs only AOT-native plugins in the app process) is TRUE and is the INVERSE of NativeScript's DENY_EXECMEM,
   so wall-2 = confirmed;
   only the citation URL is mis-attributed,
   not the conclusion.
   SECONDARY:
   WebKit blog 10300 is correctly cited ONLY for the JavaScriptCore JIT tiers (LLInt/Baseline/DFG/FTL,
   all verified present) but it says NOTHING about WKWebView,
   out-of-process rendering,
   or iOS entitlements,
   so it does not independently support the entitlement sub-claim.
   MINOR (wall-1):
   the cited Onsen Cordova guide (onsen.
  io/v2/guide/hybrid/cordova.
  html) does not state in words "Onsen provides no native code;
   all native via Cordova plugins";
   it documents placing .
  js/.
  css in the Cordova www folder with cordova.
  js as the bridge,
   which is consistent with the claim,
   and the repo-evidence (zero native artifacts) independently establishes it,
   so wall-1 stands.
- Sources checked:
   /tmp/agent/onsen-audit-20260612/onsenui/esm/ons/platform.
  js (verified lines 19 window.
  HTMLElement,
   83 window.
  cordova||phonegap||PhoneGap isWebView,
   98 navigator.
  userAgent,
   117 window.
  screen.
  width:
   all exactly as cited,
   DOM-only);
   /tmp/agent/onsen-audit-20260612/onsenui/`package.json` (verified main=js/onsenui.
  js,
   module=esm/index.
  js,
   build:
  umd=rollup,
   no gypfile/native artifact,
   keywords phonegap/cordova/web-components);
   repo-wide rg for .
  xcframework/staticlib/dart:
  ffi/gomobile/JSI/TurboModule/cinterop/.
  framework over /tmp/agent/onsen-audit-20260612 excluding node_modules+lockfile:
   EMPTY (sanity-checked:
   rg for 'cordova' returns 10+ files,
   so tooling valid);
   find for *.
  node/*.
  a/*.
  so/*.
  dylib/binding.
  gyp/*.
  xcframework/*.
  framework + rg gypfile:
   none present;
   <https://webkit.org/blog/10300/speculation-in-javascriptcore/> (confirms 4 JIT tiers LLInt/Baseline/DFG/FTL;
   explicitly does NOT mention WKWebView or iOS entitlements);
   <https://developer.apple.com/documentation/.../com.apple.security.cs.allow-jit> (JS-rendered,
   WebFetch returns title only;
   substance obtained from better primary source below);
   <https://github.com/WebKit/WebKit/blob/main/Source/JavaScriptCore/entitlements.plist> (PRIMARY:
   dynamic-codesigning=true,
   com.
  apple.
  private.
  verified-jit=true,
   com.
  apple.
  security.
  cs.
  single-jit=true;
   allow-jit ABSENT);
   WebSearch:
   dynamic-codesigning gates MAP_JIT/MobileSafari (HN 18429757),
   newosxbook WebContent.
  xpc entitlement DB,
   saagarjha Jailed-JIT,
   Mozilla platform-tilt issue #3 (allow-jit is the macOS model iOS does not give third parties);
   <https://github.com/mozilla/platform-tilt/issues/3> (iOS makes JIT exception only for Safari/WebKit;
   third-party browsers blocked at mmap MAP_JIT level);
   <https://onsen.io/v2/guide/hybrid/cordova.html> (Onsen = .
  js/.
  css in Cordova www folder + cordova.
  js bridge;
   does not verbatim state 'no native code' but consistent)
