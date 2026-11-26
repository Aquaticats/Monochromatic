//! Controls for the preflight's own shape:
//!  it makes the delegation read once,
//!  that read answers before the claim is taken,
//!  a removal's plan that still names the clone stops the preflight after the
//!  device opens,
//!  in the stop's own words and with the command that ends the plan,
//!  a checkout whose source the fill's own restore would refuse makes the
//!  preflight decline, so the add takes the ladder and nothing of the route's is
//!  written,
//!  the clone container is refused before any write for each property the design
//!  does not serve,
//!  a source the route serves that names the person's own lock word is refused in
//!  the route's own sentence,
//!  a source the route declines that names one takes the ladder with its words,
//!  a checkout with an ordinary uncommitted change is served,
//!  and a sparse checkout,
//!  a worktree-scope configuration and an unfinished merge still fail the add.
//!
//! Why the source's answers are driven on fixtures:
//!  the source check and the slot's parent stand before the device is opened,
//!  so a fixture decides every answer they give and no control needs a pool.
//!  The plan's stop stands after the device,
//!  so its sentence is controlled in `add_plan_stop_tests.rs`,
//!  and what only the preflight can show is the source read below.
//!  The route's entry point reads this host's mount table and answers no route on
//!  any machine before plan step 19's migration, so the controls start at the
//!  preflight and, in `add_route_run_tests.rs`, at the runner that entry point
//!  calls.
//!
//! Why a source scan for the delegation read:
//!  the preflight opens the kernel's device and reads a live pool,
//!  so no host control can drive it that far,
//!  and this crate's own idiom for the shape a release file must have is
//!  `source_guard`.
//!  The arm fails when the read is dropped from the preflight,
//!  which is the defect the read was built for.
//!
//! Why the plan's stop is driven on fixtures and read in the source:
//!  the command a person reads beside a standing plan needs the clone the pool can
//!  find,
//!  which is a request through the device the preflight opens
//!  (`doc/decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md`,
//!  choice 2C),
//!  so a host control cannot reach the preflight's plan stop at all.
//!  The sentence's own controls stand in `add_plan_stop_tests.rs`,
//!  and this file controls what only the preflight's code can show:
//!  the stop sits after the device opens,
//!  and what the add hands over is the stop's sentence rather than a Rust
//!  `Debug` value (issue 6450).

use super::preflight_the_add;
use super::refuse_a_container_that_cannot_serve_clones;
use super::OpenedDevice;
use crate::doctor_zfs_route::route_test_facts::{self, CONTAINER_DATASET, DatasetBuilder};
use crate::doctor_zfs_route::{DatasetFacts, DatasetRole, Reading};
use crate::test_support::{fixture, git, git_context, remove, repository};
use crate::worktree_clone_add_route::add_git_words::PERSON_LOCK_WORDS;
use crate::worktree_clone_add_route::add_route_steps::StepRefusal;
use crate::worktree_add_plan::AddPlan;
use crate::worktree_clone_add_route::add_route_gate::RouteFacts;
use crate::worktree_clone_binding_record::registration_home::home_folder;
use std::ffi::OsString;
use std::path::PathBuf;

/// What:
///  A main checkout in the shape the route serves,
///  with the add plan of the worktree it would make,
///  and nothing yet written into it.
/// Why:
///  Every control of the source's answer starts from this one checkout,
///  and what a control then puts into it decides which answer the source check gives.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { root, route, plan } = aCheckoutTheRouteServes('preflight-source');
/// ```
fn a_checkout_the_route_serves(name: &str) -> (PathBuf, RouteFacts, AddPlan) {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = std::fs::canonicalize(repository(root.as_path(), "checkout")).expect("the checkout's path");
    let route: RouteFacts = RouteFacts {
        source_root: repo.clone(),
        common_dir: repo.join(".git"),
        git_dir: repo.join(".git"),
        destination: repo.join(".claude").join("worktrees").join("topic"),
        slot: String::from("topic"),
        checkout_dataset: String::from("preflightfixturepool/checkouts/checkout"),
        container_dataset: String::from("preflightfixturepool/clones"),
        pool: String::from("preflightfixturepool"),
    };
    let plan: AddPlan = AddPlan { destination: route.destination.clone(), insert_at: 2, quiet: false, target: Some(OsString::from("main")), checks_out_a_branch: true };
    return (root, route, plan);
}

