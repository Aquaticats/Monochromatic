//! The one place that turns a native method's per-call JVM handle into a usable `Env`.
//!
//! jni 0.22 split the old `JNIEnv` into `EnvUnowned` (the argument the JVM passes) and `Env`
//! (the type that carries the JNI calls,
//! lent only to a body value that implements `JniBody`).
//! Every entry that calls
//! into the JVM (string reads,
//! array copies) goes through `run_with_env`,
//! so a JNI error or a
//! caught panic is logged through `tracing` (which reaches logcat) and the entry returns its
//! own sentinel,
//! exactly as the former `match` on each `Result` did.

/// The `Result` a body returns when a call into the host runtime fails.
use jni::errors::Result as JniResult;
/// The two handle types a body reads through the lent `Env`:
/// a Java string and a Java `float[]`.
use jni::objects::{JFloatArray, JString};
/// The call's `Env` lent to the body,
/// the per-call handle it arrives in,
/// and the outcome of running the body under the panic guard.
use jni::{Env, EnvUnowned, Outcome};

/// What:
///      `pub(crate) trait JniBody<'local, T> { fn run(self, env) -> JniResult<T>; }`.
///  A piece of work that needs the call's `Env`,
///  written as a value instead of a closure.
///  `run` consumes the value,
///  uses the lent `Env`,
///  and returns the result or a JNI error.
/// Why:
///       `run_with_env` used to take a closure that captured the Java handles it read;
///  a value that holds those handles as fields says the same thing with a name,
///  and a named type replaces every call-site closure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface JniBody<T> { run(env: Env): T; }
/// ```
pub(crate) trait JniBody<'local, T> {
    /// What:
    ///      `fn run(self, env: &mut Env<'local>) -> JniResult<T>`.
    ///  Do the work with the lent `Env`.
    /// Why:
    ///       The `Env` exists only inside `run_with_env`,
    ///  so the work must arrive as a value that takes it as an argument.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// run(env: Env): T;
    /// ```
    fn run(self, env: &mut Env<'local>) -> JniResult<T>;
}

/// What:
///      `pub(crate) struct ReadJavaString<'a, 'local> { text }`.
///  A body that reads one Java string into an owned Rust `String`.
///  The field is a borrowed `JString`
///  (a read-only loan;
///  sibling `JString` by value would take ownership of the handle,
///  which the entry point still needs to return).
/// Why:
///       Three entry points turn a Kotlin `String` argument into a Rust `String`;
///  one named body replaces their three identical closures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ReadJavaString { constructor(readonly text: JString) {} }
/// ```
pub(crate) struct ReadJavaString<'a, 'local> {
    /// The Java string to read.
    pub(crate) text: &'a JString<'local>,
}

/// What:
///      `impl JniBody<'local, String> for ReadJavaString`.
///  Makes the string reader a body that `run_with_env` can run.
/// Why:
///       `run_with_env` accepts any `JniBody`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ReadJavaString implements JniBody<string> { ... }
/// ```
impl<'local> JniBody<'local, String> for ReadJavaString<'_, 'local> {
    /// What:
    ///      `fn run(self, env) -> JniResult<String>`.
    ///  Copy the Java string into an owned `String`.
    /// Why:
    ///       The Rust side needs text it can keep after the JVM handle is gone.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// run(env: Env): string { return env.getString(this.text); }
    /// ```
    fn run(self, env: &mut Env<'local>) -> JniResult<String> {
        return self.text.try_to_string(env)
    }
}

/// What:
///      `pub(crate) struct ArrayLength<'a, 'local> { array }`.
///  A body that asks the JVM how many floats a Java `float[]` holds.
/// Why:
///       The synthetic true-peak entry sizes its host buffer from the array's length.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ArrayLength { constructor(readonly samples: Float32Array) {} }
/// ```
pub(crate) struct ArrayLength<'a, 'local> {
    /// The Java array to measure.
    pub(crate) array: &'a JFloatArray<'local>,
}

/// What:
///      `impl JniBody<'local, usize> for ArrayLength`.
///  Makes the length reader a body that `run_with_env` can run.
/// Why:
///       `run_with_env` accepts any `JniBody`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class ArrayLength implements JniBody<number> { ... }
/// ```
impl<'local> JniBody<'local, usize> for ArrayLength<'_, 'local> {
    /// What:
    ///      `fn run(self, env) -> JniResult<usize>`.
    ///  Read the array length (jni 0.22 returns `usize`).
    /// Why:
    ///       The buffer that receives the copy must hold exactly that many floats.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// run(env: Env): number { return this.samples.length; }
    /// ```
    fn run(self, env: &mut Env<'local>) -> JniResult<usize> {
        return self.array.len(env)
    }
}

