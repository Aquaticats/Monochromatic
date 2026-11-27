//! Native bridge for the full-Rust engine flavor.
//!
//! JNI entry points plus the ported decode path (symphonia + libopus).
//!  The
//! self-tests prove the toolchain and decoders cross-compile and run on the
//! GrapheneOS device;
//!  `nativeDecodeBenchmark` times native decode-to-PCM so it
//! can be compared head to head against the Media3 MediaCodec baseline.
//!  This file
//! declares the crate's modules and holds the entries that take no handle:
//! the
//! self-tests,
//!  the two decode benchmarks,
//!  the synthetic true-peak measurement and the
//! output latency probe.
//!  The entries that drive a playback engine handle live in
//! `engine_jni.rs`,
//!  and the true-peak service's entries in `service_jni.rs`,
//!  so this
//! file stays under the per-file code-line budget (`AGENTS.md`,
//! MXL).
//!
//! Dum-dum orientation for a TypeScript reader:
//!  "JNI" (Java Native Interface) is
//! the bridge that lets Android's Kotlin/Java code call into compiled native code
//! (this Rust,
//!  built into a `.so` shared library) and get values back.
//!  Every
//! `pub extern "system" fn Java_dev_monochromatic_..._native...` function below is
//! one callable slot on the Kotlin `NativeBridge` class:
//!  the long mangled name IS
//! the wiring (package path + class + method,
//!  joined by underscores),
//!  so Kotlin's
//! `external fun nativePing(): Int` finds this `..._nativePing` by name at runtime.
//! Mentally,
//!  picture this whole file as a set of exported functions in a `.node`
//! native addon that a TS file imports and calls;
//!  the parameter and return types
//! are deliberately limited to the handful the JVM understands (`jint`,
//!  `jlong`,
//! `jdouble`,
//!  `jfloat`,
//!  `jboolean`,
//!  `JString`),
//!  which are just the platform's
//! fixed-width integers/floats and an opaque Java-string handle.