/// The words of the add these controls drive,
/// which is the form the plan above was read from.
fn the_adds_words(route: &RouteFacts) -> Vec<OsString> {
    return [String::from("worktree"), String::from("add"), route.destination.display().to_string(), String::from("main")].iter().map(OsString::from).collect();
}

/// The preflight makes the delegation read once,
/// inside itself,
/// before the runner takes the claim.
#[test]
fn the_preflight_refuses_through_the_grant_read() {
    let source: &str = include_str!("add_preflight.rs");
    let code: Vec<&str> = source
        .lines()
        .map(str::trim_start)
        .filter(|line| return !line.starts_with("//"))
        .collect();
    let joined: String = code.join("\n");
    assert_eq!(joined.matches("refuse_without_the_decided_grants(").count(), 1, "the preflight makes the delegation read once: {joined}");
    let read: usize = joined.find("refuse_without_the_decided_grants(").expect("the read is in the preflight");
    let preflight_function: usize = joined.find("fn preflight_the_add(").expect("the preflight is a function of this file");
    assert!(preflight_function < read, "the read sits inside the preflight's own function");
    let runner: &str = include_str!("add_route_run.rs");
    let runner_code: Vec<&str> = runner
        .lines()
        .map(str::trim_start)
        .filter(|line| return !line.starts_with("//"))
        .collect();
    let runner_joined: String = runner_code.join("\n");
    let preflight_call: usize = runner_joined.find("preflight_the_add(").expect("the runner calls the preflight");
    let claim_taken: usize = runner_joined.find("take_the_claim(").expect("the runner takes the claim");
    assert!(preflight_call < claim_taken, "the runner calls the preflight before it takes the claim");
}

/// What:
///  The preflight reads a standing plan after it opens the device,
///  and what it hands over is the stop's own sentence,
///  never a Rust `Debug` rendering of it.
/// Why:
///  The command the person reads beside a plan needs the clone the pool can find,
///  so the read stands after `OpenedDevice::open`
///  (`doc/planning/cli-git-zfs-step-10-open-choices.md`,
///  "What 2C costs"),
///  and no host control can drive the preflight that far:
///  the device is the real `/dev/zfs`.
///  The refusal is the whole of what the person gets,
///  so a `Debug` value there would name types instead of the removal that ends
///  the plan (issue 6450),
///  and the stop's return type is the sentence itself, which is what makes that
///  unreachable rather than merely absent today.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(source.indexOf('OpenedDevice::open(')).toBeLessThan(source.indexOf('refuseWhileAPlanStands('));
/// expect(source).not.toContain('{stop:?}');
/// ```
#[test]
fn the_preflight_reads_the_plan_after_the_device_opens_with_the_stops_sentence() {
    let source: &str = include_str!("add_preflight.rs");
    let code: Vec<&str> = source.lines().map(str::trim_start).filter(|line| return !line.starts_with("//")).collect();
    let joined: String = code.join("\n");
    let device_opened: usize = joined.find("OpenedDevice::open(").expect("the preflight opens the device");
    let plan_stop: usize = joined.find("refuse_while_a_plan_stands(").expect("the preflight calls the plan's stop");
    assert!(device_opened < plan_stop, "the plan's stop follows the device it asks the pool with");
    let stop_line: &str = code.iter().find(|line| return line.contains("refuse_while_a_plan_stands(")).expect("the stop's own call line");
    assert!(stop_line.ends_with("refuse_while_a_plan_stands(route, &device)?;"), "the call hands over the stop's own refusal: {stop_line}");
    for file in ["add_preflight.rs", "add_route_run.rs"] {
        let text: &str = match file {
            "add_preflight.rs" => source,
            _ => include_str!("add_route_run.rs"),
        };
        assert!(!text.contains("{stop:?}"), "`{file}` renders no plan stop with a `Debug` value (issue 6450)");
    }
}

