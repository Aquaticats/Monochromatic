# iOS source-audit: NativeScript

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
   Interpreter (managed/scripted,
   NOT AOT).
   App logic is JavaScript executed by an embedded V8 that is forced into jitless mode on every device build,
   so JS runs on V8's Ignition bytecode interpreter with TurboFan/Sparkplug/Liftoff disabled (no JIT,
   no WebAssembly).
   Native iOS APIs are reached at runtime through a metadata-driven libffi bridge (no AOT-generated glue).
   Engine:
   V8 (built v8_enable_lite_mode=true,
   v8_enable_webassembly=false) started with --jitless.
- Minimum iOS deployment:
   iOS 13.0 (IPHONEOS_DEPLOYMENT_TARGET = 13.0 in project-template-ios/__PROJECT_NAME__.
  xcodeproj/project.
  pbxproj:
  356,410;
   ios_deployment_target=13 in build_v8_source.
  sh:
  32)
- Gate expectation:
   needs-device
- Confidence:
   high
- Key finding:
   NativeScript runs V8 with --jitless on every iOS build,
   unconditionally (Runtime.
  mm:
  255),
   so app JS executes on V8's Ignition bytecode interpreter with no JIT and no executable-memory codegen,
   and its native-callback trampolines use libffi's FFI_EXEC_TRAMPOLINE_TABLE,
   which vm_remaps a static pre-signed code page instead of generating code (closures.
  c:
  204-213).
   Therefore it needs NO JIT/dynamic-codesigning entitlement and clears iOS wall 2,
   the exact inverse of the Android DENY_EXECMEM disqualification,
   while its metadata + dlsym + libffi bridge can link and call a Rust/Go C-ABI static lib (kopia,
   and the symphonia+cpal audio core) from app code,
   clearing wall 1.

## Wall 2: JIT / executable memory

Verdict:
 interpreter-fallback

Survives wall 2 by running interpreted,
 not by AOT compiling.
 Two halves,
 both sourced.
 (1) JS execution:
 /tmp/agent/nativescript-audit-20260612/NativeScript/runtime/Runtime.
mm:
255 passes '--jitless' to V8:
:
SetFlagsFromString UNCONDITIONALLY (both IsDebug and release branches:
 debug='--expose_gc --jitless',
 release='--expose_gc --jitless --no-lazy').
 V8 is also built v8_enable_lite_mode=true / v8_enable_webassembly=false (build_v8_source.
sh).
 Jitless V8 = Ignition bytecode interpreter only,
 TurboFan/Sparkplug/Liftoff off,
 zero executable-memory allocation for JS.
 The project's own README:
101 states 'The --jitless mode in which V8 is running is explained in...'.
 This is the exact iOS twin of the Android DENY_EXECMEM situation,
 except here NativeScript pre-emptively runs jitless so it does NOT need W^X/executable-codegen pages,
 which is why it ships on the App Store and on locked-down iOS where Android NativeScript was disqualified.
 (2) Native-callback trampolines:
 every JS-backed ObjC delegate/handler is a libffi closure (Interop.
mm:
66-71 ffi_closure_alloc/ffi_prep_closure_loc).
 The NativeScript libffi fork (branch darind/v8-ios) compiles the aarch64 Apple port
  with FFI_EXEC_TRAMPOLINE_TABLE (src/aarch64/ffitarget.
h:
58-65,
 guarded on __MACH__).
 closures.
c:
204-213 allocates closures via vm_remap of a STATIC,
 already-executable,
 code-signed template page (&ffi_closure_trampoline_table_page,
 declared closures.
c:
153) on top of a placeholder page;
 per-closure data goes in a separate writable,
 non-executable config page (closures.
c:
161).
 No machine code is generated at runtime and no fresh PROT_EXEC page is mmap'd,
 so the callback path is W^X-compliant and needs no dynamic-codesigning (JIT) entitlement.
 Net:
 no JIT entitlement required;
 perf cost is interpreted JS.

Source:
 /tmp/agent/nativescript-audit-20260612/NativeScript/runtime/Runtime.