/// What:
///      `mod bench;` declares the decode-benchmark submodule (the sibling file
///           `bench.rs`).
///  `mod NAME;` pulls that file in as a private child module.
/// Why:
///       `benchmark_decode` was split out of this file to keep it under the
///           max-lines budget;
///  the JNI benchmark exports below call `bench::...`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as bench from "./bench";
/// ```
mod bench;
/// What:
///      `mod decode;` declares a child module named `decode` and tells the
///           compiler its code lives in the sibling file `decode.rs`.
///  A "module" is
///           Rust's namespace/file-grouping unit.
///  The other `mod` lines do the same
///           for `engine.rs`,
///  `engine_worker.rs`,
///  `error.rs`,
///  `opus.rs`,
///  `output.rs`,
///           and `truepeak.rs`.
/// Why:
///       This file (the crate root) is the only place that lists the crate's
///           modules;
///  without these lines those sibling files are never compiled and
///           `decode::open`,
///  `engine::Engine`,
///  etc. below would not resolve.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // No runtime statement; the bundler discovers ./decode.ts when it is imported.
/// // Mentally: `import * as decode from "./decode";` makes `decode.open` reachable.
/// ```
mod decode;
/// What:
///      `mod engine;` declares the `engine` child module,
///  compiled from
///           `engine.rs`.
///  It holds the playback `Engine` type the JNI handle wraps.
/// Why:
///       So `engine::Engine::new()` and the `engine_ref.*` method calls below
///           resolve to real code.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as engine from "./engine";
/// ```
mod engine;
/// What:
///      `mod engine_jni;` declares the playback-engine JNI entry points (`engine_jni.rs`):
///           the create,
///  load,
///  play,
///  pause,
///  seek,
///  volume and gain setters,
///           the position/duration/flags readers and release.
///  `mod NAME;` pulls that file
///           in as a private child module;
///  the entries are still exported from the `.so`,
///           because `#[unsafe(no_mangle)]` keeps each symbol's name regardless of the
///           module's visibility.
/// Why:
///       Those entries were split out of this file to keep it under the per-file
///           code-line budget (MXL);
///  nothing here calls them,
///  the JVM finds them by symbol
///           name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./engine_jni"; // its exports register themselves
/// ```
mod engine_jni;
/// What:
///      `mod engine_worker;` declares the `engine_worker` child module
///           (`engine_worker.rs`),
///  the background thread the engine drives.
/// Why:
///       The `engine` module spawns it;
///  declaring it here puts it in the build.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as engine_worker from "./engine_worker";
/// ```
mod engine_worker;
/// What:
///      `mod error;` declares the `error` child module (`error.rs`),
///  home of
///           the shared `PlayerError` type that all fallible calls funnel into.
/// Why:
///       Many functions below return `Result<_, PlayerError>`;
///  this brings that
///           type's definition into the crate.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as error from "./error";
/// ```
mod error;
/// What:
///      `mod fingerprint;` declares the `fingerprint` child module
///           (`fingerprint.rs`),
///  which holds the `nativeFingerprint` JNI entry
///           over the shared XXH3-64 cache-key fingerprint.
///  The entry is `#[unsafe(no_mangle)]`,
///  so the JVM
///           finds its symbol in the `.so` even though the module is private here.
/// Why:
///       Keeps the new JNI export and its hashing out of this file (and under the
///           per-file code-line budget);
///  nothing in this file calls it directly.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as fingerprint from "./fingerprint"; // its native export is auto-registered
/// ```
mod fingerprint;
/// What:
///      `mod jni_call;` declares the JNI environment helper submodule (the sibling file
///           `jni_call.rs`).
/// Why:
///       jni 0.22 only lends an `Env` inside a closure;
///  the helper keeps that closure plumbing and the failure logging in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as jniCall from "./jni_call";
/// ```
mod jni_call;
/// What:
///      `mod logging;` declares the logcat subscriber submodule (the sibling file
///           `logging.rs`).
///  It installs the `tracing` -> Android logcat sink once,
///  guarded by a
///           `OnceLock`,
///  so every `tracing` event from this crate and `truepeak-core` reaches
///           logcat (stderr does not on Android).
/// Why:
///       Keep the subscriber setup out of this file;
///  the JNI create entries call
///           `logging::init()` once at startup.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as logging from "./logging";
/// ```
mod logging;
/// What:
///      `mod opus;` declares a LOCAL child module named `opus` (`opus.rs`),
///           our own Opus glue.
///  Note:
///  there is ALSO an external crate also named
///           `opus` (libopus bindings);
///  this local module shadows that name at the
///           crate root,
///  which is why `nativeOpusSelfTest` reaches the external one
///           with the leading-`::` form `::opus` (see its comment).
/// Why:
///       Our decode path uses this local wrapper around libopus.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as opus from "./opus";
/// ```
mod opus;
/// What:
///      `mod output;` declares the `output` child module (`output.rs`),
///  the
///           AAudio (Android's low-latency audio) output backend.
/// Why:
///       `output::measure_output_latency_ms()` below lives here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as output from "./output";
/// ```
mod output;
/// What:
///      `mod peak_cache;` declares the peek and invalidate JNI entries (`peak_cache.rs`).
/// Why:
///       Keeps the per-track peak reads out of the service module,
/// which is near its line budget.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as peakCache from "./peak_cache";
/// ```
mod peak_cache;
/// What:
///      `mod service;` declares the native true-peak service submodule (the sibling file
///           `service.rs`).
///  It holds the `TruePeakService` cache-actor handle,
///  its request messages and
///           the actor thread that serves them;
///  the JNI entries Kotlin calls to create,
///           release,
///  resolve and warm that service live in `service_jni.rs`.
/// Why:
///       Keep the Turso-backed service out of this file (max-lines) while its
///           `#[unsafe(no_mangle)]` exports still land in the cdylib.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as service from "./service";
/// ```
mod service;
/// What:
///      `mod service_jni;` declares the true-peak service's JNI entry points
///           (`service_jni.rs`):
///  `nativeTruePeakServiceCreate`/`Release`,
///  `nativeResolveGain`
///           and `nativeWarmTrack`,
///  plus the resolve-then-cache body they share.
///  `mod NAME;`
///           pulls that file in as a private child module;
///  the entries are still exported from
///           the `.so`,
///  because `#[unsafe(no_mangle)]` keeps each symbol's name regardless of
///           the module's visibility.
/// Why:
///       Those entries were split out of `service.rs` to keep it under the per-file
///           code-line budget (MXL);
///  nothing here calls them,
///  the JVM finds them by symbol
///           name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import "./service_jni"; // its exports register themselves
/// ```
mod service_jni;
/// What:
///      `mod truepeak;` declares the `truepeak` child module (`truepeak.rs`),
///  the
///           shared-source adapter and the `resolve_current`/`resolve_full` resolvers plus
///           the re-exported `true_peak_interleaved` helper.
/// Why:
///       `service.rs` drives `truepeak::resolve_current`/`resolve_full`,
///  and the synthetic
///           JNI entry below calls `truepeak::true_peak_interleaved(...)`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as truepeak from "./truepeak";
/// ```
mod truepeak;

