//! The JNI entries of the true-peak service,
//! and the resolve-then-cache body they share.
//!
//! Kotlin creates the service once (`nativeTruePeakServiceCreate`),
//! keeps the opaque
//! `jlong` handle,
//! asks for a normalization gain on every track load (`nativeResolveGain`),
//! upgrades estimate decisions in the background (`nativeWarmTrack`),
//! and releases the
//! handle on shutdown (`nativeTruePeakServiceRelease`).
//!  The handle type itself,
//!  its two
//! channels,
//!  the cache and its actor thread live in `service.rs`;
//!  this file is only the
//! boundary the JVM calls,
//!  kept out of `service.rs` so that file stays under the
//! per-file code-line budget (`AGENTS.md`,
//! MXL).
//!
//! Dum-dum orientation for a TypeScript reader:
//!  a cache lookup is cheap and blocking (a
//! channel round trip),
//!  while a cache MISS needs a decode,
//!  so `resolve_and_cache` is the
//! one function that decides between them:
//!  answer from the cache,
//!  else open the Android
//! file descriptor,
//!  measure,
//!  store the decision,
//!  and answer with the gain.
//!  Every
//! failure along that path returns the shared `CEILING` gain (the -1 dBTP ceiling)
//!  instead of an error,
//!  because the Kotlin side always needs a gain to apply:
//!  playing a track
//! slightly quiet beats playing nothing.
//!  Timestamps,
//!  file names and stack strings never cross this
//! boundary;
//!  it carries one number in and one number out.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // export function nativeResolveGain(handle: bigint, fd: number, fingerprint: bigint): number
//! ```

/// What:
///      `use std::os::fd::RawFd;`.
///  The Unix raw-file-descriptor alias (an `i32`,
/// not the
///           sibling `u32`,
///  because the OS reserves `-1` for "no fd").
/// Why:
///       `resolve_and_cache` hands the JVM's `jint` descriptor to the decoder,
///  which
///           speaks in `RawFd`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RawFd = number;
/// ```
use std::os::fd::RawFd;

