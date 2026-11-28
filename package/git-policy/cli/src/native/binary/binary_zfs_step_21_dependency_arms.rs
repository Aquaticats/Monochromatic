//! What:
//!  Plan step 21's dependency-preparation arms:
//!  pnpm's offline frozen-lockfile install after an add,
//!  the Cargo clean with its lock recovered across file systems,
//!  and the mise launcher repair,
//!  each beside an interleaved control on the checkout's own dataset.
//! Why:
//!  The plan's acceptance asks for "real dependency preparation after add"
//!  with "the same-dataset control ... using the same lockfile and home store"
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 21),
//!  and `AGENTS.md` reports a timing only as a ratio to a control arm run in the
//!  same round.
//!  The control of each arm is the same tool doing the same work on the same
//!  bytes,
//!  on the checkout's own dataset,
//!  so a ratio says what the mounted clone costs and never what the tool's own
//!  work costs.
//!  Every install reads one home store,
//!  the fixture's,
//!  so both arms of a round prepare the same dependency set.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const control = time(() => pnpmInstall(controlFolder)); const arm = timeAdd('pnpm-0'); report(arm, control);
//! ```

#![cfg(target_os = "linux")]

use super::zfs_step_21_guest_fixture::{
    AddRound,
    Fixture,
    ROUNDS,
    device_of,
};
use std::path::{
    Path,
    PathBuf,
};
use std::time::Instant;

/// The pnpm the guest driver installed,
/// which the repair finds on the fixture's search path.
const PNPM: &str = "/usr/local/bin/pnpm";

/// The Node the pnpm arm resolves one dependency through,
/// which is what a relink that did not happen fails at.
const NODE: &str = "/usr/bin/node";

/// The guest's Cargo,
/// which the repair finds on the fixture's search path.
const CARGO: &str = "/root/.cargo/bin/cargo";

/// The tar a control round extracts,
/// taken once from the checkout's own `node_modules`,
/// so both arms of a round start from the same bytes.
const PRISTINE_MODULES: &str = "/root/step21-node-modules.tar";

/// Where the fixture writes the argument lines of every stand-in `mpm-shadow`
/// start.
const MPM_START_LOG: &str = "/root/step21-mpm-start.log";

/// The folder a lock of another checkout hangs under,
/// which the launcher arm's record names.
const ELSEWHERE: &str = "/root/step21-elsewhere";

/// The Node project the pnpm arm adds a worktree of.
const PACKAGE: &str = "{\n  \"name\": \"step21-fixture\",\n  \"private\": true,\n  \"version\": \"0.0.0\",\n  \"dependencies\": {\n    \"is-odd\": \"3.0.1\"\n  }\n}\n";

/// The one dependency the pnpm arm resolves after its install,
/// so a relink that never ran is a failure and not a silence.
const RESOLVED: &str = "is-odd";

/// The workspace manifest the Cargo arm's fixture holds.
const WORKSPACE_MANIFEST: &str =
    "[workspace]\nmembers = [\"crates/one\", \"crates/two\"]\nresolver = \"2\"\n";

/// The root member's manifest,
/// depending on its sibling through the workspace.
const ONE_MANIFEST: &str = "[package]\nname = \"one\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[dependencies]\ntwo = { path = \"../two\" }\n";

/// The sibling the lock does not name,
/// which is what makes the clean rewrite the lock.
const TWO_MANIFEST: &str = "[package]\nname = \"two\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

/// The one source file of the root member.
const ONE_SOURCE: &str = "/// One fixture crate.\npub fn one() -> u32 {\n    return 2;\n}\n";

/// The one source file of the sibling.
const TWO_SOURCE: &str =
    "/// The sibling the lock does not name.\npub fn two() -> u32 {\n    return 3;\n}\n";

/// The lock file the toolchain arm declares a launcher for.
const TOOLCHAIN_LOCK: &str =
    "{\"schemaVersion\":1,\"tools\":{\"step21:own\":{\"bins\":[\"own\"]}}}\n";

/// The launcher name the toolchain arm's lock declares.
const LAUNCHER: &str = "own";

