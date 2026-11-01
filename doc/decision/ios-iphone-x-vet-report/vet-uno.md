# iOS source-audit: Uno Platform

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
   App code is C#/.
  NET compiled to native machine code by Mono full-AOT on the iOS device.
   Uno rides the standard .
  NET-for-iOS (Microsoft.
  iOS) workload:
   its iOS heads target net9.0-ios18.0 / net10.0-ios26.0 (src/targetframework-override.
  props:
  15-16).
   The Mono interpreter is enabled ONLY in debug for C# Hot Reload (src/Uno.
  Sdk/targets/Uno.
  SingleProject.
  iOS.
  targets:
  4:
   `<UseInterpreter Condition="'$(Optimize)' != 'true'">True</UseInterpreter>`);
   release/device builds use full AOT (aot-only) producing native ARM64 code with no runtime code generation.
   The engine is Mono (the .
  NET runtime flavor used by Microsoft.
  iOS),
   identical to MAUI.
   UI is rendered by Uno's own SkiaSharp-on-Metal head (UnoSKMetalView :
   MTKView,
   GRContext.
  CreateMetal) in the AppleUIKit runtime,
   distinct from MAUI's native-UIKit controls;
   a legacy native-UIKit head also exists.
- Minimum iOS deployment:
   14.2 (Uno default SupportedOSPlatformVersion;
   src/Uno.
  Sdk/targets/Uno.
  Common.
  iOS.
  targets:
  5).
   iPhone X on iOS 16.7 clears it.
- Gate expectation:
   expected-pass
- Confidence:
   high
- Key finding:
   Uno is a .
  NET/C# UI framework on the standard .
  NET-for-iOS (Microsoft.
  iOS) workload,
   so it inherits Mono full-AOT on device (interpreter is debug-only per Uno.
  SingleProject.
  iOS.
  targets:
  4;
   dotnet/runtime confirms 'iphone prohibit JITted code'),
   clearing the no-JIT wall,
   and it links+calls native static libs from C# via [DllImport(\"__Internal\")] (proven in-repo
    by Uno's own ICU static linking at UnicodeText.
  ICU.
  skia.
  cs:
  448).
   That single FFI mechanism satisfies BOTH the kopia-as-static-lib requirement AND lets the existing symphonia+cpal
    Rust audio core be reused behind Uno's Skia/Metal UI rather than rewritten on AVAudioEngine.

## Wall 2: JIT / executable memory

Verdict:
 no-jit-needed

Uno's app code is Mono full-AOT compiled to native machine code on the device,
 so no JIT and no executable/writable memory is needed for managed code in a release build.
 The interpreter (which would be the slow fallback) is gated to debug-only (Optimize !
= true) in src/Uno.
Sdk/targets/Uno.
SingleProject.
iOS.
targets:
4;
 device/Release uses full AOT.
 dotnet/runtime's Mono AOT design doc states the platform reason:
 'Some platforms like the iphone prohibit JITted code,
 using technical and/or legal means,
' which is exactly why full AOT (aot-only) is mandatory and used.
 This clears the iOS no-W^X / no-execmem wall the same way MAUI and Avalonia do.

Source:
 src/Uno.
Sdk/targets/Uno.
SingleProject.
iOS.
targets:
4 (UseInterpreter debug-only);
 src/targetframework-override.
props:
15-16 (net9.0-ios18.0 / net10.0-ios26.0 TFM = Microsoft.
iOS workload);
 dotnet/runtime doc/design/mono/web/`aot.md` ('iphone prohibit JITted code ... full AOT')

## Wall 1: link and call a Go/Rust static library

Feasible:
 yes

Mechanism:
 P/Invoke against a statically-linked native library via [DllImport("__Internal")],
 with the .
a/.
xcframework added as <NativeReference Kind="Static"> (the .
NET-for-iOS native-references mechanism).
 Uno itself proves this in-repo:
 it statically links the native ICU library and calls its C entry points via [DllImport("__Internal")] in src/Uno.
