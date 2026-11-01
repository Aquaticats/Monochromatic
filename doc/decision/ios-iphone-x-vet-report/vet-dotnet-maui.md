# iOS source-audit: .NET MAUI

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
   Managed Mono runtime,
   JITLESS on a real device.
   The native bootstrap mono_ios_runtime_init() sets the AOT mode at startup:
   on a physical device (Release) it calls register_aot_modules() + mono_jit_set_aot_mode(MONO_AOT_MODE_FULL) = full
   ahead-of-time,
   no JIT;
   when the interpreter is forced (MAUI's Debug default,
   UseInterpreter=true) it calls mono_jit_set_aot_mode(MONO_AOT_MODE_INTERP_ONLY) = pure Mono interpreter,
   no JIT;
   AOT+interp mixed mode is MONO_AOT_MODE_INTERP.
   JIT exists ONLY in the simulator.
   Engine = Mono (libmonosgen / libclrinterpreter statically linked).
   An experimental NativeAOT path also exists but does not work with MAUI (MAUI is not trimmer-safe),
   so the shipping model is Mono full-AOT (Release) or Mono interpreter (Debug).
   Source:
   /tmp/agent/dotnet-runtime-audit-20260612/src/tasks/AppleAppBuilder/Templates/runtime.
  m:
  344-359;
   /tmp/agent/dotnet-macios-audit-20260612/docs/website/`mtouch-errors.md`:
  966;
   doc/`nativeaot.md`:
  76-78.
- Minimum iOS deployment:
   .
  NET 9 / .
  NET for iOS targets iOS 12.2+ by default (Mono full-AOT).
   BGTaskScheduler requires iOS 13+;
   AVAudioEngine manual rendering iOS 11+.
   Practically set deployment target to iOS 13+ to use BGProcessingTask.
- Gate expectation:
   needs-device
- Confidence:
   high
