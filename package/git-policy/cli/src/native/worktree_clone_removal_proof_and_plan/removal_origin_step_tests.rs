//! What:
//!  The controls of the origin step:
//!  its own tests live with the cleanup folder's attempt,
//!  and this sidecar holds the one rule the step adds.
//! Why:
//!  The record's moment before the first destroy and the attempt's seven steps are the
//!  cleanup folder's own,
//!  so this file names only what the removal adds to them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(theRecordStands()).toBe(true);
//! ```
