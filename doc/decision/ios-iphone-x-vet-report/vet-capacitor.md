# iOS source-audit: Capacitor

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
   App UI/logic is HTML/CSS/JS run entirely inside an out-of-process WKWebView content process
   (the one surface holding iOS dynamic-codesigning/JIT).
   Native side calls JS via WKWebView.
  evaluateJavaScript;
   JS calls native via WKScriptMessageHandler.
   There is no in-app-process JS engine:
   the iOS core imports only WebKit,
   with zero JavaScriptCore/JSContext/JSVirtualMachine usage.
   Native plugin code is plain Swift/ObjC compiled AOT into the app,
   like any UIKit app.
- Minimum iOS deployment:
   15.0
- Gate expectation:
   expected-pass
- Confidence:
   high
- Key finding:
   Capacitor's iOS shell is a native UIKit app whose app code runs as HTML/JS inside an out-of-process WKWebView
   (the only JIT-entitled surface) with zero in-app-process JS engine,
   so wall 2 is structurally a non-issue (no-jit-needed);
   native plugins are plain Swift/ObjC compiled AOT,
   so linking and calling a Go/Rust static lib (kopia,
   symphonia+cpal) over FFI is ordinary native iOS work.
   The real cost is everything heavy (kopia compute,
   the in-app S3/HTTP server,
   streaming uploads,
   background scheduling,
   and audio) lives entirely in custom native plugins that Capacitor neither provides nor constrains;
   Capacitor only contributes the WebView UI and a thin message bridge.

## Wall 2: JIT / executable memory

Verdict:
 no-jit-needed

