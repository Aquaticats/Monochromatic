# iOS source-audit: Lynx

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
   AOT-native at the rendering layer plus a jitless scripting runtime.
   Lynx ships a C++ layout/rendering engine compiled AOT to a native iOS static lib that draws
   into native UIKit views (LynxView :
   UIView,
   LynxTextView :
   UIView at platform/darwin/ios/lynx/public/LynxView.
  h:
  41 and .../text/LynxTextView.
  h:
  13),
   not a WebView.
   App/business JS runs in one of two jitless engines depending on a build flag.
   (1) The Lepus/LepusNG main-thread script and,
   when FORCE_USE_LIGHT_WEIGHT_JS_ENGINE is set,
   the background runtime run on PrimJS,
   a QuickJS-derived bytecode VM whose 'template interpreter' is precompiled AArch64 assembly checked
   into the repo as src/interpreter/primjs/ios/embedded.
  S and assembled at build time (no runtime codegen).
   (2) On iOS the background (BTS) JS runtime defaults to JavaScriptCore (core/Lynx.
  gni:
  62-64),
   which in a non-WKWebView app process runs interpreter-only (LLInt,
   no JIT) because the app process lacks the dynamic-codesigning entitlement.
   Either path is jitless on a device.
- Minimum iOS deployment:
   iOS 10 (`README.md`:
   "Lynx apps may target iOS 10 and Android 5.0 (API 21) or newer";
   PrimJS.
  podspec sets ios.
  deployment_target = "9.0")
- Gate expectation:
   expected-pass
- Confidence:
   high
- Key finding:
   Lynx is jitless on iOS by construction:
   it renders to native UIKit views (LynxView :
   UIView),
   and its PrimJS scripting engine is a QuickJS bytecode VM whose 'template interpreter' is PRECOMPILED AArch64
   assembly checked into the repo (src/interpreter/primjs/ios/embedded.
  S) and assembled at build time into the static lib .
  text section,
   with no runtime codegen,
   mprotect(PROT_EXEC),
   or MAP_JIT anywhere in the source.
   So it passes wall 2 with no JIT entitlement;
   but Lynx provides no audio and no kopia plumbing,
   so both apps' real work (symphonia+cpal audio core,
   kopia staticlib,
   in-app HTTP server,
   background URLSession) must be done in native Obj-C++/Rust modules behind Lynx,
   which the LynxModule selector-dispatch bridge supports.

## Wall 2: JIT / executable memory

Verdict:
 no-jit-needed

