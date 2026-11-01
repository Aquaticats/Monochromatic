# iOS source-audit: Slint

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

## Device-gate result: DISQUALIFIED (overrides this audit's expected-pass)

This source audit concluded expected-pass.
 The on-device gate disproves it.
 The Slint `energy-monitor`
demo builds and the process launches,
 but it crashes before rendering on iOS 16.7 because Slint's iOS
support is iOS 17+ in two independent places,
 both verified on the device:

- `accesskit_ios` (the a11y backend,
   a hard requirement) references four iOS-17 symbols unconditionally:
  `UIAccessibilityPriorityHigh`/`Low`,
   `UIAccessibilitySpeechAttributeAnnouncementPriority`
  (`event.rs`) and `UIAccessibilityTraitToggleButton` (`node.rs`).
   dyld `Symbol not found:
  _UIAccessibilityPriorityHigh`,
   SIGKILL before any UI.
- Slint's own `internal/backends/winit/ios/color_scheme.rs:37` uses the iOS-17 `UITrait` API
  (`UITraitUserInterfaceStyle::class()` / `registerForTraitChanges`).
   Runtime panic on iOS 16.7:
  `class UITraitUserInterfaceStyle could not be found`.

The iPhone X (A11) never gets iOS 17,
 and a11y cannot be dropped,
 so Slint is disqualified for this
device.
 Reviving it needs a maintained downport fork of both accesskit and Slint's winit iOS backend.
See `device-gate-results.md` for the full evidence.

## Verdict

- iOS runtime model:
   AOT-native machine code.
   The iOS app is a plain Rust binary cross-compiled
   with `cargo build --target aarch64-apple-ios --bin` (scripts/build_for_ios_with_cargo.
  bash:
  57),
   lipo'd,
   dSYM'd,
   and codesigned,
   then wrapped by an Xcode/XcodeGen project that only signs and packages it.
   The shipped binary has an ordinary Rust `fn main()` (tools/viewer/`main.rs`:
  153) that runs the Slint event loop (`component.run()` -> winit iOS event loop,
   `event_loop.rs`:
  700 `run_app`).
   There is no managed runtime,
   no bytecode VM,
   no JIT,
   and no interpreter for app code:
   Slint's UI is Rust-compiled,
   .
  slint markup is compiled ahead of time by slint-build into Rust at host build time.
   The iOS backend drives UIKit directly through native objc2 bindings (objc2-ui-kit:
   UIApplication/UIView/UIScreen,
   objc2-quartz-core:
   CADisplayLink) (internal/backends/winit/`Cargo.toml`:
  130-135).
- Minimum iOS deployment:
   12.0 (deploymentTarget in the documented XcodeGen `project.yml`,
   doc/astro/.../platforms/mobile/ios.
  mdx:
  53;
   winit iOS path builds for aarch64-apple-ios and aarch64-apple-ios-sim)
- Gate expectation:
   expected-pass
- Confidence:
   high
- Key finding:
   Slint already has a first-class,
   CI-tested,
   TestFlight-shipping iOS path that produces a device-installable .
  app from a plain native Rust binary (cargo build --target aarch64-apple-ios --bin;
   UIKit driven via objc2).
   Because the entire app,
   UI plus the music-player's symphonia+cpal core,
   compiles into that one AOT-native binary,
   all three iOS walls collapse to ordinary Rust-on-iOS work:
   no JIT/interpreter exists to trip wall 2,
   kopia links as a standard static-archive FFI dependency (wall 1),
   and cpal's existing CoreAudio iOS backend (src/host/coreaudio/ios/`mod.rs`) means audio is REUSED,
   not rewritten on AVAudioEngine.
   Since music-player is already Slint,
   this is the lowest-friction iOS port of any candidate;
   the one concrete change is that Skia/Metal replaces femtovg automatically on iOS (internal/backends/winit/`build.rs`:
  10,13).

## Wall 2: JIT / executable memory

Verdict:
 no-jit-needed

App code executes as AOT-compiled native ARM64.
 The whole app,
 including Slint's runtime and the AOT-compiled .
slint UI,
 is one cargo-built Rust executable (build_for_ios_with_cargo.
bash:
57 `cargo build --target aarch64-apple-ios --bin`).
 No interpreter or JIT runs in-process:
 there is no managed runtime to AOT-mitigate.
 The .
slint UI is compiled to Rust at host build time by slint-build (music-player `Cargo.toml`:
143 slint-build git pin),
 so there is no runtime codegen path.
 This sails past the W^X/DENY_EXECMEM wall that disqualifies scripted runtimes.

