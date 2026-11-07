//! What:
//!  Call a function with its state and get the panic it raised as a value,
//! not as an unwind.
//! Why:
//!  `std::panic::catch_unwind` accepts only a callable with no arguments,
//! so a guarded call that needs the caller's borrows
//!  would have to be a closure at every place that guards one.
//! This is the one place that makes the callable,
//! so the guarded
//!  calls elsewhere are a function and the state it reads.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! try { return { ok: call(state) }; } catch (panic) { return { panic }; }
//! ```

/// What:
///  Run `call(state)` and return what it returned,
/// or the payload of the panic it raised.
/// Why:
///   The state is asserted to be safe to look at after a panic,
/// as the callers did when they wrapped a closure the same way:
///  each caller discards or reports the panicked work and never reads half-updated state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const caught = callCatchingPanics(state, call); // { ok } or { panic }
/// ```
pub fn call_catching_panics<S, R>(state: S, call: fn(S) -> R) -> std::thread::Result<R> {
    #[cfg_attr(
        dylint_lib = "monochromatic_lint_closure",
        expect(
            avoidable_closure,
            reason = "`std::panic::catch_unwind` accepts only a callable with no arguments, and it must carry the state and the function; see package/git-policy/cli/doc/closure-lint-suppressions.md"
        )
    )]
    let caught: std::thread::Result<R> = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || { return call(state); }));
    return caught;
}

/// Panic catching controls stay out of the release executable.
#[cfg(test)]
#[path = "call_catching_panics_tests.rs"]
mod tests;