/// What:
///  Every sentence the device step answers for this fixture's own dataset names on
///  this machine.
/// Why:
///  The preflight's first request after the source check is the device's own:
///  it runs the start-time gate and reads the checkout's and the pool's identities,
///  so a machine whose `/dev/zfs` does not open answers with the gate's sentence
///  and one where it does answers with the missing dataset's.
///  The fixture's pool name is one no machine holds, so every one of those reads
///  refuses there and the sentence the preflight ends with is one of these,
///  whichever read the preflight makes first.
///  The clone container's read is left out because it stands behind them and its
///  refusal carries words of its own.
fn the_devices_answers_for_the_fixture(route: &RouteFacts) -> Vec<String> {
    let mut answers: Vec<String> = Vec::new();
    let device: OpenedDevice = match OpenedDevice::open() {
        Ok(device) => device,
        Err(message) => {
            answers.push(message);
            return answers;
        }
    };
    for name in [route.checkout_dataset.as_str(), route.pool.as_str()] {
        if let Err(message) = device.identity_of(name) {
            answers.push(message);
        }
    }
    return answers;
}

/// What:
///  A main checkout with one uncommitted file is a source the preflight serves:
///  the source check records the changes instead of refusing them,
///  the refusal the preflight ends with is the device step's own,
///  which stands after the source check,
///  and nothing of the route's own was written.
/// Why:
///  This is the sentence plan step 11 replaces the interim with: until the
///  verified dirty restore landed, an add from a dirty main checkout took rung 3
///  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`, lines 1858 to
///  1862), and the route now restores the copy instead of declining it.
///  The fixture names a pool no machine holds, so the device step is where this
///  add stops on both kinds of machine:
///  the start-time compatibility gate's sentence where `/dev/zfs` does not open
///  (the gate's container),
///  and the missing dataset's where it does (a ZFS host).
///  `StepRefusal` carries the person's words and an exit status, and no step of
///  its own, so the control names the step by asking the device layer the same
///  questions the preflight asks and reading the preflight's sentence among the
///  answers:
///  every one of them is this machine's, written down nowhere,
///  and no refusal of the source check can be one of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const source = sourceOrDirty(context, identity); expect(source.dirty).toBeDefined();
/// expect(preflight(dirtyCheckout).message).not.toContain('cannot be the source of a clone');
/// expect(deviceAnswersFor(fixturePool)).toContain(preflight(dirtyCheckout).message);
/// ```
#[test]
fn an_uncommitted_file_is_a_source_the_preflight_serves() {
    let (root, route, plan) = a_checkout_the_route_serves("preflight-dirty");
    std::fs::write(route.source_root.join("notes.txt"), b"one uncommitted line\n").expect("the uncommitted file");
    let identity: crate::worktree_identity::WorktreeIdentity = crate::worktree_identity::WorktreeIdentity::MainWorktree {
        common_dir: route.common_dir.clone(),
        git_dir: route.git_dir.clone(),
        worktree_root: route.source_root.clone(),
    };
    let source = crate::worktree_snapshot_source::source_or_dirty(&git_context(), &identity).expect("a dirty checkout is a source the route reads");
    assert!(source.dirty.is_some(), "the source check records the changes it will restore away");
    let refusal: StepRefusal = match preflight_the_add(&route, &plan, &git_context(), the_adds_words(&route).as_slice()) {
        Ok(_) => panic!("this fixture has no pool, so the preflight cannot prove the datasets"),
        Err(refusal) => refusal,
    };
    assert!(!refusal.message.contains("cannot be the source of a clone"), "the source check served the dirty checkout, and the preflight refused it for another reason: {}", refusal.message);
    let devices_own: Vec<String> = the_devices_answers_for_the_fixture(&route);
    assert!(devices_own.contains(&refusal.message), "the preflight's refusal is the device step's own, which stands after the source check; the device answers {devices_own:#?}");
    assert!(!home_folder(route.common_dir.as_path()).exists(), "the claim, and the record written after it, stay unwritten");
    assert!(!route.destination.exists(), "the slot stays empty before the claim");
    remove(root.as_path());
}