/// What:
///      `use std::os::fd::RawFd;` imports the Unix raw-file-descriptor type.
///           A file descriptor is a small integer the OS uses to name an open
///           file/stream.
///  `RawFd` is a plain type alias for `i32` (a 32-bit signed
///           integer;
///  the OS reserves `-1` for "no fd",
///  which is why it is signed,
///           not the sibling `u32`).
///  `use` just brings the name into scope so we can
///           write `RawFd` instead of the full `std::os::fd::RawFd` path.
/// Why:
///       We convert the JVM's `jint` fd into a `RawFd` before handing it to the
///           decoder/engine,
///  which speak in `RawFd`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RawFd = number; // a bare OS file-descriptor integer
/// ```
use std::os::fd::RawFd;
/// What:
///      `use std::path::Path;` imports the borrowed filesystem-path type.
///           `Path` is an unsized,
///  borrowed VIEW of a path (its owned,
///  growable
///           sibling is `PathBuf`,
///  exactly like `&str` is to `String`).
/// Why:
///       `decode::open` takes `&Path`,
///  so we wrap the decoded path string in a
///           `Path` reference before calling it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Path = string;
/// ```
use std::path::Path;

/// Runs a body with the call's `Env`,
/// catching a panic and logging it,
/// as every entry point of this crate does.
use crate::jni_call::{
    ArrayLength,
    CopyArrayRegion,
    ReadJavaString,
    run_with_env,
};
/// What:
///      `use jni::EnvUnowned;` imports the per-call interface pointer the JVM hands
///           every native method (jni 0.22 renamed the former `EnvUnowned` argument type).
///  `EnvUnowned<'local>` is the raw argument;
///  the JNI calls (read a string,
///  copy an array,
///  throw) live on the `Env` that `jni_call::run_with_env` lends inside a closure.
///  It is valid only for the
///           duration of one native call and only on the calling thread.
/// Why:
///       The string and array entry points pass it to `run_with_env` to pull a
///           Rust string or float slice out of the JVM handles.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EnvUnowned = RuntimeContext; // per-call handle to talk to the host runtime
/// ```
use jni::EnvUnowned;
/// What:
///      `use jni::objects::{JClass, JString};` imports two handle types from the
///           `jni` crate.
///  `JClass<'local>` is a borrowed handle to the Java/Kotlin
///           class object that invoked us;
///  `JString<'local>` is a borrowed handle to
///           a Java string passed across the boundary (NOT a Rust `String` yet,
///  it
///           must be converted).
///  The `{A, B}` braces import several names in one line.
/// Why:
///       Every JNI entry point receives the calling class,
///  and the path/string
///           functions also receive a `JString` argument;
///  we need these types named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type JClass = OpaqueHandle;  // the calling class object
/// type JString = OpaqueHandle; // a Java string handle, convert before use
/// ```
use jni::objects::{
    JClass,
    JFloatArray,
    JString,
};
/// What:
///      `use jni::sys::{jdouble, jfloat, jint};` imports the JVM's
///           fixed-width primitive types as Rust aliases.
///  `jint` is a 32-bit
///           signed integer (Java `int`),
///  `jdouble` a 64-bit float (Java `double`),
///  and
///           `jfloat` a 32-bit float (Java `float`).
///  Siblings a reader might expect on the Rust
///           side are `i32`/`f64`/`f32`;
///  we use the `j*` aliases because the function
///           signatures must match exactly what the JVM passes.
///  The wider `jlong` (a 64-bit
///           signed integer,
/// Java `long`) and `jboolean` (an alias of Rust `bool` since
///           jni 0.22) are no longer named here:
///  only `engine_jni.rs` needs them now.
/// Why:
///       The JNI functions can only speak these types across the boundary;
///  using
///           the aliases documents "this is a JVM-ABI value",
///  not a free Rust value.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type jint = number;     // 32-bit signed
/// type jdouble = number;  // 64-bit float
/// type jfloat = number;   // 32-bit float
/// ```
use jni::sys::{
    jdouble,
    jfloat,
    jint,
};