/// A stand-in `mpm-shadow`,
/// which is no real installer:
/// it records its arguments,
/// prints the summary line a real install prints,
/// and writes the record of the name its own lock declares.
/// It is a stand-in,
/// and nothing here claims what a real `mpm-shadow` does beyond that summary
/// line,
/// which `package_repair_toolchain_tests.rs` copies from a real run.
const MPM_STAND_IN: &str = "#!/bin/sh\nprintf '%s|%s|%s\\n' \"$*\" \"$(pwd -P)\" \"$MONOCHROMATIC_WORKTREE_ROOT\" >> /root/step21-mpm-start.log\necho 'toolchain install: 1 installed, 2 unchanged, 0 failed'\nprintf '{\"toolId\":\"step21:own\",\"lockFile\":\"%s/toolchain.lock.json\",\"real\":\"/tools/own\",\"scope\":\"repo\"}\\n' \"$MONOCHROMATIC_WORKTREE_ROOT\" > \"$MONOCHROMATIC_WORKTREE_ROOT/.cache/toolchain-launchers/own.launch\"\nexit 0\n";

/// A time the repair line credits one tool with,
/// in milliseconds.
struct TimedTool {
    /// The words the repair used for it.
    detail: String,
    /// The milliseconds it ran for.
    millis: u128,
}

/// What:
///  The success line a repair printed,
///  or nothing when every repair failed.
/// Why:
///  A failure is a line of its own,
///  so an arm that reads only the success line must say the repair failed
///  rather than read no time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const line = said.split('\n').find((text) => text.includes('package-manager state of'));
/// ```
pub(super) fn the_repair_line(said: &str) -> Option<&str> {
    return said
        .lines()
        .find(|line| return line.contains("package-manager state of"));
}

/// What:
///  Every tool the repair line credits,
///  with the milliseconds it names.
/// Why:
///  `render` writes one detail per tool,
///  each followed by ` in <seconds> s`,
///  so the pairs are read from one line instead of guessed from order
///  (`package_repair_report.rs`,
///  `render`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const worked = line.split('work at its new location: ')[1].split('; ').map((chunk) => chunk.rsplit(' in ', 1));
/// ```
fn the_tools_credited(line: &str) -> Vec<TimedTool> {
    let mut credited: Vec<TimedTool> = Vec::new();
    let Some((_before, rest)) = line.split_once("work at its new location: ") else {
        return credited;
    };
    for chunk in rest.trim_end().trim_end_matches('.').split("; ") {
        let Some((detail, time)) = chunk.rsplit_once(" in ") else {
            continue;
        };
        let Some(number) = time.strip_suffix(" s") else {
            continue;
        };
        credited.push(TimedTool {
            detail: String::from(detail),
            millis: seconds_to_millis(number),
        });
    }
    return credited;
}

/// The milliseconds a `x.y` seconds reading names.
fn seconds_to_millis(text: &str) -> u128 {
    let (whole, tenths): (&str, &str) = match text.split_once('.') {
        Some((whole, tenths)) => (whole, tenths),
        None => (text, "0"),
    };
    let seconds: u128 = whole.trim().parse().unwrap_or(0);
    let fraction: u128 = tenths.trim().parse().unwrap_or(0);
    return seconds * 1_000 + fraction * 100;
}

/// What:
///  The milliseconds a tool's own line credits it with,
///  or a panic naming what the add said.
/// Why:
///  An arm whose tool never ran would otherwise report a silence as a time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const millis = tools.find((tool) => tool.detail.startsWith('pnpm ')).millis;
/// ```
pub(super) fn the_time_of(
    said: &str,
    needle: &str,
) -> u128 {
    let Some(line) = the_repair_line(said) else {
        panic!("the add printed no repair line: {said}");
    };
    for tool in the_tools_credited(line) {
        if tool.detail.starts_with(needle) {
            return tool.millis;
        }
    }
    panic!("the repair line names no tool starting {needle:?}: {line}");
}