/// What:
///  An add that names the person's own lock word from a source the route declines
///  takes the ladder with its words,
///  and reads no sentence of a route that is not serving it.
/// Why:
///  The refusal's sentence says the ZFS clone route takes the managed lock itself,
///  which is a rule of that route:
///  the ladder hands the person's words to Git,
///  whose `worktree add` takes the lock and the reason by itself
///  (`SPEC.md`, the ladder's forms).
///  So a source the fill's own restore cannot prove declines before the route's own
///  invocation rules are judged (issue 6453).
///  Every word the constant names stands once on this side,
///  because a refusal judged for one of them ahead of the source's answer would fail
///  here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const word of PERSON_LOCK_WORDS) expect(preflight(nestedRepositoryCheckout, wordsWith(word))).toBeNull();
/// ```
#[test]
fn a_lock_word_from_a_declined_source_takes_the_ladder() {
    for word in PERSON_LOCK_WORDS {
        let given: &str = std::str::from_utf8(word).expect("every lock word is ASCII");
        let (root, route, plan): (PathBuf, RouteFacts, AddPlan) = a_checkout_the_route_serves(format!("preflight-declined-{}", given.trim_start_matches('-')).as_str());
        let nested: PathBuf = route.source_root.join("nested");
        std::fs::create_dir(nested.as_path()).expect("the repository's directory");
        git(nested.as_path(), &["init", "--quiet"]);
        std::fs::write(nested.join("inside.txt"), b"inside\n").expect("the repository's own file");
        let mut words: Vec<OsString> = the_adds_words(&route);
        words.insert(2, OsString::from(given));
        match preflight_the_add(&route, &plan, &git_context(), words.as_slice()) {
            Ok(None) => {}
            Ok(Some(_)) => panic!("this fixture has no pool, so a served source would have been stopped by the pool's read, and `{given}` was not the reason"),
            Err(refusal) => panic!("the route fails an add from a source its own restore cannot prove, in a sentence about `{given}`: {}", refusal.message),
        }
        assert!(!home_folder(route.common_dir.as_path()).exists(), "the claim, and the record written after it, stay unwritten for `{given}`");
        assert!(!route.destination.exists(), "the slot stays empty when the route declines an add given `{given}`");
        remove(root.as_path());
    }
}

/// What:
///  An add that names the person's own lock word from a source the route serves is
///  still refused in the route's own sentence.
/// Why:
///  The control beside this one shows a decline,
///  and this one shows the same question answering a refusal,
///  so the lock rule is not lost with the order:
///  an add the route serves and that locked itself would leave a registration whose
///  words the wrapper's proof cannot read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(preflight(cleanCheckout, wordsWith('--lock'))).toThrow(/takes the managed lock itself/);
/// ```
#[test]
fn a_lock_word_from_a_served_source_keeps_the_routes_own_sentence() {
    let (root, route, plan): (PathBuf, RouteFacts, AddPlan) = a_checkout_the_route_serves("preflight-served-lock-word");
    let mut words: Vec<OsString> = the_adds_words(&route);
    words.insert(2, OsString::from("--lock"));
    let refusal: StepRefusal = match preflight_the_add(&route, &plan, &git_context(), words.as_slice()) {
        Ok(_) => panic!("the route takes the managed lock itself, so an add it serves and that names `--lock` is refused before the claim"),
        Err(refusal) => refusal,
    };
    assert!(refusal.message.contains("takes the managed lock itself"), "the sentence is the route's own: {}", refusal.message);
    assert!(refusal.message.contains("--lock"), "the sentence names the word the person gave: {}", refusal.message);
    assert!(!home_folder(route.common_dir.as_path()).exists(), "a refused add writes nothing, so the claim is not taken");
    remove(root.as_path());
}