PrimJS is a QuickJS-derived bytecode VM;
 its high-performance 'template interpreter' is generated at BUILD time into assembly (doc/`template_interpreter.md`:
 handler generator 'writes it into the embedded.
S assembly file,
 which is then mixed with other parts of QuickJs').
 The iOS file src/interpreter/primjs/ios/embedded.
S is precompiled AArch64 (.
text/.
align/.
global + literal .
word opcodes),
 guarded by #if defined(__aarch64__) && defined(ENABLE_PRIMJS_SNAPSHOT),
 and listed in PrimJS.
podspec source_files so Xcode assembles it into the static lib .
text.
 Execution is jumps among AOT handlers via a dispatch_table;
 no runtime codegen.
 No mprotect(PROT_EXEC)/MAP_JIT/pthread_jit_write anywhere in src (only GC mprotect with PROT_READ).
 The alternate iOS BTS engine,
 JavaScriptCore,
 runs interpreter-only in a non-WebView app process.
 No JIT entitlement needed:
 passes wall 2.

Source:
 /tmp/agent/primjs-audit-20260612/docs/`template_interpreter.md`;
 /tmp/agent/primjs-audit-20260612/src/interpreter/primjs/ios/embedded.
S:
1-4;
 /tmp/agent/primjs-audit-20260612/PrimJS.
podspec;
 /tmp/agent/primjs-audit-20260612/src/gc/mem_map.
cc:
91;
 /tmp/agent/lynx-audit-20260612/core/Lynx.
gni:
62-64

## Wall 1: link and call a Go/Rust static library

Feasible:
 yes

Mechanism:
 iOS native modules are Obj-C/Obj-C++ classes conforming to @protocol LynxModule with a 'name'
 and a 'methodLookup' dict mapping JS method names to ObjC selectors.
 A .
mm (Obj-C++) module can directly call any linked C/C++ symbol,
 hence a Rust staticlib (via cbindgen C header) or a Go gomobile c-archive,
 and return results as NSData/NSDictionary.
 First-party modules already use this pattern.

Source:
 /tmp/agent/lynx-audit-20260612/platform/darwin/common/lynx/public/module/LynxModule.
h:
25-51;
 /tmp/agent/lynx-audit-20260612/platform/darwin/ios/lynx/module/LynxFetchModule.
m:
30-40

## Wall 3: background execution

Lynx provides no long-running background capability;
 networking is host-provided via LynxServiceHttpProtocol,
 and the bundled LynxHttpService builds an NSURLSession from defaultSessionConfiguration (foreground)
 with NSURLSessionDataTask.
 No background session or BGProcessingTask exists in the framework.
 A multi-hour kopia snapshot must be restructured per the standard iOS wall:
 a native module owning URLSessionConfiguration.
background + BGProcessingTask scheduling,
 driven by the app,
 not by Lynx JS.

Source:
 /tmp/agent/lynx-audit-20260612/platform/darwin/ios/lynx_service/http/LynxHttpService.
m:
46-47;
 /tmp/agent/lynx-audit-20260612/platform/darwin/ios/lynx_service/http/LynxNSUrlSessionDelegate.
h:
8

## In-app HTTP server (kopia S3 target)

Feasible:
 yes

Mechanism:
 Not a Lynx feature,
 but trivially reachable through the same native-module bridge as wall 1.
 An in-app HTTP/S3 endpoint that kopia targets would be a Rust (hyper/tiny_http) or C listener compiled into the app
 and exposed via a LynxModule .
mm wrapper,
 or simply run as part of the linked kopia/Rust core;
 it listens on 127.0.0.1 in-process.
 Lynx provides no server primitive itself (its only HTTP is the outbound LynxHttpService client),
 so the server lives entirely in the linked native lib,
 which iOS allows for loopback in-process listeners.

Source:
 /tmp/agent/lynx-audit-20260612/platform/darwin/common/lynx/public/module/LynxModule.
h:
25 (native module bridge to link C/C++/Rust);
 /tmp/agent/lynx-audit-20260612/platform/darwin/ios/lynx_service/http/LynxHttpService.
m (only an outbound client exists,
 no server)

## HTTPS streaming client (pCloud)

Feasible:
 yes

Mechanism:
 Two viable routes.
 (1) Lynx's own fetch:
 LynxFetchModule supports streaming responses (enableFetchAPIStandardStreaming),
 delivering chunks to JS via LynxHttpStreamingDelegate.
onData(NSData),
 backed by NSURLSession over the host LynxServiceHttpProtocol/LynxHttpService.
 (2) For a true streaming upload to pCloud you would instead drive NSURLSession
 (or a Rust reqwest/hyper client in the linked core) from a native module,
 since Lynx fetch is request/response oriented.
 Either way HTTPS streaming is expressible;
 the production path for backup is the native-module/Rust client,
 not Lynx JS fetch.

Source:
 /tmp/agent/lynx-audit-20260612/platform/darwin/ios/lynx/module/LynxHttpStreamingDelegate.
m:
28-45 (onData/onEnd/onError streaming callbacks);
 /tmp/agent/lynx-audit-20260612/platform/darwin/ios/lynx/module/LynxFetchModule.
m:
18-40 (standardStreamingFlag,
 fetch over LynxServiceHttpProtocol)

## Audio (music-player port)

Decode:
 Not provided by Lynx.
 The framework has no audio decode pipeline,
 no <audio> element,
 no symphonia/opus equivalent.
 Decode must be done by the existing Rust core (symphonia 0.6 + libopus) compiled into the app
 and called via a native module;
 Lynx is UI/script only.

Output:
 Not provided by Lynx as a high-level API,
 but the iOS service layer exposes raw CoreAudio AudioUnit control:
 LynxServiceSystemInvokeProtocol declares startAudioOutputUnit:
(void*)audioUnitPtr / stopAudioOutputUnit:
(void*) taking a raw AudioUnit pointer.
 Low-latency output therefore happens in native code (CoreAudio AudioUnit / cpal's CoreAudio backend),
 wired by the app,
 not by Lynx.

Rust core reuse:
 Reused via FFI,
 NOT rewritten on AVAudioEngine.
 Because audio is entirely outside Lynx's responsibility and native modules are Obj-C++ that link arbitrary C/C++/Rust,
 the existing Rust core (symphonia decode + cpal 0.18,
 whose CoreAudio backend targets Apple) can be compiled as a Rust staticlib and called from a LynxModule .
mm.
 cpal already drives CoreAudio AudioUnits on Apple,
 matching the void* AudioUnit surface Lynx exposes.
 No AVAudioEngine rewrite is forced by Lynx.
 The only Lynx-driven work is the UI (transport,
 queue,
 pagination) as a Lynx view layered over the FFI'd Rust audio engine;
 Slint itself is not used (Lynx replaces the UI layer).

Source:
 /tmp/agent/lynx-audit-20260612/platform/darwin/common/lynx/public/service/LynxServiceSystemInvokeProtocol.
h:
71-81 (startAudioOutputUnit/stopAudioOutputUnit on raw AudioUnit void*);
 /tmp/agent/lynx-audit-20260612/platform/darwin/common/lynx/public/module/LynxModule.
h:
25 (native module links Rust staticlib);
 grep:
 no AVAudio/decode element in platform/darwin

## Gate probe and toolchain

Minimal app:
 an iOS app embedding LynxView that renders a trivial Lynx page (text + a button) and registers one custom LynxModule (.
mm) that links a tiny Rust staticlib exporting a C function.
 On-device signals:
 (1) the UI renders as native UIView/LynxTextView (verify in Xcode Debug View Hierarchy that there is NO WKWebView)
 and the button tap fires JS handlers without crashing,
 proving Lynx executes app JS jitless on device (PrimJS template interpreter / interpreter-only JSC);
 (2) invoking the native module from JS returns the Rust function's value,
 proving the Rust-staticlib FFI bridge (wall 1).
 Watch device console for absence of EXC_BAD_ACCESS / dynamic-codesign faults on first JS run.
 Secondary:
 link the real symphonia+cpal core and play ~2s audio through the AudioUnit to confirm CoreAudio output behind the
 module.

Toolchain:
 macOS with Xcode + command line tools;
 CocoaPods (Lynx and PrimJS ship .
podspec;
 pod install pulls PrimJS);
 Ruby/Bundler (Gemfile present).
 Node + pnpm for building the Lynx JS bundle (rspeedy/ReactLynx toolchain).
 Rust with the aarch64-apple-ios target (and aarch64-apple-ios-sim) plus cbindgen to produce the staticlib + C header
 for the native module.
 A valid Apple developer signing identity + provisioning for on-device install.
 Optionally gn/ninja only if building Lynx engine from source rather than consuming the pod.

## Supporting-stack vets this framework drags in

- kopia as Go gomobile c-archive xcframework or Rust staticlib linked + called from a LynxModule .
  mm (FFI marshalling,
   threading,
   error propagation across the ObjC boundary)
- In-app loopback HTTP/S3 server (Rust hyper/tiny_http) living in the linked native core,
   lifecycle-bound to the app,
   that kopia targets
- Streaming HTTPS upload client to pCloud via NSURLSession background config
   or Rust reqwest/hyper in native code (chunked upload,
   TLS,
   resumable)
- Background transfer restructuring:
   URLSessionConfiguration.
  background + BGProcessingTask scheduling in a native module (no foreground-service equivalent)
- symphonia 0.6 + libopus decode and cpal 0.18 CoreAudio output reused as a Rust staticlib behind a LynxModule,
   with true-peak normalization and on-disk peak cache (verify low-latency AudioUnit callback works behind the bridge)
- Lynx UI re-authoring:
   the music-player UI (transport,
   two-axis paginated folder queue,
   session persistence) rebuilt as a Lynx/ReactLynx page since Slint is not used;
   verify Lynx list/scroll perf for large folder scans
- JS bundle build + delivery pipeline (rspeedy/ReactLynx,
   bytecode caching ENABLE_CODECACHE) and choice of BTS engine (JSC default vs FORCE_USE_LIGHT_WEIGHT_JS_ENGINE PrimJS)
- QA:
   native-module unit tests (the repo's *UnitTest.
  mm pattern),
   in-process Lynx view UI test,
   e2e device test of the FFI round-trip,
   fuzz/property tests on the kopia S3 endpoint and the streaming parser (LynxHttpStreamingDelegate chunk parsing)

## Cited sources

- PrimJS is a QuickJS-based bytecode engine with a build-time-generated assembly template interpreter,
   not a JIT:
   /tmp/agent/primjs-audit-20260612/`README.md`:
  11-16 and doc/`template_interpreter.md` (handler -> embedded.
  S,
   mixed with QuickJs)
- iOS template interpreter is precompiled AArch64 machine code assembled into the static lib at build time:
   /tmp/agent/primjs-audit-20260612/src/interpreter/primjs/ios/embedded.
  S:
  1-4;
   PrimJS.
  podspec source_files lists embedded.
  S
- No runtime executable-memory / JIT primitives in PrimJS;
   only GC mprotect with PROT_READ:
   /tmp/agent/primjs-audit-20260612/src/gc/mem_map.
  cc:
  91,
   collector_ms.cc:
  320,
   space.
  cc:
  238 (grep found no PROT_EXEC/MAP_JIT/pthread_jit_write)
- On iOS the background JS runtime defaults to JavaScriptCore;
   PrimJS/QuickJS selectable via FORCE_USE_LIGHT_WEIGHT_JS_ENGINE:
   /tmp/agent/lynx-audit-20260612/core/Lynx.
  gni:
  62-64;
   core/shell/runtime/bts/bts_runtime.
  cc:
  64-75
- Lynx renders to native UIKit views,
   not a WebView (app shell):
   /tmp/agent/lynx-audit-20260612/platform/darwin/ios/lynx/public/LynxView.
  h:
  41 (LynxView :
   UIView),
   .../text/LynxTextView.
  h:
  13
- Native modules are Obj-C/Obj-C++ classes with selector dispatch,
   able to link Rust/Go static libs (wall 1 FFI):
   /tmp/agent/lynx-audit-20260612/platform/darwin/common/lynx/public/module/LynxModule.
  h:
  25-51;
   platform/darwin/ios/lynx/module/LynxFetchModule.
  m:
  30-40
- Default networking is a foreground NSURLSession;
   no background-session/BGProcessingTask in framework (wall 3):
   /tmp/agent/lynx-audit-20260612/platform/darwin/ios/lynx_service/http/LynxHttpService.
  m:
  46-47
- Lynx supports streaming fetch responses via LynxHttpStreamingDelegate onData callbacks:
   /tmp/agent/lynx-audit-20260612/platform/darwin/ios/lynx/module/LynxHttpStreamingDelegate.
  m:
  28-45;
   LynxFetchModule.
  m standardStreamingFlag
- Lynx has no audio decode/playback;
   iOS service layer only exposes raw AudioUnit start/stop,
   so Rust symphonia+cpal core is reused via FFI not rewritten on AVAudioEngine:
   /tmp/agent/lynx-audit-20260612/platform/darwin/common/lynx/public/service/LynxServiceSystemInvokeProtocol.
  h:
  71-81;
   grep:
   no AVAudio/decode element in platform/darwin
- Min iOS deployment iOS 10 (PrimJS pod allows 9.0):
   /tmp/agent/lynx-audit-20260612/`README.md`:
  45;
   /tmp/agent/primjs-audit-20260612/PrimJS.
  podspec ios.
  deployment_target 9.0

## Adversarial cite-check

- Wall 2 confirmed:
   confirmed
- Wall 1 confirmed:
   confirmed
- Corrections:
   All load-bearing claims are supported by their cited sources;
   nothing fabricated,
   unsupported,
   or contradicted.
   The decisive wall-2 evidence holds:
   a repo-wide grep for PROT_EXEC / MAP_JIT / pthread_jit_write returns ZERO matches in the entire primjs repo
   (not just src/),
   and all three mprotect call sites are in the GC subsystem (src/gc/{mem_map.
  cc:
  91,
  collector_ms.cc:
  320,
  space.
  cc:
  238}) with PROT_READ -- so 'no runtime codegen / jitless' is verified against primary source,
   not just prose.

Minor citation imprecisions found (cosmetic;
 none flip a verdict,
 recorded for honesty under the adversarial framing):
1) mem_map.
   cc citation:
    audit cites line 91,
    which is the generic wrapper `mprotect(addr, size, prot)` with a VARIABLE prot,
    not a literal PROT_READ.
    The PROT_READ literal is actually at mem_map.
   cc:
   97 (ProtectAllMem) and collector_ms.cc:
   320.
    The 'GC mprotect with PROT_READ' substance is correct;
    only the line pin is slightly off.
2) embedded.
   S guard:
    audit cites lines 1-4 and lists the guard as `#if defined(__aarch64__) && defined(ENABLE_PRIMJS_SNAPSHOT)`.
    The real guard at line 1 has a THIRD condition the audit omits:
    `&& !defined(ENABLE_QUICKJS_DEBUGGER)`.
    Also the literal `.word` opcodes start at line 6,
    just outside the cited 1-4 range (.
   text/.
   align/.
   global at 2-4 are in range).
    Substance (precompiled AArch64,
    guarded,
    .
   word opcodes) confirmed.