/// What:
///  Whether the add's own diagnostic log holds a line with all these words.
/// Why:
///  The repair says which branch it took only in that log,
///  and a branch is evidence while a duration is not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const said = add.said.split('\n').some((line) => words.every((word) => line.includes(word)));
/// ```
fn the_log_says(
    said: &str,
    words: &[&str],
) -> bool {
    return said
        .lines()
        .any(|line| return words.iter().all(|word| return line.contains(word)));
}

/// The ratio of one arm's time to its control's,
/// or not-a-number when the control reads zero.
fn ratio_of(
    arm: u128,
    control: u128,
) -> f64 {
    return if control == 0 {
        f64::NAN
    } else {
        arm as f64 / control as f64
    };
}

/// The middle of a list of readings,
/// which is the median of an odd count.
fn middle(values: &[u128]) -> u128 {
    let mut sorted: Vec<u128> = values.to_vec();
    sorted.sort_unstable();
    return sorted.get(sorted.len() / 2).copied().unwrap_or(0);
}

/// What:
///  Print an arm's readings,
///  the run-to-run spread of each,
///  and the ratio of each round to the control beside it.
/// Why:
///  `AGENTS.md` reports a timing only as a ratio to a control arm run
///  interleaved in the same round,
///  with its range,
///  and the spread of the arm on unchanged input says first whether a
///  difference is larger than the noise.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// console.log(`${arm} control-ms=[${controls}] arm-ms=[${measured}] ratio per-round=[${ratios}] median=${median} range=${min}..${max}`);
/// ```
pub(super) fn report(
    arm: &str,
    controls: &[u128],
    measured: &[u128],
) {
    let ratios: Vec<f64> = measured
        .iter()
        .zip(controls.iter())
        .map(|(one, control)| return ratio_of(*one, *control))
        .collect();
    let lowest: f64 = ratios.iter().copied().fold(f64::INFINITY, f64::min);
    let highest: f64 = ratios.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    println!(
        "STEP21 {arm} control-ms={controls:?} arm-ms={measured:?} control-spread={}..{} arm-spread={}..{}",
        controls.iter().copied().min().unwrap_or(0),
        controls.iter().copied().max().unwrap_or(0),
        measured.iter().copied().min().unwrap_or(0),
        measured.iter().copied().max().unwrap_or(0)
    );
    println!(
        "STEP21 {arm} ratio per-round={ratios:?} median={:.3} range={lowest:.3}..{highest:.3} control-median={} arm-median={}",
        ratio_of(middle(measured), middle(controls)),
        middle(controls),
        middle(measured)
    );
}

/// What:
///  The node project the pnpm and queued arms prepare:
///  one dependency,
///  its lockfile committed,
///  its installed `node_modules` kept as the tar both arms of a round start
///  from.
/// Why:
///  The repair relinks a `node_modules` the clone carried,
///  so the store is warmed in the checkout and the bytes a control starts from
///  are the same bytes,
///  not a second install.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// commit({ 'package.json': manifest }); pnpmInstall(); commit({ 'pnpm-lock.yaml': ... }); tar('node_modules');
/// ```
pub(super) fn the_node_project(fixture: &Fixture) {
    fixture.commit(
        &[
            ("package.json", PACKAGE),
            ("src/main.txt", "the node project\n"),
        ],
        "the node project",
    );
    // The store is warmed once, in the checkout, so both arms of every round install offline from it.
    fixture.run(Path::new(PNPM), &["install"], fixture.root());
    fixture.git(&["add", "--all"]);
    fixture.git(&[
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "commit",
        "--quiet",
        "--no-verify",
        "-m",
        "the lockfile",
    ]);
    fixture.run(
        Path::new("/usr/bin/tar"),
        &["--create", "--file", PRISTINE_MODULES, "node_modules"],
        fixture.root(),
    );
}