Source:
 /tmp/agent/slint-audit-20260612/scripts/build_for_ios_with_cargo.
bash:
57 and /tmp/agent/slint-audit-20260612/tools/viewer/`main.rs`:
153 (fn main -> component.
run());
 UI AOT via slint-build (package/music-player/desktop-app/`Cargo.toml`:
143)

## Wall 1: link and call a Go/Rust static library

Feasible:
 yes

Mechanism:
 This is a standard Rust-on-iOS FFI capability,
 not a Slint feature,
 and the single-native-binary model is exactly what makes it clean.
 Because the app IS a cargo-built Rust executable with no managed boundary,
 a Go gomobile c-archive (.
a/.
xcframework) or a Rust staticlib is linked the ordinary way:
 `build.rs` emits `cargo:rustc-link-search` + `cargo:rustc-link-lib=static=kopia`,
 app code declares `extern "C"` signatures and calls them directly.
 kopia as a LINKED static lib (wall 1) is satisfied by linking its gomobile c-archive into the Rust binary.
 The kopia-Go-runtime-on-iOS specifics (gomobile c-archive init,
 goroutine scheduler under iOS) belong to kopia's own vet,
 not Slint's;
 Slint neither provides nor blocks the link step.

Source:
 /tmp/agent/slint-audit-20260612/scripts/build_for_ios_with_cargo.
bash:
57 (the app is `cargo build --target aarch64-apple-ios --bin`,
 a native binary into which any static archive links via standard Rust FFI)

## Wall 3: background execution

Slint provides NO background-execution abstraction;
 the foreground UI event loop is all it owns.
 Background transfer is app-level work done from Rust.
 The winit iOS backend already links objc2-foundation
 with NSRunLoop/NSOperation/NSNotification features (internal/backends/winit/`Cargo.toml`:
132),
 so the app can reach NSURLSession background configurations
 and BGTaskScheduler either directly via objc2/objc2-foundation bindings from Rust
 or through a thin Swift/ObjC shim added to the XcodeGen project.
 The iOS wall stands unchanged:
 a multi-hour kopia snapshot is NOT expressible as a foreground service;
 it must be restructured around background URLSession uploads and short BGProcessingTask windows.
 That restructuring is identical work regardless of UI framework and is not eased or worsened by Slint.

Source:
 /tmp/agent/slint-audit-20260612/internal/backends/winit/`Cargo.toml`:
132 (objc2-foundation with NSRunLoop/NSOperation linked;
 background APIs reachable via objc2,
 but no Slint-provided background mechanism)

## In-app HTTP server (kopia S3 target)

Feasible:
 yes

Mechanism:
 App-level Rust capability,
 not Slint-provided.
 Because the app is a native Rust binary,
 an in-process HTTP server crate (tiny_http / hyper / axum)
 that builds for aarch64-apple-ios listens on 127.0.0.1 as the S3/HTTP endpoint kopia targets.
 iOS permits in-process loopback listeners (no entitlement needed for localhost).
 Feasibility is a property of the chosen Rust crate compiling for the iOS target,
 confirmed by the fact that arbitrary Rust crates compile into the same `cargo build --target aarch64-apple-ios` binary;
 verify the specific server crate cross-compiles cleanly.

Source:
 /tmp/agent/slint-audit-20260612/scripts/build_for_ios_with_cargo.
