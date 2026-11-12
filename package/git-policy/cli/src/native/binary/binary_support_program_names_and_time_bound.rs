//! What:
//!  The names of the two programs a binary-level control starts,
//! and the time bound every wrapped command runs under.
//! Why:
//!  The fixture,
//! the observation and the helpers that start commands all need them and none depends on another's files,
//! so
//!  they live in a file that imports nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! export const WRAPPER = "monochromatic-git-policy-cli"; export const REAL_GIT = "/usr/bin/git"; export const TIME_BOUND_SECONDS = 5;
//! ```

/// The native executable Cargo built for this test run.
pub const WRAPPER: &str = env!("CARGO_BIN_EXE_monochromatic-git-policy-cli");

/// The real Git 2.56.0 executable installed in the verification image.
pub const REAL_GIT: &str = "/usr/bin/git";

/// The wall-clock bound of one native Git command this harness runs for its own oracle or its fixtures,
/// in seconds,
/// as the text `timeout` takes.
/// Why not the shared backstop:
///  A fixture command is one native Git run and never the service work of a wrapped commit,
/// and a control that waits for it should fail in a minute rather than in ten.
/// Why sixty:
///  The slowest of the 1,036 oracle reads and 28 native commits that the conclusion controls made
/// took 463 ms on 2026-11-11 on a host other sessions load
/// (`git-policy-cli-service-landing-state-control-times-out-alone`),
/// and the longest disk stall one `fsync` of a trivial command showed on this host is 4.74 s
/// (`doc/troubleshooting/native-binary-tests-hit-the-shared-bound-under-host-load.md`),
/// so sixty seconds is more than ten times either.
pub const NATIVE_GIT_BOUND_SECONDS: &str = "60";

/// The wall-clock backstop,
/// in seconds,
/// of the shared bound:
/// a hang guard of the test harness,
/// not a requirement on the
///  wrapper,
/// which has no time limit on a wrapped command;
///  the one latency requirement is that a commit made through the wrapper takes less than 5 seconds longer than the
///  same commit made with native Git.
/// Real commands here finish in milliseconds.
/// The bound that fails a hung command quickly is not this one but the
///  supervisor's (`binary_supervisor.rs`):
/// it ends a command whose process tree shows no progress for five seconds or has
///  used ten seconds of processor time,
/// and it does not count time spent waiting for the disk or for the processor,
/// which
///  other sessions on a shared host stretch without limit.
/// A mutant that loops or deadlocks still fails every affected
///  control within seconds.
pub const TIME_BOUND_SECONDS: &str = super::supervisor::SHARED_BACKSTOP_SECONDS;
