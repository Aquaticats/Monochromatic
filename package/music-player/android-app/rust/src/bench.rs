//! Decode-throughput micro-benchmark shared by the path and fd JNI benchmark
//! entry points.
//!  Split out of `lib.rs` to keep that file under the max-lines
//! budget;
//!  the JNI exports stay in `lib.rs` and call `bench::benchmark_decode`.

/// What:
///      `use crate::decode;` brings the crate's `decode` module into scope so
///           this file can name `decode::Source` (the decoder trait-object type).
///           `crate::` is the absolute path to this crate's root module.
/// Why:
///       `benchmark_decode` takes a `Box<dyn decode::Source>` and drives its
///           trait methods (`spec`,
///  `next_chunk`,
///  `seek`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as decode from "./decode";
/// ```
use crate::decode;
/// What:
///      `use jni::sys::jdouble;` imports the JVM `double` alias (a 64-bit float;
///           the plain-Rust sibling is `f64`).
///  The `j*` alias documents "this value
///           crosses the JVM ABI boundary".
/// Why:
///       `benchmark_decode` returns a `jdouble` (microseconds per sample,
///  or a
///           negative sentinel) straight back to the calling JNI export.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type jdouble = number; // 64-bit float
/// ```
use jni::sys::jdouble;
/// What:
///      `use std::time::Instant;` imports a monotonic clock reading (`Instant`
///           never runs backwards,
///  unlike its wall-clock sibling `SystemTime`).
/// Why:
///       The benchmark records `Instant::now()` before the decode loop and reads
///           the elapsed `Duration` after it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Instant = number; // a performance.now() timestamp, monotonic
/// ```
use std::time::Instant;

