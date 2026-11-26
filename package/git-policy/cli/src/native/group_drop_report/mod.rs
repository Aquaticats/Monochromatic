//! What:
//!  The account companion's group drop as `doctor` reports it:
//!  one probe of this host and the report a skipped drop prints,
//!  with the reason it is skipped and what a person can do about it.
//! Why:
//!  The companion's `deny` write is refused on a stock Ubuntu 24.04 before it is
//!  reached,
//!  at the open of `/proc/self/setgroups` for write,
//!  which asks for `CAP_SYS_ADMIN` inside the new user namespace
//!  (`doc/decision/cli-git-zfs-start-probe.md`,
//!  "The account companion's group drop on Ubuntu 24.04").
//!  `doctor` may not write anything, claim a slot or run an add,
//!  so the fact is measured the way an add measures it and printed here,
//!  and the person reads that the group drop was skipped, why,
//!  and what to do,
//!  without running an add to find out.
//!  The remedies are measured ones
//!  (`doc/troubleshooting/ubuntu-apparmor-userns-restriction-refuses-the-zfs-account-companion.md`).
//!
//!  Why this folder repeats the companion's two calls instead of using them:
//!  the layout rule finds the companion's folder and `invocation` inside one
//!  loop its allow list tolerates today,
//!  and a tolerated loop may only lose uses,
//!  so a new use between them may not be added
//!  (`doc/decision/cli-git-layout-rules-on-with-a-shrinking-allow-list.md`).
//!  Everything here is therefore one code path of its own,
//!  and the untangling round that takes the companion out of that loop brings
//!  the probe's calls and its words back onto the companion's own
//!  (`group_drop_probe.rs` says which).
//!
//! Files in this folder,
//! each using only files listed before it:
//!
//! - `group_drop_probe.rs`:
//!   the probe, the child, its bounded wait and what an answer means.
//! - `group_drop_probe_tests.rs`:
//!   the controls of each answer and of the digits the child writes.
//! - `group_drop_report_line.rs`:
//!   the four lines a skipped drop prints, its settings and its remedies.
//! - `group_drop_report_line_tests.rs`:
//!   the controls of the report, one per answer.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const report = groupDropReport(); // '' on a host that wrote the deny
//! ```

/// The probe of this host's group drop.
pub mod group_drop_probe;

/// The report a skipped group drop prints.
pub mod group_drop_report_line;