/// What:
///  The Cargo workspace the Cargo and queued arms prepare:
///  a member depending on its sibling,
///  a built build directory the clone's snapshot carries,
///  and a lock the manifest no longer matches.
/// Why:
///  `cargo clean` rewrites a lock that disagrees with its manifest unless it is
///  given `--locked`,
///  and the add's own clean is not,
///  so every round has a lock for the repair to put back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// commit(manifests); cargo build --offline; stripTheSiblingFromTheLock(); commit(lock);
/// ```
pub(super) fn the_cargo_workspace(fixture: &Fixture) -> String {
    fixture.commit(
        &[
            ("Cargo.toml", WORKSPACE_MANIFEST),
            ("crates/one/Cargo.toml", ONE_MANIFEST),
            ("crates/one/src/lib.rs", ONE_SOURCE),
            ("crates/two/Cargo.toml", TWO_MANIFEST),
            ("crates/two/src/lib.rs", TWO_SOURCE),
            ("src/main.txt", "the cargo workspace\n"),
        ],
        "the cargo workspace",
    );
    fixture.run(Path::new(CARGO), &["build", "--offline"], fixture.root());
    let complete: String =
        std::fs::read_to_string(fixture.root().join("Cargo.lock")).expect("the lock Cargo wrote");
    let stripped: String = without_the_sibling(complete.as_str());
    assert_ne!(
        stripped, complete,
        "the lock Cargo wrote names no sibling to strip"
    );
    std::fs::write(fixture.root().join("Cargo.lock"), stripped.as_bytes())
        .expect("the lock without the sibling");
    fixture.git(&["add", "--all"]);
    fixture.git(&[
        "-c",
        "user.name=Fixture",
        "-c",
        "user.email=fixture@example.invalid",
        "commit",
        "--quiet",
        "--no-verify",
        "-m",
        "the lock the manifest no longer matches",
    ]);
    return stripped;
}

/// What:
///  A lock whose bytes do not name the sibling,
///  which the clean of a lockless manifest rewrites.
/// Why:
///  `cargo clean` refuses a stale lock only with `--locked`,
///  and the add's own clean does not pass it,
///  so the lock it rewrites is the one the arm has to displace.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const kept = text.split('[[package]]').filter((part) => !part.includes('name = "two"')).join('[[package]]');
/// ```
fn without_the_sibling(text: &str) -> String {
    let mut kept: Vec<&str> = Vec::new();
    for part in text.split("[[package]]") {
        if part.contains("name = \"two\"") {
            continue;
        }
        kept.push(part);
    }
    return kept.join("[[package]]");
}

/// What:
///  Run pnpm's offline frozen-lockfile install in a folder the arm prepared,
///  and time it.
/// Why:
///  This is the control of the pnpm arm:
///  the same command,
///  the same lockfile,
///  the same home store,
///  on the checkout's own dataset.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const started = now(); fixture.run(PNPM, ['install', '--offline', '--frozen-lockfile'], control); return started.elapsed();
/// ```
fn pnpm_in_a_control_folder(
    fixture: &Fixture,
    round: usize,
) -> u128 {
    let control: PathBuf = fixture.control("pnpm", round);
    let _gone: std::io::Result<()> = std::fs::remove_dir_all(control.as_path());
    std::fs::create_dir_all(control.as_path()).expect("the control folder");
    for name in ["package.json", "pnpm-lock.yaml"] {
        std::fs::copy(fixture.root().join(name), control.join(name))
            .unwrap_or_else(|error| panic!("{name} could not be copied: {error}"));
    }
    fixture.run(
        Path::new("/usr/bin/tar"),
        &[
            "--extract",
            "--file",
            PRISTINE_MODULES,
            "--directory",
            control.to_str().expect("a text path"),
        ],
        control.as_path(),
    );
    let started: Instant = Instant::now();
    fixture.run(
        Path::new(PNPM),
        &["install", "--offline", "--frozen-lockfile"],
        control.as_path(),
    );
    let elapsed: u128 = started.elapsed().as_millis();
    std::fs::remove_dir_all(control.as_path()).expect("the control folder goes");
    println!("STEP21 pnpm control round={round} ms={elapsed}");
    return elapsed;
}

