# iOS source-audit: Avalonia

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
   Managed full-AOT.
   App code is C#/MSIL compiled Ahead-of-Time to a native ARM64 binary by the Mono AOT compiler in the .
  NET-for-iOS (Microsoft.
  iOS) workload;
   no JIT on device.
   An optional non-JIT Mono bytecode INTERPRETER (UseInterpreter/MtouchInterpreter) is available as a fallback for code
   that needs dynamic codegen (off by default in release).
   Engine:
   MonoVM full-AOT.
   Avalonia itself contributes no runtime;
   it is an ordinary .
  NET-for-iOS app head (AvaloniaAppDelegate :
   UIResponder,
   IUIApplicationDelegate).
   JIT only happens in the x64 simulator,
   never on device or ARM64 simulator.
- Minimum iOS deployment:
   iOS 13.0 (AvsMinSupportedIOSVersion in build/TargetFrameworks.
  props:
  21);
   builds against ios26.0 SDK (build/TargetFrameworks.
  props:
  8).
- Gate expectation:
   expected-pass
- Confidence:
   high
- Key finding:
   Avalonia is a pure UI/graphics framework riding the .
  NET-for-iOS (Microsoft.
  iOS) workload,
   so on a device its app code runs as Mono FULL-AOT native ARM64 (no JIT,
   wall 2 satisfied) and it links/calls native Go
   or Rust static libs via ordinary P/Invoke (proven in-tree by the bundled native SkiaSharp/HarfBuzz/Metal bindings,
   wall 1 satisfied).
   It contributes NO audio subsystem,
   which means the existing Rust symphonia+cpal core is reused unchanged behind FFI
   (cpal already has a CoreAudio backend) rather than rewritten on AVAudioEngine.
   All three iOS walls are clearable;
   the differentiator versus MAUI is only the UI layer
   (Avalonia draws its own controls with Skia/Metal instead of mapping to native UIKit),
   not the runtime/FFI/background story,
   which are identical.

## Wall 2: JIT / executable memory

Verdict:
 no-jit-needed

Apple forbids dynamically generated code on device,
 so .
NET-for-iOS uses Mono full-AOT to produce a native ARM64 binary;
 no W^X/executable-memory violation.
 Microsoft docs:
 'there is a security restriction on iOS,
 set by Apple,
 which disallows the execution of dynamically generated code on a device...
 iOS and Mac Catalyst apps use an Ahead of Time (AOT) compiler to compile the managed code.
' A JIT attempt under aot-only mode throws System.
ExecutionEngineException 'Attempting to JIT compile method while running in aot-only mode'.
 An optional Mono INTERPRETER (UseInterpreter/MtouchInterpreter) covers Reflection.
Emit-style dynamic-codegen needs WITHOUT JIT (it interprets bytecode,
 satisfying iOS execmem rules) at a speed cost.
 Avalonia inherits this unchanged as a normal app head;
 nothing in its iOS backend needs JIT.

Source:
 Microsoft Learn <https://learn.microsoft.com/en-us/dotnet/maui/macios/interpreter> (AOT-required + no-JIT statements;
 UseInterpreter/MtouchInterpreter);
 Avalonia app head /tmp/agent/avalonia-audit/src/iOS/Avalonia.
iOS/AvaloniaAppDelegate.
cs (AvaloniaAppDelegate<TApp> :
 UIResponder,
 IUIApplicationDelegate)

## Wall 1: link and call a Go/Rust static library

Feasible:
 yes

Mechanism:
 P/Invoke (DllImport / LibraryImport) plus the ObjCRuntime/Foundation Objective-C bindings.
 A Go gomobile c-archive or Rust staticlib is added to the app project as a NativeReference / native lib and called
 from C# via [DllImport("__Internal")] extern declarations against its exported C ABI
 (with delegate* unmanaged<> callbacks).
 This native-link-and-call path is already proven in-tree:
 Avalonia links the native SkiaSharp (libSkiaSharp C/C++) and HarfBuzzSharp libraries
 and drives Apple's Metal framework through bound native types (IMTLDevice/IMTLCommandQueue),
 and CoreFoundation/libc via DllImport.
 So kopia-as-Go-c-archive or a Rust staticlib is callable via the identical P/Invoke mechanism.