/// What:
///  A checkout holding an untracked Git repository of its own makes the preflight
///  decline the whole route, before the claim,
///  so nothing of the route's own is written.
/// Why:
///  The fill's `git clean --force -d` keeps such a repository and the restore's
///  proof then names it as still untracked,
///  so the restore would end after the clone and the origin snapshot were made,
///  and nothing of this release settles a failed restore at a slot
///  (`SPEC.md`, "What this release does not yet do").
///  The ladder settles the same refusal in its own stage and finishes with Git's
///  own checkout.
///  On this machine the pool's read is what stops a served source,
///  so a control that read a refusal here would pass with the defect in place;
///  what it reads instead is the preflight's own decline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(preflight(nestedRepositoryCheckout)).toBeNull();
/// expect(existsSync(homeFolder(commonDir))).toBe(false);
/// ```
#[test]
fn an_untracked_repository_declines_the_preflight() {
    let (root, route, plan) = a_checkout_the_route_serves("preflight-nested-repository");
    let nested: PathBuf = route.source_root.join("nested");
    std::fs::create_dir(nested.as_path()).expect("the repository's directory");
    git(nested.as_path(), &["init", "--quiet"]);
    std::fs::write(nested.join("inside.txt"), b"inside\n").expect("the repository's own file");
    match preflight_the_add(&route, &plan, &git_context(), the_adds_words(&route).as_slice()) {
        Ok(None) => {}
        Ok(Some(_)) => panic!("this fixture has no pool, so a served source would have been stopped by the pool's read"),
        Err(refusal) => panic!("the route fails an add from a source its own restore cannot prove: {}", refusal.message),
    }
    assert!(!home_folder(route.common_dir.as_path()).exists(), "the claim, and the record written after it, stay unwritten");
    assert!(!route.destination.exists(), "the slot stays empty when the route declines");
    remove(root.as_path());
}

/// What:
///  A checkout whose tracked file has become a directory makes the preflight
///  decline the whole route, before the claim.
/// Why:
///  The restore removes the files its reset cannot see as files or links,
///  and a directory cannot be removed that way,
///  so the restore would end after the clone and the origin snapshot were made,
///  with nothing of this release to settle it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(preflight(directoryAtATrackedPath)).toBeNull();
/// expect(existsSync(homeFolder(commonDir))).toBe(false);
/// ```
#[test]
fn a_directory_where_the_index_records_a_file_declines_the_preflight() {
    let (root, route, plan) = a_checkout_the_route_serves("preflight-directory-at-a-file");
    let tracked: PathBuf = route.source_root.join("tracked.txt");
    std::fs::write(tracked.as_path(), b"first\n").expect("the tracked file");
    git(route.source_root.as_path(), &["add", "--", "tracked.txt"]);
    git(route.source_root.as_path(), &["commit", "--quiet", "--message=first"]);
    std::fs::remove_file(tracked.as_path()).expect("the file goes");
    std::fs::create_dir(tracked.as_path()).expect("the directory takes its name");
    std::fs::write(tracked.join("inside.txt"), b"inside\n").expect("a file inside it");
    match preflight_the_add(&route, &plan, &git_context(), the_adds_words(&route).as_slice()) {
        Ok(None) => {}
        Ok(Some(_)) => panic!("this fixture has no pool, so a served source would have been stopped by the pool's read"),
        Err(refusal) => panic!("the route fails an add from a source its own restore cannot prove: {}", refusal.message),
    }
    assert!(!home_folder(route.common_dir.as_path()).exists(), "the claim, and the record written after it, stay unwritten");
    assert!(!route.destination.exists(), "the slot stays empty when the route declines");
    remove(root.as_path());
}