/// What:
///  Add a worktree of the node project and read the time the repair credits
///  pnpm with,
///  then resolve one dependency through the clone's own `node_modules`.
/// Why:
///  The install is the add's own,
///  so the arm times it from the repair's line instead of from a second run of
///  the command.
///  A resolution through the clone is what a relink that never ran fails at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const added = fixture.add('pnpm-0'); const millis = theTimeOf(added.said, 'pnpm relinked'); resolveThroughTheClone();
/// ```
fn pnpm_after_an_add(
    fixture: &Fixture,
    round: usize,
) -> (u128, u128) {
    let name: String = format!("pnpm-{round}");
    let added: AddRound = fixture.add(name.as_str(), None);
    assert_eq!(
        added.status,
        Some(0),
        "the add of {name} was refused: {}",
        added.said
    );
    let millis: u128 = the_time_of(added.said.as_str(), "pnpm relinked node_modules");
    let slot: PathBuf = fixture.slot(name.as_str());
    let resolved: std::process::Output = fixture.run(
        Path::new(NODE),
        &["--print", format!("require.resolve('{RESOLVED}')").as_str()],
        slot.as_path(),
    );
    let answer: String = String::from_utf8_lossy(resolved.stdout.as_slice())
        .trim()
        .to_string();
    assert!(
        answer.starts_with(slot.to_str().expect("a text path")),
        "the clone resolves {RESOLVED} outside its own tree: {answer}"
    );
    println!("STEP21 pnpm arm round={round} ms={millis} resolves={answer}");
    fixture.clear(name.as_str());
    return (millis, added.elapsed.as_millis());
}

/// What:
///  The pnpm arm:
///  one home store and one lockfile prepared once,
///  then interleaved rounds of the control and the add.
/// Why:
///  The plan asks for the offline frozen-lockfile install after an add beside a
///  same-dataset control,
///  and the rounds alternate which arm goes first so neither is warmed by the
///  other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const store = warmTheStore(); for (let n = 0; n < ROUNDS; n += 1) { n % 2 === 0 ? controlThenAdd(n) : addThenControl(n); }
/// ```
pub(super) fn the_pnpm_arm(fixture: &Fixture) {
    fixture.forget(&[], "nothing to remove before the node project");
    the_node_project(fixture);
    fixture.assert_clean();
    let mut controls: Vec<u128> = Vec::new();
    let mut measured: Vec<u128> = Vec::new();
    for round in 0..ROUNDS {
        if round % 2 == 0 {
            controls.push(pnpm_in_a_control_folder(fixture, round));
            measured.push(pnpm_after_an_add(fixture, round).0);
        } else {
            measured.push(pnpm_after_an_add(fixture, round).0);
            controls.push(pnpm_in_a_control_folder(fixture, round));
        }
    }
    report("pnpm", controls.as_slice(), measured.as_slice());
}

/// What:
///  Run the Cargo clean in a copy of the workspace on the checkout's dataset,
///  and time it.
/// Why:
///  This is the control of the Cargo arm:
///  the same clean,
///  the same lock,
///  the same built build directory,
///  on the dataset the clone was snapshotted from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const copy = reflinkOf(workspace); const started = now(); fixture.run(CARGO, ['clean', '--offline', '--package', 'one'], copy); return started.elapsed();
/// ```
fn cargo_in_a_control_folder(
    fixture: &Fixture,
    round: usize,
) -> u128 {
    let control: PathBuf = fixture.control("cargo", round);
    let _gone: std::io::Result<()> = std::fs::remove_dir_all(control.as_path());
    std::fs::create_dir_all(control.as_path()).expect("the control folder");
    for name in ["Cargo.toml", "Cargo.lock", "target", "crates"] {
        fixture.run(
            Path::new("/usr/bin/cp"),
            &[
                "--archive",
                "--reflink=auto",
                name,
                control.to_str().expect("a text path"),
            ],
            fixture.root(),
        );
    }
    let started: Instant = Instant::now();
    fixture.run(
        Path::new(CARGO),
        &["clean", "--offline", "--package", "one"],
        control.as_path(),
    );
    let elapsed: u128 = started.elapsed().as_millis();
    std::fs::remove_dir_all(control.as_path()).expect("the control folder goes");
    println!("STEP21 cargo control round={round} ms={elapsed}");
    return elapsed;
}