Capacitor does not embed a managed/scripted runtime in the app process.
 All app JS executes in WKWebView,
 which is the out-of-process WebContent process that already carries the JIT (dynamic-codesigning) entitlement on iOS.
 The app process itself runs only AOT-compiled Swift/ObjC (the CapacitorBridge + native plugins).
 Native-to-JS is evaluateJavaScript;
 JS-to-native is the WKScriptMessageHandler bridge.
 Grepping the iOS core for an in-process JS engine
 (import JavaScriptCore / JSContext( / JSVirtualMachine) returns empty,
 so no app-process interpreter or JIT is needed.
 This is the structurally safest answer to wall 2 of any candidate:
 Capacitor lands directly on the only JIT-blessed surface and needs nothing
 from the app process beyond ordinary native AOT code.

Source:
 /tmp/agent/capacitor-audit-20260612/ios/Capacitor/Capacitor/CapacitorBridge.
swift:
643,659 (evaluateJavaScript);
 /tmp/agent/capacitor-audit-20260612/ios/Capacitor/Capacitor/WebViewDelegationHandler.
swift:
7,186 (WKScriptMessageHandler userContentController);
 rg 'import JavaScriptCore|JSContext\(|JSVirtualMachine' over ios/ returns empty

## Wall 1: link and call a Go/Rust static library

Feasible:
 yes

Mechanism:
 Swift/ObjC bridge to a C ABI.
 A Capacitor plugin is a plain NSObject/Swift class (CAPPlugin),
 ordinary native code compiled into the app target.
 It links and calls a Go gomobile c-archive .
xcframework or a Rust staticlib exactly like any native iOS app:
 add the .
a/.
xcframework to the generated App.
xcworkspace (CocoaPods Podfile '# Add your Pods here',
 or the SPM template Package.
swift),
 expose the C functions via a bridging header/modulemap,
 and call them from the plugin's Swift/ObjC.
 kopia (Go) or any Rust core is reached over FFI from the plugin,
 then surfaced to the WebView UI via the message bridge.
 Capacitor adds no obstacle here;
 this is just native iOS linking.

Source:
 /tmp/agent/capacitor-audit-20260612/ios/Capacitor/Capacitor/CAPPlugin.
h (plugin is @interface CAPPlugin :
 NSObject);
 /tmp/agent/capacitor-audit-20260612/ios-pods-template/App/Podfile:
18 ('# Add your Pods here');
 /tmp/agent/capacitor-audit-20260612/ios-spm-template/App/CapApp-SPM/Package.
swift

## Wall 3: background execution

Capacitor core ships no background scheduler.
 Grepping the iOS core finds only URLSession.
shared (in WebViewAssetHandler's HTTP interceptor) and no BGTaskScheduler/BGProcessingTask/beginBackgroundTask usage.
 Background work is a native-plugin concern,
 written in Swift exactly as in any iOS app,
 and is bound by the same platform wall:
 only background URLSession (for transfers) and BGProcessingTask/BGAppRefreshTask (short,
 idle/charging) exist;
 there is no foreground-service equivalent.
 A multi-hour kopia snapshot cannot run continuously in the background and must be restructured:
 do chunked work under a BGProcessingTask
 and push the actual pCloud upload through a background URLSession (which the OS continues after suspension).
 The pCloud transfer maps cleanly to background URLSession;
 the kopia compute (dedup/snapshot via the linked Go lib) does not get unbounded background CPU
 and must checkpoint/resume across BGProcessingTask windows.
 The community @capacitor/background-runner plugin exists but runs JS,
 not the native kopia compute,
 so the heavy path stays in a custom native plugin.

Source:
 /tmp/agent/capacitor-audit-20260612/ios/Capacitor/Capacitor/WebViewAssetHandler.
swift:
142 (URLSession.
shared,
 only URLSession in core);
 rg 'BGTaskScheduler|BGProcessingTask|beginBackgroundTask'
 over ios/Capacitor returns empty (no background scheduler in core);
 Apple docs <https://developer.apple.com/documentation/backgroundtasks/bgprocessingtask>
 and <https://developer.apple.com/documentation/foundation/url_loading_system/downloading_files_in_the_background>

## In-app HTTP server (kopia S3 target)

Feasible:
 partial

Mechanism:
 Capacitor itself does NOT run a real TCP HTTP server.
 WebViewAssetHandler is a WKURLSchemeHandler registered via WKWebViewConfiguration.
setURLSchemeHandler for the custom app scheme (capacitor:
//);
 it only intercepts requests originating inside that one WebView and serves local files / proxies fetch,
 with no loopback socket.
 kopia needs to POST to a real S3/HTTP endpoint over a socket,
 which the URLSchemeHandler cannot provide to an out-of-WebView client.
 So the in-app S3/HTTP target kopia hits must be a separate embedded TCP server bound to 127.0.0.1,
 written in the native plugin:
 either Go net/http inside the same gomobile lib (recommended;
 keeps the loop in-process with kopia),
 or a Swift Network.
framework NWListener.
 iOS permits binding 127.0.0.1 in-process;
 the constraint is Capacitor provides none of this,
 it is plain native plugin work.

Source:
 /tmp/agent/capacitor-audit-20260612/ios/Capacitor/Capacitor/CAPBridgeViewController.
swift:
297 (setURLSchemeHandler);
 /tmp/agent/capacitor-audit-20260612/ios/Capacitor/Capacitor/WebViewAssetHandler.
swift:
6,27 (WKURLSchemeHandler,
 no socket);
 rg 'GCDWebServer|CocoaHTTPServer|Telegraph|NWListener' over repo returns empty

## HTTPS streaming client (pCloud)

Feasible:
 yes

Mechanism:
 Two viable paths.
 (1) From the WebView UI,
 ordinary fetch/XHR over HTTPS (optionally routed through Capacitor's CapacitorHttp interceptor in WebViewAssetHandler.
handleCapacitorHttpRequest,
 which proxies via URLSession.
shared and merges headers).
 (2) For the kopia/pCloud streaming upload,
 the native side uses Foundation URLSession (incl.
 background URLSession with streamed bodies) or Go net/http inside the linked lib.
 cploud-bound streaming HTTPS with backpressure is best done natively
 (URLSession upload tasks or the Go http client in the gomobile lib),
 not from JS.
 ATS (App Transport Security) applies to HTTPS the same as any iOS app.

Source:
 /tmp/agent/capacitor-audit-20260612/ios/Capacitor/Capacitor/WebViewAssetHandler.
swift:
132-187 (handleCapacitorHttpRequest using URLSession.
shared.
dataTask);
 Apple docs <https://developer.apple.com/documentation/foundation/urlsession>

## Audio (music-player port)

Decode:
 symphonia 0.6 (all features) + libopus reused as-is via the Rust core compiled to a staticlib;
 decode runs in the app process as native Rust,
 no WebView involvement.

Output:
 cpal 0.18 reused as-is:
 cpal has a dedicated iOS CoreAudio/AudioUnit backend (RemoteIO + AVAudioSession) selected by
 cfg(not(target_os="macos")),
 so low-latency output works through the same crate.
 No AVAudioEngine rewrite of the core is required;
 AVAudioSession category/activation may need a thin Swift shim in the plugin.

Rust core reuse:
 reuse-via-ffi

Source:
 cpal coreaudio host has an ios submodule:
 <https://raw.githubusercontent.com/RustAudio/cpal/master/src/host/coreaudio/mod.rs> (comment:
 'iOS and tvOS share the same CoreAudio / AudioUnit surface (RemoteIO,
 AVAudioSession)';
 cfg(not(target_os="macos")) mod ios);
 Capacitor plugin FFI bridge:
 /tmp/agent/capacitor-audit-20260612/ios/Capacitor/Capacitor/CAPPlugin.
h

## Gate probe and toolchain

Minimal app:
 Capacitor iOS scaffold (npx cap add ios) opened as App.
xcworkspace,
 with a custom CAPPlugin that links a Rust staticlib (rustup target aarch64-apple-ios)
 or Go gomobile c-archive via a bridging header.
 On-device signals on the iPhone X:
 (1) WebView UI shows a value returned from the linked Rust/Go function
 (wall-1 FFI through the WKScriptMessageHandler bridge),
 (2) audible tone via cpal's iOS RemoteIO backend (native audio output works),
 (3) an in-process 127.0.0.1 HTTP 200 from a plugin-hosted listener (embedded-server path for kopia).
 Pass = all three observed with no execmem/codesign kill (none expected,
 no app-process JIT).
 Fail = staticlib link failure or bridge round-trip never returns.

Toolchain:
 macOS with Xcode (iOS 15+ SDK) + Command Line Tools;
 CocoaPods (or use the SPM template);
 Node.
js + @capacitor/cli and @capacitor/ios;
 Rust with aarch64-apple-ios target (rustup target add aarch64-apple-ios) for the Rust staticlib,
 and/or Go + gomobile (gomobile bind -target=ios) for the kopia c-archive .
xcframework;
 a valid signing identity/provisioning for device install.

## Supporting-stack vets this framework drags in

- Native FFI plugin vet:
   Go gomobile c-archive (kopia) and Rust staticlib (symphonia+cpal) packaged as .
  xcframework,
   linked into the Capacitor App target via Podfile/SPM,
   called from a CAPPlugin Swift bridge
- In-app TCP HTTP/S3 server vet:
   embed Go net/http (in the gomobile lib) or Swift Network.
  framework NWListener bound to 127.0.0.1 as kopia's target endpoint
  (Capacitor's URLSchemeHandler does not provide a real socket)
- Streaming HTTPS upload vet:
   background URLSession upload tasks (and ATS config) for the pCloud transfer,
   plus checkpoint/resume of kopia compute across BGProcessingTask windows
- Background-transfer restructuring vet:
   BGProcessingTask scheduling + background URLSession continuation,
   since no foreground-service / long-running background exists
- Audio vet:
   cpal iOS RemoteIO/AVAudioSession output latency + AVAudioSession category shim,
   true-peak normalization + on-disk peak cache running in the Rust core
- Bridge-throughput vet:
   WKScriptMessageHandler + evaluateJavaScript round-trip cost for high-frequency UI<->native audio/queue state
   (two-axis pagination,
   session persistence)
- QA:
   in-process UI test (XCUITest driving the WKWebView),
   e2e (device automation),
   fuzzing the kopia S3 endpoint + range/Content-Range handler,
   property tests on the Rust audio/normalization core,
   mutation testing of the FFI bridge marshalling

## Cited sources

- App JS executes in WKWebView via evaluateJavaScript;
   no in-app JS engine:
   /tmp/agent/capacitor-audit-20260612/ios/Capacitor/Capacitor/CapacitorBridge.
  swift:
  643,659;
   rg 'import JavaScriptCore|JSContext\(|JSVirtualMachine' over ios/ empty
- JS-to-native bridge is WKScriptMessageHandler:
   /tmp/agent/capacitor-audit-20260612/ios/Capacitor/Capacitor/WebViewDelegationHandler.
  swift:
  7,186
- Native plugin is a plain NSObject (Swift/ObjC),
   so it can link a C-ABI static lib like any native app:
   /tmp/agent/capacitor-audit-20260612/ios/Capacitor/Capacitor/CAPPlugin.
  h
- Generated App workspace is standard CocoaPods/SPM where any pod/xcframework can be added:
   /tmp/agent/capacitor-audit-20260612/ios-pods-template/App/Podfile:
  18;
   /tmp/agent/capacitor-audit-20260612/ios-spm-template/App/CapApp-SPM/Package.
  swift
- In-app asset server is a WKURLSchemeHandler (custom scheme interception),
   not a real TCP server:
   /tmp/agent/capacitor-audit-20260612/ios/Capacitor/Capacitor/CAPBridgeViewController.
  swift:
  297;
   /tmp/agent/capacitor-audit-20260612/ios/Capacitor/Capacitor/WebViewAssetHandler.
  swift:
  6,27
- CapacitorHttp proxy uses URLSession.
  shared (HTTPS client path):
   /tmp/agent/capacitor-audit-20260612/ios/Capacitor/Capacitor/WebViewAssetHandler.
  swift:
  132-187
- Capacitor core ships no background scheduler (no BGTaskScheduler/BGProcessingTask):
   rg 'BGTaskScheduler|BGProcessingTask|beginBackgroundTask'
   over /tmp/agent/capacitor-audit-20260612/ios/Capacitor empty
- cpal has an iOS CoreAudio/AudioUnit (RemoteIO/AVAudioSession) backend,
   so the Rust audio core is reusable via FFI without AVAudioEngine rewrite:
   <https://raw.githubusercontent.com/RustAudio/cpal/master/src/host/coreaudio/mod.rs>
   (cfg(not(target_os="macos")) mod ios;
   RemoteIO/AVAudioSession comment)
- Min iOS deployment target 15.0:
   /tmp/agent/capacitor-audit-20260612/ios/Capacitor.
  podspec:
  15;
   /tmp/agent/capacitor-audit-20260612/ios-pods-template/App/Podfile:
  3
- App boots as a native UIKit @UIApplicationMain AppDelegate:
   /tmp/agent/capacitor-audit-20260612/ios-pods-template/App/App/AppDelegate.
  swift

## Adversarial cite-check

- Wall 2 confirmed:
   confirmed
- Wall 1 confirmed:
   confirmed
- Corrections:
   No false,
   fabricated,
   or contradicted claims found.
   Every line-level citation verifies exactly against the genuine ionic-team/capacitor clone
   (remote = <https://github.com/ionic-team/capacitor.git>,
   HEAD 28bb2c6).

Verified true:
- CapacitorBridge.
  swift:
  643 and :
  659 are both `self.getWebView()?.evaluateJavaScript(...)` calls (native-to-JS),
   as cited.
- WebViewDelegationHandler.
  swift:
  7 declares the class conforming to `WKScriptMessageHandler`;
   line 186 is `open func userContentController(_:didReceive:)` (JS-to-native),
   as cited.
- The wall-2 grep `import JavaScriptCore|JSContext\(|JSVirtualMachine` over ios/ returns empty (exit 1).
   I broadened it:
   no `import JavaScriptCore`,
   no ObjC `#import <JavaScriptCore>`,
   no JSContext/JSVirtualMachine/JSGlobalContext anywhere in the whole ios/ tree (incl.
   CapacitorCordova).
   Distinct Swift imports are only Foundation/WebKit/UIKit/Combine/Cordova/etc. Capacitor's JSExport.
  swift,
   JS.
  swift,
   JSTypes.
  swift,
   JSValueEncoder/Decoder are its own Foundation/Combine-based bridge types,
   NOT Apple's JavaScriptCore framework,
   so they do not contradict the no-in-process-engine finding.
- CAPPlugin.
  h:
  9 is `@interface CAPPlugin : NSObject`.
- ios-pods-template/App/Podfile:
  18 is `  # Add your Pods here` (inside `target 'App'`),
   as cited.
- ios-spm-template/App/CapApp-SPM/Package.
  swift exists and declares the Capacitor product dependency.

Caveat worth surfacing to a reader making the toolchain-install call (per schema:
 unsupported-by-cited-source,
 though not false):
 the wall-2 verdict's actual linchpin is the POSITIVE claim that "WKWebView is the out-of-process WebContent process
 that already carries the JIT (dynamic-codesigning) entitlement.
" None of the cited Capacitor sources state this;
 the grep only proves the NEGATIVE (the app process embeds no in-process JS engine).
 The positive claim is nonetheless correct as iOS platform architecture
 (WebContent runs out-of-process with dynamic-codesigning;
 third-party in-app JavaScriptCore is jitless),
 which is precisely why no-jit-needed is the safe answer,
 so wall-2 stays confirmed,
 but the proof rests on platform fact,
 not on the cited repo files.

Minor (does not affect wall-1 feasibility=yes):
 the SPM template Package.
swift opens with `// DO NOT MODIFY THIS FILE - managed by Capacitor CLI commands`,
 so the audit's "add the dep to the SPM template Package.
swift" is the wrong manual extension point for the SPM path (CLI-managed);
 the Podfile `# Add your Pods here` is the correct clean extension point and is cited accurately.
 Linking native static libs/xcframeworks via a CAPPlugin remains feasible.
- Sources checked:
   /tmp/agent/capacitor-audit-20260612 (git remote ionic-team/capacitor,
   HEAD 28bb2c6);
   ios/Capacitor/Capacitor/CapacitorBridge.
  swift:
  585,607,643,659,693 (evaluateJavaScript);
   ios/Capacitor/Capacitor/WebViewDelegationHandler.
  swift:
  1-12,184,186 (WKScriptMessageHandler,
   userContentController didReceive);
   ios/Capacitor/Capacitor/CAPPlugin.
  h:
  9 (@interface CAPPlugin :
   NSObject);
   ios/Capacitor/Capacitor/JSExport.
  swift (Capacitor's own class,
   imports nothing/Foundation);
   ios/Capacitor/Capacitor/JS.
  swift,
   JSTypes.
  swift (import Foundation only);
   ios/Capacitor/Capacitor/Codable/JSValueEncoder.
  swift,
   JSValueDecoder.
  swift (import Foundation,
   Combine);
   rg 'import JavaScriptCore|JSContext\(|JSVirtualMachine' over ios/ -> empty (exit 1);
   rg 'import JavaScriptCore|#import <JavaScriptCore' over ios/ -> empty;
   rg JSContext|JSVirtualMachine|JSGlobalContext|jitless over ios/ -> empty;
   distinct Swift imports across ios/:
   Foundation/XCTest/Capacitor/WebKit/UIKit/Cordova/Combine/UniformTypeIdentifiers/Dispatch/CommonCrypto;
   ios-pods-template/App/Podfile:
  18 ('# Add your Pods here');
   ios-spm-template/App/CapApp-SPM/Package.
  swift (exists;
   'DO NOT MODIFY - managed by Capacitor CLI';
   Capacitor product dep)