// What:     `#[unsafe(no_mangle)]` is an ATTRIBUTE (a compiler annotation, written
//           `#[...]`) telling the compiler "do NOT rename this function's symbol".
//           Rust normally scrambles ("mangles") symbol names for uniqueness; the JVM
//           must find this function by the exact name `Java_dev_..._nativePing`, so
//           mangling must be off.
// Why:      Without `#[unsafe(no_mangle)]` the JVM's `System.loadLibrary` + `native` lookup
//           would fail to find the symbol and the call would crash at link time.
//
// In TS you'd write (pseudocode):
// ```ts
// // no annotation needed; `export function nativePing()` keeps its name
// ```
#[unsafe(no_mangle)]
/// What:
///      `pub extern "system" fn Java_..._nativePing<'local>(...) -> jint`
///           declares the function.
///  `pub` = visible outside this module.
///  `extern
///           "system"` = use the platform's C/JVM calling convention so the JVM can
///           call it (NOT Rust's internal convention).
///  The long name is the JNI
///           wiring:
///  `Java_` + package path + class + method,
///  underscore-joined.
///           `<'local>` introduces a LIFETIME parameter named `local` (a label,
///  not a
///           value) used by the borrowed JVM handle types.
///  `-> jint` returns a 32-bit
///           signed JVM int.
///  `_env` / `_class` are the two params every JNI method
///           gets;
///  the leading `_` marks them deliberately unused.
/// Why:
///       This is the first slot Kotlin calls to prove the `.so` loaded and an int
///           survives the round trip;
///  it just returns a known constant.
/// Gotcha:
///    `extern "system"` means a panic must NEVER cross this boundary (it would
///           abort the process);
///  this function only returns a literal,
///  so it is safe.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nativePing(_env: EnvUnowned, _class: JClass): number {
///   return 42;
/// }
/// ```
pub extern "system" fn Java_dev_monochromatic_musicplayer_NativeBridge_nativePing<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
) -> jint {
    // What:     `42` is a bare literal at the end of the function body with no `;`.
    //           In Rust, the final expression of a block WITHOUT a trailing semicolon
    //           IS the return value (a "tail expression"), so this returns 42.
    // Why:      Hand the Kotlin side a known sentinel it can assert on.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return 42;
    // ```
    return 42;
}

// What:     `#[unsafe(no_mangle)]` again: keep the symbol name unmangled so the JVM finds it.
// Why:      Same reason as `nativePing`: the JVM looks this function up by exact name.
//
// In TS you'd write (pseudocode):
// ```ts
// // no annotation needed
// ```
#[unsafe(no_mangle)]
/// What:
///      Same declaration shape as `nativePing`:
///  `pub extern "system"`,
///  the JNI
///           mangled name,
///  a `<'local>` lifetime,
///  the two unused `_env`/`_class`
///           params,
///  and a `-> jint` return.
/// Why:
///       A second self-test slot Kotlin calls;
///  returns 1 or 0 as success/failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nativeOpusSelfTest(_env: EnvUnowned, _class: JClass): number { ... }
/// ```
pub extern "system" fn Java_dev_monochromatic_musicplayer_NativeBridge_nativeOpusSelfTest<
    'local,
>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
) -> jint {
    // What:     `match ::opus::Decoder::new(48_000, ::opus::Channels::Stereo) { ... }`
    //           builds an Opus decoder and inspects the `Result` it returns. `::opus`
    //           with a LEADING `::` means "the EXTERNAL crate named opus", not our
    //           local `mod opus` (which shadows the bare name `opus` here). `::` is
    //           the path separator (like `.` in TS module paths). `Decoder::new(...)`
    //           is an associated constructor function. `48_000` is the sample rate;
    //           the `_` in the number is just a digit separator (= 48000).
    //           `::opus::Channels::Stereo` is an enum VARIANT (one named choice of the
    //           `Channels` enum, here "two channels"). `Decoder::new` returns a
    //           `Result<Decoder, Error>` (success-or-failure container); `match`
    //           branches on which case it is.
    // Why:      Prove the bundled C libopus links and a decoder constructs on-device.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try {
    //   new opus.Decoder(48000, opus.Channels.Stereo);
    //   return 1;
    // } catch {
    //   return 0;
    // }
    // ```
    match ::opus::Decoder::new(48_000, ::opus::Channels::Stereo) {
        // What:     `Ok(_decoder) => 1` is the success arm of the `match`. `Ok(...)`
        //           is the success variant of `Result`; we DESTRUCTURE the decoder out
        //           of it into `_decoder`, whose leading `_` says "I do not use this
        //           value, just confirm it exists". The arm's value `1` (no `;`) is
        //           what the whole `match` evaluates to.
        // Why:      Construction succeeded, so report success (1) to Kotlin.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // succeeded: return 1;
        // ```
        Ok(_decoder) => return 1,
        // What:     `Err(_error) => 0` is the failure arm. `Err(...)` is the failure
        //           variant of `Result`; we destructure the error into `_error` and
        //           ignore it (leading `_`). The arm yields `0`.
        // Why:      Construction failed, so report failure (0) to Kotlin.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // failed: return 0;
        // ```
        Err(_error) => return 0,
    }
}