mm:
255 ;
 `README.md`:
101 ;
 build_v8_source.
sh (v8_enable_lite_mode=true,
 v8_enable_webassembly=false) ;
 /tmp/agent/nativescript-audit-20260612/NativeScript/runtime/Interop.
mm:
66-71 ;
 /tmp/agent/libffi-ns-20260612/src/aarch64/ffitarget.
h:
58-65 ;
 /tmp/agent/libffi-ns-20260612/src/closures.
c:
144-153,204-213

## Wall 1: link and call a Go/Rust static library

Feasible:
 yes

Mechanism:
 Metadata-driven libffi bridge plus dlsym,
 the same path used for every ObjC/C call.
 A Rust staticlib (crate-type=staticlib) or Go gomobile c-archive that exposes a C-ABI header is bridged in two steps:
 (a) build time,
 the metadata-generator parses the C header with clang into FunctionMeta (MetaFactory:
:
createFromFunction,
 metadata-generator/src/Meta/MetaFactory.
h:
49;
 FunctionMeta/MetaType:
:
Function,
 MetaEntities.
h:
276-280) so the C functions and FunctionPointerType structs become callable metadata;
 (b) runtime,
 the symbol is resolved by SymbolLoader and the call is marshalled through libffi.
 SymbolLoader.
mm:
131-137 falls back to dlsym(RTLD_DEFAULT,
 symbolName),
 which resolves symbols statically linked into the app's main executable,
 exactly where a Rust staticlib / Go c-archive's exported C symbols live.
 The actual invocation is FFICall (runtime/FFICall.
cpp builds the ffi_cif from TypeEncoding) and Interop.
mm:
137 ffi_call(cif,
 FFI_FN(functionPointer),
 ...).
 So the test 'can this framework link and call a Go/Rust static lib from app code' is YES,
 via the libffi/dlsym C-function bridge (not P/Invoke,
 not dart:
ffi,
 not JSI;
 it is NativeScript's own metadata+libffi interop).
 kopia as a gomobile c-archive .
xcframework,
 or a Rust staticlib,
 links into the app and is called this way.

Source:
 /tmp/agent/nativescript-audit-20260612/NativeScript/runtime/SymbolLoader.
mm:
131-137 (dlsym RTLD_DEFAULT) ;
 /tmp/agent/nativescript-audit-20260612/NativeScript/runtime/FFICall.
cpp ;
 /tmp/agent/nativescript-audit-20260612/NativeScript/runtime/Interop.
mm:
137 (ffi_call) ;
 metadata-generator/src/Meta/MetaFactory.
h:
49 ;
 metadata-generator/src/Meta/MetaEntities.
h:
276-280

## Wall 3: background execution

NativeScript adds no background runtime of its own;
 it exposes the full iOS SDK to JS via the metadata bridge,
 so the background story is iOS-native and must be restructured the same way as for a native app.
 A multi-hour kopia snapshot cannot run as a foreground service.
 The expressible mechanisms,
 all reachable from JS through the bridge,
 are:
 NSURLSession background sessions (URLSessionConfiguration.
background) for upload/download that the system continues after suspension,
 and BGTaskScheduler (BGProcessingTaskRequest,
 requiresExternalPower/requiresNetworkConnectivity) for short,
 idle/charging-gated work.
 The runtime itself (Runtime.
mm) does not extend background execution.
 So the kopia-to-pCloud transfer must be decomposed
  into discrete background-URLSession transfers plus BGProcessingTask resumption,
 not a single long-lived process.
 There is no NativeScript-specific blocker here and no NativeScript-specific help;
 the constraint is purely Apple's.
 Note:
 jitless interpreted JS is also CPU-slower,
 which matters for any in-process hashing/chunking the snapshot would do during those windows.

Source:
 Architectural:
 NativeScript exposes BGTaskScheduler/NSURLSession via the metadata bridge (metadata-generator parses the iOS SDK
  headers;
 SymbolLoader.
mm resolves system frameworks,
 e.g. CFBundle path /System/Library/Frameworks SymbolLoader.
mm:
102).
 No background-extension code exists in /tmp/agent/nativescript-audit-20260612/NativeScript/runtime (grep
  for BGTask/URLSession in runtime returns no runtime-side background logic).
 Apple platform fact:
 developer.
apple.
com/documentation/backgroundtasks and URLSessionConfiguration.
background.

## In-app HTTP server (kopia S3 target)

Feasible:
 yes

Mechanism:
 No HTTP server ships in the runtime,
 but the full iOS networking stack is callable from JS via the metadata bridge,
 so an in-app HTTP/S3 endpoint that kopia targets at 127.0.0.1 can be built on Network.
framework (NWListener) or CFSocket/POSIX sockets,
 all exposed through metadata + libffi.
 Alternatively an Objective-C/Swift micro-server (e.g. GCDWebServer / Telegraph) is linked as a pod and driven
  from JS the same way.
 There is no iOS prohibition on listening on a loopback socket in-process.
 Pure-JS server frameworks would run on the slow jitless interpreter,
 so the native-socket route is preferred for the streaming endpoint kopia hammers.

Source:
 Bridge mechanism:
 SymbolLoader.
mm:
91-111 (resolves system frameworks via CFBundle,
 e.g. /System/Library/Frameworks) + Interop.
mm/FFICall.
cpp (libffi marshalling).
 No server in /tmp/agent/nativescript-audit-20260612/NativeScript/runtime (grep).
 Capability is the iOS Network.
framework,
 callable through NativeScript's metadata bridge.

## HTTPS streaming client (pCloud)

Feasible:
 yes

Mechanism:
 NSURLSession (foreground + background) and its streaming/delegate APIs (URLSessionStreamTask,
 dataTask with delegate callbacks) are reachable from JS through the metadata bridge.
 Streaming upload to pCloud is expressed as an NSURLSession upload task with an InputStream/file body;
 progress and chunk callbacks arrive as JS-backed ObjC delegate methods,
 which run through the libffi closure trampolines (the FFI_EXEC_TRAMPOLINE_TABLE path verified for wall 2).
 TLS is handled by the OS (no in-app crypto codegen).
 So an HTTPS streaming client to pCloud,
 including background transfer,
 is feasible with native URLSession behind a thin JS layer.

Source:
 Delegate-callback mechanism:
 Interop.
mm:
66-71 (ffi_closure_alloc/ffi_prep_closure_loc) + /tmp/agent/libffi-ns-20260612/src/closures.
c:
204-213 (static trampoline table).
 System framework resolution:
 SymbolLoader.
mm:
91-111.
 Capability is iOS NSURLSession,
 callable via the metadata bridge.

## Audio (music-player port)

Decode:
 symphonia 0.6 (all features) + libopus decode entirely inside the Rust staticlib;
 symphonia is pure Rust and needs no executable-memory generation,
 so it runs unchanged under iOS.
 True-peak normalization and the on-disk peak cache also stay in Rust.

Output:
 cpal 0.18 has an iOS/CoreAudio (AudioUnit) backend,
 so low-latency output stays in Rust through cpal;
 AVAudioSession category/activation (low-latency session setup,
 route handling) is an ObjC API set once from the NativeScript metadata bridge,
 not from cpal.
 No AVAudioEngine rewrite of the audio core is required.

Rust core reuse:
 REUSED via FFI,
 not rewritten.
 The existing symphonia+cpal Rust core (the music-player core) compiles as a staticlib and is driven
  through the exact same dlsym + libffi C-function bridge as kopia (wall 1).
 NativeScript is only the UI shell on top;
 it neither helps nor hinders the Rust audio pipeline.
 The Slint/winit/femtovg UI layer does NOT come along (NativeScript supplies its own UI),
 so the port reuses the Rust audio/normalization core behind a C-ABI and rebuilds the UI in NativeScript.

Source:
 Reuse mechanism identical to wall 1:
 /tmp/agent/nativescript-audit-20260612/NativeScript/runtime/SymbolLoader.
mm:
131-137 (dlsym RTLD_DEFAULT) + FFICall.
cpp/Interop.
mm:
137 (ffi_call).
 cpal CoreAudio backend is upstream cpal (coreaudio-rs);
 AVAudioSession reachable via the NativeScript ObjC metadata bridge.

## Gate probe and toolchain

Build the smallest NativeScript iOS app (ns create,
 ns build ios / open the generated .
xcworkspace,
 sign,
 deploy to the iPhone X) that does BOTH wall-stressing things in one launch:
 (a) link a tiny Rust staticlib (crate-type=staticlib) exposing one extern \"C\" function like int rust_add(int,
int) plus a generated C header,
 register it via NativeScript metadata,
 and call it from app JS,
 confirming the returned value (proves wall 1:
 static-lib FFI through dlsym/libffi);
 and (b) register a JS-backed ObjC callback that actually fires,
 e.g. an NSURLSession (or NWListener) completion/delegate handler,
 or a UIButton tap handler,
 confirming the JS callback runs (proves the libffi FFI_EXEC_TRAMPOLINE_TABLE closure path works under jitless
  on a real,
 code-signed device).
 The decisive on-device signal:
 both the Rust return value AND the JS callback fire with NO EXC_BAD_ACCESS,
 NO codesigning/AMFI kill,
 and NO 'cannot allocate executable memory' crash.
 Because jitless V8 + the static trampoline table never request a JIT entitlement,
 this is expected to pass on a stock signed device;
 it must be confirmed on hardware (simulator is permissive about codesigning,
 so simulator success is not proof).