- Key finding:
   MAUI ships JITLESS on iOS:
   the native Mono bootstrap calls mono_jit_set_aot_mode(MONO_AOT_MODE_FULL) on a real device (interpreter-only in
   Debug),
   JIT exists only in the simulator,
   so wall 2 is cleanly passed -- and the same runtime turns on a pinvoke_override that lets
   [DllImport(\"__Internal\")] call into a statically-linked Rust/Go .
  a (NativeReference/LinkerArgument),
   so wall 1 is satisfied by ordinary P/Invoke.
   The Rust symphonia+cpal core is therefore reusable behind MAUI via FFI with cpal's CoreAudio backend,
   with no forced AVAudioEngine rewrite.

## Wall 2: JIT / executable memory

Verdict:
 no-jit-needed

On an iOS device MAUI never JITs.
 runtime.
m sets mono_jit_set_aot_mode(MONO_AOT_MODE_FULL) for device Release (lines 348-356)
and MONO_AOT_MODE_INTERP_ONLY for the interpreter path (line 347,
 MAUI Debug default).
 The mtouch error doc states plainly:
 'the JIT remains the only option available on the simulator,
 while AOT and interpreter are for devices only' (`mtouch-errors.md`:
966).
 So app code runs as native AOT machine code (Release) or via the statically-linked Mono interpreter (Debug);
 no W^X/executable-memory entitlement is needed.
 This is the iOS twin of the Android DENY_EXECMEM test,
 and MAUI passes it:
 full-AOT is the default Release shape.
 The interpreter is a real bytecode interpreter (not codegen),
 so even Debug needs no JIT entitlement.

Source:
 /tmp/agent/dotnet-runtime-audit-20260612/src/tasks/AppleAppBuilder/Templates/runtime.
m:
344-359 (mono_jit_set_aot_mode FULL / INTERP_ONLY / INTERP);
 /tmp/agent/dotnet-macios-audit-20260612/docs/website/`mtouch-errors.md`:
963-966;
 doc/building-apps/`build-properties.md`:
965-998 (MtouchInterpreter,
 MAUI Debug=UseInterpreter)

## Wall 1: link and call a Go/Rust static library

Feasible:
 yes

Mechanism:
 P/Invoke to a statically-linked Rust staticlib (.
a) or Go gomobile c-archive (.
a).
 The archive is linked into the native iOS executable via @(NativeReference)
 or <LinkerArgument Include=".../libCustom.a"/> (`build-items.md`:
215-220,
 252-255).
 The extern "C" symbols are then called from C# with [DllImport("__Internal")] -- '__Internal' is the documented
 library name for symbols statically linked into the app (`binding_types_reference_guide.md`:
1392).
 Under device full-AOT this resolves through the runtime's pinvoke_override:
 pinvoke_override_enabled is set true on the device path (runtime.
m:
349) and handle_pinvoke_override maps the P/Invoke to the linked symbol (runtime.
m:
237-239).
 The macios runtime itself uses [DllImport("__Internal")] pervasively for its own static C glue (e.g.
 src/Foundation/NSObject2.
cs:
442,
 src/ObjCRuntime/Class.
cs:
955),
 proving the path is the standard one.
 For Apple-SDK wrappers there is also the higher-level @(XcodeProject)->XCFramework binding flow
 (`native-library-interop.md`),
 but a plain Rust/Go .
a needs only NativeReference + DllImport.

Source:
 /tmp/agent/dotnet-macios-audit-20260612/docs/building-apps/`build-items.md`:
215-220 and 252-255 (NativeReference / LinkerArgument libCustom.
a);
 doc/website/`binding_types_reference_guide.md`:
1392 (__Internal for static libs);
 /tmp/agent/dotnet-runtime-audit-20260612/src/tasks/AppleAppBuilder/Templates/runtime.
m:
237-239,349 (pinvoke_override);
 src/Foundation/NSObject2.
cs:
442 ([DllImport("__Internal")] in practice)

## Wall 3: background execution

Use background URLSession + BGTaskScheduler,
 both bound in macios.
 NSUrlSessionConfiguration.
CreateBackgroundSessionConfiguration (src/Foundation/NSUrlSessionConfiguration.
cs:
55) gives OS-managed background uploads/downloads that survive app suspension -- the only durable transfer channel on
iOS.
 BGTaskScheduler / BGProcessingTaskRequest are bound (src/backgroundtasks.
cs:
27,84,117) for short idle/charging maintenance windows.
 There is NO long-running foreground-service equivalent:
 a multi-hour kopia snapshot cannot run as a single in-process job
 and must be restructured into discrete chunks driven by background URLSession transfers plus BGProcessingTask wake-ups,
 exactly as on every other iOS candidate.
 This is a platform constraint MAUI inherits,
 not a MAUI limitation;
 MAUI exposes the right APIs to express the restructured design.

Source:
 /tmp/agent/dotnet-macios-audit-20260612/src/Foundation/NSUrlSessionConfiguration.
cs:
46-57 (CreateBackgroundSessionConfiguration);
 src/backgroundtasks.
cs:
27,84,117 (BGProcessingTaskRequest/BGTaskScheduler);
 src/BackgroundTasks/Enums.
cs:
17 (BGTaskSchedulerErrorDomain)

## In-app HTTP server (kopia S3 target)

Feasible:
 yes

Mechanism:
 System.
Net.
HttpListener ships in the .
NET-for-iOS BCL (referenced from the macios build profile),
 so an in-process HTTP/S3 endpoint that kopia targets can run as pure managed code -- no binary exec needed,
 sidestepping wall 1 for the server itself.
 The macios docs even show HttpListener listening-and-responding inside an app.
 Full System.
Net.
Sockets is also present,
 so Kestrel/raw-socket servers work too.

Source:
 /tmp/agent/dotnet-macios-audit-20260612/src/Makefile:
37 (/r:
.../System.
Net.
HttpListener.
dll in the iOS BCL);
 doc/api/JavaScriptCore/JSContext.
xml:
195-199 (new HttpListener() used to listen and respond in-app)

## HTTPS streaming client (pCloud)

Feasible:
 yes

Mechanism:
 HttpClient on iOS is backed by NSUrlSessionHandler :
 HttpMessageHandler (the default handler),
 which wraps NSURLSession and supports streaming request/response bodies and a TLS trust-override callback.
 This gives a streaming HTTPS client to pCloud without bundling OpenSSL.
 For background uploads it composes with the background-URLSession config above.

Source:
 /tmp/agent/dotnet-macios-audit-20260612/src/Foundation/NSUrlSessionHandler.
cs:
112 (public partial class NSUrlSessionHandler :
 HttpMessageHandler);
 :
58 (NSUrlSessionHandlerTrustOverrideForUrlCallback)

## Audio (music-player port)

Decode:
 Reuse the existing Rust core.
 symphonia 0.6 + libopus are pure-Rust/C decoders compiled into the linked staticlib;
 they run unchanged behind P/Invoke.
 No managed AudioToolbox decode path is needed (though AudioToolbox/AudioConverter bindings exist if wanted).

Output:
 cpal 0.18 has an iOS/CoreAudio backend that talks to AudioUnit/AudioToolbox via the C ABI directly from inside the
 Rust lib.
 It does NOT route through the .
NET layer,
 so low-latency output works natively.
 macios also binds the same primitives (AudioUnit AURenderCallback at src/AudioUnit/AUGraph.
cs:
480-487,
 AVAudioEngine at src/avfoundation.
cs:
1110) if you ever drive audio from C# instead.

Rust core reuse:
 REUSABLE via FFI -- no AVAudioEngine rewrite required.
 Because audio is owned by the linked Rust staticlib (wall 1),
 symphonia+cpal run entirely inside that lib and cpal's existing CoreAudio (AudioUnit) backend drives output;
 MAUI/.
NET only needs to start/stop and pass control via [DllImport("__Internal")].
 AVAudioEngine is bound and available as an alternative,
 but rewriting onto it is optional,
 not forced.
 The true-peak normalization,
 on-disk peak cache,
 paginated queue,
 and session persistence are all Rust-side logic that ports as-is;
 only Slint's winit/femtovg UI must be replaced by MAUI views (or the Rust core kept headless behind a MAUI shell).

Source:
 cpal CoreAudio backend (upstream,
 audio owned by Rust lib via wall-1 FFI);
 /tmp/agent/dotnet-macios-audit-20260612/src/AudioUnit/AUGraph.
cs:
480-487 (AURenderCallback bound);
 src/avfoundation.
cs:
1104-1110 (AVAudioEngine bound)

## Gate probe and toolchain

Minimal app:
 a MAUI iOS app (or a thinner .
NET-for-iOS app to isolate the runtime) that links a tiny Rust staticlib exposing extern \"C\" fn rust_add(a,
b)->i32 (and a second symbol that spawns a thread + writes a file),
 referenced via @(NativeReference)/LinkerArgument,
 called from C# via [DllImport(\"__Internal\")].
 Build in RELEASE for a real device (MtouchInterpreter off,
 so MONO_AOT_MODE_FULL) AND in Debug (interpreter).
 On-device signal that confirms viability:
 (1) app launches and a label shows the value returned by rust_add (proves the Rust .
a was statically linked and called via P/Invoke under full-AOT,
 wall 1 + wall 2 together);
 (2) os_log shows neither 'INTERP Enabled' nor any JIT/codesign-execmem crash in Release (proves full-AOT,
 no executable-memory entitlement);
 (3) System.