/// What:
///  Add a worktree of the Cargo workspace and read the time the repair credits
///  its clean with,
///  then check the lock the clean displaced.
/// Why:
///  The clone's Git directory lies on the checkout's dataset while the worktree
///  lies on its own,
///  so the exchange that would put the checked-out lock back cannot happen and
///  its bytes are written over the lock instead;
///  the arm says so from the two devices and from the copy the repair kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const added = fixture.add('cargo-0'); const millis = theTimeOf(added.said, 'Cargo kept'); keptLockExists(); devicesDiffer();
/// ```
fn cargo_after_an_add(
    fixture: &Fixture,
    round: usize,
) -> u128 {
    let name: String = format!("cargo-{round}");
    let added: AddRound = fixture.add(name.as_str(), None);
    assert_eq!(
        added.status,
        Some(0),
        "the add of {name} was refused: {}",
        added.said
    );
    let millis: u128 = the_time_of(
        added.said.as_str(),
        "Cargo kept dependency builds and cleaned the worktree's own crates",
    );
    assert!(
        the_log_says(
            added.said.as_str(),
            &["packageRepairCargo", "the clean runs here instead"]
        ),
        "the add did not take the clean-in-place branch: {}",
        added.said
    );
    let slot: PathBuf = fixture.slot(name.as_str());
    let checked_out: Vec<u8> =
        std::fs::read(fixture.root().join("Cargo.lock")).expect("the checkout's lock");
    let in_the_clone: Vec<u8> = std::fs::read(slot.join("Cargo.lock")).expect("the clone's lock");
    assert_eq!(
        in_the_clone, checked_out,
        "the clone's lock is not the checked-out one"
    );
    let displaced: PathBuf = fixture
        .common()
        .join("monochromatic-git-policy-cli-displaced-locks");
    let kept: usize = std::fs::read_dir(displaced.as_path())
        .map(|entries| return entries.count())
        .unwrap_or(0);
    let worktree_device: u64 = device_of(slot.as_path());
    let git_device: u64 = device_of(fixture.common().as_path());
    println!(
        "STEP21 cargo arm round={round} ms={millis} displaced-folders={kept} worktree-device={worktree_device} git-device={git_device} exchange-possible={}",
        worktree_device == git_device
    );
    assert!(
        kept > 0,
        "the clean displaced no lock, so nothing was kept under {}",
        displaced.display()
    );
    assert_ne!(
        worktree_device, git_device,
        "the clone and its Git directory are on one device, so the exchange that puts the lock back can happen"
    );
    fixture.clear(name.as_str());
    return millis;
}

/// What:
///  The Cargo arm:
///  the node project taken away,
///  the workspace built once,
///  its lock stripped of the sibling the manifest names,
///  a staging name planted so the add cleans where the state stands,
///  then interleaved rounds of the control and the add.
/// Why:
///  The plan asks for the Cargo prewarm and the cross-filesystem lock recovery;
///  a clean that runs in the add is the only path where the add itself reads
///  and puts back the lock,
///  and the add takes it when the copied build state cannot be moved aside.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// forgetTheNodeProject(); const built = buildTheWorkspace(); stripTheSiblingFromTheLock(); plantTheStagingName(); rounds();
/// ```
pub(super) fn the_cargo_arm(fixture: &Fixture) {
    fixture.forget(
        &["package.json", "pnpm-lock.yaml"],
        "the node project goes, so one add runs one tool",
    );
    fixture.remove_ignored(&["node_modules"]);
    let lock: String = the_cargo_workspace(fixture);
    let staging: PathBuf = fixture.root().join("target.monochromatic-prewarm");
    // A name this wrapper did not stage makes the add leave the copied state where it is and clean it here instead.
    std::fs::create_dir_all(staging.as_path()).expect("the planted staging name");
    fixture.assert_clean();
    let probe: std::process::Output = fixture.run(
        Path::new(CARGO),
        &["metadata", "--locked", "--offline", "--no-deps"],
        fixture.root(),
    );
    println!(
        "STEP21 cargo lock-probe metadata-locked-bytes={} lock-bytes={}",
        probe.stdout.len(),
        lock.len()
    );
    let mut controls: Vec<u128> = Vec::new();
    let mut measured: Vec<u128> = Vec::new();
    for round in 0..ROUNDS {
        if round % 2 == 0 {
            controls.push(cargo_in_a_control_folder(fixture, round));
            measured.push(cargo_after_an_add(fixture, round));
        } else {
            measured.push(cargo_after_an_add(fixture, round));
            controls.push(cargo_in_a_control_folder(fixture, round));
        }
    }
    report("cargo", controls.as_slice(), measured.as_slice());
}