bash:
57 (native cargo iOS binary hosts arbitrary Rust crates;
 HTTP-server feasibility is a Rust-crate property,
 scope to that crate's vet)

## HTTPS streaming client (pCloud)

Feasible:
 yes

Mechanism:
 App-level Rust capability.
 A streaming HTTPS client to pCloud uses reqwest+rustls (or hyper) compiled for aarch64-apple-ios;
 rustls avoids OpenSSL cross-compile pain and is the common iOS-Rust choice.
 Slint imposes nothing here;
 the constraint is that the client crate builds for the iOS target and
 that long uploads move to a background URLSession (wall 3).
 Confirm the streaming/chunked-upload crate cross-compiles for aarch64-apple-ios in the supporting-stack vet.

Source:
 /tmp/agent/slint-audit-20260612/scripts/build_for_ios_with_cargo.
bash:
57 (native Rust binary;
 HTTPS-client feasibility is a Rust-crate/TLS-backend property)

## Audio (music-player port)

Decode:
 symphonia 0.6 (all features) decodes unchanged:
 it is pure-Rust and compiles into the same aarch64-apple-ios binary.
 libopus via opusic-sys/audiopus is a bundled C lib cross-compiled by cc for the iOS target like any other C dep.
 No decode path needs rewriting.

Output:
 Low-latency output via cpal's dedicated CoreAudio iOS backend.
 cpal lists aarch64-apple-ios as a supported target and gates a CoreAudio host on cfg(any(target_os="ios",
 target_os="tvos")),
 with the iOS-specific submodule at src/host/coreaudio/ios/`mod.rs` (it drives AudioUnit/AVAudioSession internally).
 The OS audio engine sample-rate-converts to the hardware clock,
 matching the existing desktop CoreAudio path.

Rust core reuse:
 REUSED via FFI-free single-binary,
 NOT rewritten on AVAudioEngine.
 The existing music-player Rust core (symphonia + cpal in src/`output_cpal.rs`) already targets CoreAudio on Apple via
 cpal's macOS host;
 on iOS cpal selects its iOS CoreAudio host automatically under the same cfg(not(target_os="linux")) dependency table
 the package already uses (music-player `Cargo.toml`:
111 cpal=0.18).
 The core compiles into the Slint iOS binary directly:
 no separate FFI bridge,
 no AVAudioEngine rewrite.
 The developer does NOT reimplement output on AVAudioEngine;
 cpal's iOS backend already wraps AudioUnit/AVAudioSession.
 Caveat:
 cpal's iOS host has narrower device/format support than macOS
 and needs AVAudioSession category/activation set up (often a small objc2 or Swift shim);
 verify on device.

Source:
 /tmp/agent/cpal-audit-20260612/src/host/coreaudio/ios/`mod.rs` and `Cargo.toml`:
160 (cfg(any(target_os="ios",
target_os="tvos")) coreaudio deps) and `Cargo.toml`:
234 (aarch64-apple-ios listed target);
 reuse path:
 /var/home/user/Monochromatic/package/music-player/desktop-app/src/`output_cpal.rs` + `Cargo.toml`:
111

## Gate probe and toolchain

Slint already ships a real device build path (doc/`ios.md` documents a CI `build_ios` job producing a signed-locally .
xcarchive/.
ipa for dev.
slint.
slint-viewer on TestFlight;
 doc/astro mobile/ios.
mdx walks the XcodeGen + cargo aarch64-apple-ios flow),
 so bare UI-on-device is near-certain.
 The discriminating gate,
 exercising UI + wall-1 + audio together:
 start from slint-rust-template,
 build for the device with `cargo build --target aarch64-apple-ios` wrapped by the documented build_for_ios_with_cargo.
bash + XcodeGen project,
 then (a) link one trivial `extern \"C\"` function
 from a Go gomobile c-archive (or a Rust staticlib) via `build.rs` `rustc-link-lib=static`,
 call it from Rust,
 and render the returned value in the Slint UI,
 and (b) play a cpal CoreAudio sine wave.
 On-device signal:
 app installs and launches on the iPhone,
 the Slint UI renders the value returned from the linked static lib,
 and the device emits the tone.
 That single run confirms AOT-native execution (no JIT crash),
 static-lib FFI linking (wall 1),
 and CoreAudio output reuse.

Toolchain:
 macOS with Xcode (full,
 for device archive + codesign),
 XcodeGen (brew install xcodegen),
 Rust via rustup with `rustup target add aarch64-apple-ios aarch64-apple-ios-sim`,
 rsvg-convert (only for the viewer's app-icon render step),
 and an Apple Developer team + Apple Distribution cert + App Store provisioning profile for signing/upload.
 For the music-player port:
 a C cross-compiler (Xcode's clang) for libopus via cc,
 and (for the gate) gomobile if linking kopia's Go c-archive.

## Supporting-stack vets this framework drags in

- kopia static-lib vet:
   build kopia as a gomobile c-archive (.
  xcframework) for aarch64-apple-ios and confirm the Go runtime initializes
  and goroutines schedule in-process inside the Rust binary
- in-app HTTP server crate vet:
   confirm tiny_http/hyper/axum cross-compiles for aarch64-apple-ios
   and binds 127.0.0.1 as kopia's S3/HTTP target on-device
- HTTPS streaming client vet:
   confirm reqwest+rustls (or hyper) cross-compiles for aarch64-apple-ios and streams chunked uploads to pCloud
- background-transfer vet:
   drive NSURLSession background config + BGTaskScheduler from Rust via objc2 (or a Swift shim)
   and confirm uploads continue when suspended;
   restructure the multi-hour snapshot around it
- Rust FFI binding vet:
   `build.rs` static linking of Go c-archive and Rust staticlib,
   extern "C" ABI,
   plus AVAudioSession setup shim for cpal
- cpal iOS CoreAudio device vet:
   confirm low-latency output,
   correct AVAudioSession category/activation,
   and acceptable buffer sizes on a real device (narrower than macOS)
- renderer vet:
   confirm Skia/Metal (force-enabled on iOS) renders the music-player UI;
   femtovg GL path is unavailable on iOS
- QA:
   in-process Rust UI test (slint testing backend,
   internal/backends/testing),
   on-device e2e via XCUITest over the Xcode wrapper,
   cargo-fuzz on symphonia/peak-cache parsing,
   mutation testing on pagination/normalization logic,
   property tests on two-axis pagination + true-peak normalization

## Cited sources

- iOS app is an AOT-native cargo binary,
   lipo'd + codesigned (no JIT/interpreter):
   /tmp/agent/slint-audit-20260612/scripts/build_for_ios_with_cargo.
  bash:
  57,63,79
- Shipped iOS binary is a Rust fn main running the Slint event loop:
   /tmp/agent/slint-audit-20260612/tools/viewer/`main.rs`:
  153 (fn main),
   :
  278 (component.
  run());
   internal/backends/winit/`event_loop.rs`:
  700 (run_app)
- winit backend has a full iOS module wired via objc2/UIKit (UIView,
   UIScreen,
   CADisplayLink,
   keyboard,
   color scheme,
   touch):
   /tmp/agent/slint-audit-20260612/internal/backends/winit/`lib.rs`:
  35-36 (mod ios),
   internal/backends/winit/ios/`keyboard_animator.rs`:
  10-17,
   internal/backends/winit/`Cargo.toml`:
  130-135
- Documented,
   CI-tested iOS build producing a signed .
  xcarchive/.
  ipa for TestFlight (dev.
  slint.
  slint-viewer):
   /tmp/agent/slint-audit-20260612/docs/`ios.md`:
  1-76 and doc/astro/src/content/docs/guide/platforms/mobile/ios.
  mdx:
  14-39
- Skia/Metal is force-enabled on iOS;
   OpenGL (femtovg) path is unavailable on iOS:
   /tmp/agent/slint-audit-20260612/internal/backends/winit/`build.rs`:
  9 (ios_and_friends),
   :
  10 (enable_skia_renderer includes ios_and_friends),
   :
  13 (supports_opengl false on ios_and_friends);
   internal/backends/winit/`Cargo.toml`:
  126-128 (i-slint-renderer-skia pulled for apple-non-macos)
- cpal has a dedicated CoreAudio iOS backend;
   aarch64-apple-ios is a supported target:
   /tmp/agent/cpal-audit-20260612/src/host/coreaudio/ios/`mod.rs`;
   `Cargo.toml`:
  160 (cfg(any(target_os=ios,
   tvos)) coreaudio deps);
   `Cargo.toml`:
  234 (aarch64-apple-ios)
- music-player already targets CoreAudio via cpal 0.18 on non-Linux and pins Slint winit;
   femtovg+software renderers (femtovg unused on iOS):
   /var/home/user/Monochromatic/package/music-player/desktop-app/`Cargo.toml`:
  75 (slint git pin,
   renderer-femtovg+software),
   :
  111 (cpal=0.18 under cfg(not linux)),
   src/`output_cpal.rs`:
  1 (CoreAudio/WASAPI)
- Background URLSession/BGTask reachable from Rust via objc2-foundation (NSRunLoop/NSOperation linked);
   no Slint-provided background mechanism:
   /tmp/agent/slint-audit-20260612/internal/backends/winit/`Cargo.toml`:
  132 (objc2-foundation features NSRunLoop,
   NSOperation,
   NSNotification)

## Adversarial cite-check

- Wall 2 confirmed:
   confirmed
- Wall 1 confirmed:
   confirmed
- Corrections:
   The wall-2 "no-jit-needed" verdict and wall-1 "yes" verdict both hold,
   but two cited sources are mis-attributed and one objc2 feature is overstated.
   Details:

1. MIS-CITATION (contradicted sub-claim,
    does NOT move the verdict):
    The audit cites `tools/viewer/main.rs:153 (fn main -> component.run())` as evidence for the AOT runtime model.
    Line 153 is indeed `fn main() -> Result<()>` (confirmed),
    but the "-> component.
   run()" part is wrong for the iOS code path of that very binary.
    On iOS,
    viewer main forces `--remote` (`main.rs`:
   170-171),
    returns early via `remote::run(...)` (`main.rs`:
   194),
    which calls `slint_interpreter::run_event_loop()` (`remote.rs`:
   56) and dynamically COMPILES .
   slint at runtime via `slint_interpreter::Compiler` (`remote.rs`:
   217,238).
    The `component.run()` line is `main.rs`:
   278 and is NEVER reached on iOS.
    So the audit cited the one shipped Slint iOS binary whose iOS path is interpreter-mode,
    not AOT.
    The correct evidence for the AOT `.run()` pattern is the music-player's own `src/main.rs:2004 app.run()`
    (with `slint::include_modules!()` at src/`main.rs`:
   19).
    Corrected fact/source:
    AOT model is proven by music-player `build.rs`:
   5 `slint_build::compile("ui/app.slint")` + `main.rs`:
   19 `include_modules!()` + `main.rs`:
   2004 `app.run()` + zero interpreter deps in its `Cargo.toml`;
    the viewer is the wrong citation for it.

2. EVIDENCE/SHIPS GAP:
    The cited `scripts/build_for_ios_with_cargo.bash` is the slint-VIEWER's iOS build script,
    invoked as `slint-viewer --features remote` (tools/viewer/`ios-project.yml`:
   57),
    not the music-player's.
    Line 57 (`cargo build --target $CARGO_TARGET --bin "$1"`)
    and the `aarch64-apple-ios` target (line 46) accurately demonstrate the native-cargo-binary model
    that BOTH walls rest on,
    so the wall-1 premise (native --bin into which any static archive links via Rust FFI) is supported.
    But what that script actually builds is the interpreter-mode viewer;
    the music-player has no iOS build script in-repo yet.
    The verdict still holds because (a) music-player is AOT (verified independently)
    and (b) even the interpreter has NO JIT/cranelift/wasmtime/PROT_EXEC/mprotect
    (internal/interpreter is a tree-walking `eval.rs`),
    so no Slint code path trips W^X/DENY_EXECMEM regardless.

3. MINOR OVERSTATEMENT:
    iosRuntimeModel lists "objc2-ui-kit:
    UIApplication/UIView/UIScreen".
    At internal/backends/winit/`Cargo.toml`:
   133,
    the objc2-ui-kit feature list includes UIScreen,
    UIWindow,
    UIView,
    UIResponder,
    UIInterface,
    etc.,
    but NOT UIApplication.
    UIView/UIScreen and CADisplayLink (objc2-quartz-core,
    line 134) are confirmed;
    UIApplication is not an enabled feature there.
- Sources checked:
   /tmp/agent/slint-audit-20260612/scripts/build_for_ios_with_cargo.
  bash (line 57 cargo build --bin confirmed;
   line 46 aarch64-apple-ios target);
   /tmp/agent/slint-audit-20260612/tools/viewer/`main.rs` (line 153 fn main confirmed;
   component.
  run() at 278 NOT reached on iOS;
   iOS forces --remote at 170-171,
   returns at 194);
   /tmp/agent/slint-audit-20260612/tools/viewer/`remote.rs` (iOS path:
   slint_interpreter:
  :
  run_event_loop() at 56,
   runtime Compiler at 217/238);
   /tmp/agent/slint-audit-20260612/tools/viewer/`ios-project.yml`
   (line 57 invokes build script with slint-viewer --features remote);
   /tmp/agent/slint-audit-20260612/tools/viewer/`Cargo.toml` (slint-interpreter dependency at line 80);
   /tmp/agent/slint-audit-20260612/internal/backends/winit/`Cargo.toml`
   (lines 130-135 iOS objc2/objc2-ui-kit/objc2-quartz-core;
   UIApplication absent from feature list);
   /tmp/agent/slint-audit-20260612/internal/backends/winit/`event_loop.rs` (line 700 .
  run_app,
   ios_and_friends-gated at 698);
   /tmp/agent/slint-audit-20260612/internal/interpreter/ (no JIT/cranelift/PROT_EXEC;
   tree-walking `eval.rs`);
   /var/home/user/Monochromatic/package/music-player/desktop-app/`Cargo.toml` (line 143 slint-build git pin confirmed;
   line 75 slint git pin;
   no interpreter/jit deps);
   /var/home/user/Monochromatic/package/music-player/desktop-app/`build.rs` (line 5 slint_build:
  :
  compile);
   /var/home/user/Monochromatic/package/music-player/desktop-app/src/`main.rs` (line 19 include_modules!
  ;
   line 1181 fn main;
   line 2004 app.
  run())