/// Runs a body with the call's `Env`,
/// catching a panic and logging it,
/// as every entry point of this crate does.
use crate::jni_call::{
    ReadJavaString,
    run_with_env,
};
/// The service handle type,
/// and the helper that rebuilds a shared reference to it from the
///  `jlong` Kotlin passes back.
use crate::service::{
    TruePeakService,
    service_ref,
};
/// What:
///      `use crate::{decode, truepeak};`.
///  The Android decoder opener and the shared-source
///           resolvers.
/// Why:
///       A cache miss opens the fd and drives it through `resolve_current`/`resolve_full`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as decode from "./decode"; import * as truepeak from "./truepeak";
/// ```
use crate::{
    decode,
    truepeak,
};
/// What:
///      `use jni::EnvUnowned;`.
///  The per-call JVM argument (the former `JNIEnv`,
/// renamed in
///           jni 0.22).
/// Why:
///       The create entry reads its `JString` through `run_with_env`,
///  which needs it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type EnvUnowned = RuntimeContext;
/// ```
use jni::EnvUnowned;
/// What:
///      `use jni::objects::{JClass, JString};`.
///  The calling class handle and a borrowed
///           Java string argument.
/// Why:
///       The create entry reads the database path as a `JString`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // opaque host-runtime handles
/// ```
use jni::objects::{
    JClass,
    JString,
};
/// What:
///      `use jni::sys::{jfloat, jint, jlong};`.
///  The JNI 32-bit float (Kotlin `Float`),
///           32-bit int (`Int`),
///  and 64-bit int (`Long`).
/// Why:
///       Handles and fingerprints are `jlong`,
///  the fd is `jint`,
///  and gains are `jfloat`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type jlong = number; type jint = number; type jfloat = number;
/// ```
use jni::sys::{
    jfloat,
    jint,
    jlong,
};
/// What:
///      `use truepeak_core::{CEILING, DecisionKind};`.
///  The ceiling gain returned on every
///           failure,
///  and the tag that says how exact a cached decision is.
/// Why:
///       A miss that fails to decode answers with `CEILING`,
///  and warming reuses a
///           decision only when its kind is an exact one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import { CEILING, DecisionKind } from "truepeak-core";
/// ```
use truepeak_core::{
    CEILING,
    DecisionKind,
};
/// What:
///      `fn resolve_and_cache(service, fd, fingerprint, full) -> f32`.
///  The shared body of
///           both JNI resolve entries:
///  cache hit -> its gain;
///  miss -> open the fd,
///  resolve
///           (probe-or-full when `full` is false,
///  always-exact when true),
///  cache,
///  and return
///           the gain;
///  any failure -> the ceiling fallback.
/// Why:
///       One place holds the get/decode/put orchestration for foreground and warming.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function resolveAndCache(service, fd, fingerprint, full): number { ... }
/// ```
fn resolve_and_cache(
    service: &TruePeakService,
    fd: jint,
    fingerprint: u64,
    full: bool,
) -> f32 {
    // What:     `if let Some(decision) = service.get(fingerprint) { ... }`. Cache hit.
    // Why:      Warming skips an already-EXACT track; foreground reuses any decision.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const hit = service.get(fingerprint);
    // ```
    if let Some(decision) = service.get(fingerprint) {
        // What:     `if !full || matches!(decision.kind, ShortFullScan | FullScanExact) { ... }`.
        //           Foreground uses any hit; warming reuses only an exact hit and otherwise
        //           re-scans to upgrade a probe estimate.
        // Why:      Warming's job is to make probe estimates exact.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // if (!full || isExact(decision.kind)) return decision.gain;
        // ```
        if !full
            || matches!(
                decision.kind,
                DecisionKind::ShortFullScan | DecisionKind::FullScanExact
            )
        {
            return decision.gain;
        }
    }
    // What:     `if fd < 0 { return CEILING; }`. A negative fd cannot be decoded.
    // Why:      Fall back to the safe -1 dBTP ceiling gain rather than panic.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (fd < 0) return CEILING;
    // ```
    if fd < 0 {
        tracing::debug!(fd, "negative fd; using ceiling gain");
        return CEILING;
    }
    // What:     `let source = match decode::open_borrowed_fd(fd as RawFd) { ... };`. Open the
    //           decoder over the duplicated fd; fall back on failure.
    // Why:      We only decode on a miss (or a warming upgrade).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let source; try { source = decode.openBorrowedFd(fd); } catch { return CEILING; }
    // ```
    let source = match decode::open_borrowed_fd(fd as RawFd) {
        Ok(source) => source,
        Err(error) => {
            tracing::warn!(fd, cause = %error, "could not open fd for decode; using ceiling gain");
            return CEILING;
        }
    };
    // What:     `let decision = match resolve { ... };`. Drive the source through the shared
    //           resolver: `resolve_full` (always exact) for warming, else `resolve_current`
    //           (probe-or-full); fall back on a decode error.
    // Why:      Produce the fresh decision to cache and return.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let decision; try { decision = full ? resolveFull(source) : resolveCurrent(source); } catch { return CEILING; }
    // ```
    let resolved = if full {
        truepeak::resolve_full(source)
    } else {
        truepeak::resolve_current(source)
    };
    let decision = match resolved {
        Ok(decision) => decision,
        Err(error) => {
            tracing::warn!(fingerprint, cause = %error, "resolve failed; using ceiling gain");
            return CEILING;
        }
    };
    // What:     `service.put(fingerprint, decision);`. Persist it (fire-and-forget).
    // Why:      Later plays and sweeps hit the cache.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // service.put(fingerprint, decision);
    // ```
    if full {
        service.put_and_wait(fingerprint, decision);
    } else {
        service.put(fingerprint, decision);
    }
    // What:     `decision.gain`. The resolved gain. Tail -> return.
    // Why:      Hand it back to Kotlin to apply (foreground) or log (warming).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return decision.gain;
    // ```
    return decision.gain;
}

/// What:
///      `#[unsafe(no_mangle)] pub extern "system" fn Java_..._nativeTruePeakServiceCreate(...)
///           -> jlong`.
///  Open a service for the given database path and return its handle,
///  or
///           `0` on failure.
/// Why:
///       Kotlin creates one service at startup with its app-private `decisions.db` path.
/// Gotcha:
///    `extern "system"`:
///  no panic may cross this boundary;
///  string read failure returns
///           `0`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nativeTruePeakServiceCreate(env, _class, dbPath: JString): jlong { ... }
/// ```
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_monochromatic_musicplayer_NativeBridge_nativeTruePeakServiceCreate<
    'local,
>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    db_path: JString<'local>,
) -> jlong {
    // Install the logcat subscriber once (idempotent) before any native work logs.
    crate::logging::init();
    // What:     `let Some(path) = run_with_env(&mut env, ..., ReadJavaString { text: &db_path }) else { ... };`.
    //           Read the Java string; `run_with_env` logs and the `else` arm returns `0` on the
    //           (unreachable) read failure.
    // Why:      The actor needs an owned path string.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let path; try { path = env.getString(dbPath); } catch { return 0; }
    // ```
    let Some(path) = run_with_env(
        &mut env,
        "nativeTruePeakServiceCreate db path",
        ReadJavaString {
            text: &db_path,
        },
    ) else {
        tracing::warn!("could not read db path string; no service created");
        return 0;
    };
    // What:     `Box::into_raw(Box::new(TruePeakService::open(path))) as jlong`. Open the
    //           service, box it onto the heap, and leak the pointer as the handle. Tail ->
    //           return.
    // Why:      A stable numeric handle Kotlin holds until release.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return boxIntoRaw(TruePeakService.open(path));
    // ```
    let handle = Box::into_raw(Box::new(TruePeakService::open(path))) as jlong;
    // The service is live; log the handle Kotlin will hold.
    tracing::info!(handle, "truepeak service created");
    return handle;
}