// What:     `#[unsafe(no_mangle)]`: keep the symbol name as-is for JVM lookup.
// Why:      Same as the other entry points.
//
// In TS you'd write (pseudocode):
// ```ts
// // no annotation needed
// ```
#[unsafe(no_mangle)]
/// What:
///      Same JNI-entry declaration shape:
///  `pub extern "system"`,
///  mangled name,
///           `<'local>` lifetime,
///  unused `_env`/`_class`,
///  `-> jint`.
/// Why:
///       A self-test slot that forces symphonia's registries to initialize.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nativeSymphoniaSelfTest(_env: EnvUnowned, _class: JClass): number { ... }
/// ```
pub extern "system" fn Java_dev_monochromatic_musicplayer_NativeBridge_nativeSymphoniaSelfTest<
    'local,
>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
) -> jint {
    // What:     `std::hint::black_box(symphonia::default::get_probe());`. The inner
    //           `symphonia::default::get_probe()` builds the container prober.
    //           `std::hint::black_box(x)` is a compiler hint that takes a value and
    //           hands it straight back, but makes the optimizer treat it as "used for
    //           real" so it cannot delete the call as dead code. `::` are path
    //           separators (module/function navigation).
    // Why:      We want the prober to actually initialize on-device; without
    //           `black_box`, the optimizer could notice we throw the result away and
    //           skip the work, defeating the self-test.
    // Gotcha:   `black_box` is NOT a no-op you can delete; removing it changes whether
    //           the optimizer runs the call at all.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // symphonia.getProbe(); // (no black_box concept needed in TS)
    // ```
    std::hint::black_box(symphonia::default::get_probe());
    // What:     `std::hint::black_box(symphonia::default::get_codecs());` does the
    //           same for the codec registry: build it, and prevent the optimizer from
    //           eliding the call.
    // Why:      Force the codec registry to initialize on-device too.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // symphonia.getCodecs();
    // ```
    std::hint::black_box(symphonia::default::get_codecs());
    // What:     `1` is the tail expression (no trailing `;`), so it is the function's
    //           return value: report success.
    // Why:      Both registries initialized; tell Kotlin "ok" (1).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return 1;
    // ```
    return 1;
}

// What:     `#[unsafe(no_mangle)]`: keep the symbol name for JVM lookup.
// Why:      Same as the other JNI entry points.
//
// In TS you'd write (pseudocode):
// ```ts
// // no annotation needed
// ```
#[unsafe(no_mangle)]
/// What:
///      The JNI entry declaration.
///  Same shape as before,
///  but with a THIRD
///           parameter `path: JString<'local>` (a borrowed Java-string handle),
///  and
///           `env` is taken WITHOUT a leading `_` this time because we actually use it.
///           `-> jdouble` returns a 64-bit float (the throughput or a negative error).
/// Why:
///       Kotlin calls this with a filesystem path string to benchmark a file;
///  it
///           times the decode loop only (not the open/probe) and returns throughput,
///           or a negative sentinel:
///  -1 bad path string,
///  -2 open failed,
///  plus the
///           shared codes from `benchmark_decode`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nativeDecodeBenchmark(env: EnvUnowned, _class: JClass, path: JString): number { ... }
/// ```
pub extern "system" fn Java_dev_monochromatic_musicplayer_NativeBridge_nativeDecodeBenchmark<
    'local,