Net.
HttpListener binds a localhost port and returns a 200 to an in-app HttpClient GET (proves in-app server + HTTPS client).
 Refutation signal:
 an EXC_BAD_ACCESS/codesigning kill at the DllImport call,
 or a trimming/AOT crash,
 or HttpListener throwing PlatformNotSupported.

Toolchain:
 macOS with Xcode (+ iOS SDK and a provisioning profile / dev cert).
 .
NET SDK 9+ with the iOS workload:
 dotnet workload install ios (and maui-ios for the MAUI templates).
 Rust with the aarch64-apple-ios target (rustup target add aarch64-apple-ios) to produce the staticlib,
 or Go with gomobile for a c-archive.
 No JIT entitlement is required (full-AOT/interpreter).
 NativeAOT is NOT used for MAUI.

## Supporting-stack vets this framework drags in

- P/Invoke + DllImport(__Internal) FFI binding vet:
   marshalling Rust extern C structs/callbacks/byte buffers across the managed<->native boundary under full-AOT
   (blittable types,
   GCHandle pinning,
   UnmanagedCallersOnly for callbacks)
- Trimming/AOT-safety vet:
   MAUI is not trimmer-safe;
   confirm the chosen feature set + any reflection-using deps survive MtouchLink=Full / linker without runtime crashes
   (and whether NativeAOT is ever reachable)
