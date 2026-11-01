# iOS source-audit: Apache Cordova

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
   WKWebView-JIT.
   App code is HTML/CSS/JavaScript executed by the out-of-process WebKit content process inside WKWebView;
   Cordova ships no in-process managed/scripted runtime.
   CordovaLib is a thin native Objective-C shell (Foundation/UIKit/WebKit).
   The bridge registers a WKScriptMessageHandler named "cordova" (CDVWebViewEngine.
  m line 29 CDV_BRIDGE_NAME,
   line 217 addScriptMessageHandler:
  name:
  CDV_BRIDGE_NAME);
   JS calls window.
  webkit.
  messageHandlers.
  cordova.
  postMessage(command) (cordova-js-src/exec.
  js line 113);
   native dispatches to a CDVPlugin method via objc_msgSend (CDVCommandQueue.
  m line 188);
   native-to-JS callbacks go back through WKWebView evaluateJavaScript (CDVCommandDelegateImpl.
  m,
   CDVCommandQueue.
  m line 99).
   There is no JavaScriptCore/JSContext/JSValue anywhere in CordovaLib (grep returned zero matches),
   so no in-process JS engine needs executable memory.
- Minimum iOS deployment:
   iOS 13.0 (default IPHONEOS_DEPLOYMENT_TARGET in templates/project/App.
  xcodeproj/project.
  pbxproj lines 347,416;
   overridable via the deployment-target preference,
   lib/prepare.
  js line 323)
- Gate expectation:
   expected-pass
- Confidence:
   high
- Key finding:
   Architecturally Cordova is the same WKWebView-JIT substrate as Capacitor on all three walls:
   app code is JavaScript executing only in the out-of-process WebKit content process (no in-process JS/managed runtime
   in CordovaLib,
   confirmed by zero JavaScriptCore hits),
   native plugins link Go/Rust static libs exactly like a plain native iOS app (pluginHandlers.
  js framework/customFramework/podspec) and bridge over objc_msgSend (CDVCommandQueue.
  m:
  188),
   and Cordova core ships zero background-transfer machinery so background URLSession/BGProcessingTask is
   plugin-authored.
   The only material difference from Capacitor is project management:
   Cordova generates and owns the Xcode project,
   Podfile and `Info.plist` via prepare.
  js/Podfile.
  js,
   whereas Capacitor hands you the native project to own.
   Wall 2 is satisfied (no app-process execmem);
   walls 1,
   3,
   the in-app server,
   HTTPS client and the symphonia+cpal audio core are all feasible only as native plugin code,
   not Cordova built-ins.

## Wall 2: JIT / executable memory

Verdict:
 wkwebview-jit

App logic is JavaScript and genuinely relies on a JIT,
 but it runs only inside the out-of-process WebKit content process of WKWebView,
 which is the one process Apple grants the dynamic-codesigning (JIT) entitlement to.
 Cordova's own app-process code is pure native Objective-C/Swift (AOT-compiled),
 and ships no in-process JavaScriptCore,
 Mono,
 V8,
 or bytecode interpreter (grep for JavaScriptCore|JSContext|JSValue|evaluateScript across CordovaLib
 and cordova-js-src returned zero hits).
 So the app process never needs W^X-violating executable memory;
 the only JIT lives in the sanctioned out-of-process WebKit,
 exactly like Capacitor.
 This is the iOS-legal path,
 not the NativeScript/DENY_EXECMEM failure mode.

Source:
 /tmp/agent/cordova-audit/CordovaLib/Classes/Private/Plugins/CDVWebViewEngine/CDVWebViewEngine.
m (lines 29,217,490 the WKScriptMessageHandler bridge);
 /tmp/agent/cordova-audit/cordova-js-src/exec.
js line 113 (postMessage transport);
 zero matches for grep -rln 'JavaScriptCore|JSContext|JSValue|evaluateScript' CordovaLib/ cordova-js-src/

## Wall 1: link and call a Go/Rust static library

Feasible:
 yes

Mechanism:
 A Cordova plugin includes native Objective-C/Swift source plus a static library or .
xcframework.
 plugin.
