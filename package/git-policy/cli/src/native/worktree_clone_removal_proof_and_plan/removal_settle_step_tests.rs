//! What:
//!  The controls of the settle step's own refusal,
//!  which the live settle's tests in `removal_settle.rs` cannot reach without a pool.
//! Why:
//!  The step proves the clone's absence again before it hands the live settle its request,
//!  and a clone that still answers must stop it there.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(settleTheRegistration(input, plan)).toThrow();
//! ```
