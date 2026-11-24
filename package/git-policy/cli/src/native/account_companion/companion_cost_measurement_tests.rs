//! A measurement,
//! not a control:
//! what the account companion costs one managed add,
//! as a ratio to the same add without it,
//! every arm interleaved in the same round on one build.
//! Why:
//!  The design states no duration for the companion's cost and asks step 10 to
//!  measure it as a ratio to the same add with the companion disabled,
//!  interleaved in the same round,
//! with the range between rounds,
//! and to size
//!  the per-add deadline from that measurement and from no other
//!  (`doc/planning/cli-git-zfs-start-probe-design.md`,
//! M11).
//!  This host cannot serve the ZFS route:
//! the route's gate serves only a
//!  checkout on the design's own dataset,
//!  which plan step 19's migration makes (`add_route_gate.rs`).
//!  So the measurement times the two terms the ratio is made of,
//! in one round
//!  and on one build:
//!  the real `git worktree add` through the built wrapper,
//!  and the companion's own run exactly as the preflight makes it.
//!  The identical-binary control is the second copy of each arm in the same
//!  round:
//! a difference between two copies of one command is the harness's
//!  spread,
//! and the ratio stands only above it.
//!
//! Run:
//! `cargo test --release --lib account_companion -- --ignored --nocapture companion_cost_measurement`
//! with `CLI_GIT_ACCOUNT_COMPANION_MEASUREMENT_ROOT` naming a scratch folder
//! outside the repository,
//! and the release wrapper built by
//! `mise run //package/git-policy/cli:native:build:release`.

use super::{Switch, run_under};
use crate::account_companion::companion_deadline::COMPANION_DEADLINE;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// How many rounds the arms are interleaved over.
const ROUNDS: usize = 5;

/// The number of samples one arm of one round is made of.
const SAMPLES: usize = 3;

/// Where the fixture repository and its worktrees live,
/// which the run names.
const ROOT_VARIABLE: &str = "CLI_GIT_ACCOUNT_COMPANION_MEASUREMENT_ROOT";

/// One arm's samples in one round.
#[derive(Clone, Debug, Default)]
struct Arm {
    /// Every sample this arm took.
    samples: Vec<Duration>,
}

impl Arm {
    /// The middle sample,
    /// or nothing when the arm took none.
    fn median(&self) -> Duration {
        let mut sorted: Vec<Duration> = self.samples.clone();
        sorted.sort_unstable();
        return sorted.get(sorted.len() / 2).copied().unwrap_or_default();
    }

    /// The shortest and the longest sample.
    fn range(&self) -> (Duration, Duration) {
        let mut sorted: Vec<Duration> = self.samples.clone();
        sorted.sort_unstable();
        return (sorted.first().copied().unwrap_or_default(), sorted.last().copied().unwrap_or_default());
    }
}

/// What:
///  One add through the built wrapper,
///  timed from the start of its process to its end.
fn time_an_add(binary: &Path, repository: &Path, slot: &Path) -> Duration {
    let began: Instant = Instant::now();
    let started = Command::new(binary)
        .args(["worktree", "add", "--no-checkout"])
        .arg(slot)
        .current_dir(repository)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let taken: Duration = began.elapsed();
    assert!(started.map(|status| return status.success()).unwrap_or(false), "the add at {} failed, so no sample was taken", slot.display());
    let _removed = Command::new(binary).args(["worktree", "remove", "--force"]).arg(slot).current_dir(repository).output();
    return taken;
}

/// What:
///  One companion run,
///  timed as the preflight spends it.
fn time_the_companion(switch: Switch) -> Duration {
    let began: Instant = Instant::now();
    let verdict = run_under(switch, COMPANION_DEADLINE);
    let taken: Duration = began.elapsed();
    assert!(verdict.the_managed_attach_goes_ahead(), "the companion refused on this host, so no sample was taken: {:?}", verdict.lines());
    return taken;
}