/// What:
///  A sparse checkout still fails the add,
///  in the source check's own sentence,
///  because that is a source no copy reproduces and not a source that is dirty.
/// Why:
///  Only a refusal about uncommitted changes makes the route decline;
///  a decline here would send the add to the ladder for a reason the ladder cannot
///  serve either, and the person would read Git's failure instead of the reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(preflight(sparseCheckout).message).toContain('the source has a sparse checkout');
/// ```
#[test]
fn a_sparse_checkout_still_refuses_the_add() {
    let (root, route, plan) = a_checkout_the_route_serves("preflight-sparse");
    let info: PathBuf = route.common_dir.join("info");
    std::fs::create_dir_all(info.as_path()).expect("the Git info folder");
    std::fs::write(info.join("sparse-checkout"), b"/*\n!/*/\n").expect("the sparse-checkout patterns");
    let refusal: StepRefusal = match preflight_the_add(&route, &plan, &git_context(), the_adds_words(&route).as_slice()) {
        Ok(_) => panic!("a sparse checkout is no source of a copy, so the route refuses it"),
        Err(refusal) => refusal,
    };
    assert!(refusal.message.contains("the source has a sparse checkout"), "the refusal names the sparse checkout: {}", refusal.message);
    assert!(!home_folder(route.common_dir.as_path()).exists(), "a refusal writes nothing either, so the claim is not taken");
    remove(root.as_path());
}

/// What:
///  A worktree-scope configuration still fails the add,
///  because a copy reproduces the repository's configuration and not the one
///  setting Git keeps outside the files it copies.
/// Why:
///  This is the other shape the source check refuses as unsupported,
///  and only uncommitted changes may make the route decline.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(preflight(worktreeScopeConfig).message).toContain('the worktree-specific configuration sets gc.auto');
/// ```
#[test]
fn a_worktree_scope_configuration_still_refuses_the_add() {
    let (root, route, plan) = a_checkout_the_route_serves("preflight-worktree-config");
    git(route.source_root.as_path(), &["config", "extensions.worktreeConfig", "true"]);
    std::fs::write(route.common_dir.join("config.worktree"), b"[gc]\n\tauto = 1\n").expect("the worktree-scope configuration");
    let refusal: StepRefusal = match preflight_the_add(&route, &plan, &git_context(), the_adds_words(&route).as_slice()) {
        Ok(_) => panic!("a worktree-scope configuration is no source of a copy, so the route refuses it"),
        Err(refusal) => refusal,
    };
    assert!(refusal.message.contains("the worktree-specific configuration sets gc.auto"), "the refusal names the setting: {}", refusal.message);
    remove(root.as_path());
}

/// What:
///  A merge left unfinished still fails the add,
///  because an operation in progress is a state a restore cannot undo,
///  so it is not dirt.
/// Why:
///  The code's own line between dirt and every other refusal is
///  `CleanRefusalKind::restorable`,
///  and this control sits on its refused side:
///  without it, a decline for an unfinished merge would leave the person reading
///  Git's own refusal instead of the route's sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(preflight(midMerge).message).toContain('MERGE_HEAD exists in');
/// ```
#[test]
fn an_unfinished_merge_still_refuses_the_add() {
    let (root, route, plan) = a_checkout_the_route_serves("preflight-mid-merge");
    std::fs::write(route.common_dir.join("MERGE_HEAD"), b"0000000000000000000000000000000000000000\n").expect("the merge's mark");
    let refusal: StepRefusal = match preflight_the_add(&route, &plan, &git_context(), the_adds_words(&route).as_slice()) {
        Ok(_) => panic!("an operation in progress is not dirt, so the route refuses the source"),
        Err(refusal) => refusal,
    };
    assert!(refusal.message.contains("the main checkout cannot be the source of a clone"), "the refusal names the source check: {}", refusal.message);
    assert!(refusal.message.contains("MERGE_HEAD exists in"), "the refusal names the unfinished merge: {}", refusal.message);
    remove(root.as_path());
}