/// What:
///  The launcher arm's state in the checkout:
///  a lock that declares one launcher,
///  a record naming another checkout's lock,
///  and the worktree's own stand-in `mpm-shadow`.
/// Why:
///  The repair rewrites the links of a worktree whose records name another
///  lock,
///  and the program is the one the worktree's own link names when the search
///  path holds none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// writeLock(); writeRecord(ELSEWHERE); writeStandIn('.cache/toolchain-bin/mpm-shadow');
/// ```
fn the_launcher_state(fixture: &Fixture) {
    fixture.commit(
        &[
            ("toolchain.lock.json", TOOLCHAIN_LOCK),
            ("src/main.txt", "the toolchain\n"),
        ],
        "the toolchain lock",
    );
    std::fs::create_dir_all(ELSEWHERE).expect("the other checkout's folder");
    std::fs::write(
        Path::new(ELSEWHERE).join("toolchain.lock.json"),
        TOOLCHAIN_LOCK.as_bytes(),
    )
    .expect("the other checkout's lock");
    let launchers: PathBuf = fixture.root().join(".cache").join("toolchain-launchers");
    std::fs::create_dir_all(launchers.as_path()).expect("the launcher folder");
    std::fs::write(
        launchers.join(format!("{LAUNCHER}.launch")),
        format!(
            "{{\"toolId\":\"step21:own\",\"lockFile\":\"{ELSEWHERE}/toolchain.lock.json\",\"real\":\"/tools/own\",\"scope\":\"repo\"}}\n"
        )
        .as_bytes(),
    )
    .expect("the record naming another lock");
    let bin: PathBuf = fixture.root().join(".cache").join("toolchain-bin");
    std::fs::create_dir_all(bin.as_path()).expect("the toolchain bin folder");
    let stand_in: PathBuf = bin.join("mpm-shadow");
    std::fs::write(stand_in.as_path(), MPM_STAND_IN.as_bytes()).expect("the stand-in");
    std::fs::set_permissions(
        stand_in.as_path(),
        std::os::unix::fs::PermissionsExt::from_mode(0o755),
    )
    .expect("the stand-in's mode");
}

/// What:
///  Run the stand-in the way the repair runs it,
///  in a folder of the arm's own,
///  and time it.
/// Why:
///  This is the launcher arm's control:
///  the same program with the same arguments and the same root,
///  started by the arm instead of by the add's repair.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const folder = controlFolder(round); writeRecord(folder); const started = now(); run(standIn, ['install', '--frozen-lockfile'], folder, {MONOCHROMATIC_WORKTREE_ROOT: folder}); return started.elapsed();
/// ```
fn launcher_in_a_control_folder(
    fixture: &Fixture,
    round: usize,
) -> u128 {
    let control: PathBuf = fixture.control("launcher", round);
    let _gone: std::io::Result<()> = std::fs::remove_dir_all(control.as_path());
    let launchers: PathBuf = control.join(".cache").join("toolchain-launchers");
    std::fs::create_dir_all(launchers.as_path()).expect("the control's launcher folder");
    std::fs::write(
        launchers.join(format!("{LAUNCHER}.launch")),
        format!(
            "{{\"toolId\":\"step21:own\",\"lockFile\":\"{ELSEWHERE}/toolchain.lock.json\",\"real\":\"/tools/own\",\"scope\":\"repo\"}}\n"
        )
        .as_bytes(),
    )
    .expect("the control's record");
    let stand_in: PathBuf = fixture
        .root()
        .join(".cache")
        .join("toolchain-bin")
        .join("mpm-shadow");
    let started: Instant = Instant::now();
    let status: std::process::ExitStatus = std::process::Command::new(stand_in)
        .args(["install", "--frozen-lockfile"])
        .current_dir(control.as_path())
        .env("MONOCHROMATIC_WORKTREE_ROOT", control.as_path())
        .status()
        .expect("the stand-in starts");
    let elapsed: u128 = started.elapsed().as_millis();
    assert!(status.success(), "the control's stand-in refused");
    std::fs::remove_dir_all(control.as_path()).expect("the control folder goes");
    println!("STEP21 launcher control round={round} ms={elapsed}");
    return elapsed;
}