UI/UI/Xaml/Documents/UnicodeText.
ICU.
skia.
cs (10+ imports starting line 448).
 The same path links a Go gomobile c-archive (kopia) or a Rust staticlib and calls exported C symbols from C# app code;
 callbacks back into managed code use delegates/function pointers (P/Invoke reverse marshalling).
 This is the kopia-as-linked-static-lib requirement satisfied with no exec of a bundled binary.

Source:
 src/Uno.
UI/UI/Xaml/Documents/UnicodeText.
ICU.
skia.
cs:
448-475 ([DllImport("__Internal")] static-link P/Invoke);
 src/Uno.
UI.
Runtime.
Skia.
AppleUIKit/UI/Xaml/Controls/TextBox/NativeTextSelection.
cs:
16 (objc_msgSend P/Invoke);
 Microsoft.
iOS native-references doc + dotnet/docs `pinvoke.md`

## Wall 3: background execution

Uno provides NO iOS background-transfer abstraction.
 The framework only surfaces the WinRT BackgroundTask shape,
 and on Apple it is unimplemented (src/Uno.
UWP/ApplicationModel/Background has only Android/shared stubs;
 generated WinRT BackgroundTask types throw NotImplemented).
 There is no BGTaskScheduler,
 BGProcessingTask,
 or background-URLSession wiring in the Uno source.
 So a multi-hour kopia snapshot cannot run as a long-lived background job;
 it must be restructured by the app directly on the .
NET-for-iOS Foundation bindings:
 NSUrlSession with a background NSUrlSessionConfiguration for transfers,
 and BGProcessingTaskRequest (short,
 OS-scheduled,
 idle/charging) registered via the iOS BGTaskScheduler binding.
 NSUrlSessionHandler even exposes BypassBackgroundSessionCheck,
 confirming background-session awareness in the binding layer.
 All of this is done by P/Invoke/binding to native iOS APIs,
 not via any Uno helper.

Source:
 src/Uno.
UWP/ApplicationModel/Background/ (Android-only / generated-throwing WinRT stubs,
 no iOS impl);
 Microsoft.
iOS Foundation.
NSUrlSessionHandler doc (BypassBackgroundSessionCheck property);
 no BGTask/URLSession references found in Uno src (rg over src/ returned none)

## In-app HTTP server (kopia S3 target)

Feasible:
 yes

Mechanism:
 Not an Uno feature;
 provided by the .
NET BCL on the .
NET-for-iOS runtime.
 A managed in-process listener (System.
Net.
Sockets.
TcpListener / Socket,
 or ASP.
NET Core Kestrel hosted in-process on 127.0.0.1) runs as ordinary AOT-compiled managed code with no JIT,
 so kopia's local S3/HTTP target endpoint can be hosted in-app.
 This must be vetted as a supporting-stack item (build a tiny AOT iOS app that binds a localhost TcpListener
  and serves a request) since it is not exercised by Uno's own source.
 Verdict 'yes' on mechanism;
 exact in-process socket binding to be confirmed on device.

Source:
 dotnet/docs `pinvoke.md` + .
NET BCL System.
Net.
Sockets (BCL,
 runs under Mono full-AOT confirmed by src/Uno.
Sdk/targets/Uno.
SingleProject.
iOS.
targets:
4);
 not present in Uno repo,
 so flagged as supporting-stack vet

## HTTPS streaming client (pCloud)

Feasible:
 yes

Mechanism:
 BCL HttpClient on .
NET-for-iOS routes through NSUrlSessionHandler (a HttpMessageHandler subclass that wraps native NSURLSession),
 the default handler on Microsoft.
iOS.
 It supports streaming request/response content,
 client certs,
 TLS (SslProtocols),
 and background-session configuration,
 so a streaming HTTPS client to pCloud is straightforward.
 Available to Uno app code with no Uno-specific glue.

Source:
 Microsoft.
iOS Foundation.
NSUrlSessionHandler API doc (public class NSUrlSessionHandler :
 System.
Net.
Http.
HttpMessageHandler;
 SendAsync,
 ClientCertificates,
 SslProtocols,
 BypassBackgroundSessionCheck)