Toolchain:
 macOS + Xcode (15+),
 Xcode command-line tools;
 Homebrew + CMake (README:
 brew install cmake,
 symlink to /usr/local/bin/cmake);
 CocoaPods + xcodeproj gem (sudo gem install xcodeproj cocoapods);
 Node.
js + the NativeScript CLI (npm i -g nativescript,
 the `ns` command);
 for the gate's Rust staticlib:
 rustup with aarch64-apple-ios target (rustup target add aarch64-apple-ios) and cbindgen for the C header;
 a valid Apple signing identity / provisioning profile for the iPhone X.
 (To rebuild the runtime from source,
 also:
 the libffi submodule build via build_libffi.
sh and V8 via build_v8_source.
sh with download_llvm.
sh for the metadata generator,
 but consuming the prebuilt @nativescript/ios npm package avoids that.
)

## Supporting-stack vets this framework drags in

- In-process HTTP/S3 endpoint vet:
   build the loopback server kopia targets on Network.
  framework/NWListener (or a pinned ObjC pod like GCDWebServer) driven from JS,
   and prove kopia's S3 client reaches it
- Streaming + background transfer vet:
   NSURLSession background upload to pCloud with delegate progress callbacks crossing the JS<->libffi-closure boundary,
   plus BGTaskScheduler resumption windows