3) 'generated at BUILD time':
    embedded.
   S is a checked-in artifact (`template_interpreter.md`:
   61 confirms a handler generator 'writes it into the embedded.
   S assembly file'),
    i.e. dev-time generated and build-time ASSEMBLED.
    The load-bearing conclusion 'no runtime codegen' holds under either framing.
4) wall-1 .
   mm example:
    the mechanism prose says a '.
   mm (Obj-C++) module' can call linked C/C++;
    the cited example LynxFetchModule is a plain '.
   m' (Obj-C),
    used only as evidence of the protocol pattern (name + methodLookup),
    which it confirms exactly at lines 30-40.
    The .
   mm capability statement is a correct general claim,
    not contradicted.
5) One inferred (not directly read) link:
    that FORCE_USE_LIGHT_WEIGHT_JS_ENGINE selects PrimJS vs JSC was inferred from the flag name,
    use_quickjs_engine (bts_runtime.
   cc:
   71-72),
    and JSExecutor storing force_use_light_weight_js_engine (js_executor.
   cc:
   18-27);
    the exact PrimJS-vs-JSC construction branch was not located.
    Does not affect the verdict because BOTH candidate engines are jitless on iOS,
    so routing is immaterial to 'no-jit-needed'.

Confirmed exactly as cited:
 `template_interpreter.md`:
61 ('writes it into the embedded.
S assembly file,
 which is then mixed with other parts of QuickJs') and dispatch_table (lines 49,73,82);
 PrimJS.
podspec:
63 lists src/interpreter/primjs/ios/embedded.
S in sp.
source_files;
 core/Lynx.
gni:
62-64 (`if (is_ios) { if (jsengine_type == \"none\") { jsengine_type = \"jsc\" } }`);
 LynxModule.
h @protocol LynxModule with name + methodLookup (JS method names -> ObjC selectors);
 LynxFetchModule.
m:
30-40 name/methodLookup;
 LynxView.
h:
41 (@interface LynxView :
 UIView) and LynxTextView.
h:
13 (@interface LynxTextView :
 UIView).
 The JSC interpreter-only/LLInt-no-JIT statement is an accurate external iOS-platform fact
 (third-party non-WebView app process lacks dynamic-codesigning entitlement),
 correctly not cited to a repo source.
- Sources checked:
   /tmp/agent/primjs-audit-20260612/docs/`template_interpreter.md`
   (lines 49,61,73,82 -- handler generator writes embedded.
  S;
   dispatch_table;
   confirmed);
   /tmp/agent/primjs-audit-20260612/src/interpreter/primjs/ios/embedded.
  S:
  1-30 (guard + .
  text/.
  align/.
  global + .
  word opcodes;
   guard has extra !
  ENABLE_QUICKJS_DEBUGGER;
   .
  word starts line 6);
   /tmp/agent/primjs-audit-20260612/PrimJS.
  podspec:
  63 (embedded.
  S in sp.
  source_files -- confirmed);
   /tmp/agent/primjs-audit-20260612/src/gc/mem_map.
  cc:
  90-98 (mprotect wrapper at 91,
   PROT_READ literal at 97 not 91);
   /tmp/agent/primjs-audit-20260612/src/gc/collector_ms.cc:
  320 and src/gc/space.
  cc:
  238 (other GC mprotect sites);
   primjs repo-wide grep PROT_EXEC|MAP_JIT|pthread_jit_write -> ZERO matches (decisive jitless evidence);
   /tmp/agent/primjs-audit-20260612/docs/`wasm.md` (no JIT/executable-memory reference);
   /tmp/agent/lynx-audit-20260612/core/Lynx.
  gni:
  62-64 (iOS default jsengine_type=jsc -- confirmed);
   /tmp/agent/lynx-audit-20260612/platform/darwin/common/lynx/public/module/LynxModule.
  h (@protocol LynxModule,
   name,
   methodLookup -- confirmed);
   /tmp/agent/lynx-audit-20260612/platform/darwin/ios/lynx/module/LynxFetchModule.
  m:
  30-40 (.
  m not .
  mm;
   name+methodLookup pattern confirmed);
   /tmp/agent/lynx-audit-20260612/platform/darwin/ios/lynx/public/LynxView.
  h:
  41 (LynxView :
   UIView -- confirmed);
   /tmp/agent/lynx-audit-20260612/platform/darwin/ios/lynx/public/ui/text/LynxTextView.
  h:
  13 (LynxTextView :
   UIView -- confirmed);
   /tmp/agent/lynx-audit-20260612/core/shell/runtime/bts/bts_runtime.
  {h:
  56,
  cc:
  71-72,205} + core/runtime/js/js_executor.
  cc:
  18-27 (FORCE_USE_LIGHT_WEIGHT_JS_ENGINE flag real and wired;
   PrimJS-vs-JSC branch inferred not read)