xml framework tags add a custom framework/static lib to the generated Xcode project with link:
true,
 embed,
 and sign (pluginHandlers.
js lines 96-145,
 opt {customFramework:
true,
 embed,
 link,
 sign:
true} at line 142);
 the podspec tag (line 155) lets the plugin pull a CocoaPods dependency,
 and SwiftPackage.
js adds SPM support.
 The native plugin therefore links a Go gomobile c-archive .
xcframework or a Rust staticlib exactly like any native iOS app,
 and exposes it to app JS by declaring a CDVPlugin method that the bridge invokes via objc_msgSend (CDVCommandQueue.
m line 188),
 returning results through CDVPluginResult/evaluateJavaScript.
 kopia as a linked static lib called via FFI from a native plugin is fully expressible;
 it is plugin-authored native code,
 not a Cordova built-in.

Source:
 /tmp/agent/cordova-audit/lib/plugman/pluginHandlers.
js lines 96-145,155 (framework/customFramework/link/embed/sign + podspec);
 /tmp/agent/cordova-audit/lib/SwiftPackage.
js (SPM);
 /tmp/agent/cordova-audit/CordovaLib/Classes/Public/CDVCommandQueue.
m line 188 (objc_msgSend dispatch into the CDVPlugin);
 /tmp/agent/cordova-audit/Cordova.
podspec and Package.
swift (CordovaLib itself is a normal native lib)

## Wall 3: background execution

Cordova-ios core ships NO background-transfer machinery.
 grep across CordovaLib/,
 templates/,
 lib/ for
 BGTaskScheduler|BGProcessingTask|UIBackgroundModes|beginBackgroundTask|backgroundSessionConfiguration|backgroundFetch
 returned zero matches (exit 1),
 and the project template `Info.plist` (templates/project/App/`App-Info.plist`) declares no UIBackgroundModes.
 Background work is therefore per-plugin native code,
 identical to a plain native iOS app:
 a plugin would create a background URLSessionConfiguration for pCloud uploads
 and schedule BGProcessingTask for idle/charging windows,
 adding UIBackgroundModes / BGTaskSchedulerPermittedIdentifiers via the plugin's config-file edits to the generated
 `Info.plist`.
 The multi-hour kopia snapshot must be restructured around background URLSession + BGProcessingTask just as on any iOS
 framework;
 Cordova neither helps nor hinders this beyond giving the plugin a normal native surface to write it.

Source:
 empty grep (BGTaskScheduler|BGProcessingTask|UIBackgroundModes|beginBackgroundTask|backgroundSessionConfiguration over
 /tmp/agent/cordova-audit/CordovaLib /tmp/agent/cordova-audit/templates /tmp/agent/cordova-audit/lib,
 exit 1);
 /tmp/agent/cordova-audit/templates/project/App/`App-Info.plist` (no UIBackgroundModes);
 plugin native mechanism per lib/plugman/pluginHandlers.
js

## In-app HTTP server (kopia S3 target)

Feasible:
 partial

Mechanism:
 Not provided by Cordova core as a real listening socket.
 Cordova's CDVURLSchemeHandler (CDVURLSchemeHandler.
m) serves the app's web assets by intercepting WKWebView requests for the app:
//localhost custom scheme in-process via WKURLSchemeHandler;
 it is an in-WebView request interceptor,
 NOT a TCP listener,
 so kopia (a separate native process/lib) cannot POST to it as an S3/HTTP endpoint.
 A real loopback HTTP/S3 endpoint that kopia targets must be a native plugin embedding GCDWebServer or an Apple Network.
framework nw_listener,
 added through the plugin's podspec tag (pluginHandlers.
js line 155) or framework tag.
 Feasible as ordinary native code,
 but it is plugin-authored,
 not a Cordova built-in.

Source:
 /tmp/agent/cordova-audit/CordovaLib/Classes/Private/Plugins/CDVWebViewEngine/CDVURLSchemeHandler.
m (WKURLSchemeHandler asset interception,
 not a TCP socket);
 /tmp/agent/cordova-audit/lib/plugman/pluginHandlers.
js line 155 (podspec tag to add a server pod)

## HTTPS streaming client (pCloud)

Feasible:
 yes

