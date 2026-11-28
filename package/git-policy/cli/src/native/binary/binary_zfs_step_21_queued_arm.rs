//! What:
//!  Plan step 21's queued repair:
//!  the same add with a commit service answering beside the same add with none,
//!  so the relink and the prewarm are the service's work in one round and the
//!  add's own in the other.
//! Why:
//!  The plan asks to check "queued and inline repair"
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  step 21),
//!  and `SPEC.md` "Package-manager repair of a new worktree" says the two tools
//!  that rewrite link layouts in place are the service's work whenever a service
//!  answers and the add's own when none does.
//!  A service is one mode of the same executable,
//!  started here with the address the wrapper's own route would find,
//!  so the hand-over is the shipped one and not a stand-in
//!  (`src/native/service/service_main.rs`,
//!  `--path` and `--instance`).
//!  The queued add returns before either tool runs and the inline add waits for
//!  pnpm,
//!  which is the difference the two arms report as a ratio.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const service = startService(); const queued = add(); await recordsSettled(); service.stop(); const inline = add(); report(queued.elapsed, inline.elapsed);
//! ```

#![cfg(target_os = "linux")]

use super::zfs_step_21_dependency_fixtures::{
    RESOLVED,
    the_cargo_workspace,
    the_node_project,
};
use super::zfs_step_21_guest_fixture::{
    AddRound,
    Fixture,
};
use super::zfs_step_21_guest_layout::{
    ROUNDS,
    mounted_at,
};
use super::zfs_step_21_queued_service::{
    TheService,
    wait_until_settled,
};
use super::zfs_step_21_repair_readings::{
    report,
    the_repair_line,
    the_time_of,
};
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The checkout's state for this arm:
///  the node project and the Cargo workspace,
///  with neither the staging name planted nor a service's records left.
/// Why:
///  One add of this state hands over one relink and one prewarm,
///  so a round shows both tools on each side of the queued and inline pair.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theNodeProject(); theCargoWorkspace();
/// ```
fn the_queued_state(fixture: &Fixture) {
    // The launcher arm's record and stand-in are no part of this pair, so the adds here run the two tools it is about.
    fixture.forget(
        &["toolchain.lock.json"],
        "the toolchain lock goes, so these adds are about the two tools this pair measured",
    );
    fixture.remove_ignored(&[".cache"]);
    the_node_project(fixture);
    let _lock: String = the_cargo_workspace(fixture);
    fixture.assert_clean();
}

/// What:
///  Add a worktree while a service answers,
///  and read what it did after the add returned.
/// Why:
///  The queued repair's whole claim is that the add returns before the tools run
///  and that the service runs them in the worktree the record names,
///  so the arm reads the queue,
///  the clone's own `node_modules`,
///  and the build
///  directory the filling published.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const added = fixture.add(name); await settled(); assert(pnpmResolves()); assert(targetPublished()); assert(lockRestored());
/// ```
fn queued_after_an_add(
    fixture: &Fixture,
    round: usize,
) -> u128 {
    let name: String = format!("queued-{round}");
    let added: AddRound = fixture.add(name.as_str(), None);
    assert_eq!(
        added.status,
        Some(0),
        "the queued add of {name} was refused: {}",
        added.said
    );
    let Some(line) = the_repair_line(added.said.as_str()) else {
        panic!("the queued add printed no repair line: {}", added.said);
    };
    assert!(
        line.contains("the commit service runs it after this command returns"),
        "the add did not hand the relink over: {line}"
    );
    assert!(
        line.contains("handed the filling of 1 build directory to the commit service"),
        "the add did not hand the filling over: {line}"
    );
    wait_until_settled(fixture);
    let slot: PathBuf = fixture.slot(name.as_str());
    let resolved: std::process::Output = fixture.run(
        Path::new("/usr/bin/node"),
        &["--print", format!("require.resolve('{RESOLVED}')").as_str()],
        slot.as_path(),
    );
    let answer: String = String::from_utf8_lossy(resolved.stdout.as_slice())
        .trim()
        .to_string();
    assert!(
        answer.starts_with(slot.to_str().expect("a text path")),
        "the service's relink left the clone resolving is-odd elsewhere: {answer}"
    );
    let published: bool = slot.join("target").is_dir();
    let staged_gone: bool = !slot.join("target.monochromatic-prewarm").exists();
    let checked_out: Vec<u8> =
        std::fs::read(fixture.root().join("Cargo.lock")).expect("the checkout's lock");
    let in_the_clone: Vec<u8> = std::fs::read(slot.join("Cargo.lock")).expect("the clone's lock");
    let kept: usize = std::fs::read_dir(
        fixture
            .common()
            .join("monochromatic-git-policy-cli-displaced-locks")
            .as_path(),
    )
    .map(|entries| return entries.count())
    .unwrap_or(0);
    println!(
        "STEP21 queued round={round} add-ms={} target-published={published} staging-gone={staged_gone} lock-restored={} displaced-folders={kept}",
        added.elapsed.as_millis(),
        in_the_clone == checked_out
    );
    assert!(
        published,
        "the filling published no build directory in the clone"
    );
    assert!(
        staged_gone,
        "the filling left the staging name the add made in the clone"
    );
    assert_eq!(
        in_the_clone, checked_out,
        "the filling left the clone's lock as the clean wrote it"
    );
    assert!(kept > 0, "the filling's clean displaced no lock");
    fixture.clear(name.as_str());
    return added.elapsed.as_millis();
}