/// What:
///      `fn benchmark_decode(mut source: Box<dyn decode::Source>) -> jdouble`
///           declares a crate-visible helper (`pub(crate)`:
///  callable from other
///           modules of this crate,
///  namely the JNI benchmark exports in `lib.rs`).
///  `mut source` = the parameter is mutable (we call mutating methods
///           on it).
///  `Box<dyn decode::Source>` is an OWNING heap pointer to "some
///           value that implements the `Source` trait,
///  exact type chosen at runtime"
///           (`dyn` = dynamic dispatch,
///  like a TS interface reference;
///  `Box` is the
///           owned heap box,
///  siblings `Rc<T>`/`Arc<T>` would be shared-ownership
///           pointers,
///  which we do not want here because exactly one owner runs the
///           benchmark).
///  `-> jdouble` returns a 64-bit float (the JVM `double`).
/// Why:
///       Both the path and fd benchmarks open a decoder and then run the SAME
///           timed loop;
///  factoring it here avoids duplicating the loop twice.
///  It
///           returns microseconds per interleaved sample (comparable to the Media3
///           MediaCodec ~0.33 baseline),
///  or a negative sentinel:
///  -3 decode error,
///           -4 zero samples.
///  It also exercises seek once untimed so the seek path
///           is covered on-device.
/// Gotcha:
///    `Box<dyn Source>` is an OWNED value moved INTO this function;
///  the caller
///           gives it up.
///  In TS the caller would still hold a reference afterward.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function benchmarkDecode(source: Source): number { ... }
/// ```
pub(crate) fn benchmark_decode(mut source: Box<dyn decode::Source>) -> jdouble {
    // What:     `let spec = source.spec();` calls the trait method `spec()` to read
    //           the audio format (rate/channels/duration) and binds it to the
    //           immutable local `spec`. `let` introduces a binding; without `mut` it
    //           is read-only.
    // Why:      We touch the spec next so the decoder definitely parsed the header.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const spec = source.spec();
    // ```
    let spec = source.spec();
    // What:     `std::hint::black_box((spec.rate, spec.channels, spec.duration_secs));`
    //           builds a TUPLE `(a, b, c)` (an anonymous fixed-size group of values)
    //           of the three spec fields and feeds it to `black_box` so the optimizer
    //           cannot decide reading the spec was pointless and delete it.
    // Why:      Prove the header was really parsed on-device, untimed, before the
    //           decode loop.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // void [spec.rate, spec.channels, spec.durationSecs];
    // ```
    std::hint::black_box((spec.rate, spec.channels, spec.duration_secs));
    // What:     `let start = Instant::now();` reads the monotonic clock and binds the
    //           moment to `start`. `Instant::now()` is the associated constructor on
    //           the `Instant` type (`::` navigates into the type).
    // Why:      Mark the start of the timed window so we can measure decode time only.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const start = performance.now();
    // ```
    let start = Instant::now();
    // What:     `let mut total_samples: u64 = 0;` declares a MUTABLE counter with an
    //           explicit type `u64` (unsigned 64-bit integer; siblings: `u32` would
    //           overflow on long tracks, `usize` is platform-width, `i64` allows
    //           negatives we never need). `mut` is required because we add to it in
    //           the loop.
    // Why:      Accumulate how many interleaved samples we decoded, the denominator of
    //           the per-sample timing.
    // Gotcha:   `u64` WRAPS on overflow in release builds (no auto-widening to bigint
    //           like TS would do); chosen wide enough that a real track cannot reach it.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let totalSamples = 0;
    // ```
    let mut total_samples: u64 = 0;
    // What:     `loop { ... }` is Rust's infinite loop (runs until an inner `break`
    //           or `return`). There is no condition; this is the bare keyword form.
    // Why:      Pull decoded chunks until the decoder signals end-of-stream.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // while (true) { ... }
    // ```
    loop {
        // What:     `match source.next_chunk() { ... }` calls the trait method
        //           `next_chunk()` (which returns `Result<Vec<f32>, PlayerError>`, a
        //           success-holding-a-vector-of-floats or failure container) and
        //           branches on the outcome. `match` is exhaustive pattern dispatch.
        // Why:      Decode the next block of PCM and decide: stop, accumulate, or fail.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // let chunk: number[];
        // try { chunk = source.nextChunk(); } catch { return -3.0; }
        // ```
        match source.next_chunk() {
            // What:     `Ok(chunk) if chunk.is_empty() => break` is a GUARDED success
            //           arm. `Ok(chunk)` destructures the decoded `Vec<f32>` out of the
            //           success variant into `chunk`; the `if chunk.is_empty()` is a
            //           MATCH GUARD (extra condition the arm requires). `break` exits
            //           the `loop`.
            // Why:      An empty chunk is the decoder's end-of-stream signal; stop the
            //           loop when it arrives.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // if (chunk.length === 0) break;
            // ```
            Ok(chunk) if chunk.is_empty() => break,
            // What:     `Ok(chunk) => { ... }` is the non-empty success arm: we got a
            //           real `Vec<f32>` of samples named `chunk`, and run the block.
            // Why:      Count these samples and keep the decode work non-elidable.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // else {
            //   totalSamples += chunk.length;
            //   noInline(chunk);
            // }
            // ```
            Ok(chunk) => {
                // What:     `total_samples += chunk.len() as u64;`. `chunk.len()`
                //           returns the element count as `usize` (platform-width
                //           unsigned int). `as u64` is an EXPLICIT numeric cast from
                //           `usize` to `u64` (Rust never auto-converts integer types).
                //           `+=` adds into the mutable counter.
                // Why:      Grow the running sample total by this chunk's length.
                // Gotcha:   `as u64` is a real cast; on a 64-bit OS `usize` is already
                //           64-bit, but the cast is required to satisfy the type checker.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // totalSamples += chunk.length;
                // ```
                total_samples += chunk.len() as u64;
                // What:     `std::hint::black_box(&chunk);`. `&chunk` is a read-only
                //           BORROW of the vector (we LEND it, not give it away or copy
                //           it). `black_box` consumes the borrow so the optimizer treats
                //           the decoded data as genuinely observed.
                // Why:      Stop the optimizer from skipping decode work it thinks is
                //           unused; without this the benchmark could time nothing.
                // Gotcha:   `&chunk` does NOT copy the data; it is a temporary loan that
                //           ends at the end of this statement.
                //
                // In TS you'd write (pseudocode):
                // ```ts
                // noInline(chunk);
                // ```
                std::hint::black_box(&chunk);
            }
            // What:     `Err(_) => return -3.0`. `Err(_)` is the failure variant of the
            //           `Result`; the `_` discards the actual `PlayerError` (we do not
            //           inspect it). `return -3.0` exits the WHOLE function (not just
            //           the match) with the error sentinel `-3.0` (a `jdouble`).
            // Why:      Any decode failure ends the benchmark with the agreed "-3 decode
            //           error" code that Kotlin checks for.
            //
            // In TS you'd write (pseudocode):
            // ```ts
            // catch { return -3.0; }
            // ```
            Err(_) => return -3.0,
        }
    }
    // What:     `let elapsed = start.elapsed();` asks the `start` `Instant` how much
    //           time passed since it was taken; binds the resulting `Duration` to the
    //           immutable `elapsed`.
    // Why:      This is the measured decode time (the loop above is the only timed
    //           work).
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const elapsed = performance.now() - start;
    // ```
    let elapsed = start.elapsed();
    // Exercise seek once (untimed) so the seek path is covered on-device too; the
    // engine (task #12) drives it for real.
    // What:     `let _ = source.seek(0.0);`. `source.seek(0.0)` returns a
    //           `Result<(), PlayerError>` (success carrying the empty tuple `()` =
    //           "nothing", or an error). `let _ = ...` is the DISCARD pattern: run the
    //           expression but explicitly throw the result away, which also silences
    //           the "unused must-use Result" warning.
    // Why:      Run the seek path once so it is exercised on-device, but we do not care
    //           whether it succeeded here (the real engine handles seek for keeps).
    // Gotcha:   `let _ =` is NOT a real variable; it binds to nothing and immediately
    //           drops the value. It exists only to consume a must-use `Result`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // try { source.seek(0.0); } catch {}
    // ```
    let _ = source.seek(0.0);
    // What:     `if total_samples == 0 { return -4.0; }`. A plain conditional; `==` is
    //           ordinary equality. On a true condition, `return -4.0` exits the whole
    //           function with the "-4 zero samples" sentinel.
    // Why:      Avoid dividing by zero below, and report the empty-decode case.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (totalSamples === 0) return -4.0;
    // ```
    if total_samples == 0 {
        return -4.0;
    }
    // What:     `(elapsed.as_nanos() as f64) / 1000.0 / (total_samples as f64)` is the
    //           function's tail expression (no `;`), so it is the return value.
    //           `elapsed.as_nanos()` converts the `Duration` to an integer count of
    //           nanoseconds (`u128`); `as f64` casts that to a 64-bit float; `/ 1000.0`
    //           turns nanoseconds into microseconds; `total_samples as f64` casts the
    //           `u64` counter to a float so the final division yields microseconds per
    //           sample. Every `as` is an explicit numeric cast (Rust never converts
    //           number types implicitly).
    // Why:      Produce the single comparable figure: microseconds of decode time per
    //           interleaved sample.
    // Gotcha:   The two casts to `f64` matter: integer division would truncate; we want
    //           a fractional microseconds-per-sample result.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // return (elapsedNanos / 1000) / totalSamples;
    // ```
    return (elapsed.as_nanos() as f64) / 1000.0 / (total_samples as f64)
}