Mechanism:
 A native plugin uses NSURLSession (including background URLSessionConfiguration for resumable/streamed uploads to
 pCloud),
 exposed to JS over the CDVPlugin bridge;
 or the linked Rust core can drive its own HTTPS client (e.g. reqwest) from the static lib.
 ATS applies;
 HTTPS streaming to pCloud is ordinary native networking.
 Not a Cordova built-in,
 but trivially plugin-authored.

Source:
 /tmp/agent/cordova-audit/lib/plugman/pluginHandlers.
js (native source/framework plugin mechanism);
 /tmp/agent/cordova-audit/CordovaLib/Classes/Public/CDVCommandQueue.
m line 188 (bridge to native)

## Audio (music-player port)

Decode:
 Reuse the existing Rust core.
 A native plugin links the Rust staticlib so symphonia 0.6 (all features) + libopus decode runs unchanged via FFI;
 no rewrite of decode logic.

Output:
 cpal 0.18 has an iOS/CoreAudio backend,
 so audio output is driven by the same Rust core through cpal -> CoreAudio.
 A thin native plugin must still own the iOS-only audio glue that Rust cannot express:
 AVAudioSession category/activation,
 the UIBackgroundModes 'audio' entry for background playback,
 and MPNowPlayingInfoCenter / MPRemoteCommandCenter for lock-screen controls.
 CoreAudio low-latency output is reachable from cpal;
 AVAudioEngine is not required for the decode/output path.

Rust core reuse:
 Yes,
 reusable via FFI.
 The symphonia + cpal core links into a Cordova native plugin as a static library exactly as wall 1 (pluginHandlers.
js framework/customFramework + podspec).
 Audio decode and CoreAudio output are NOT rewritten on AVAudioEngine;
 only the iOS session/now-playing/background-mode shell is new native code.
 The Slint UI itself would be replaced by the WKWebView web UI (Cordova is a web UI shell),
 so only the Rust audio/decode/normalization core is reused,
 fronted by the WebView for the queue/pagination/session UI.

Source:
 /tmp/agent/cordova-audit/lib/plugman/pluginHandlers.
js lines 96-145 (framework/customFramework/link to embed the Rust staticlib);
 cpal iOS/CoreAudio backend granted by task;
 AVAudioSession/MPNowPlaying glue is plugin-authored native code per CDVPlugin model (CDVCommandQueue.
m line 188)

## Gate probe and toolchain

Minimal app:
 `cordova create gate && cordova platform add ios`,
 then add one native plugin that links a tiny Rust staticlib (a single exported `extern \"C\" fn add(a,b)->i64`)
 packaged as an .
xcframework via plugin.
xml's framework tag (customFramework:
true),
 and a CDVPlugin method that calls it over the exec bridge and returns the result to JS.
 Build + install on the iPhone X with the established CLI signing path.
 On-device signal that confirms viability:
 the app launches,
 the WebView page calls cordova.
exec(...) into the plugin,
 the plugin invokes the Rust symbol,
 and the device shows the FFI-computed value rendered in the WebView (proves wall 1 link+call works
 and wall 2 is satisfied because JS runs in WebKit while native AOT code does the FFI).
 A failure would be a link error at build time or an EXC_BAD_ACCESS/codesign-execmem crash on call,
 neither of which is expected given the standard native link path.
 Because it needs the physical iPhone X + CLI signing,
 it is buildable but the confirming signal is on-device.

Toolchain:
 macOS with Xcode (+ Command Line Tools),
 Node.
js + npm with the cordova CLI (`npm i -g cordova`),
 CocoaPods (`pod`) and/or Swift Package Manager (bundled with Xcode),
 and the Rust iOS targets (`rustup target add aarch64-apple-ios`) plus cargo to build the staticlib/.
xcframework (or Go + gomobile for a Go c-archive).
 ios-deploy or devicectl for on-device install.

## Supporting-stack vets this framework drags in

- Native-plugin FFI binding vet:
   marshalling kopia/Rust results across the objc_msgSend + CDVPluginResult boundary,
   ArrayBuffer base64 round-trips (exec.
  js massageArgsJsToNative),
   and memory ownership across the Swift/ObjC<->Rust/Go boundary