## Audio (music-player port)

Decode:
 Reuse the existing Rust core:
 symphonia 0.6 (all features) + libopus run as a linked Rust staticlib called from C# via P/Invoke,
 exactly the wall-1 [DllImport("__Internal")] static-link mechanism Uno already uses for ICU.
 No managed audio decode needed;
 Uno is only the UI layer.

Output:
 cpal 0.18 has an iOS/CoreAudio (AudioUnit) backend;
 low-latency output runs on a native realtime audio thread via AudioUnit render callbacks,
 which are native code with no managed code in the realtime path and no JIT,
 so it is compatible with the iOS no-execmem rule.
 Output stays in the Rust core,
 driven through cpal,
 not through Uno or AVFoundation.

Rust core reuse:
 REUSE via FFI,
 not rewrite on AVAudioEngine.
 The symphonia+cpal Rust core links as a static library and is called from Uno's C# app code
  by the same P/Invoke __Internal static-link path that satisfies wall 1 for kopia;
 cpal's existing iOS/CoreAudio backend drives output.
 Consistency check:
 if a Go/Rust kopia staticlib links (wall 1 = yes),
 the symphonia+cpal staticlib links by the identical mechanism,
 so audio is not rewritten on AVAudioEngine.
 AVAudioEngine/AVFoundation would only enter the picture if you adopted Uno's built-in MediaPlayerElement,
 which this player does not.

Source:
 src/Uno.
UI/UI/Xaml/Documents/UnicodeText.
ICU.
skia.
cs:
448 ([DllImport("__Internal")] static-link P/Invoke,
 the identical mechanism for a symphonia+cpal staticlib);
 cpal CoreAudio backend (given).
 Note:
 Uno's own MediaPlayerElement uses AVFoundation/AudioToolbox (src/Uno.
UWP/Media/Playback/MediaPlayer.
Apple.
cs),
 but app B does NOT use it.

## Gate probe and toolchain