- FFI binding ergonomics vet:
   cbindgen-generated C header -> NativeScript metadata regeneration -> typed-array/pointer marshalling for kopia and
    for the symphonia/cpal audio core (struct/by-ref encodings in FFICall.
  cpp/TypeEncoding)
- kopia static-lib packaging vet:
   gomobile c-archive .
  xcframework (or Rust staticlib) link + symbol visibility under dlsym(RTLD_DEFAULT),
   and snapshot/restart restructuring around background URLSession
- Audio core reuse vet:
   symphonia+libopus+cpal staticlib on aarch64-apple-ios,
   AVAudioSession category/activation via the ObjC bridge,
   true-peak/peak-cache behavior,
   and interpreted-JS UI latency under playback
- Jitless performance vet:
   measure interpreted-V8 CPU cost for any in-JS hot path (hashing/chunking glue,
   UI),
   since TurboFan/Sparkplug are off
- QA stack:
   in-process UI test (NativeScript test runner / appium),
   end-to-end device test of backup+playback,
   fuzz the C-ABI boundary (malformed pointers/lengths across libffi),
   property tests on normalization,
   mutation testing of the JS glue

## Cited sources

- V8 started with --jitless unconditionally in both debug and release:
   /tmp/agent/nativescript-audit-20260612/NativeScript/runtime/Runtime.
  mm:
  255-256