/// What:
///  A container the design does not serve is refused in the preflight,
///  for each of the four properties the design names,
///  before any write.
/// Why:
///  A clone inherits the container's `mountpoint`, `setuid` and `devices`,
///  so a container set wrongly gives every clone below it the same fault,
///  and the design refuses it here rather than after the clone exists
///  (`doc/decision/cli-git-zfs-clone-inherits-legacy-from-the-container.md`).
///  The sentence must name the container, the property and the value it needs,
///  because that is the whole of what a person acts on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const wrong of [notLegacy, canmountOn, setuidOn, devicesOn]) expect(() => refuseContainer(wrong)).toThrow(/clone container/);
/// ```
#[test]
fn a_container_the_design_does_not_serve_is_refused_before_any_write() {
    let cases: [(crate::worktree_clone_mount_handler::sweep_dataset_facts::CloneObjectFacts, &str, &str); 4] = [
        (DatasetBuilder::container().mountpoint("/srv/elsewhere").build(), "`mountpoint` is `/srv/elsewhere`", "the clone container must be `legacy`"),
        (DatasetBuilder::container().canmount(1).build(), "`canmount` is 1", "the container must be off"),
        (DatasetBuilder::container().setuid(1).build(), "`setuid` is 1", "`setuid` must be off"),
        (DatasetBuilder::container().devices(1).build(), "`devices` is 1", "`devices` must be off"),
    ];
    for (facts, found, required) in cases {
        let container: DatasetFacts = route_test_facts::dataset(DatasetRole::Container, facts);
        let refusal: StepRefusal = refuse_a_container_that_cannot_serve_clones(&container).expect_err("a container the design does not serve is refused");
        assert!(refusal.message.contains(CONTAINER_DATASET), "the container is named: {}", refusal.message);
        assert!(refusal.message.contains(found), "the property and its value are named: {}", refusal.message);
        assert!(refusal.message.contains(required), "the value the design needs is named: {}", refusal.message);
        assert!(refusal.message.contains("No claim, slot, Git registration, snapshot or clone was written"), "the sentence says where the refusal stands: {}", refusal.message);
    }
}

/// What:
///  The container the design serves passes the same check,
///  which is what keeps the refusal from passing every container.
/// Why:
///  A check that refused everything would satisfy the control above,
///  so the accepted shape is read here beside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(refuseContainer(healthyContainer)).toBeUndefined();
/// ```
#[test]
fn a_container_the_design_serves_passes_the_check() {
    let container: DatasetFacts = route_test_facts::dataset(DatasetRole::Container, DatasetBuilder::container().build());
    assert!(refuse_a_container_that_cannot_serve_clones(&container).is_ok(), "the design's own container passes");
    let unreadable: DatasetFacts = DatasetFacts { role: DatasetRole::Container, name: String::from(CONTAINER_DATASET), reading: Reading::Unreadable { why: String::from("permission denied") } };
    let refusal: StepRefusal = refuse_a_container_that_cannot_serve_clones(&unreadable).expect_err("a container this account cannot read proves nothing and is refused");
    assert!(refusal.message.contains("permission denied"), "the read's own answer is named: {}", refusal.message);
}

/// What:
///  The preflight itself judges the container,
///  inside its own function,
///  before it reads the delegation.
/// Why:
///  The check is only worth its sentence if the preflight runs it,
///  and the preflight opens the kernel's device to reach it,
///  so the shape a release file must have is read from the source,
///  as the grant read's control beside this one does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(source.match(/refuseAContainer/g)).toHaveLength(2); // the definition and one call
/// expect(source.indexOf('refuseAContainer(')).toBeLessThan(source.indexOf('refuseWithoutTheDecidedGrants('));
/// ```
#[test]
fn the_preflight_judges_the_container_before_it_reads_the_grants() {
    let source: &str = include_str!("add_preflight.rs");
    let code: Vec<&str> = source
        .lines()
        .map(str::trim_start)
        .filter(|line| return !line.starts_with("//"))
        .collect();
    let joined: String = code.join("\n");
    assert_eq!(joined.matches("refuse_a_container_that_cannot_serve_clones(").count(), 2, "the container check is defined once and called once: {joined}");
    let preflight_function: usize = joined.find("fn preflight_the_add(").expect("the preflight is a function of this file");
    let call: usize = preflight_function + joined[preflight_function..].find("refuse_a_container_that_cannot_serve_clones(").expect("the call is in the preflight");
    let container_read: usize = joined.find("facts_of(route.container_dataset.as_str())").expect("the preflight reads the container's facts");
    assert!(container_read < call, "the check judges the reply the preflight read");
    let grants: usize = joined.find("refuse_without_the_decided_grants(").expect("the grant read is in the preflight");
    assert!(call < grants, "the container is judged before the delegation is read, so a wrong container is refused without a grant answer a person would have to read twice");
}
