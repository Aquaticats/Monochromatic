//! JNI entries that read one track's cached true peak and clear it for a re-analysis.
//!
//! Both entries use the process-wide service handle that `nativeResolveGain` and
//!  `nativeWarmTrack` use,
//! so they read and clear the same cache the sweep writes.
//!  Neither starts a measurement:
//! the peek only reads,
//! and the invalidate only removes.

/// The `Result` a body returns when a call into the host runtime fails.
use jni::errors::Result as JniResult;
/// The class and float-array references the entry points receive from the host runtime.
use jni::objects::{JClass, JFloatArray};
/// The raw boolean,
/// float-array and long types of the entry points' signatures.
use jni::sys::{jboolean, jfloatArray, jlong};
/// The per-call handle and the `Env` it lends.
use jni::{Env, EnvUnowned};
/// The peak kind a cached decision reports,
/// and the peak measurement a miss falls back to.
use truepeak_core::{PeakKind, peak_dbtp};

/// Runs a body with the call's `Env`,
/// catching a panic and logging it.
use crate::jni_call::{JniBody, run_with_env};
/// The handle to the decision cache service the entry points read and clear.
use crate::service::service_ref;

/// What:
///      `const PEAK_READING_LEN: usize = 2;`.
///  The slots of a peek reply:
/// the peak in dBTP,
/// then the kind code.
/// Why:
///       A fixed two-slot array keeps the Kotlin side to one call that returns both values together.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const PEAK_READING_LEN = 2;
/// ```
const PEAK_READING_LEN: usize = 2;

/// What:
///      `const KIND_CODE_EXACT: f32 = 1.0;`.
///  The kind code for an exact peak,
/// in the second slot.
/// Why:
///       Kotlin reads the kind as a float,
/// so the code is a float too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const KIND_CODE_EXACT = 1.0;
/// ```
const KIND_CODE_EXACT: f32 = 1.0;

/// What:
///      `const KIND_CODE_SAMPLED: f32 = 0.0;`.
///  The kind code for a sampled peak,
/// in the second slot.
/// Why:
///       A probe estimate is a lower bound,
/// and the reader must see that.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const KIND_CODE_SAMPLED = 0.0;
/// ```
const KIND_CODE_SAMPLED: f32 = 0.0;

/// What:
///      `fn kind_code(kind: PeakKind) -> f32`.
///  Map a peak kind to its code in the reply.
/// Why:
///       Keeps the two codes next to each other in one match.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function kindCode(kind: PeakKind): number { return kind === "exact" ? 1.0 : 0.0; }
/// ```
fn kind_code(kind: PeakKind) -> f32 {
    match kind {
        PeakKind::Exact => return KIND_CODE_EXACT,
        PeakKind::Sampled => return KIND_CODE_SAMPLED,
    }
}

/// What:
///      `struct PeakReplyBody<'a> { slots }`.
///  A body that builds the reply array from the two reply slots.
///  `slots` is a borrowed fixed-size array
///  (`&[f32; PEAK_READING_LEN]`;
///  sibling `&[f32]` would not fix the length).
/// Why:
///       `run_with_env` takes its work as a value;
///  this one carries the slots that the reply copies into the Java array.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PeakReplyBody { constructor(readonly slots: number[]) {} }
/// ```
struct PeakReplyBody<'a> {
    /// The `[peakDbtp, kindCode]` pair the reply carries.
    slots: &'a [f32; PEAK_READING_LEN],
}