- Project documents that its V8 runs in --jitless mode:
   /tmp/agent/nativescript-audit-20260612/`README.md`:
  101
- V8 built in lite mode with WebAssembly disabled (no Liftoff/TurboFan codegen):
   /tmp/agent/nativescript-audit-20260612/build_v8_source.
  sh (v8_enable_lite_mode=true,
   v8_enable_webassembly=false)
- Native callbacks created as libffi closures:
   /tmp/agent/nativescript-audit-20260612/NativeScript/runtime/Interop.
  mm:
  66-71 (ffi_closure_alloc/ffi_prep_closure_loc)
- libffi aarch64 Apple port uses FFI_EXEC_TRAMPOLINE_TABLE (static trampolines,
   no runtime codegen):
   /tmp/agent/libffi-ns-20260612/src/aarch64/ffitarget.
  h:
  58-65 ;
   /tmp/agent/libffi-ns-20260612/src/closures.
  c:
  144-153
- Closure trampoline page is a vm_remap of a static pre-existing executable template page,
   not freshly generated/mprotected memory:
   /tmp/agent/libffi-ns-20260612/src/closures.
  c:
  204-213
- C calls dispatched via libffi ffi_call on a cif built from metadata TypeEncoding:
   /tmp/agent/nativescript-audit-20260612/NativeScript/runtime/Interop.
  mm:
  137 ;
   /tmp/agent/nativescript-audit-20260612/NativeScript/runtime/FFICall.
  cpp
- Symbols of a statically linked Rust/Go staticlib resolved via dlsym(RTLD_DEFAULT):
   /tmp/agent/nativescript-audit-20260612/NativeScript/runtime/SymbolLoader.
  mm:
  131-137
- Metadata generator parses plain C functions from headers into callable FunctionMeta:
   /tmp/agent/nativescript-audit-20260612/metadata-generator/src/Meta/MetaFactory.
  h:
  49 ;
   /tmp/agent/nativescript-audit-20260612/metadata-generator/src/Meta/MetaEntities.
  h:
  276-280
- Minimum iOS deployment target 13.0:
   /tmp/agent/nativescript-audit-20260612/project-template-ios/__PROJECT_NAME__.
  xcodeproj/project.
  pbxproj:
  356,410 ;
   build_v8_source.
  sh:
  32
- libffi iOS device build targets aarch64-apple-darwin13 (triggers Apple trampoline-table port):
   /tmp/agent/nativescript-audit-20260612/build_libffi.
  sh
- Runtime contains no audio/HTTP-server/background-extension code of its own;
   iOS SDK is reached via the bridge:
   grep over /tmp/agent/nativescript-audit-20260612/NativeScript/runtime (no AVAudio/URLSession server/BGTask logic)

## Adversarial cite-check

- Wall 2 confirmed:
   confirmed
- Wall 1 confirmed:
   confirmed
