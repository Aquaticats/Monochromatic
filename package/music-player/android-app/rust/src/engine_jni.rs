//! The JNI entries that drive one playback engine handle.
//!
//! Kotlin creates the engine once (`nativeEngineCreate`),
//! hands the opaque `jlong` handle
//! back on every later call,
//! and releases it last (`nativeEngineRelease`).
//!  This file holds
//! those entries and nothing else,
//! so the crate root (`lib.rs`) keeps only its module
//! list and the free-standing entries:
//! the self-tests,
//! the two decode benchmarks,
//! the
//! synthetic true-peak measurement and the output latency probe.
//!
//! Dum-dum orientation for a TypeScript reader:
//!  an "opaque handle" is a number that
//! stands in for an object the other side must not look inside,
//!  the way a TS
//! `Map<number, Engine>` hands back a key instead of the object itself.
//!  Every entry
//! here turns that number back into a `&mut Engine` and calls exactly one method on
//! it;
//!  the audio work itself happens on the engine's own worker thread,
//! not on the
//! thread that makes these calls.
//!  The integer return codes (`0` ok,
//!  `-1` bad fd,
//!  `-2`
//! dispatch failed,
//!  `-3` no handle) are the exception to "no result":
//! an entry that
//! can fail reports why through them,
//! because a Kotlin `void` and a Kotlin `Int` are
//! different declarations on the other side.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const engines = new Map<number, Engine>();
//! // export function nativeEnginePlay(handle: bigint): void { engines.get(handle)?.play(); }
//! ```

/// What:
///      `use crate::engine;` names the sibling module `engine`,
///  compiled from
///           `engine.rs`.
///  `crate::` means "starting from this crate's root",
///  which is
///           `lib.rs`;
///  Rust paths reach the root from anywhere inside the crate that way.
/// Why:
///       Every entry below writes `engine::Engine` to name the type behind the
///           handle;
///  this import makes that path resolve here,
///  so the moved code keeps
///           reading exactly as it did in the crate root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as engine from "./engine";
/// ```
use crate::engine;
/// What:
///      `use crate::logging;` names the sibling module `logging` (`logging.rs`),
///           which installs the `tracing` sink that sends log lines to Android
///           logcat.
/// Why:
///       `nativeEngineCreate` installs that sink once before the engine logs
///           anything;
///  without the import the call would not resolve outside the crate
///           root.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as logging from "./logging";
/// ```
use crate::logging;
/// What:
///      `use jni::EnvUnowned;` imports the per-call interface pointer the JVM passes
///           to every native method (jni 0.22 renamed the former `JNIEnv` argument).
///  It
///           is valid only for the duration of one native call and only on the calling
///           thread.
/// Why:
///       Every entry's signature takes one as its first parameter,
///  even the ones that
///           ignore it (those name it `_env`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EnvUnowned = RuntimeContext; // per-call handle to talk to the host runtime
/// ```
use jni::EnvUnowned;
/// What:
///      `use jni::objects::JClass;` imports the borrowed handle to the Java/Kotlin
///           class object that invoked the native method.
/// Why:
///       Every JNI entry receives one as its second parameter;
///  none of these entries
///           uses it,
///  so each names it `_class`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type JClass = OpaqueHandle; // the calling class object
/// ```
use jni::objects::JClass;
/// What:
///      `use jni::sys::{jboolean, jdouble, jfloat, jint, jlong};` imports the JVM's
///           fixed-width primitive types as Rust aliases.
///  `jlong` is a 64-bit signed integer
///           (wide enough to hold a pointer,
///  so it carries the engine handle),
///  `jint` a 32-bit
///           signed integer,
///  `jdouble` a 64-bit float,
///  `jfloat` a 32-bit float,
///  and `jboolean` an
///           alias of Rust `bool` since jni 0.22.
///  Siblings a Rust reader might expect are
///           `u32`/`i64`/`f64`/`f32`;
///  the `j*` aliases are used because the signatures must
///           match exactly what the JVM passes.
/// Why:
///       The return codes and the `save position`/`duration` read-backs have to
///           marshal through these exact types.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type jlong = number;    // 64-bit signed
/// type jint = number;     // 32-bit signed
/// type jdouble = number;  // 64-bit float
/// type jfloat = number;   // 32-bit float
/// type jboolean = boolean; // a Rust bool since jni 0.22
/// ```
use jni::sys::{
    jboolean,
    jdouble,
    jfloat,
    jint,
    jlong,
};
/// What:
///      `use std::os::fd::RawFd;` imports the Unix raw-file-descriptor type,
///  a
///           plain alias for `i32` (a 32-bit signed integer;
///  the OS reserves `-1` for
///           "no fd",
///  which is why it is signed and not the sibling `u32`).
/// Why:
///       `nativeEngineLoad` casts the JVM's `jint` to `RawFd` before handing it to
///           the engine,
///  which speaks in `RawFd`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RawFd = number; // a bare OS file-descriptor integer
/// ```
use std::os::fd::RawFd;
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
///  No extra params;
///  returns `jlong` (a 64-bit signed
///           int) that secretly holds a raw pointer to a heap `Engine`,
///  used as an
///           opaque handle Kotlin passes back in later calls.
///  We use `jlong` (not a
///           narrower `jint`) because a pointer needs 64 bits on a 64-bit device.
/// Why:
///       Kotlin calls this once to create the engine and stash the handle (or 0 if
///           the worker thread could not spawn).
///  The handle must be released exactly
///           once with `nativeEngineRelease` and only used from the one Kotlin thread
///           that owns it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nativeEngineCreate(_env: EnvUnowned, _class: JClass): number { ... }
/// ```
pub extern "system" fn Java_dev_monochromatic_musicplayer_NativeBridge_nativeEngineCreate<
    'local,