/// What:
///      `pub(crate) struct CopyArrayRegion<'a, 'local> { array, buffer }`.
///  A body that copies the first `buffer.len()` floats of a Java `float[]` into a host slice.
///  `array` is a borrowed `JFloatArray`;
///  `buffer` is an exclusive borrow (`&mut [f32]`) of the slice that receives the copy.
/// Why:
///       The synthetic true-peak entry brings the samples into Rust before metering them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CopyArrayRegion { constructor(readonly array: Float32Array, readonly buffer: Float32Array) {} }
/// ```
pub(crate) struct CopyArrayRegion<'a, 'local> {
    /// The Java array the floats come from.
    array: &'a JFloatArray<'local>,
    /// The host slice the floats are copied into.
    buffer: &'a mut [f32],
}

/// What:
///      `impl CopyArrayRegion { fn new }`.
///  The constructor of the region copy.
/// Why:
///       The fields stay private to this module,
///  so the entry point builds the body only through `new`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CopyArrayRegion { static of(array: Float32Array, buffer: Float32Array): CopyArrayRegion { ... } }
/// ```
impl<'a, 'local> CopyArrayRegion<'a, 'local> {
    /// What:
    ///      `pub(crate) fn new(array, buffer) -> CopyArrayRegion`.
    ///  Pair a Java array with the host slice that receives its floats.
    /// Why:
    ///       One call states both borrows at the entry point.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const copy = (array: Float32Array, buffer: Float32Array) => new CopyArrayRegion(array, buffer);
    /// ```
    pub(crate) fn new(array: &'a JFloatArray<'local>, buffer: &'a mut [f32]) -> CopyArrayRegion<'a, 'local> {
        return CopyArrayRegion { array, buffer }
    }
}

/// What:
///      `impl JniBody<'local, ()> for CopyArrayRegion`.
///  Makes the region copy a body that `run_with_env` can run.
/// Why:
///       `run_with_env` accepts any `JniBody`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class CopyArrayRegion implements JniBody<void> { ... }
/// ```
impl<'local> JniBody<'local, ()> for CopyArrayRegion<'_, 'local> {
    /// What:
    ///      `fn run(self, env) -> JniResult<()>`.
    ///  Copy the array's elements from index 0 into the host slice.
    /// Why:
    ///       One region copy is one JNI call and leaves the array on the JVM heap.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// run(env: Env): void { this.buffer.set(this.array.subarray(0, this.buffer.length)); }
    /// ```
    fn run(self, env: &mut Env<'local>) -> JniResult<()> {
        return self.array.get_region(env, 0, self.buffer)
    }
}

/// What:
///      `pub(crate) fn run_with_env<'local, T>(env, operation, body) -> Option<T>`.
///  Lend `body` the call's `Env`,
///  and return its value,
///  or `None` after logging when the JNI call failed or `body` panicked.
/// Why:
///       Each JNI entry maps failure to its own sentinel (`-1.0`,
/// `0`,
/// a null array);
///  returning `None` lets the entry keep that mapping in one `let ... else`.
/// Gotcha:
///       `EnvUnowned::with_env` catches panics,
/// so a panic no longer aborts the process
///  at the `extern "system"` boundary;
/// it is logged here and reported as `None`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runWithEnv<T>(env: Env, operation: string, body: JniBody<T>): T | null { ... }
/// ```
pub(crate) fn run_with_env<'local, T>(
    env: &mut EnvUnowned<'local>,
    operation: &str,
    body: impl JniBody<'local, T>,
) -> Option<T> {
    // What:     `env.with_env(|jni_env| body.run(jni_env))`. The jni crate's `with_env` is the
    //           only way to obtain the lent `Env` and it takes a callable that captures `body`.
    // Why:      The one closure in this module; no named function can carry `body` into
    //           `with_env`, and the jni crate offers no entry that takes a value instead
    //           (package doc `doc/closure-lint-suppressions.md`).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const outcome = env.withEnv((jniEnv) => body.run(jniEnv));
    // ```
    #[cfg_attr(
        dylint_lib = "monochromatic_lint_closure",
        expect(
            avoidable_closure,
            reason = "jni's EnvUnowned::with_env takes a callable that must carry the body value; see doc/closure-lint-suppressions.md"
        )
    )]
    let outcome = env.with_env(|jni_env| return body.run(jni_env)).into_outcome();
    match outcome {
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