Source:
 /tmp/agent/avalonia-audit/src/iOS/Avalonia.
iOS/Interop.
cs (DllImport against CoreFoundation/libc with delegate* unmanaged callbacks);
 /tmp/agent/avalonia-audit/src/iOS/Avalonia.
iOS/Metal/MetalDevice.
cs (Metal IMTLDevice/IMTLCommandQueue bindings);
 /tmp/agent/avalonia-audit/build/SkiaSharp.
props + Directory.
Packages.
props:
53 SkiaSharp 3.119.4 native lib linked into the app

## Wall 3: background execution

Avalonia provides no background-task abstraction;
 its iOS backend only observes UIApplication.
DidEnterBackground to pause the CADisplayLink render loop (DisplayLinkTimer.
cs / AvaloniaAppDelegate.
cs).
 Background transfer is done with the platform APIs the .
NET-for-iOS bindings expose directly from app code:
 Foundation.
NSUrlSessionConfiguration.
CreateBackgroundSessionConfiguration(...) for background URLSession,
 and BGTaskScheduler/BGProcessingTaskRequest (BackgroundTasks namespace) bound in Microsoft.
iOS.
dll.
 A multi-hour kopia snapshot is NOT expressible as a long-running foreground task;
 it must be restructured around background URLSession chunked uploads + BGProcessingTask (short,
 idle/charging) windows,
 identical to every other iOS candidate.

Source:
 /tmp/agent/avalonia-audit/src/iOS/Avalonia.
iOS/DisplayLinkTimer.
cs (ObserveDidEnterBackground pauses render link);
 /tmp/agent/avalonia-audit/src/iOS/Avalonia.
iOS/AvaloniaAppDelegate.
cs (DidEnterBackground observer);
 Microsoft.
iOS binding
<https://learn.microsoft.com/en-us/dotnet/api/foundation.nsurlsessionconfiguration.backgroundsessionconfiguration>
(Assembly Microsoft.
iOS.
dll,
 namespace Foundation)

## In-app HTTP server (kopia S3 target)

Feasible:
 yes

Mechanism:
 Avalonia adds no constraint here;
 an in-process HTTP/S3 endpoint kopia targets is a plain .
NET listener running in the AOT-compiled managed process.
 Either System.
Net.
HttpListener / a Kestrel-style socket server,
 or (cleaner on iOS) a CFNetwork/NSURLSession-backed loopback listener via the bound Foundation/CoreFoundation APIs.
 No exec of a bundled binary is involved,
 so wall 1's no-exec rule is not triggered:
 the server is in-process managed code on 127.0.0.1.

Source:
 /tmp/agent/avalonia-audit/src/iOS/Avalonia.
iOS/Interop.
cs (CoreFoundation/CFRunLoop + libc dispatch bindings available to app code);
 .
NET BCL System.
Net sockets ship in the Microsoft.
iOS runtime (general .
NET-for-iOS capability)

## HTTPS streaming client (pCloud)

Feasible:
 yes

Mechanism:
 Streaming HTTPS to pCloud uses .