Build ONE minimal Uno AppleUIKit-head app in Release (full AOT) that exercises both surviving walls at once:
 (1) renders a frame through the Skia/Metal head (UnoSKMetalView -> proves the UI runtime AOT-executes on device),
 and (2) shows a XAML label whose text comes from a P/Invoke call into a linked Go or Rust static lib (a tiny .
a / c-archive .
xcframework added as <NativeReference Kind=\"Static\"> and called via [DllImport(\"__Internal\")]).
 On-device signal on the iPhone X:
 the app launches at all (proves the Mono AOT image runs
  with no execmem/JIT fault) AND the label shows the exact value returned
  by the native FFI function (proves wall 1 static linking,
 and therefore that symphonia+cpal reuse is mechanically real).
 iPhone X (arm64,
 max iOS 16.7) clears Uno's 14.2 minimum deployment target,
 so no deployment-target blocker.

Toolchain:
 macOS with Xcode + Apple command-line tools (Apple build/sign tooling is Mac-only).
 .
NET SDK (net9 or net10) with the Microsoft.
iOS workload (`dotnet workload install ios` plus the Uno workload,
 or `dotnet workload install uno`).
 The Uno.
Sdk / Uno templates (`dotnet new install Uno.Templates`).
 For the gate's native lib:
 Rust with the aarch64-apple-ios target (and cargo to build a staticlib) or Go with gomobile to emit a c-archive .
xcframework.
 A provisioning profile + signing identity for on-device install (iPhone X).

## Supporting-stack vets this framework drags in

- In-app HTTP/S3 server:
   bind a localhost TcpListener/Socket or in-process Kestrel under iOS AOT and confirm kopia can target it
- HTTPS streaming client to pCloud via HttpClient+NSUrlSessionHandler:
   verify streaming upload/download and TLS on device
- Rust/Go static-lib FFI binding:
   build symphonia+cpal and kopia as .
  a/.
  xcframework,
   wire <NativeReference Kind=Static> + [DllImport(__Internal)],
   confirm symbol resolution and reverse-callback marshalling under AOT
- Background transfer:
   NSUrlSession background NSUrlSessionConfiguration + BGTaskScheduler/BGProcessingTaskRequest binding,
   restructure multi-hour kopia snapshot around OS-scheduled chunks
- Audio output:
   cpal 0.18 CoreAudio/AudioUnit backend low-latency render-callback validation on device,
   true-peak normalization + on-disk peak cache from Rust core
- AOT trimming/linker correctness:
   ILLink/full-AOT may break P/Invoke marshalling or reflection in the Rust-interop and HttpClient paths;
   verify TrimmerRootAssembly needs
- QA:
   in-process UI test (Uno.
  UITest / Appium),
   end-to-end device automation,
   fuzzing the FFI boundary marshalling,
   property tests for audio pipeline,
   mutation testing of the C# interop shims

## Cited sources

- Interpreter is debug-only;
   release/device uses full AOT:
   src/Uno.
  Sdk/targets/Uno.
  SingleProject.
  iOS.
  targets:
  4 (UseInterpreter Condition Optimize!
  =true)
- Uno iOS heads target the Microsoft.
  iOS workload TFMs:
   src/targetframework-override.
  props:
  15-16 (net9.0-ios18.0 / net10.0-ios26.0)
- iOS forbids JIT,
   so Mono full AOT is mandatory;
   this is the runtime model:
   dotnet/runtime doc/design/mono/web/`aot.md`:
   'Some platforms like the iphone prohibit JITted code,
   using technical and/or legal means' (full AOT / aot-only)
- Wall 1:
   native static lib linked + called from C# via __Internal P/Invoke,
   proven by Uno's own ICU linking:
   src/Uno.
  UI/UI/Xaml/Documents/UnicodeText.
  ICU.
  skia.
  cs:
  448-475 ([DllImport("__Internal")])
- P/Invoke (DllImport/LibraryImport) calls native C functions including statically linked libs from managed .
  NET:
   dotnet/docs standard/native-interop/`pinvoke.md`
- UI is Uno's own SkiaSharp-on-Metal render head (distinct from MAUI native UIKit):
   src/Uno.
  UI.
  Runtime.
  Skia.
  AppleUIKit/Rendering/UnoSKMetalView.
  cs (UnoSKMetalView :
   MTKView;
   GRContext.
  CreateMetal)
- HttpClient on iOS routes HTTPS through native NSURLSession via NSUrlSessionHandler (default handler):
   Microsoft.
  iOS Foundation.
  NSUrlSessionHandler API doc (public class NSUrlSessionHandler :
   HttpMessageHandler)
- Uno provides no iOS background abstraction;
   only WinRT BackgroundTask stubs,
   Apple unimplemented:
   src/Uno.
  UWP/ApplicationModel/Background/ (Android-only / generated-throwing stubs;
   rg for BGTask/URLSession in src returned no iOS impl)
- Audio MediaPlayerElement on Apple uses AVFoundation/AudioToolbox (relevant only if app uses Uno's player,
   which app B does not):
   src/Uno.
  UWP/Media/Playback/MediaPlayer.
  Apple.
  cs (using AVFoundation;
   AudioToolbox callbacks)
- Minimum iOS deployment 14.2:
   src/Uno.
  Sdk/targets/Uno.
  Common.
  iOS.
  targets:
  5 (SupportedOSPlatformVersion 14.2)

## Adversarial cite-check

- Wall 2 confirmed:
   confirmed
- Wall 1 confirmed:
   confirmed
- Corrections:
   Wall-1 cited NativeTextSelection.
  cs:
  16 as "objc_msgSend P/Invoke",
   but the actual symbol at that line is objc_msgSendSuper,
   not bare objc_msgSend:
   [DllImport(Constants.
  ObjectiveCLibrary,
   EntryPoint = "objc_msgSendSuper")].
   Minor and corroborating-only (it is still a valid P/Invoke into the Objective-C runtime
    and does not affect the wall-1 conclusion).
   Source:
   /tmp/agent/uno-audit-20260612/src/Uno.
  UI.
  Runtime.
  Skia.
  AppleUIKit/UI/Xaml/Controls/TextBox/NativeTextSelection.
  cs:
  16.
   ||| Framing/precision note (not a factual error in the verdict,
   but worth recording so the team does not over-attribute):
   the cited Uno.
  SingleProject.
  iOS.
  targets:
  4 line proves ONLY that the Mono interpreter is debug-gated (Optimize !
  = true),
   not that release uses full AOT;
   "release/device uses full AOT" is a sound inference from that gate plus the Microsoft.
  iOS workload default plus Apple's JIT ban,
   and it is independently confirmed by Uno's own docs
    at /tmp/agent/uno-audit-20260612/doc/articles/`api-differences.md`:
  31 ("iOS is AOT-only ... .
  NET code must be Ahead-Of-Time (AOT) compiled to run on iOS,
   as a fundamental platform limitation").
   ||| Edge case found in repo-wide grep (does NOT contradict wall-2):
   SamplesApp.
  netcoremobile.
  csproj:
  92 enables UseInterpreter=true + MtouchInterpreter=all under Condition BuildForTestFlight==true.
   This is the sample app's own csproj (not the SDK default real consumer apps inherit),
   and the Mono iOS interpreter is a full-AOT-compatible,
   no-runtime-codegen mode (Apple permits it precisely because it is not a JIT),
   so it does not introduce any JIT/W^X/execmem requirement.
   The no-jit-needed verdict stands.
- Sources checked:
   /tmp/agent/uno-audit-20260612/src/Uno.
  Sdk/targets/Uno.
  SingleProject.
  iOS.
  targets:
  4 (UseInterpreter Condition Optimize !
  = true;
   comment 'Required for C# Hot Reload') -- supports wall-2;
   /tmp/agent/uno-audit-20260612/src/targetframework-override.
  props:
  15-16 (net9.0-ios -> net9.0-ios18.0,
   net10.0-ios -> net10.0-ios26.0 Microsoft.
  iOS TFMs) -- supports iosRuntimeModel;
   /tmp/agent/uno-audit-20260612/src/Uno.
  UI/UI/Xaml/Documents/UnicodeText.
  ICU.
  skia.
  cs:
  198,245,446-490 (line 198 'ICU is included in the unoicu.
  a static library';
   line 245 iOS path dispatches to IOSICUSymbols;
   19 [DllImport("__Internal")] entries,
   first at line 448) -- supports wall-1;
   /tmp/agent/uno-audit-20260612/src/Uno.
  UI.
  Runtime.
  Skia.
  AppleUIKit/UI/Xaml/Controls/TextBox/NativeTextSelection.
  cs:
  16 (DllImport objc_msgSendSuper,
   NOT bare objc_msgSend as cited) -- corroborating wall-1,
   naming discrepancy;
   /tmp/agent/uno-audit-20260612/src/Uno.
  UI.
  Runtime.
  Skia.
  AppleUIKit/Rendering/UnoSKMetalView.
  cs:
  19,53 (class UnoSKMetalView :
   MTKView;
   GRContext.
  CreateMetal) -- supports iosRuntimeModel Skia/Metal head;
   /tmp/agent/dotnet-runtime-audit-20260612/docs/design/mono/web/`aot.md`:
  108 ('Some platforms like the iphone prohibit JITted code,
   using technical and/or legal means... full-aot or aot-only mode... at runtime,
   no code needs to be generated') -- verbatim match,
   supports wall-2;
   /tmp/agent/uno-audit-20260612/doc/articles/`api-differences.md`:
  31 ('iOS is AOT-only',
   fundamental platform limitation) -- independent in-repo corroboration of wall-2;
   /tmp/agent/uno-audit-20260612/src/Directory.
  Build.
  targets:
  79,182 (UnoICUVersion 77.2.1,
   PackageReference Uno.
  icu-ios) -- corroborates ICU static-link wiring for wall-1;
   repo-wide rg 'UseInterpreter' across /tmp/agent/uno-audit-20260612 -- confirmed no iOS Release/device force-enable
    of interpreter;
   only debug-gated SDK default plus sample-app/TestFlight-conditioned and test-script occurrences