/// What:
///      `impl JniBody<'local, JFloatArray<'local>> for PeakReplyBody`.
///  Makes the reply builder a body that `run_with_env` can run.
/// Why:
///       `run_with_env` accepts any `JniBody`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class PeakReplyBody implements JniBody<Float32Array | null> { ... }
/// ```
impl<'local> JniBody<'local, JFloatArray<'local>> for PeakReplyBody<'_> {
    /// What:
    ///      `fn run(self, env) -> JniResult<JFloatArray<'local>>`.
    ///  Build the reply array;
    ///  `peak_array` maps its own JNI failures to a null array,
    ///  so this never returns an error.
    /// Why:
    ///       A failed allocation must not throw across JNI.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// run(env: Env): Float32Array | null { return peakArray(env, this.slots); }
    /// ```
    fn run(self, env: &mut Env<'local>) -> JniResult<JFloatArray<'local>> {
        return Ok(peak_array(env, self.slots))
    }
}

/// What:
///      `fn peak_array<'local>(env, slots) -> JFloatArray<'local>`.
///  Copy the reply slots into a new Java `float[]`,
///  or return a null array when the copy cannot be made.
/// Why:
///       A failed allocation must not throw across JNI;
/// Kotlin reads null as "no peak".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function peakArray(slots: number[]): Float32Array | null { ... }
/// ```
fn peak_array<'local>(env: &mut Env<'local>, slots: &[f32; PEAK_READING_LEN]) -> JFloatArray<'local> {
    let array = match JFloatArray::new(env, PEAK_READING_LEN) {
        Ok(array) => array,
        Err(error) => {
            tracing::warn!(cause = %error, "peek peak: could not allocate the reply array");
            return JFloatArray::default()
        }
    };
    if let Err(error) = array.set_region(env, 0, slots) {
        tracing::warn!(cause = %error, "peek peak: could not fill the reply array");
        return JFloatArray::default();
    }
    return array
}

/// What:
///      `#[unsafe(no_mangle)] pub extern "system" fn Java_..._nativePeekPeak(...) -> jfloatArray`.
///  Read the cached peak of one track fingerprint without measuring it.
///  The reply is `[peakDbtp, kindCode]`,
/// or null when no decision is cached or the handle is null.
/// Why:
///       The track rows and the details menu show the peak only when it is already known,
///  so the read must never start a decode.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// declare function nativePeekPeak(handle: bigint, fingerprint: bigint): Float32Array | null;
/// ```
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_monochromatic_musicplayer_NativeBridge_nativePeekPeak<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
    fingerprint: jlong,
) -> jfloatArray {
    tracing::debug!(fingerprint, "peek peak");
    let Some(service) = service_ref(handle) else {
        tracing::warn!("peek peak: null service handle; no peak");
        return JFloatArray::default().into_raw();
    };
    let Some(decision) = service.get(fingerprint as u64) else {
        tracing::debug!(fingerprint, "peek peak: no cached decision");
        return JFloatArray::default().into_raw();
    };
    let slots: [f32; PEAK_READING_LEN] = [
        peak_dbtp(f64::from(decision.measured_peak)) as f32,
        kind_code(decision.peak_kind()),
    ];
    // `peak_array` handles its own JNI failures (null array); `run_with_env` only adds the lent `Env`.
    let reply = run_with_env(&mut env, "nativePeekPeak reply", PeakReplyBody { slots: &slots }).unwrap_or_default();
    return reply.into_raw()
}

/// What:
///      `#[unsafe(no_mangle)] pub extern "system" fn Java_..._nativeInvalidateTrack(...) -> jboolean`.
///  Remove the cached decision of one track fingerprint,
/// so its next resolve measures it again.
///  Returns true when a row was removed;
/// false when none was stored or the handle is null.
/// Why:
///       A re-analysis clears one track and no other,
/// and the removal finishes before the
///  caller measures again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// declare function nativeInvalidateTrack(handle: bigint, fingerprint: bigint): boolean;
/// ```
#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_monochromatic_musicplayer_NativeBridge_nativeInvalidateTrack<'local>(
    _env: EnvUnowned<'local>,
    _class: JClass<'local>,
    handle: jlong,
    fingerprint: jlong,
) -> jboolean {
    tracing::info!(fingerprint, "invalidate track");
    let Some(service) = service_ref(handle) else {
        tracing::warn!("invalidate track: null service handle; nothing removed");
        return false;
    };
    if service.forget(fingerprint as u64) {
        return true;
    }
    return false
}