/// What:
///      `#[unsafe(no_mangle)] pub extern "system" fn Java_..._nativeTruePeakServiceRelease(...)`.
///           Reclaim the boxed service,
///  closing its channels so the actor thread exits.
/// Why:
///       Kotlin releases the one service on shutdown.
/// Gotcha:
///    Releasing twice is a use-after-free;
///  Kotlin must release exactly once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nativeTruePeakServiceRelease(_env, _class, handle: jlong): void { ... }
/// ```
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_monochromatic_musicplayer_NativeBridge_nativeTruePeakServiceRelease<
    'local,
>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
) {
    // What:     `if handle == 0 { return; }`. Nothing to free for the sentinel.
    // Why:      Never turn `0` into a pointer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (handle === 0) return;
    // ```
    if handle == 0 {
        return;
    }
    // The handle is being released; log it before the actor thread stops.
    tracing::info!(handle, "releasing truepeak service");
    // What:     `unsafe { drop(Box::from_raw(handle as *mut TruePeakService)); }`. Rebuild the
    //           box and drop it, closing the senders so the actor loop breaks.
    // Why:      Reclaim the leaked heap allocation and stop the thread.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // handleTable.delete(handle); // runs the destructor
    // ```
    unsafe {
        drop(Box::from_raw(handle as *mut TruePeakService));
    }
}

/// What:
///      `#[unsafe(no_mangle)] pub extern "system" fn Java_..._nativeResolveGain(...) -> jfloat`.
///           Foreground:
///  return the normalization gain for the current track,
///  resolving and
///           caching a probe-or-full decision on a miss.
/// Why:
///       Kotlin calls this on track load,
///  then applies the gain to the engine.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nativeResolveGain(_env, _class, handle: jlong, fd: jint, fingerprint: jlong): jfloat { ... }
/// ```
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_monochromatic_musicplayer_NativeBridge_nativeResolveGain<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
    fd: jint,
    fingerprint: jlong,
) -> jfloat {
    // What:     `let Some(service) = service_ref(handle) else { return CEILING; };`. Rebuild the
    //           service, or fall back to the ceiling for a null handle.
    // Why:      A missing service must still yield a safe gain.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const service = serviceRef(handle); if (!service) return CEILING;
    // ```
    let Some(service) = service_ref(handle) else {
        tracing::warn!("resolve gain: null service handle; using ceiling gain");
        return CEILING;
    };
    // What:     `resolve_and_cache(service, fd, fingerprint as u64, false)`. Foreground resolve.
    //           Tail -> return.
    // Why:      Probe-or-full; a cache hit skips decode.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return resolveAndCache(service, fd, fingerprint, false);
    // ```
    return resolve_and_cache(service, fd, fingerprint as u64, false);
}

/// What:
///      `#[unsafe(no_mangle)] pub extern "system" fn Java_..._nativeWarmTrack(...) -> jfloat`.
///           Background warming:
///  full-scan a track to an EXACT decision and cache it,
///  skipping
///           tracks that are already exact.
/// Why:
///       Kotlin's sweep calls this per track to upgrade probe estimates over idle time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function nativeWarmTrack(_env, _class, handle: jlong, fd: jint, fingerprint: jlong): jfloat { ... }
/// ```
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_monochromatic_musicplayer_NativeBridge_nativeWarmTrack<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
    fd: jint,
    fingerprint: jlong,
) -> jfloat {
    // What:     `let Some(service) = service_ref(handle) else { return CEILING; };`. Rebuild the
    //           service, or fall back for a null handle.
    // Why:      A missing service is a no-op warm.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const service = serviceRef(handle); if (!service) return CEILING;
    // ```
    let Some(service) = service_ref(handle) else {
        tracing::warn!("warm track: null service handle; using ceiling gain");
        return CEILING;
    };
    // What:     `resolve_and_cache(service, fd, fingerprint as u64, true)`. Warming resolve
    //           (always exact, skip-if-exact). Tail -> return.
    // Why:      Upgrade a probe estimate to an exact cached gain.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return resolveAndCache(service, fd, fingerprint, true);
    // ```
    return resolve_and_cache(service, fd, fingerprint as u64, true);
}