/// What:
///  Add a worktree whose launcher record names another checkout's lock,
///  and read what the repair did:
///  the time it credited `mpm-shadow`,
///  the arguments and the root the stand-in was given,
///  and the record the install left.
/// Why:
///  The plan asks to check the mise launcher repair;
///  the record naming this worktree's own lock after the add is the repair's
///  result,
///  and the root the program was given is how it knew which checkout to relink.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const added = fixture.add('launcher-0'); const millis = theTimeOf(added.said, 'mpm-shadow rebuilt'); recordNamesTheClone(); startLineNamesTheRoot();
/// ```
fn launcher_after_an_add(
    fixture: &Fixture,
    round: usize,
) -> u128 {
    let name: String = format!("launcher-{round}");
    let _cleared: std::io::Result<()> = std::fs::remove_file(MPM_START_LOG);
    let added: AddRound = fixture.add(name.as_str(), None);
    assert_eq!(
        added.status,
        Some(0),
        "the add of {name} was refused: {}",
        added.said
    );
    let millis: u128 = the_time_of(
        added.said.as_str(),
        "mpm-shadow rebuilt the toolchain links",
    );
    let slot: PathBuf = fixture.slot(name.as_str());
    let record: String = std::fs::read_to_string(
        slot.join(".cache")
            .join("toolchain-launchers")
            .join(format!("{LAUNCHER}.launch")),
    )
    .expect("the record the install left");
    assert!(
        record.contains(slot.to_str().expect("a text path")),
        "the record does not name the clone's own lock: {record}"
    );
    let started: String = std::fs::read_to_string(MPM_START_LOG).expect("the stand-in's own line");
    println!(
        "STEP21 launcher arm round={round} ms={millis} start={started} record-names-the-clone={}",
        record.contains(slot.to_str().expect("a text path"))
    );
    assert!(
        started.contains("install --frozen-lockfile"),
        "the stand-in was not asked for the install: {started}"
    );
    assert!(
        started.contains(slot.to_str().expect("a text path")),
        "the stand-in was given no root naming the clone: {started}"
    );
    fixture.clear(name.as_str());
    return millis;
}

/// What:
///  The launcher arm:
///  the record naming another lock prepared once,
///  then interleaved rounds of the control and the add.
/// Why:
///  The plan asks to check the mise launcher repair,
///  and a ratio to a control run in the same round is how a timing here is
///  reported.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theLauncherState(); for (let n = 0; n < ROUNDS; n += 1) { alternate(); }
/// ```
pub(super) fn the_launcher_arm(fixture: &Fixture) {
    fixture.forget(
        &[
            "Cargo.toml",
            "Cargo.lock",
            "crates/one/Cargo.toml",
            "crates/two/Cargo.toml",
            "crates/one/src/lib.rs",
            "crates/two/src/lib.rs",
        ],
        "the cargo workspace goes, so one add runs one tool",
    );
    fixture.remove_ignored(&[
        "target",
        "target.monochromatic-prewarm",
        "crates",
        ".step21",
    ]);
    the_launcher_state(fixture);
    fixture.assert_clean();
    let mut controls: Vec<u128> = Vec::new();
    let mut measured: Vec<u128> = Vec::new();
    for round in 0..ROUNDS {
        if round % 2 == 0 {
            controls.push(launcher_in_a_control_folder(fixture, round));
            measured.push(launcher_after_an_add(fixture, round));
        } else {
            measured.push(launcher_after_an_add(fixture, round));
            controls.push(launcher_in_a_control_folder(fixture, round));
        }
    }
    report("launcher", controls.as_slice(), measured.as_slice());
}