- In-app HTTP/S3 server vet:
   GCDWebServer vs Network.
  framework nw_listener as a Cordova plugin,
   loopback binding,
   ATS/app-bound-domain interaction with WKWebView
- HTTPS streaming client + background URLSession vet:
   native background URLSessionConfiguration plugin,
   resumable uploads to pCloud,
   BGProcessingTask scheduling and `Info.plist` UIBackgroundModes/BGTaskSchedulerPermittedIdentifiers injection via
   plugin config-file edits
- Audio plugin vet:
   AVAudioSession category/activation,
   UIBackgroundModes audio for background playback,
   MPNowPlayingInfoCenter/MPRemoteCommandCenter,
   cpal CoreAudio latency under WKWebView,
   true-peak normalization peak-cache on-disk access from the native side
- Build-system vet:
   CocoaPods (Podfile.
  js/PodsJson.
  js) vs Swift Package Manager (SwiftPackage.
  js) for embedding the .
  xcframework,
   codesign/embed/sign of custom frameworks,
   generated-project drift since Cordova owns the pbxproj
- QA:
   in-process UI test via WKWebView automation (WebDriver/Appium against the WebView),
   e2e on device,
   fuzz of the JS<->native bridge JSON,
   property tests for the FFI marshalling,
   mutation testing of the Rust core

## Cited sources

- App JS executes only via WKScriptMessageHandler named 'cordova';
   native dispatch is the bridge:
   /tmp/agent/cordova-audit/CordovaLib/Classes/Private/Plugins/CDVWebViewEngine/CDVWebViewEngine.
  m lines 29,217,490
- JS->native transport is window.
  webkit.
  messageHandlers.
  cordova.
  postMessage:
   /tmp/agent/cordova-audit/cordova-js-src/exec.
  js line 113
- Native command dispatch into a CDVPlugin via objc_msgSend (the FFI entry to native,
   where a Rust/Go staticlib is called):
   /tmp/agent/cordova-audit/CordovaLib/Classes/Public/CDVCommandQueue.
  m line 188
- No in-process JavaScriptCore/JSContext/JSValue in CordovaLib (app code runs only in out-of-process WebKit):
   grep -rln 'JavaScriptCore|JSContext|JSValue|evaluateScript' /tmp/agent/cordova-audit/CordovaLib
   /tmp/agent/cordova-audit/cordova-js-src -> zero matches (exit 1)
- Plugins link custom frameworks/static libs (customFramework,
   link,
   embed,
   sign) and CocoaPods podspecs:
   /tmp/agent/cordova-audit/lib/plugman/pluginHandlers.
  js lines 96-145,
   115,
   155
- Swift Package Manager support for embedding native code:
   /tmp/agent/cordova-audit/lib/SwiftPackage.
  js
- CordovaLib is a native lib linking Foundation/UIKit/WebKit (normal native static-lib surface):
   /tmp/agent/cordova-audit/Cordova.
  podspec;
   /tmp/agent/cordova-audit/Package.
  swift
- Cordova core ships no background-transfer machinery;
   no UIBackgroundModes in template:
   grep BGTaskScheduler|BGProcessingTask|UIBackgroundModes|beginBackgroundTask|backgroundSessionConfiguration over
   /tmp/agent/cordova-audit/{CordovaLib,
  templates,
  lib} -> exit 1;
   /tmp/agent/cordova-audit/templates/project/App/`App-Info.plist`