/// What:
///  Add the same worktree with no service answering,
///  and read what the add did itself.
/// Why:
///  With no service the add relinks the clone itself and leaves the prewarm
///  record queued,
///  which is the inline half of the plan's "queued and inline repair".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const added = fixture.add(name); assert(line.includes('pnpm relinked')); assert(line.includes('could not hand to the commit service'));
/// ```
fn inline_after_an_add(
    fixture: &Fixture,
    round: usize,
) -> u128 {
    let name: String = format!("inline-{round}");
    let added: AddRound = fixture.add(name.as_str(), None);
    assert_eq!(
        added.status,
        Some(0),
        "the inline add of {name} was refused: {}",
        added.said
    );
    let pnpm_millis: u128 = the_time_of(added.said.as_str(), "pnpm relinked node_modules");
    let Some(line) = the_repair_line(added.said.as_str()) else {
        panic!("the inline add printed no repair line: {}", added.said);
    };
    assert!(
        line.contains("could not hand to the commit service"),
        "the inline add handed the filling to a service that was not there: {line}"
    );
    let slot: PathBuf = fixture.slot(name.as_str());
    let resolved: std::process::Output = fixture.run(
        Path::new("/usr/bin/node"),
        &["--print", format!("require.resolve('{RESOLVED}')").as_str()],
        slot.as_path(),
    );
    let answer: String = String::from_utf8_lossy(resolved.stdout.as_slice())
        .trim()
        .to_string();
    assert!(
        answer.starts_with(slot.to_str().expect("a text path")),
        "the add's own relink left the clone resolving is-odd elsewhere: {answer}"
    );
    println!(
        "STEP21 inline round={round} add-ms={} pnpm-ms={pnpm_millis} staging-left={}",
        added.elapsed.as_millis(),
        slot.join("target.monochromatic-prewarm").exists()
    );
    fixture.clear(name.as_str());
    return added.elapsed.as_millis();
}

/// What:
///  The queued and inline pair,
///  one round each way,
///  and the ratio of the add's own time on each side.
/// Why:
///  The queued add returns before pnpm runs and the inline add waits for it,
///  so the two adds' own times are the measure of what queuing buys,
///  reported as a ratio to the arm beside it in the same round.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (let n = 0; n < ROUNDS; n += 1) { n % 2 === 0 ? queuedThenInline(n) : inlineThenQueued(n); }
/// ```
pub(super) fn the_queued_and_inline_arms(fixture: &Fixture) {
    the_queued_state(fixture);
    let mut queued: Vec<u128> = Vec::new();
    let mut inline: Vec<u128> = Vec::new();
    for round in 0..ROUNDS {
        if round % 2 == 0 {
            let service: TheService = TheService::start(fixture);
            queued.push(queued_after_an_add(fixture, round));
            service.stop();
            inline.push(inline_after_an_add(fixture, round));
        } else {
            inline.push(inline_after_an_add(fixture, round));
            let service: TheService = TheService::start(fixture);
            queued.push(queued_after_an_add(fixture, round));
            service.stop();
        }
        assert!(
            !mounted_at(fixture.slot(format!("queued-{round}").as_str()).as_path()),
            "a slot of round {round} still holds a mount"
        );
    }
    report("queued-vs-inline", inline.as_slice(), queued.as_slice());
}