>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
) -> jlong {
    // What:     `match engine::Engine::new() { ... }`. `Engine::new()` is the
    //           associated constructor; it returns `Result<Engine, std::io::Error>`
    //           (it can fail only if the OS refuses to spawn the worker thread). The
    //           `match` is the tail expression, so its value is returned.
    // Why:      Build the engine and either box it into a handle or report failure (0).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try {
    //   const engine = new Engine();
    //   return boxIntoRaw(engine); // stash the object, return an opaque handle id
    // } catch {
    //   return 0;
    // }
    // ```
    // Install the logcat subscriber once (idempotent) before any native work logs.
    logging::init();
    match engine::Engine::new() {
        // What:     `Ok(engine_value) => Box::into_raw(Box::new(engine_value)) as jlong`.
        //           `Box::new(engine_value)` MOVES the engine onto the heap and returns
        //           an owning `Box<Engine>` (sibling `Rc`/`Arc` would be shared
        //           ownership, but exactly one owner, Kotlin, holds this handle).
        //           `Box::into_raw(...)` CONSUMES that box and returns a raw `*mut
        //           Engine` pointer, deliberately LEAKING ownership so Rust will not
        //           free it at end of scope (Kotlin frees it later via Release).
        //           `as jlong` casts the raw pointer to a 64-bit int to hand across the
        //           JVM boundary.
        // Why:      Turn the engine into a stable numeric handle Kotlin can hold and
        //           pass back, surviving past this function's stack frame.
        // Gotcha:   `Box::into_raw` INTENTIONALLY leaks: the heap `Engine` is now
        //           nobody's responsibility until `nativeEngineRelease` reclaims it.
        //           Forgetting to release it is a memory leak; releasing twice is a
        //           use-after-free.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // return boxIntoRaw(engineValue); // leak on purpose; Kotlin frees later
        // ```
        Ok(engine_value) => {
            let handle = Box::into_raw(Box::new(engine_value)) as jlong;
            tracing::info!(handle, "engine created");
            return handle;
        }
        // What:     `Err(_) => 0`. Failure variant, error discarded; yield the handle
        //           value `0`, which the contract treats as "no engine".
        // Why:      The worker thread could not spawn; tell Kotlin construction failed.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // catch (error) { logger.error(error); return 0; }
        // ```
        Err(error) => {
            tracing::error!(cause = %error, "could not create engine (worker spawn failed)");
            return 0;
        }
    }
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
///      JNI entry declaration with THREE extra params:
///  `handle: jlong` (the
///           opaque engine handle from create),
///  `fd: jint` (the file descriptor),
///  and
///           `play: jboolean` (a Rust `bool`,
///  whether to start playing).
///           Returns `jint` (0 ok,
///  or a negative error code).
/// Why:
///       Kotlin calls this to hand a borrowed `content://` fd (a
///           `ParcelFileDescriptor.getFd()`,
///  duplicated synchronously) to the engine
///           and optionally play it.
///  Returns 0 on success,
///  -1 bad fd,
///  -2 dup/dispatch
///           failed,
///  -3 null handle.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nativeEngineLoad(_env, _class, handle: number, fd: number, play: number): number { ... }
/// ```
pub extern "system" fn Java_dev_monochromatic_musicplayer_NativeBridge_nativeEngineLoad<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
    fd: jint,
    play: jboolean,
) -> jint {
    // What:     `if handle == 0 { return -3; }`. Guard: a `0` handle means "no engine"
    //           (the create sentinel). Return the "-3 null handle" code.
    // Why:      We must not turn `0` into a pointer and dereference it; reject it first.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (handle === 0) return -3;
    // ```
    if handle == 0 {
        return -3;
    }
    // What:     `if fd < 0 { return -1; }`. Guard: reject a negative file descriptor
    //           with the "-1 bad fd" code.
    // Why:      Same panic-avoidance as the benchmarks: a negative fd would panic the
    //           borrow/dup path and abort across the boundary.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (fd < 0) return -1;
    // ```
    if fd < 0 {
        return -1;
    }
    // What:     `let engine_ref = unsafe { &mut *(handle as *mut engine::Engine) };`.
    //           This rebuilds a usable reference from the opaque handle. Reading it
    //           inside-out: `handle as *mut engine::Engine` casts the 64-bit int back
    //           to a raw MUTABLE pointer to an `Engine`. `*(...)` DEREFERENCES that raw
    //           pointer (follows it to the actual `Engine` in heap memory). `&mut ...`
    //           takes a MUTABLE BORROW of that `Engine` (we will call `&mut self`
    //           methods). The whole thing sits in an `unsafe { ... }` block: a region
    //           where the compiler trusts US to guarantee the pointer is valid, since
    //           it cannot prove it (raw-pointer deref is unchecked).
    // Why:      The engine lives behind a numeric handle; to call its methods we must
    //           turn that number back into a borrow of the real `Engine`.
    // Gotcha:   `unsafe` here is a PROMISE, not a bypass: if `handle` is stale or
    //           released, this is undefined behaviour. The safety contract (one valid
    //           handle, used from one thread) is enforced by Kotlin, not the compiler.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const engineRef = handleTable.get(handle); // assume valid; no compiler check
    // ```
    let engine_ref = unsafe { &mut *(handle as *mut engine::Engine) };
    // What:     `match engine_ref.load(fd as RawFd, play) { ... }`. We call the
    //           engine's `load` method. `fd as RawFd` casts the JVM int to the fd
    //           alias. `play` is already a Rust `bool` (jni 0.22 makes `jboolean` an alias
    //           of `bool`). `load` returns `Result<(), PlayerError>` (success carries the empty
    //           tuple `()` = "nothing"). The
    //           `match` is the tail expression, so its value is returned.
    // Why:      Hand the fd and play-intent to the engine and translate its result into
    //           the integer code contract.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { engineRef.load(fd, play); return 0; } catch { return -2; }
    // ```
    match engine_ref.load(fd as RawFd, play) {
        // What:     `Ok(()) => 0`. Success arm. `Ok(())` matches the success variant
        //           whose inner value is the empty tuple `()` (i.e. "succeeded, no
        //           payload"). The arm yields `0`, the success code.
        // Why:      Load dispatched successfully; report 0.
        // Gotcha:   `()` is the unit value ("nothing"), not a typo; it is what a
        //           `Result<(), _>` carries on success.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // // success: return 0;
        // ```
        Ok(()) => return 0,
        // What:     `Err(_) => -2`. Failure variant, error discarded; yield `-2`, the
        //           "dup/dispatch failed" code.
        // Why:      The dup or the send-to-worker failed; report -2.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // catch (error) { logger.warn(error); return -2; }
        // ```
        Err(error) => {
            tracing::warn!(cause = %error, "engine load failed");
            return -2;
        }
    }
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
///      JNI entry declaration taking the engine `handle: jlong` and returning
///           NOTHING (no `-> ...`,
///  so the return type is `()`,
///  Rust's "unit"/void).
/// Why:
///       Kotlin calls this to resume playback of the loaded track;
///  it is
///           fire-and-forget,
///  no result.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nativeEnginePlay(_env: EnvUnowned, _class: JClass, handle: number): void { ... }
/// ```
pub extern "system" fn Java_dev_monochromatic_musicplayer_NativeBridge_nativeEnginePlay<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
) {
    // What:     `if handle == 0 { return; }`. Guard: a `0` handle is "no engine"; the
    //           bare `return;` exits this void function early without doing anything.
    // Why:      Avoid dereferencing a null handle; silently no-op when there is no
    //           engine.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (handle === 0) return;
    // ```
    if handle == 0 {
        return;
    }
    // What:     `let engine_ref = unsafe { &mut *(handle as *mut engine::Engine) };`.
    //           Same handle-to-reference reconstruction as in `load`: cast the int to a
    //           raw `*mut Engine`, dereference it with `*`, take a mutable borrow with
    //           `&mut`, all inside an `unsafe` block where we vouch for validity.
    // Why:      We need a usable engine reference to call `play()` on it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const engineRef = handleTable.get(handle);
    // ```
    let engine_ref = unsafe { &mut *(handle as *mut engine::Engine) };
    // What:     `engine_ref.play();` calls the engine's `play` method on the borrowed
    //           reference. This is a plain method call with a trailing `;`, so it is a
    //           statement (no return value used).
    // Why:      Tell the engine to resume sounding the loaded track.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // engineRef.play();
    // ```
    engine_ref.play();
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
///      JNI entry declaration taking the engine `handle: jlong` and returning
///           nothing (unit/void).
/// Why:
///       Kotlin calls this to pause playback (keeping the loaded track and
///           buffered audio);
///  fire-and-forget.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nativeEnginePause(_env: EnvUnowned, _class: JClass, handle: number): void { ... }
/// ```
pub extern "system" fn Java_dev_monochromatic_musicplayer_NativeBridge_nativeEnginePause<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
) {
    // What:     `if handle == 0 { return; }`. Guard: no-op early on a null handle.
    // Why:      Avoid dereferencing a null handle.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (handle === 0) return;
    // ```
    if handle == 0 {
        return;
    }
    // What:     `let engine_ref = unsafe { &mut *(handle as *mut engine::Engine) };`.
    //           Same handle-to-mutable-reference reconstruction inside `unsafe`.
    // Why:      We need a usable engine reference to call `pause()`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const engineRef = handleTable.get(handle);
    // ```
    let engine_ref = unsafe { &mut *(handle as *mut engine::Engine) };
    // What:     `engine_ref.pause();` calls the engine's `pause` method (statement,
    //           trailing `;`).
    // Why:      Tell the engine to stop sounding while keeping the loaded track and
    //           buffered audio.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // engineRef.pause();
    // ```
    engine_ref.pause();
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
///      JNI entry declaration taking the engine `handle: jlong` and a target
///           `position_sec: jdouble` (64-bit float seconds);
///  returns nothing (void).
/// Why:
///       Kotlin calls this to seek;
///  fire-and-forget.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nativeEngineSeek(_env, _class, handle: number, positionSec: number): void { ... }
/// ```
pub extern "system" fn Java_dev_monochromatic_musicplayer_NativeBridge_nativeEngineSeek<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
    position_sec: jdouble,
) {
    // What:     `if handle == 0 { return; }`. Guard: no-op early on a null handle.
    // Why:      Avoid dereferencing a null handle.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (handle === 0) return;
    // ```
    if handle == 0 {
        return;
    }
    // What:     `let engine_ref = unsafe { &mut *(handle as *mut engine::Engine) };`.
    //           Same handle-to-mutable-reference reconstruction inside `unsafe`.
    // Why:      We need a usable engine reference to call `seek_to(...)`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const engineRef = handleTable.get(handle);
    // ```
    let engine_ref = unsafe { &mut *(handle as *mut engine::Engine) };
    // What:     `engine_ref.seek_to(position_sec);` calls the engine's `seek_to`
    //           method with the requested position (statement, trailing `;`).
    // Why:      Tell the engine to jump to `position_sec` in the loaded track.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // engineRef.seekTo(positionSec);
    // ```
    engine_ref.seek_to(position_sec);
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
///      JNI entry declaration taking the engine `handle: jlong` and a `volume:
///           jfloat` (32-bit float,
///  linear gain 0.0..1.0);
///  returns nothing (void).
/// Why:
///       Kotlin calls this to set user volume;
///  fire-and-forget.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nativeEngineSetVolume(_env, _class, handle: number, volume: number): void { ... }
/// ```
pub extern "system" fn Java_dev_monochromatic_musicplayer_NativeBridge_nativeEngineSetVolume<
    'local,
>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
    volume: jfloat,
) {
    // What:     `if handle == 0 { return; }`. Guard: no-op early on a null handle.
    // Why:      Avoid dereferencing a null handle.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (handle === 0) return;
    // ```
    if handle == 0 {
        return;
    }
    // What:     `let engine_ref = unsafe { &mut *(handle as *mut engine::Engine) };`.
    //           Same handle-to-mutable-reference reconstruction inside `unsafe`.
    // Why:      We need a usable engine reference to call `set_volume(...)`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const engineRef = handleTable.get(handle);
    // ```
    let engine_ref = unsafe { &mut *(handle as *mut engine::Engine) };
    // What:     `engine_ref.set_volume(volume);` calls the engine's `set_volume`
    //           method (statement, trailing `;`).
    // Why:      Apply the user's requested linear volume gain.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // engineRef.setVolume(volume);
    // ```
    engine_ref.set_volume(volume);
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
///      JNI entry declaration taking the engine `handle: jlong` and a `gain:
///           jfloat` (32-bit float,
///  linear normalization gain 0.0..1.0);
///  returns
///           nothing (void).
/// Why:
///       Kotlin calls this to set the per-track loudness-normalization gain;
///           fire-and-forget.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nativeEngineSetNormalizationGain(_env, _class, handle: number, gain: number): void { ... }
/// ```
pub extern "system" fn Java_dev_monochromatic_musicplayer_NativeBridge_nativeEngineSetNormalizationGain<
    'local,
>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
    gain: jfloat,
) {
    // What:     `if handle == 0 { return; }`. Guard: no-op early on a null handle.
    // Why:      Avoid dereferencing a null handle.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (handle === 0) return;
    // ```
    if handle == 0 {
        return;
    }
    // What:     `let engine_ref = unsafe { &mut *(handle as *mut engine::Engine) };`.
    //           Same handle-to-mutable-reference reconstruction inside `unsafe`.
    // Why:      We need a usable engine reference to call `set_normalization_gain(...)`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const engineRef = handleTable.get(handle);
    // ```
    let engine_ref = unsafe { &mut *(handle as *mut engine::Engine) };
    // What:     `engine_ref.set_normalization_gain(gain);` calls the engine's
    //           `set_normalization_gain` method (statement, trailing `;`).
    // Why:      Apply the per-track normalization gain, combined with the user volume.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // engineRef.setNormalizationGain(gain);
    // ```
    engine_ref.set_normalization_gain(gain);
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
///      JNI entry declaration taking the engine `handle: jlong` and returning a
///           `jdouble` (64-bit float,
///  current position in seconds).
/// Why:
///       Kotlin polls this to show the current playback position.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nativeEnginePositionSec(_env: EnvUnowned, _class: JClass, handle: number): number { ... }
/// ```
pub extern "system" fn Java_dev_monochromatic_musicplayer_NativeBridge_nativeEnginePositionSec<
    'local,
>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jdouble {
    // What:     `if handle == 0 { return 0.0; }`. Guard: with no engine, report
    //           position `0.0`.
    // Why:      Avoid dereferencing a null handle, and 0.0 is the sensible "nothing
    //           loaded" position.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (handle === 0) return 0.0;
    // ```
    if handle == 0 {
        return 0.0;
    }
    // What:     `let engine_ref = unsafe { &mut *(handle as *mut engine::Engine) };`.
    //           Same handle-to-mutable-reference reconstruction inside `unsafe`.
    // Why:      We need a usable engine reference to read `position_sec()`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const engineRef = handleTable.get(handle);
    // ```
    let engine_ref = unsafe { &mut *(handle as *mut engine::Engine) };
    // What:     `engine_ref.position_sec()` calls the engine's `position_sec` reader.
    //           No trailing `;`, so it is the tail expression and its `f64` value is
    //           returned.
    // Why:      Hand the current playback position back to Kotlin.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return engineRef.positionSec();
    // ```
    return engine_ref.position_sec();
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
///      JNI entry declaration taking the engine `handle: jlong` and returning a
///           `jdouble` (64-bit float,
///  track duration in seconds).
/// Why:
///       Kotlin reads this to size the seek bar / show total length.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nativeEngineDurationSec(_env: EnvUnowned, _class: JClass, handle: number): number { ... }
/// ```
pub extern "system" fn Java_dev_monochromatic_musicplayer_NativeBridge_nativeEngineDurationSec<
    'local,
>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jdouble {
    // What:     `if handle == 0 { return 0.0; }`. Guard: with no engine, report
    //           duration `0.0`.
    // Why:      Avoid dereferencing a null handle; 0.0 is the "unknown" duration.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (handle === 0) return 0.0;
    // ```
    if handle == 0 {
        return 0.0;
    }
    // What:     `let engine_ref = unsafe { &mut *(handle as *mut engine::Engine) };`.
    //           Same handle-to-mutable-reference reconstruction inside `unsafe`.
    // Why:      We need a usable engine reference to read `duration_sec()`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const engineRef = handleTable.get(handle);
    // ```
    let engine_ref = unsafe { &mut *(handle as *mut engine::Engine) };
    // What:     `engine_ref.duration_sec()` calls the engine's `duration_sec` reader.
    //           Tail expression (no `;`), so its `f64` value is returned.
    // Why:      Hand the loaded track's duration back to Kotlin.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return engineRef.durationSec();
    // ```
    return engine_ref.duration_sec();
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
///      JNI entry declaration taking the engine `handle: jlong` and returning a
///           `jboolean` (a Rust `bool` since jni 0.22;
///  the JVM sees one byte,
/// 0 or 1).
/// Why:
///       Kotlin reads this to know if audio is actually coming out right now.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nativeEngineIsPlaying(_env: EnvUnowned, _class: JClass, handle: number): boolean { ... }
/// ```
pub extern "system" fn Java_dev_monochromatic_musicplayer_NativeBridge_nativeEngineIsPlaying<
    'local,
>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jboolean {
    // What:     `if handle == 0 { return false; }`. Guard: with no
    //           engine, report "not playing" by returning `false`.
    // Why:      `jboolean` is a Rust `bool` alias since jni 0.22, so the value is
    //           returned directly with no conversion.
    // Gotcha:   Before jni 0.22 `jboolean` was a byte and needed `jboolean::from(...)`;
    //           a no-op `from` is now a clippy `useless_conversion` finding.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (handle === 0) return false;
    // ```
    if handle == 0 {
        return false;
    }
    // What:     `let engine_ref = unsafe { &mut *(handle as *mut engine::Engine) };`.
    //           Same handle-to-mutable-reference reconstruction inside `unsafe`.
    // Why:      We need a usable engine reference to read `is_playing()`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const engineRef = handleTable.get(handle);
    // ```
    let engine_ref = unsafe { &mut *(handle as *mut engine::Engine) };
    // What:     `engine_ref.is_playing()` returns a Rust `bool`, which is the
    //           `jboolean` type. Tail expression (no `;`), so the value is returned.
    // Why:      Report the engine's sounding state.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return engineRef.isPlaying();
    // ```
    return engine_ref.is_playing();
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
///      JNI entry declaration taking the engine `handle: jlong` and returning a
///           `jboolean` (a Rust `bool` since jni 0.22).
/// Why:
///       Kotlin reads this to know when to advance to the next track.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nativeEngineIsEnded(_env: EnvUnowned, _class: JClass, handle: number): boolean { ... }
/// ```
pub extern "system" fn Java_dev_monochromatic_musicplayer_NativeBridge_nativeEngineIsEnded<
    'local,
>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jboolean {
    // What:     `if handle == 0 { return false; }`. Guard: with no
    //           engine, report "not ended" by returning `false`.
    // Why:      `jboolean` is a Rust `bool` alias, so no conversion is needed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (handle === 0) return false;
    // ```
    if handle == 0 {
        return false;
    }
    // What:     `let engine_ref = unsafe { &mut *(handle as *mut engine::Engine) };`.
    //           Same handle-to-mutable-reference reconstruction inside `unsafe`.
    // Why:      We need a usable engine reference to read `is_ended()`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const engineRef = handleTable.get(handle);
    // ```
    let engine_ref = unsafe { &mut *(handle as *mut engine::Engine) };
    // What:     `engine_ref.is_ended()`. Read the Rust `bool` from
    //           `is_ended()`. Tail expression, so it is returned.
    // Why:      Report end-of-track.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return engineRef.isEnded();
    // ```
    return engine_ref.is_ended();
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
///      JNI entry declaration taking the engine `handle: jlong` and returning a `jlong`:
///           the sequence number of the most recent queued load the worker could not open for playback,
///           or 0 when none failed.
/// Why:
///       `nativeEngineLoad` returns once the load is queued,
///  so a file that opens but does not decode fails later;
///  Kotlin
///           polls this to report that failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nativeEngineFailedLoad(_env: EnvUnowned, _class: JClass, handle: number): number { ... }
/// ```
pub extern "system" fn Java_dev_monochromatic_musicplayer_NativeBridge_nativeEngineFailedLoad<
    'local,
>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jlong {
    // What:     `if handle == 0 { return 0; }`. Guard: with no engine, report that no load failed.
    // Why:      A released engine has no worker to ask.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (handle === 0) return 0;
    // ```
    if handle == 0 {
        return 0;
    }
    // What:     `let engine_ref = unsafe { &mut *(handle as *mut engine::Engine) };`.
    //           Same handle-to-mutable-reference reconstruction inside `unsafe`.
    // Why:      We need a usable engine reference to read `failed_load()`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const engineRef = handleTable.get(handle);
    // ```
    let engine_ref = unsafe { &mut *(handle as *mut engine::Engine) };
    // What:     `engine_ref.failed_load() as jlong`. The `u64` sequence number cast to the signed `jlong`;
    //           it counts loads, so it never nears the sign bit.
    // Why:      Report the worker's last load failure.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return engineRef.failedLoad();
    // ```
    return engine_ref.failed_load() as jlong;
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
///      JNI entry declaration taking the engine `handle: jlong` and returning a
///           `jboolean` (a Rust `bool` since jni 0.22).
/// Why:
///       Kotlin reads this "playWhenReady" intent (true from a play/load-and-play
///           request until a pause),
///  distinct from actual sounding.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nativeEnginePlayWhenReady(_env: EnvUnowned, _class: JClass, handle: number): boolean { ... }
/// ```
pub extern "system" fn Java_dev_monochromatic_musicplayer_NativeBridge_nativeEnginePlayWhenReady<
    'local,
>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
) -> jboolean {
    // What:     `if handle == 0 { return false; }`. Guard: with no
    //           engine, report "no play intent" by returning `false`.
    // Why:      `jboolean` is a Rust `bool` alias, so no conversion is needed.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (handle === 0) return false;
    // ```
    if handle == 0 {
        return false;
    }
    // What:     `let engine_ref = unsafe { &mut *(handle as *mut engine::Engine) };`.
    //           Same handle-to-mutable-reference reconstruction inside `unsafe`.
    // Why:      We need a usable engine reference to read `play_when_ready()`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const engineRef = handleTable.get(handle);
    // ```
    let engine_ref = unsafe { &mut *(handle as *mut engine::Engine) };
    // What:     `engine_ref.play_when_ready()`. Read the Rust `bool`
    //           from `play_when_ready()`. Tail expression, so it is returned.
    // Why:      Report the play-intent flag.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return engineRef.playWhenReady();
    // ```
    return engine_ref.play_when_ready();
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
///      JNI entry declaration taking the engine `handle: jlong` and returning
///           nothing (unit/void).
/// Why:
///       Kotlin calls this once to tear down the engine (stop the worker,
///  close
///           the AAudio stream,
///  free the handle) and reclaim the leaked box;
///  the
///           handle must not be used afterwards.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nativeEngineRelease(_env: EnvUnowned, _class: JClass, handle: number): void { ... }
/// ```
pub extern "system" fn Java_dev_monochromatic_musicplayer_NativeBridge_nativeEngineRelease<
    'local,
>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
) {
    // What:     `if handle == 0 { return; }`. Guard: a `0` handle is "no engine"; the
    //           bare `return;` exits early. This also makes release safe to call when
    //           create returned 0.
    // Why:      Never try to reclaim a null handle (that would be undefined behaviour).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (handle === 0) return;
    // ```
    if handle == 0 {
        return;
    }
    // SAFETY: `handle` came from a single `nativeEngineCreate` Box::into_raw and is
    // released exactly once; reclaiming the Box drops the Engine (joining the worker).
    // What:     `unsafe { drop(Box::from_raw(handle as *mut engine::Engine)); }`.
    //           Reading inside-out: `handle as *mut engine::Engine` casts the int back
    //           to a raw mutable pointer. `Box::from_raw(...)` REBUILDS an owning
    //           `Box<Engine>` FROM that raw pointer, taking ownership back (the exact
    //           inverse of the `Box::into_raw` in create). `drop(...)` immediately
    //           runs that box's destructor, freeing the heap `Engine` (which stops the
    //           worker thread and closes the AAudio stream). All inside `unsafe`
    //           because rebuilding a box from a raw pointer is unchecked.
    // Why:      Reclaim and free the engine we deliberately leaked at create time;
    //           dropping it is what actually tears the engine down.
    // Gotcha:   `Box::from_raw` must be called EXACTLY ONCE per `into_raw`. Calling it
    //           twice is a double-free (use-after-free); never calling it leaks the
    //           engine. `drop(x)` is just an explicit "destroy now"; it is not the same
    //           as TS `delete obj.prop`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // handleTable.delete(handle); // let the GC reclaim the engine
    // ```
    unsafe {
        drop(Box::from_raw(handle as *mut engine::Engine));
    }
}