>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    path: JString<'local>,
) -> jdouble {
    // What:     The `env` parameter is declared `mut` right in the signature, so the
    //           `run_with_env` call below (which takes `&mut EnvUnowned`) can borrow it mutably.
    // Why:      The `jni` API needs a mutable borrow to lend the `Env`. Declaring the parameter
    //           mutable carries that need directly instead of rebinding `env` to a new
    //           mutable local of the same name.
    // Gotcha:   `mut` here is NOT a copy of the JVM env; the function still holds the same
    //           handle, merely allowed to call its `&mut self` methods.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // (no-op in TS; env is already reassignable)
    // ```
    // What:     `let Some(path_str) = run_with_env(&mut env, ..., ReadJavaString { text: &path }) else { ... };`.
    //           `run_with_env` lends the `Env` to the `ReadJavaString` body, which
    //           pulls the Java string into Rust; `path` is BORROWED (read-only loan, no
    //           ownership transfer). The helper returns `None` after logging when the
    //           read fails, so the `else` arm returns the sentinel. `path_str` is an
    //           OWNED, heap, growable UTF-8 string (sibling `&str` is a borrowed view
    //           we cannot use here because the string must outlive the JVM handle).
    // Why:      Convert the opaque JVM string into a real owned Rust `String` we can
    //           build a `Path` from.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let pathStr: string;
    // try { pathStr = env.getString(path); } catch { return -1.0; }
    // ```
    let Some(path_str) = run_with_env(
        &mut env,
        "nativeDecodeBenchmark path",
        ReadJavaString {
            text: &path,
        },
    ) else {
        // What:     The `else` arm `return -1.0` exits the whole function with the
        //           "-1 bad path string" sentinel; `run_with_env` already logged the cause.
        // Why:      If the JVM string could not be read, report the bad-path code.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // catch { return -1.0; }
        // ```
        return -1.0;
    };
    // What:     `let source = match decode::open(Path::new(&path_str)) { ... };`.
    //           `Path::new(&path_str)` wraps a read-only BORROW of `path_str` (`&`) in
    //           a `Path` view (no copy, no ownership transfer). `decode::open(...)`
    //           returns `Result<Box<dyn Source>, PlayerError>`; we `match` on it and
    //           bind the opened decoder to `source`.
    // Why:      Open the file as a decoder before benchmarking it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let source: Source;
    // try { source = decode.open(pathStr); } catch { return -2.0; }
    // ```
    let source = match decode::open(Path::new(&path_str)) {
        // What:     `Ok(source) => source`. Success arm: destructure the opened
        //           `Box<dyn Source>` out of `Ok` and yield it directly as the match
        //           value (which the outer `let source = ...` then binds).
        // Why:      Open succeeded; keep the decoder.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // success: source = the opened decoder
        // ```
        Ok(source) => source,
        // What:     `Err(_) => return -2.0`. Failure variant, error discarded; `return
        //           -2.0` exits with the "-2 open failed" sentinel.
        // Why:      Could not open the file; report the open-failure code.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // catch { return -2.0; }
        // ```
        Err(_) => return -2.0,
    };
    // What:     `bench::benchmark_decode(source)` is the tail expression (no `;`), so
    //           its return value becomes this function's return value. We pass the opened
    //           `source` by VALUE (moving ownership into the helper).
    // Why:      Delegate the timed decode loop to the shared helper and return its
    //           microseconds-per-sample figure.
    // Gotcha:   `source` is MOVED here: after this call the caller no longer owns it.
    //           In TS the reference would still be usable; in Rust it is gone.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return benchmarkDecode(source);
    // ```
    return bench::benchmark_decode(source);
}

// What:     `#[unsafe(no_mangle)]`: keep the symbol name for JVM lookup.
// Why:      Same as the other JNI entry points.
//
// In TS you'd write (pseudocode):
// ```ts
// // no annotation needed
// ```
#[unsafe(no_mangle)]
/// What:
///      JNI entry declaration.
///  Third parameter is `fd: jint` (a 32-bit signed JVM
///           int holding the Android file descriptor).
///  `_env`/`_class` unused;
///  returns
///           `jdouble` (throughput or negative error code).
/// Why:
///       Kotlin calls this with a borrowed `content://` file descriptor (a
///           `ParcelFileDescriptor.getFd()`) to benchmark it;
///  `open_borrowed_fd` dups
///           the fd synchronously so the JVM keeps and closes the original.
///  Returns
///           throughput,
///  or a negative sentinel:
///  -1 bad fd,
///  -2 dup/open failed,
///  plus
///           the shared codes from `benchmark_decode`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nativeDecodeFdBenchmark(_env: EnvUnowned, _class: JClass, fd: number): number { ... }
/// ```
pub extern "system" fn Java_dev_monochromatic_musicplayer_NativeBridge_nativeDecodeFdBenchmark<
    'local,