/// What:
///  Time every arm over the rounds,
///  interleaved,
///  and print the ratio with its range.
#[test]
#[ignore = "a measurement, not a control"]
fn companion_cost_measurement() {
    let root: PathBuf = measurement_root();
    let repository: PathBuf = root.join("repository");
    let binary: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/release/monochromatic-git-policy-cli");
    assert!(binary.is_file(), "the release wrapper is not built at {}; run `mise run //package/git-policy/cli:native:build:release` first", binary.display());
    make_fixture(repository.as_path());
    let mut add_first: Arm = Arm::default();
    let mut add_second: Arm = Arm::default();
    let mut companion_first: Arm = Arm::default();
    let mut companion_second: Arm = Arm::default();
    let mut disabled: Arm = Arm::default();
    let mut ratios: Vec<f64> = Vec::new();
    for round in 1..=ROUNDS {
        for sample in 0..SAMPLES {
            let slot: PathBuf = root.join(format!("slot-{round}-{sample}"));
            add_first.samples.push(time_an_add(binary.as_path(), repository.as_path(), slot.as_path()));
            companion_first.samples.push(time_the_companion(Switch::None));
            add_second.samples.push(time_an_add(binary.as_path(), repository.as_path(), slot.as_path()));
            companion_second.samples.push(time_the_companion(Switch::None));
            disabled.samples.push(time_the_companion(Switch::Disabled));
        }
        let add: Duration = add_first.median().max(add_second.median());
        let companion: Duration = companion_first.median().max(companion_second.median());
        let ratio: f64 = ratio_of(add, companion);
        ratios.push(ratio);
        println!("COMPANION COST round={round} add_ms={:.3} companion_ms={:.3} ratio={ratio:.6}", millis(add), millis(companion));
    }
    println!(
        "COMPANION COST add_ms={:.3} range={:.3}..{:.3} companion_ms={:.3} range={:.3}..{:.3} disabled_ms={:.3}",
        millis(add_first.median().max(add_second.median())),
        millis(add_first.range().0.min(add_second.range().0)),
        millis(add_first.range().1.max(add_second.range().1)),
        millis(companion_first.median().max(companion_second.median())),
        millis(companion_first.range().0.min(companion_second.range().0)),
        millis(companion_first.range().1.max(companion_second.range().1)),
        millis(disabled.median())
    );
    let shortest: f64 = ratios.iter().copied().fold(f64::INFINITY, f64::min);
    let longest: f64 = ratios.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    println!("COMPANION COST ratio-range={shortest:.6}..{longest:.6}");
}

/// The ratio one round reports:
///  the add with the companion over the add without it,
///  both terms measured in that round.
fn ratio_of(add: Duration, companion: Duration) -> f64 {
    let without: f64 = add.as_secs_f64();
    return (add + companion).as_secs_f64() / without;
}

/// One duration in milliseconds.
fn millis(within: Duration) -> f64 {
    return within.as_secs_f64() * 1_000.0;
}

/// The scratch folder this run may write in,
/// which the run names and whose absence fails rather than guessing.
fn measurement_root() -> PathBuf {
    let Some(root) = std::env::var_os(ROOT_VARIABLE) else {
        panic!("set {ROOT_VARIABLE} to a scratch folder outside the repository, which this measurement writes its fixture in");
    };
    return PathBuf::from(root);
}

/// What:
///  A repository with one commit and no worktrees,
///  which is what an add needs.
fn make_fixture(repository: &Path) {
    let _ = std::fs::remove_dir_all(repository);
    std::fs::create_dir_all(repository).expect("the fixture root is creatable");
    let steps: [&[&str]; 3] = [&["init", "--quiet", "--initial-branch", "main"], &["commit", "--quiet", "--allow-empty", "--message", "a fixture commit"], &["config", "user.name", "a fixture"]];
    for arguments in steps {
        let status = Command::new("git").args(arguments).current_dir(repository).status().expect("git runs");
        assert!(status.success(), "git {arguments:?} failed in the fixture");
    }
}