- Corrections:
   All load-bearing claims are supported by their cited sources;
   no fabricated or contradicted claim found.
   The interpreter-fallback wall-2 verdict and the wall-1 "yes" both hold.
   Verified verbatim:
   (1) /tmp/agent/nativescript-audit-20260612/NativeScript/runtime/Runtime.
  mm:
  255 passes --jitless UNCONDITIONALLY in both branches ('--expose_gc --jitless' debug,
   '--expose_gc --jitless --no-lazy' release);
   SetFlagsFromString at :
  256 is the only V8-flag site in the runtime,
   with no jit re-enable anywhere.
   (2) build_v8_source.
  sh has v8_enable_lite_mode=true (line 44) and v8_enable_webassembly=false (line 47) under target_os="ios" (line 34),
   so these are the iOS device build,
   not a Catalyst-only variant.
   (3) `README.md`:
  101 quote exact.
   (4) Interop.
  mm:
  66-71 ffi_closure_alloc + ffi_prep_closure_loc confirmed;
   Interop.
  mm:
  137 ffi_call confirmed.
   (5) libffi clone is on branch darind/v8-ios (confirmed via git);
   ffitarget.
  h:
  58-65 shows the FFI_EXEC_TRAMPOLINE_TABLE closure path guarded on __MACH__ (with #error otherwise);
   closures.
  c:
  153 declares extern ffi_closure_trampoline_table_page;
   closures.
  c:
  204-213 vm_remaps that static template onto the placeholder page;
   no runtime PROT_EXEC mmap or machine-code generation exists,
   so the W^X / no-JIT-entitlement conclusion is sound.
   (6) Wall-1:
   SymbolLoader.
  mm:
  136 dlsym(RTLD_DEFAULT,
   symbolName) fallback (inside loadFunctionSymbol starting :
  131);
   FFICall.
  cpp:
  300 ffi_prep_cif builds the cif from TypeEncoding-derived ffi_types;
   MetaFactory.
  h:
  49 createFromFunction(FunctionDecl&,
   FunctionMeta&);
   MetaEntities.
  h:
  276-280 class FunctionMeta with type=MetaType:
  :
  Function.
   All present and matching.

ONE MINOR CITATION IMPRECISION (does not change either verdict):
 the wall-2 verdict cites closures.
c:
161 as the "separate writable,
 non-executable config page,
" but line 161 is only the `vm_address_t config_page;` struct-field declaration,
 and the comment immediately above it (line 160) literally reads "/* contiguous writable and executable pages */".
 The config page's actual non-executability is real but is established by the allocation logic
  in ffi_trampoline_table_alloc (config_page is vm_allocate'd at closures.
c:
198-200 and is never the target of the vm_remap onto the executable template at :
211-213;
 only trampoline_page is remapped executable),
 not by anything stated at the cited line 161.
 So the cited line does not by itself say what the audit attributes to it;
 the claim is true but the line pointer is loose.
 Corrected support:
 closures.
c:
185-243 (ffi_trampoline_table_alloc),
 where only trampoline_page receives the executable template via vm_remap.
- Sources checked:
   /tmp/agent/nativescript-audit-20260612/NativeScript/runtime/Runtime.
  mm:
  244-257 (jitless flags,
   both branches);
   /tmp/agent/nativescript-audit-20260612/build_v8_source.
  sh:
  34,44,47 (v8_enable_lite_mode/webassembly,
   target_os ios);
   /tmp/agent/nativescript-audit-20260612/`README.md`:
  101 (--jitless explained doc);
   /tmp/agent/nativescript-audit-20260612/NativeScript/runtime/Interop.
  mm:
  60-75 (ffi_closure_alloc,
   ffi_prep_closure_loc);
   /tmp/agent/nativescript-audit-20260612/NativeScript/runtime/Interop.
  mm:
  137 (ffi_call);
   /tmp/agent/libffi-ns-20260612 git branch (darind/v8-ios confirmed);
   /tmp/agent/libffi-ns-20260612/src/aarch64/ffitarget.
  h:
  53-70 (FFI_EXEC_TRAMPOLINE_TABLE,
   __MACH__ guard);
   /tmp/agent/libffi-ns-20260612/src/closures.
  c:
  144-243 (extern trampoline page,
   struct,
   vm_allocate,
   vm_remap of static template);
   /tmp/agent/libffi-ns-20260612/configure.
  ac:
  194-215 (FFI_EXEC_TRAMPOLINE_TABLE define site);
   /tmp/agent/nativescript-audit-20260612/NativeScript/runtime/SymbolLoader.
  mm:
  131-145 (dlsym RTLD_DEFAULT fallback);
   /tmp/agent/nativescript-audit-20260612/NativeScript/runtime/FFICall.
  cpp:
  8,300 (TypeEncoding switch,
   ffi_prep_cif);
   /tmp/agent/nativescript-audit-20260612/metadata-generator/src/Meta/MetaFactory.
  h:
  49 (createFromFunction);
   /tmp/agent/nativescript-audit-20260612/metadata-generator/src/Meta/MetaEntities.
  h:
  276-285 (FunctionMeta,
   MetaType:
  :
  Function)