>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    fd: jint,
) -> jdouble {
    // BorrowedFd::borrow_raw panics on -1, and that panic across extern "system"
    // would abort, so reject a negative fd here in the error-code convention.
    // What:     `if fd < 0 { return -1.0; }`. Plain conditional with `<` comparison.
    //           A true condition returns the "-1 bad fd" sentinel from the whole
    //           function.
    // Why:      A negative fd is invalid; we reject it BEFORE handing it to
    //           `borrow_raw`, which would otherwise panic and (across `extern
    //           "system"`) abort the whole process.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (fd < 0) return -1.0;
    // ```
    if fd < 0 {
        return -1.0;
    }
    // What:     `let source = match decode::open_borrowed_fd(fd as RawFd) { ... };`.
    //           `fd as RawFd` is an explicit cast from `jint` (i32) to `RawFd` (also
    //           i32 underneath, but a distinct alias the API wants). `open_borrowed_fd`
    //           dups the fd and returns `Result<Box<dyn Source>, PlayerError>`; we
    //           `match` and bind the decoder to `source`.
    // Why:      Open a decoder over the (duplicated) Android file descriptor.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let source: Source;
    // try { source = decode.openBorrowedFd(fd); } catch { return -2.0; }
    // ```
    let source = match decode::open_borrowed_fd(fd as RawFd) {
        // What:     `Ok(source) => source`. Success arm: destructure the decoder out of
        //           `Ok` and yield it as the match value.
        // Why:      Open succeeded; keep the decoder.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // success: source = the opened decoder
        // ```
        Ok(source) => source,
        // What:     `Err(_) => return -2.0`. Failure variant, error discarded; return
        //           the "-2 dup/open failed" sentinel.
        // Why:      Could not dup/open the fd; report the failure code.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // catch { return -2.0; }
        // ```
        Err(_) => return -2.0,
    };
    // What:     `bench::benchmark_decode(source)` is the tail expression: run the shared
    //           timed loop and return its figure. `source` is MOVED into the helper.
    // Why:      Reuse the same benchmark loop the path variant uses.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return benchmarkDecode(source);
    // ```
    return bench::benchmark_decode(source);
}

// What:     `#[unsafe(no_mangle)]`: keep the symbol name unmangled so the JVM finds it.
// Why:      Same as the other JNI entry points.
//
// In TS you'd write (pseudocode):
// ```ts
// // no annotation needed
// ```
#[unsafe(no_mangle)]
/// What:
///      `pub extern "system" fn Java_..._nativeTruePeakSynthetic<'local>(env,`
///           `_class, samples: JFloatArray, channels: jint) -> jfloat`.
///  A TEST-ONLY
///           JNI entry that measures the true peak of an IN-MEMORY interleaved-`f32`
///           array handed straight from Kotlin,
///  bypassing the decoder.
///           `samples: JFloatArray<'local>` is the JVM `float[]` handle;
///  `channels:
///           jint` the interleave width;
///  `-> jfloat` returns the measured peak (or a
///           negative sentinel on a JNI read error).
/// Why:
///       Production `nativeMeasureTruePeak` needs a real encoded file + a
///           `content://` descriptor,
///  so an instrumented test cannot assert a KNOWN
///           golden peak through it.
///  This entry lets the on-device test feed a
///           synthetic signal with a known inter-sample peak and verify the SAME
///           `TruePeakMeter` + `catmull_rom` path on the real arm64 target.
///  It is
///           exercised ONLY by `NativeBridgeTest`,
///  never by production Kotlin.
/// Gotcha:
///    Returns `-1.0` if the JVM array cannot be read;
///  a real peak is >= 0.0,
///           so the test treats any negative value as a JNI failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nativeTruePeakSynthetic(env, _class, samples: number[], channels: number): number { ... }
/// ```
pub extern "system" fn Java_dev_monochromatic_musicplayer_NativeBridge_nativeTruePeakSynthetic<
    'local,