NET System.
Net.
Http.
HttpClient,
 which on iOS is backed by NSUrlSessionHandler (Apple's CFNetwork/TLS stack) by default in the Microsoft.
iOS runtime;
 streaming upload/download via request/response content streams and,
 for resilience,
 NSUrlSession background tasks.
 Fully usable from Avalonia app code;
 Avalonia imposes nothing.

Source:
 <https://learn.microsoft.com/en-us/dotnet/api/foundation.nsurlsessionconfiguration.backgroundsessionconfiguration>
 (NSUrlSession in Microsoft.
iOS.
dll,
 the HttpClient backing handler on iOS);
 .
NET-for-iOS HttpClient/NSUrlSessionHandler default

## Audio (music-player port)

Decode:
 Avalonia has ZERO audio code (repo-wide search for AVAudio/CoreAudio/AudioUnit/opus/symphonia in src/ returns nothing;
 only Android haptic-sound + keycode hits).
 Decoding is entirely out-of-scope for the framework,
 so the existing Rust symphonia 0.6 + libopus decoder is reused as-is,
 linked as a Rust staticlib and called via P/Invoke (same mechanism as wall 1).

Output:
 No Avalonia audio output path exists.
 Output is whatever the linked Rust core uses:
 cpal 0.18 has a CoreAudio (AudioUnit) backend for Apple targets,
 so cpal drives low-latency output directly from the Rust core.
 If cpal's iOS backend proves insufficient,
 the alternative is AVAudioEngine via the Microsoft.
iOS AVFoundation bindings,
 but that is a fallback,
 not a requirement.

Rust core reuse:
 REUSED via FFI.
 Because Avalonia provides no audio at all and the runtime is full-AOT C#
 that links native libraries via P/Invoke (proven by SkiaSharp/HarfBuzz/Metal in-tree),
 the symphonia+cpal Rust core links as a staticlib and is called over its C ABI.
 Nothing forces a rewrite onto AVAudioEngine;
 cpal already targets CoreAudio on Apple.
 The only iOS-specific work is AVAudioSession category/activation (route + interruptions),
 which is a thin AVFoundation call,
 not a decoder/output rewrite.

Source:
 repo-wide null search
 (rg -i 'avaudio|coreaudio|audiounit|symphonia|opus' over /tmp/agent/avalonia-audit/src returned no audio engine);
 P/Invoke proof /tmp/agent/avalonia-audit/src/iOS/Avalonia.
iOS/Interop.
cs and SkiaSharp native link build/SkiaSharp.
props

## Gate probe and toolchain

Minimal app:
 `dotnet new` a net-ios Avalonia single-view app head (AvaloniaAppDelegate<App>) referencing Avalonia.
iOS + Avalonia.
Skia;
 add a tiny Rust staticlib exposing `extern \"C\" int add(int,int)`
 (and a second symbol that spins a CoreAudio sine via cpal) linked as a NativeReference,
 called from C# via [DllImport(\"__Internal\")];
 build in RELEASE (AOT,
 not simulator),
 code-sign,
 install on the iPhone X.
 On-device signals that confirm viability:
 (1) Avalonia UI renders via Metal/Skia at the CADisplayLink rate with no System.
ExecutionEngineException 'Attempting to JIT compile method
while running in aot-only mode' in device console (proves full-AOT executes app code);
 (2) the P/Invoke call into the Rust staticlib returns the right value (proves wall-1 native-static-lib linking);
 (3) cpal emits the sine through the speaker (proves Rust audio core reuse).
 Refute signal:
 any aot-only JIT exception or a linker failure resolving the staticlib symbol.

Toolchain:
 macOS + Xcode (iOS SDK,
 simctl,
 codesign,
 device provisioning);
 .
NET 9/10 SDK with the `ios` workload (`dotnet workload install ios`);
 Rust with the `aarch64-apple-ios` target (or Go with gomobile for the kopia c-archive) to build the staticlib;
 a paid Apple Developer signing identity + provisioning profile for the iPhone X
 (matching the existing CLI codesign path in this repo's runbook).

## Supporting-stack vets this framework drags in

- P/Invoke FFI binding vet:
   build kopia as a Go gomobile c-archive (or Rust staticlib) for aarch64-apple-ios,
   link as NativeReference,
   confirm C-ABI symbols resolve under full-AOT and callbacks marshal
- In-process HTTP/S3 endpoint vet:
   stand up the loopback server kopia targets inside the managed process and confirm kopia reaches it on 127.0.0.1
- HTTPS streaming + background URLSession vet:
   HttpClient/NSUrlSessionHandler chunked upload to pCloud with CreateBackgroundSessionConfiguration resumption
- Background execution vet:
   BGProcessingTaskRequest scheduling + URLSession background completion handlers under iOS idle/charging constraints
   (multi-hour snapshot restructure)
- Audio vet:
   link symphonia+cpal Rust staticlib,
   drive CoreAudio output,
   wire AVAudioSession category/interruption handling;
   true-peak normalization + on-disk peak cache run in Rust
- Trimming/AOT-compat vet:
   ILLink trimming + Mono full-AOT generics limitations (no Reflection.
  Emit) across Avalonia + app code;
   decide if MtouchInterpreter -all fallback is needed
- QA vets:
   in-process UI test (Avalonia.
  Headless test harness),
   e2e on-device automation (XCUITest/Appium against the app),
   property-based + fuzz tests on the Rust FFI boundary,
   mutation testing on the managed glue

## Cited sources

- iOS backend exists,
   targets net-ios/maccatalyst/tvos via the .
  NET-for-iOS workload,
   references Avalonia.
  Skia + HarfBuzz:
   /tmp/agent/avalonia-audit/src/iOS/Avalonia.
  iOS/Avalonia.
  iOS.
  csproj
- Min iOS 13.0;
   builds against ios26.0 SDK:
   /tmp/agent/avalonia-audit/build/TargetFrameworks.
  props:
  8,21
- On device Apple disallows dynamic codegen;
   .
  NET-for-iOS uses AOT;
   aot-only JIT attempt throws System.
  ExecutionEngineException;
   optional Mono interpreter (UseInterpreter/MtouchInterpreter) for dynamic-codegen needs without JIT:
   <https://learn.microsoft.com/en-us/dotnet/maui/macios/interpreter>
- Avalonia iOS app head is a normal UIApplicationDelegate (inherits the workload runtime model):
   /tmp/agent/avalonia-audit/src/iOS/Avalonia.
  iOS/AvaloniaAppDelegate.
  cs
- P/Invoke (DllImport) + delegate* unmanaged callbacks against native frameworks already used in-tree:
   /tmp/agent/avalonia-audit/src/iOS/Avalonia.
  iOS/Interop.
  cs
- Apple Metal native bindings (IMTLDevice/IMTLCommandQueue) drive rendering;
   EAGL/OpenGL fallback present:
   /tmp/agent/avalonia-audit/src/iOS/Avalonia.
  iOS/Metal/MetalDevice.
  cs and src/iOS/Avalonia.
  iOS/Eagl/
- Native SkiaSharp 3.119.4 C++ library linked into the app (proves native-lib link path):
   /tmp/agent/avalonia-audit/build/SkiaSharp.
  props + /tmp/agent/avalonia-audit/Directory.
  Packages.
  props:
  53
- Render loop driven by CADisplayLink;
   backgrounding only pauses the render link:
   /tmp/agent/avalonia-audit/src/iOS/Avalonia.
  iOS/DisplayLinkTimer.
  cs
- Background URLSession available from app code (Microsoft.
  iOS.
  dll,
   Foundation namespace):
   <https://learn.microsoft.com/en-us/dotnet/api/foundation.nsurlsessionconfiguration.backgroundsessionconfiguration>
- Avalonia has no audio subsystem (repo-wide search finds no AVAudio/CoreAudio/symphonia/opus engine,
   only Android haptic-sound + keycodes):
   rg -i over /tmp/agent/avalonia-audit/src (null result for audio engine APIs)
- Avalonia embeds native UIViews via NativeControlHostImpl (first-class UIKit interop):
   /tmp/agent/avalonia-audit/src/iOS/Avalonia.
  iOS/NativeControlHostImpl.
  cs

## Adversarial cite-check

- Wall 2 confirmed:
   confirmed
- Wall 1 confirmed:
   confirmed
- Corrections:
   No load-bearing claim is fabricated,
   unsupported,
   or contradicted;
   all cited sources exist and say what the audit reports.
   Wall-2 quotes are verbatim-faithful to the Microsoft Learn doc:
   "there is a security restriction on iOS,
   set by Apple,
   which disallows the execution of dynamically generated code on a device" and "iOS
   and Mac Catalyst apps use an Ahead of Time (AOT) compiler to compile the managed code" both appear word-for-word,
   as does the System.
  ExecutionEngineException message "Attempting to JIT compile method while running in aot-only mode".
   The UseInterpreter/MtouchInterpreter non-JIT interpreter mechanism (covering Reflection.
  Emit-style dynamic codegen while abiding by platform restrictions) is correctly described.
   Wall-1 source files all exist and support the P/Invoke claim:
   Interop.
  cs has DllImport against CoreFoundation/libc with delegate* unmanaged<> callbacks;
   MetalDevice.
  cs binds IMTLDevice/IMTLCommandQueue;
   SkiaSharp 3.119.4 is at Directory.
  Packages.
  props:
  53;
   and HarfBuzzSharp 8.3.1.3 (which I additionally verified,
   since the audit named it) is present in Directory.
  Packages.
  props and build/HarfBuzzSharp.
  props,
   parallel to SkiaSharp.
   Two minor framing caveats (NOT verdict-changing,
   both walls remain confirmed):
   (1) The cited doc is titled "Mono interpreter on iOS and Mac Catalyst - .
  NET MAUI" (a .
  NET MAUI doc),
   whereas the audit applies it to generic ".
  NET-for-iOS";
   the facts transfer because MtouchInterpreter is a workload-level Microsoft.
  iOS/mtouch property,
   but the source is MAUI-titled and the audit treats it as generic without noting this.
   (2) Interop.
  cs demonstrates [DllImport] against NAMED dynamic libraries (CoreFoundation framework path,
   /usr/lib/libc.
  dylib),
   not the [DllImport("__Internal")] static-archive variant the audit extrapolates for a kopia Go c-archive / Rust
   staticlib;
   the extrapolation is technically correct but the cited file shows the named-dylib pattern,
   not the __Internal static-link pattern.
- Sources checked:
   /tmp/agent/avalonia-audit/src/iOS/Avalonia.
  iOS/Interop.
  cs (read:
   DllImport vs CoreFoundation/libc + delegate* unmanaged callbacks confirmed,
   lines 24-57);
   /tmp/agent/avalonia-audit/src/iOS/Avalonia.
  iOS/AvaloniaAppDelegate.
  cs (read:
   AvaloniaAppDelegate<TApp> :
   UIResponder,
   IUIApplicationDelegate confirmed,
   line 21);
   /tmp/agent/avalonia-audit/src/iOS/Avalonia.
  iOS/Metal/MetalDevice.
  cs (read:
   IMTLDevice/IMTLCommandQueue bindings confirmed,
   lines 13-23);
   /tmp/agent/avalonia-audit/build/SkiaSharp.
  props (read:
   SkiaSharp PackageReference confirmed);
   /tmp/agent/avalonia-audit/build/HarfBuzzSharp.
  props (grep:
   HarfBuzzSharp PackageReference confirmed,
   parallel to SkiaSharp);
   /tmp/agent/avalonia-audit/Directory.
  Packages.
  props (read:
   SkiaSharp 3.119.4 at line 53;
   HarfBuzzSharp 8.3.1.3 confirmed);
   <https://learn.microsoft.com/en-us/dotnet/maui/macios/interpreter> (WebFetch:
   verbatim Apple-restriction + AOT + ExecutionEngineException + UseInterpreter/MtouchInterpreter text confirmed;
   doc is MAUI-titled)