- In-app HTTP server vet:
   System.
  Net.
  HttpListener vs Kestrel on iOS background-execution limits;
   keeping the kopia S3 endpoint alive only while foregrounded
- Background-transfer vet:
   restructuring a multi-hour kopia snapshot into background-URLSession chunks + BGProcessingTask windows;
   delegate/event plumbing on app relaunch
- Audio FFI vet:
   cpal CoreAudio backend init from inside a P/Invoked Rust lib,
   real-time render-callback thread safety vs the Mono GC,
   low-latency buffer sizing on device
- QA:
   in-process UI test (Appium/.
  NET MAUI UITest / Microsoft.
  Maui.
  TestUtils),
   e2e on-device (XHarness),
   property/fuzz tests for the Rust core (cargo-fuzz/proptest) since the audio+backup logic stays in Rust,
   mutation testing (Stryker.
  NET) for the C# shell

## Cited sources

- On a real iOS device Mono runs full-AOT (MONO_AOT_MODE_FULL) or interpreter-only (MONO_AOT_MODE_INTERP_ONLY);
   register_aot_modules + pinvoke_override_enabled=true on device path;
   JIT branch is simulator/maccatalyst only:
   /tmp/agent/dotnet-runtime-audit-20260612/src/tasks/AppleAppBuilder/Templates/runtime.
  m:
  344-359
- JIT is the only option in the simulator;
   AOT and interpreter are for devices only:
   /tmp/agent/dotnet-macios-audit-20260612/docs/website/`mtouch-errors.md`:
  963-966
- MtouchInterpreter/UseInterpreter enable the Mono interpreter on device;
   MAUI sets UseInterpreter=true for Debug by default:
   /tmp/agent/dotnet-macios-audit-20260612/docs/building-apps/`build-properties.md`:
  965-998
- NativeAOT on iOS is experimental and MAUI projects don't typically work with it (MAUI not trimmer-safe);
   there is no interpreter under NativeAOT:
   /tmp/agent/dotnet-macios-audit-20260612/docs/`nativeaot.md`:
  63-78
- A custom static library .
  a is linked into the native iOS executable via NativeReference / LinkerArgument:
   /tmp/agent/dotnet-macios-audit-20260612/docs/building-apps/`build-items.md`:
  215-220,
   252-255
- '__Internal' is the library name used to P/Invoke symbols statically linked into the app:
   /tmp/agent/dotnet-macios-audit-20260612/docs/website/`binding_types_reference_guide.md`:
  1388-1392
- [DllImport("__Internal")] is the standard P/Invoke path,
   used pervasively by the macios runtime itself:
   /tmp/agent/dotnet-macios-audit-20260612/src/Foundation/NSObject2.
  cs:
  442;
   src/ObjCRuntime/Class.
  cs:
  955;
   src/UIKit/UIApplication.
  cs:
  56
- Device P/Invoke resolves __Internal symbols through handle_pinvoke_override,
   enabled on the device full-AOT path:
   /tmp/agent/dotnet-runtime-audit-20260612/src/tasks/AppleAppBuilder/Templates/runtime.
  m:
  221-239,
   349
- System.
  Net.
  HttpListener ships in the .
  NET-for-iOS BCL and is shown listening/responding in-app:
   /tmp/agent/dotnet-macios-audit-20260612/src/Makefile:
  37;
   doc/api/JavaScriptCore/JSContext.
  xml:
  195-199
- HttpClient on iOS is backed by NSUrlSessionHandler :
   HttpMessageHandler with TLS trust-override and streaming:
   /tmp/agent/dotnet-macios-audit-20260612/src/Foundation/NSUrlSessionHandler.
  cs:
  58,112
