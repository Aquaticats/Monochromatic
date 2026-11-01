//! The one place that turns a native method's per-call JVM handle into a usable `Env`.
//!
//! jni 0.22 split the old `JNIEnv` into `EnvUnowned` (the argument the JVM passes) and `Env`
//! (the type that carries the JNI calls, lent only inside a closure). Every entry that calls
//! into the JVM (string reads, array copies) goes through `run_with_env`, so a JNI error or a
//! caught panic is logged through `tracing` (which reaches logcat) and the entry returns its
//! own sentinel, exactly as the former `match` on each `Result` did.

use jni::errors::Result as JniResult;
use jni::{Env, EnvUnowned, Outcome};

/// What:
///      `pub(crate) fn run_with_env<'local, T>(env, operation, body) -> Option<T>`.
///  Lend `body` the call's `Env`,
///  and return its value,
///  or `None` after logging when the JNI call failed or `body` panicked.
/// Why:
///       Each JNI entry maps failure to its own sentinel (`-1.0`, `0`, a null array);
///  returning `None` lets the entry keep that mapping in one `let ... else`.
/// Gotcha:
///       `EnvUnowned::with_env` catches panics, so a panic no longer aborts the process
///  at the `extern "system"` boundary; it is logged here and reported as `None`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runWithEnv<T>(env: Env, operation: string, body: (env: Env) => T): T | null { ... }
/// ```
pub(crate) fn run_with_env<'local, T>(
    env: &mut EnvUnowned<'local>,
    operation: &str,
    body: impl FnOnce(&mut Env<'local>) -> JniResult<T>,
) -> Option<T> {
    match env.with_env(body).into_outcome() {
        Outcome::Ok(value) => return Some(value),
        Outcome::Err(error) => {
            tracing::warn!(cause = %error, operation, "JNI call failed");
            return None;
        }
        Outcome::Panic(_) => {
            tracing::error!(operation, "JNI call panicked; the panic was caught at the boundary");
            return None;
        }
    }
}