- In-app asset serving is a WKURLSchemeHandler interceptor (app:
  //localhost),
   not a TCP listener:
   /tmp/agent/cordova-audit/CordovaLib/Classes/Private/Plugins/CDVWebViewEngine/CDVURLSchemeHandler.
  m
- Min iOS deployment target 13.0,
   overridable via deployment-target preference:
   /tmp/agent/cordova-audit/templates/project/App.
  xcodeproj/project.
  pbxproj lines 347,416;
   /tmp/agent/cordova-audit/lib/prepare.
  js lines 323-326

## Adversarial cite-check

- Wall 2 confirmed:
   confirmed
- Wall 1 confirmed:
   confirmed
- Corrections:
   Wall-1 podspec citation is wrong (minor,
   non-load-bearing).
   The audit states:
   "the podspec tag (line 155) lets the plugin pull a CocoaPods dependency.
  " But /tmp/agent/cordova-audit/lib/plugman/pluginHandlers.
  js line 155 is actually `if (obj.type !== 'podspec')` inside the framework UNINSTALL handler -- an uninstall-guard
  negation,
   not a podspec-tag install handler.
   pluginHandlers.
  js contains no `podspec:` handler entry;
   the `framework` handler explicitly emits an error that `type="podspec"` is "no longer supported.
   Please use the podspec tag" (lines 115-116).
   The asserted capability is nonetheless real:
   actual CocoaPods/podspec dependency handling lives in lib/Api.
  js,
   lib/Podfile.
  js,
   and lib/PodsJson.
  js.
   Corrected fact:
   a Cordova plugin can pull a CocoaPods dependency,
   but via the dedicated podspec-tag machinery in lib/Api.
  js + lib/Podfile.
  js (+ lib/PodsJson.
  js),
   not at pluginHandlers.
  js:
  155.
   This does not change the wall-1 "yes" verdict,
   which rests on the framework/customFramework/embed/link/sign mechanism (pluginHandlers.
  js:
  96-145,
   esp.
   line 142 verified exactly),
   SwiftPackage.
  js SPM support (verified present,
   adds plugin SPM deps at lines 36-39),
   and objc_msgSend dispatch (CDVCommandQueue.
  m:
  188 verified exactly) -- all confirmed against source.
   No other claim was unsupported,
   fabricated,
   or contradicted.
   All wall-2 / iosRuntimeModel citations verified exactly:
   CDVWebViewEngine.
  m line 29 (CDV_BRIDGE_NAME @"cordova"),
   217 (addScriptMessageHandler:
  name:
  CDV_BRIDGE_NAME),
   490 (userContentController:
  didReceiveScriptMessage:
   WKScriptMessageHandler bridge);
   cordova-js-src/exec.
  js line 113 (window.
  webkit.
  messageHandlers.
  cordova.
  postMessage);
   CDVCommandQueue.
  m line 188 (objc_msgSend) and 99 (evaluateJavaScript callback);
   CDVCommandDelegateImpl.
  m line 74 (evaluateJavaScript).
   The pivotal zero-match grep is genuine:
   searching JavaScriptCore,
   JSContext,
   JSValue,
   evaluateScript each individually across CordovaLib/ and cordova-js-src/ returns 0 files;
   broadened net (V8/Mono/Hermes/interpreter) also 0;
   no bundled .
  framework/.
  dylib/.
  a engine in CordovaLib.
   The "no in-process JS engine;
   only out-of-process WKWebView JIT" conclusion driving the wkwebview-jit verdict is sound.
   (The verdict's Apple-entitlement and "exactly like Capacitor" assertions are external-knowledge claims,
   not repo-citation claims,
   so outside repo-source scope;
   the repo evidence that they rest on holds.
  )
- Sources checked:
   /tmp/agent/cordova-audit/CordovaLib/Classes/Private/Plugins/CDVWebViewEngine/CDVWebViewEngine.
  m (lines 27,29,217,490);
   /tmp/agent/cordova-audit/cordova-js-src/exec.
  js (line 113);
   /tmp/agent/cordova-audit/CordovaLib/Classes/Public/CDVCommandQueue.
  m (lines 99,188);
   /tmp/agent/cordova-audit/CordovaLib/Classes/Private/CDVCommandDelegateImpl.
  m (line 74);
   /tmp/agent/cordova-audit/lib/plugman/pluginHandlers.
  js (lines 96-145,155,
   esp 142);
   /tmp/agent/cordova-audit/lib/SwiftPackage.
  js;
   /tmp/agent/cordova-audit/Package.
  swift;
   /tmp/agent/cordova-audit/Cordova.
  podspec;
   grep JavaScriptCore|JSContext|JSValue|evaluateScript across CordovaLib/ and cordova-js-src/ (0 matches,
   each term verified individually);
   find CordovaLib for .
  framework/.
  dylib/.
  a (none);
   git remote (confirmed apache/cordova-ios)