>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    samples: JFloatArray<'local>,
    channels: jint,
) -> jfloat {
    // What:     `if channels <= 0 { return 0.0; }`. Reject a non-positive channel
    //           count up front, returning a `0.0` (silence) peak.
    // Why:      `true_peak_interleaved` also guards `channels == 0`; rejecting here
    //           keeps the `channels as usize` cast below safe and meaningful.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (channels <= 0) return 0.0;
    // ```
    if channels <= 0 {
        return 0.0;
    }
    // What:     `let Some(len) = run_with_env(..., ArrayLength { array: &samples }) else {`
    //           `return -1.0 };`. Ask the JVM how many floats the array holds.
    //           `JFloatArray::len` borrows the array and returns `Result<usize>`
    //           (jni 0.22 returns `usize`, not `jsize`); on error `run_with_env` logs and
    //           we bail with the `-1.0` read-error sentinel.
    // Why:      We must size the destination buffer before copying the elements out.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const len = samples.length;
    // ```
    let Some(len) = run_with_env(
        &mut env,
        "nativeTruePeakSynthetic length",
        ArrayLength {
            array: &samples,
        },
    ) else {
        return -1.0;
    };
    // What:     `if len == 0 { return 0.0; }`. An empty array measures as silence.
    // Why:      No samples means no peak; `0.0` maps to unity gain on the Kotlin side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (len <= 0) return 0.0;
    // ```
    if len == 0 {
        return 0.0;
    }
    // What:     `let mut buf = vec![0.0f32; len as usize];`. Allocate a zeroed
    //           host-side `Vec<f32>` of the array's length to receive the copy.
    // Why:      `get_float_array_region` copies INTO a Rust slice we own; the JVM
    //           array itself stays on the JVM heap.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const buf = new Float32Array(len);
    // ```
    let mut buf = vec![0.0f32; len];
    // What:     `let copied = run_with_env(..., CopyArrayRegion::new(&samples, &mut buf));`
    //           then `if copied.is_none() { return -1.0; }`. Copy all `len` floats from index 0 of
    //           the JVM array into `buf`; on any JNI error, bail with the `-1.0` sentinel.
    // Why:      Brings the synthetic PCM into Rust so the meter can scan it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // copy samples[0..len] into buf
    // ```
    let copied = run_with_env(
        &mut env,
        "nativeTruePeakSynthetic copy",
        CopyArrayRegion::new(&samples, &mut buf),
    );
    if copied.is_none() {
        return -1.0;
    }
    // What:     `truepeak::true_peak_interleaved(&buf, channels as usize)`. Run the
    //           production meter over the copied samples; `channels as usize` casts
    //           the JVM int to the index type. Tail expression, so its `f32` is the
    //           returned `jfloat`.
    // Why:      Reuse the exact measurement path production uses, so the test
    //           verifies real behaviour rather than a copy of it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return truePeakInterleaved(buf, channels);
    // ```
    return truepeak::true_peak_interleaved(&buf, channels as usize);
}

// What:     `#[unsafe(no_mangle)]`: keep the symbol name for JVM lookup.
// Why:      Same as the other JNI entry points.
//
// In TS you'd write (pseudocode):
// ```ts
// // no annotation needed
// ```
#[unsafe(no_mangle)]
/// What:
///      JNI entry declaration with no extra params;
///  `_env`/`_class` unused;
///           returns `jdouble` (latency in ms,
///  or -1.0 on failure).
/// Why:
///       Kotlin calls this to probe the native (raw ndk::audio) AAudio output
///           latency on-device;
///  it opens a silent low-latency stream (inaudible,
///  it
///           writes zeros) and returns the measured latency in milliseconds,
///  or -1.0
///           on failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nativeOutputLatencyProbe(_env: EnvUnowned, _class: JClass): number { ... }
/// ```
pub extern "system" fn Java_dev_monochromatic_musicplayer_NativeBridge_nativeOutputLatencyProbe<
    'local,
>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
) -> jdouble {
    // What:     `output::measure_output_latency_ms().unwrap_or(-1.0)`. The call returns an
    //           `Option<f64>` (Rust's null-free "maybe a value" container: either `Some(x)`
    //           with a value, or `None` for "no value"), NOT a `Result`. `.unwrap_or(
    //           fallback)` yields the inner `f64` on `Some` and the eager fallback `-1.0`
    //           on `None`. This call is the tail expression, so its value is returned.
    // Why:      Open the AAudio stream, measure latency, and either return the number or
    //           signal failure with -1.0.
    // Gotcha:   `.unwrap_or` collapses what was a two-arm `match` (`Some(ms) => ms` /
    //           `None => -1.0`); the eager `-1.0` is fine for a plain literal sentinel.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const ms = output.measureOutputLatencyMs(); // number | null
    // return ms ?? -1.0;
    // ```
    return output::measure_output_latency_ms().unwrap_or(-1.0);
}
