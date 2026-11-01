//! Track cache-key fingerprint JNI entry.
//!  The hash and the hashed byte layout (a domain tag,
//!  the path length,
//!  the path
//! bytes,
//!  the size as 8 little-endian bytes,
//!  and the mtime nanoseconds as 16 little-endian bytes)
//! live in the shared `truepeak-core` crate (`track_fingerprint`,
//!  XXH3-64 with the default
//! seed),
//!  the same function the desktop crate calls,
//!  so identical inputs hash identically on both
//! flavors by construction.
//!  This file only decodes the Java arguments and calls it.
//!
//! Testing:
//!  this crate is Android-only (its `ndk`/`ndk-sys` dependency refuses to
//! compile off-Android),
//!  so there is no host `cargo test` here.
//!  The digest is pinned to the
//! xxHash reference C implementation by the host known-answer tests in
//! `truepeak-core/src/fingerprint_tests.rs`;
//!  the JNI path (argument decoding,
//!  the
//! sign-preserving `jlong` cast) is verified ON DEVICE by `NativeBridgeTest`,
//!  which asserts
//! the same known-answer value through `NativeBridge.nativeFingerprint`.
//!  The value is the raw
//! `u64` (returned as a Kotlin `Long`),
//!  which the shared `DecisionCache` keys on directly.

/// What:
///      `use jni::EnvUnowned;` imports the per-call interface pointer the JVM hands
///           every native method (jni 0.22 renamed the former `EnvUnowned` argument type;
///           the JNI calls live on the `Env` that `run_with_env` lends in a closure).
/// Why:
///       `native_fingerprint` reads its `JString` argument through this handle.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EnvUnowned = RuntimeContext; // per-call handle to talk to the host runtime
/// ```
use jni::EnvUnowned;

use crate::jni_call::run_with_env;

/// What:
///      `use jni::objects::{JClass, JString};`.
///  `JClass` is the calling Java
///           class handle (unused here);
///  `JString` is a borrowed handle to a Java
///           `String` argument (sibling:
///  `JObject`,
///  the untyped object handle).
/// Why:
///       The JNI entry's signature names both:
///  `_class` is the class handle and
///           `path` is the incoming Java string.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // both are opaque host-runtime handles
/// ```
use jni::objects::{JClass, JString};

/// What:
///      `use jni::sys::jlong;`.
///  The JNI 64-bit signed integer (maps to Kotlin `Long`;
///           sibling `jint` is 32-bit).
/// Why:
///       `size`/`mtime_nanos` arrive as `jlong`,
///  and the function RETURNS a `jlong`
///           (the raw `u64` fingerprint bit-cast),
///  not a Java string.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type jlong = bigint;
/// ```
use jni::sys::jlong;

/// What:
///      `use truepeak_core::track_fingerprint;`.
///  The shared fingerprint function
///           `track_fingerprint(path: &str, size: u64, mtime_nanos: u128) -> u64`.
/// Why:
///       The one place that owns the hashed layout and the hash,
///  shared with the desktop
///           crate,
///  so the two flavors cannot drift.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { trackFingerprint } from "truepeak-core";
/// ```
use truepeak_core::track_fingerprint;

// What:     `#[unsafe(no_mangle)]` keeps the symbol name unmangled so the JVM's
//           `System.loadLibrary` + `native` lookup finds it by the exact name below.
// Why:      Without it the linker would rename the symbol and JNI could not locate it.
//
// In TS you'd write (pseudocode):
// ```ts
// // no annotation needed
// ```
#[unsafe(no_mangle)]
/// What:
///      `pub extern "system" fn Java_dev_monochromatic_musicplayer_NativeBridge_nativeFingerprint<'local>(...)`.
///           The JNI entry.
///  The name encodes `Java_` + package + class (`NativeBridge`)
///           + method (`nativeFingerprint`),
///  underscore-joined.
///  Params:
///  `env` (the JNI
///           gateway),
///  `_class` (the calling class,
///  unused),
///  `path: JString<'local>`
///           (the borrowed Java string),
///  `size`/`mtime_nanos` (`jlong`,
///  i.e. Kotlin
///           `Long`).
///  `-> jlong` returns the raw `u64` fingerprint bit-cast to `i64`.
/// Why:
///       Kotlin's `NativeBridge.nativeFingerprint(path, size, mtimeNanos)` calls
///           this;
///  it returns the cache key that `nativeResolveGain`/`nativeWarmTrack`
///           take.
///  Returning the `u64` as a `Long` avoids a hex-string round-trip.
/// Gotcha:
///    `extern "system"` means a panic must NEVER cross this boundary;
///  the body
///           only does infallible work plus one `match` that returns `0` on the
///           (practically unreachable) JNI string-read failure.
///  A `0` key is a benign
///           collision risk only when the read fails,
///  which never happens for a real
///           Kotlin string.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nativeFingerprint(env, _class, path: JString, size: bigint, mtimeNanos: bigint): jlong { ... }
/// ```
pub extern "system" fn Java_dev_monochromatic_musicplayer_NativeBridge_nativeFingerprint<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    path: JString<'local>,
    size: jlong,
    mtime_nanos: jlong,
) -> jlong {
    // What:     `let Some(path_str) = run_with_env(&mut env, ..., |jni_env| ...) else { return 0 };` with the
    //           closure body `return path.try_to_string(jni_env)`.
    //           `path.try_to_string(env)` borrows the `JString` and returns an owned
    //           `String`; `run_with_env` logs a failure and yields `None`, so the `else`
    //           arm bails with the `0` sentinel.
    // Why:      Turn the opaque Java string into an owned Rust `String` to hash; a valid
    //           Kotlin `String` never fails here, so the sentinel path is unreachable in
    //           practice but keeps the boundary panic-free.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let pathStr: string;
    // try { pathStr = env.getString(path); } catch { return 0n; }
    // ```
    let Some(path_str) = run_with_env(&mut env, "nativeFingerprint path", |jni_env| {
        return path.try_to_string(jni_env);
    }) else {
        return 0;
    };
    // What:     `let fingerprint = track_fingerprint(&path_str, size as u64, mtime_nanos as u64 as u128);`.
    //           `&path_str` lends the string read-only. `size as u64` reinterprets the
    //           `jlong` (i64) as unsigned (sizes are never negative). `mtime_nanos as u64 as
    //           u128` reinterprets the i64 as u64 then widens to u128, matching desktop's
    //           16-byte mtime field.
    // Why:      Build the cache key from the three inputs through the shared function the desktop
    //           also calls.
    // Gotcha:   `as` casts here are bit-reinterpretations, not range checks; valid
    //           sizes/timestamps are non-negative so the unsigned view is the intended value.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const fingerprint = trackFingerprint(pathStr, BigInt.asUintN(64, size), BigInt.asUintN(64, mtimeNanos));
    // ```
    let fingerprint = track_fingerprint(&path_str, size as u64, mtime_nanos as u64 as u128);
    // What:     `fingerprint as jlong`. Bit-cast the `u64` to the `i64`/`jlong` the JVM
    //           carries (bijective; the native cache reverses it). No `;`, so this is the
    //           tail/return value.
    // Why:      Hand the cache key back to Kotlin as a `Long`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return BigInt.asIntN(64, fingerprint);
    // ```
    return fingerprint as jlong
}