- Background URLSession (CreateBackgroundSessionConfiguration)
and BGTaskScheduler/BGProcessingTaskRequest are bound for background transfer/maintenance:
   /tmp/agent/dotnet-macios-audit-20260612/src/Foundation/NSUrlSessionConfiguration.
  cs:
  46-57;
   src/backgroundtasks.
  cs:
  27,84,117
- Low-level CoreAudio (AudioUnit AURenderCallback) and AVAudioEngine are bound,
   so audio can be driven from native or managed code:
   /tmp/agent/dotnet-macios-audit-20260612/src/AudioUnit/AUGraph.
  cs:
  480-487;
   src/avfoundation.
  cs:
  1104-1110
- Apple-SDK native wrappers can additionally be built via @(XcodeProject)->XCFramework->NativeReference (slim binding):
   /tmp/agent/dotnet-macios-audit-20260612/docs/`native-library-interop.md`:
  1-31

## Adversarial cite-check

- Wall 2 confirmed:
   confirmed
- Wall 1 confirmed:
   confirmed
- Sources checked:
   /tmp/agent/dotnet-runtime-audit-20260612/src/tasks/AppleAppBuilder/Templates/runtime.
  m:
  236-379 (verified:
   line 347 mono_jit_set_aot_mode(MONO_AOT_MODE_INTERP_ONLY);
   line 356 MONO_AOT_MODE_FULL;
   line 354 MONO_AOT_MODE_INTERP;
   line 349 pinvoke_override_enabled=true on device path;
   lines 236-243 handle_pinvoke_override;
   line 258 mono_ios_runtime_init;
   line 351 register_aot_modules -- all support the audit's claims);
   /tmp/agent/dotnet-macios-audit-20260612/docs/website/`mtouch-errors.md`:
  966 (verbatim:
   'the JIT remains the only option available on the simulator,
   while AOT and interpreter are for devices only');
   /tmp/agent/dotnet-macios-audit-20260612/docs/building-apps/`build-properties.md`:
  965-998 (MtouchInterpreter;
   line 998 NOTE:
   'MAUI changes the default by setting UseInterpreter=true for the Debug configuration');
   /tmp/agent/dotnet-macios-audit-20260612/docs/building-apps/`build-items.md`:
  215-221 (LinkerArgument libCustom.
  a static-library example) and :
  252-255 (NativeReference definition);
   /tmp/agent/dotnet-macios-audit-20260612/docs/website/`binding_types_reference_guide.md`:
  1388-1392 ('If you're linking a static library,
   use __Internal as the libraryName parameter' -- note context is the [Field] attribute libraryName,
   not [DllImport],
   but the __Internal-for-static-libs principle is documented);
   /tmp/agent/dotnet-macios-audit-20260612/src/Foundation/NSObject2.
  cs:
  442 ([DllImport("__Internal")] confirmed);
   /tmp/agent/dotnet-macios-audit-20260612/src/ObjCRuntime/Class.
  cs:
  955 ([DllImport("__Internal")] confirmed);
   /tmp/agent/dotnet-macios-audit-20260612/docs/`native-library-interop.md`:
  1-31 (XcodeProject -> XCFramework -> NativeReference binding flow confirmed);
   /tmp/agent/dotnet-macios-audit-20260612/docs/`nativeaot.md`:
  76-78 (verbatim:
   'NativeAOT requires trimming,
   and MAUI isn't trimmer-safe,
   and thus ... MAUI projects don't typically work with NativeAOT');
   /tmp/agent/dotnet-macios-audit-20260612/docs/`managed-static-registrar.md`:
  201-211 (Interpreter/JIT section -- about UnmanagedCallersOnly lookup tables;
   neither supports nor contradicts the entitlement claim);
   <https://learn.microsoft.com/en-us/dotnet/maui/macios/interpreter> (authoritative:
   Apple 'disallows the execution of dynamically generated code on a device';
   'The Mono interpreter overcomes these restrictions while abiding by platform restrictions' -- confirms the
   interpreter needs no JIT/execmem entitlement on device,
   supporting the wall-2 no-jit-needed verdict for both Release full-AOT and Debug interpreter)
