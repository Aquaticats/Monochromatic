//! Controls for the preflight's own shape:
//!  it makes the delegation read once,
//!  that read answers before the claim is taken,
//!  a removal's plan that still names the clone refuses the preflight
//!  in the plan stop's own sentence,
//!  and a checkout refused only for its uncommitted changes makes the preflight
//!  decline, so the add takes the ladder's rung 3 as the owner's interim says
//!  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
//!  lines 1858 to 1862),
//!  while every other refusal of the source still fails the add.
//!
//! Why the source's answers are driven on fixtures:
//!  the source check, the plan read and the slot's parent stand before the device
//!  is opened, so a fixture decides every answer they give and no control needs a
//!  pool.
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
//! Why the plan's refusal is driven instead:
//!  the plan read sits before the device is opened,
//!  so a checkout whose source the preflight accepts reaches it on any machine,
//!  and what a person reads is then the control's own subject
//!  (issue 6450: the add printed a Rust `Debug` value here).

use super::{Preflight, preflight_the_add};
use crate::test_support::{fixture, git, git_context, remove, repository};
use crate::worktree_clone_add_route::add_route_steps::StepRefusal;
use crate::worktree_add_plan::AddPlan;
use crate::worktree_clone_add_route::add_route_gate::RouteFacts;
use crate::worktree_clone_binding_record::plan_folder_read::{PLAN_HEADER, PlanStop, refuse_while_a_plan_names_the_clone};
use crate::worktree_clone_binding_record::registration_home::{home_folder, plans_folder};
use std::ffi::OsString;
use std::path::PathBuf;

/// The plan's own file name:
///  the clone's canonical numeric GUID,
///  which is the plan's key and the only name this reader reads.
const THE_PLANS_KEY: &str = "4821193048571903102";

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
        checkout_dataset: String::from("tank/checkouts/checkout"),
        container_dataset: String::from("tank/clones"),
        pool: String::from("tank"),
    };
    let plan: AddPlan = AddPlan { destination: route.destination.clone(), insert_at: 2, quiet: false, target: Some(OsString::from("main")), checks_out_a_branch: true };
    return (root, route, plan);
}

/// The words of the add these controls drive,
/// which is the form the plan above was read from.
fn the_adds_words(route: &RouteFacts) -> Vec<OsString> {
    return [String::from("worktree"), String::from("add"), route.destination.display().to_string(), String::from("main")].iter().map(OsString::from).collect();
}

/// What:
///  A clean main checkout with one plan standing that names the clone this add would fill.
/// Why:
///  The plan read sits after the source's checks,
///  so a control reaches the refusal only with a source the preflight accepts,
///  and the plan's field names the clone's dataset because that is the name the add passes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { root, route, plan } = aCheckoutWithAStandingPlan('preflight-plan');
/// ```
fn a_checkout_with_a_standing_plan(name: &str) -> (PathBuf, RouteFacts, AddPlan) {
    let (root, route, plan): (PathBuf, RouteFacts, AddPlan) = a_checkout_the_route_serves(name);
    let folder: PathBuf = plans_folder(route.common_dir.as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the plan folder");
    let plan_text: String = format!("{PLAN_HEADER}\ncloneDataset={}\n", route.clone_dataset());
    std::fs::write(folder.join(THE_PLANS_KEY), plan_text.as_bytes()).expect("the standing plan");
    return (root, route, plan);
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
///  A plan that still names the clone stops the preflight,
///  and the words the person reads are that stop's own sentence,
///  never a Rust `Debug` rendering of it.
/// Why:
///   The refusal is the whole of what the person gets,
///  so a `Debug` value there names types instead of the removal that ends the plan
///  (issue 6450).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(preflight(standingPlan).message).toBe(stop.sentence());
/// ```
#[test]
fn a_standing_plan_refuses_the_preflight_in_the_plans_own_sentence() {
    let (root, route, plan) = a_checkout_with_a_standing_plan("preflight-plan-sentence");
    let stop: PlanStop = refuse_while_a_plan_names_the_clone(route.common_dir.as_path(), None, route.clone_dataset().as_str()).expect_err("the standing plan stops the read");
    let sentence: &str = stop.sentence();
    let arguments: Vec<OsString> = [String::from("worktree"), String::from("add"), route.destination.display().to_string(), String::from("main")].iter().map(OsString::from).collect();
    let refusal: StepRefusal = match preflight_the_add(&route, &plan, &git_context(), arguments.as_slice()) {
        Ok(_) => panic!("the standing plan stops the preflight before its first mutation"),
        Err(refusal) => refusal,
    };
    let message: &str = refusal.message.as_str();
    assert_eq!(message, sentence, "the add prints the plan stop's own sentence: {message}");
    assert!(!message.contains("NamesTheClone") && !message.contains("PlanNamesTheClone"), "no `Debug` rendering of the stop reaches the person: {message}");
    remove(root.as_path());
}

/// What:
///  A main checkout with one uncommitted file makes the preflight decline,
///  and the decline stands before every write the route makes.
/// Why:
///  The owner's interim of 2026-11-19 is that an add from a dirty main checkout
///  takes rung 3 while plan step 11 is unlanded
///  (`doc/planning/cli-git-zfs-worktree-implementation-plan.md`,
///  lines 1858 to 1862).
///  Refusing the source failed the add instead of leaving it to the ladder
///  (issue 6452).
///  The claim is the route's first write and the record's first version follows
///  it, so a record home that was never made proves no claim, and with it no
///  record, no snapshot, no clone and no mount request.
///  The one write the source check makes is the repository's own exclude line for
///  the trash place, which every ladder add makes too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(preflight(dirtyCheckout)).toBeNull();
/// expect(existsSync(homeFolder(commonDir))).toBe(false);
/// ```
#[test]
fn an_uncommitted_file_makes_the_preflight_decline_before_the_claim() {
    let (root, route, plan) = a_checkout_the_route_serves("preflight-dirty");
    std::fs::write(route.source_root.join("notes.txt"), b"one uncommitted line\n").expect("the uncommitted file");
    let answer: Result<Option<Preflight>, StepRefusal> = preflight_the_add(&route, &plan, &git_context(), the_adds_words(&route).as_slice());
    match answer {
        Ok(None) => {}
        Ok(Some(_)) => panic!("a checkout with uncommitted changes is no source of a clone until plan step 11 lands, so the preflight must decline and the add must take the ladder"),
        Err(refusal) => panic!("the route fails the add instead of leaving it to the ladder: {}", refusal.message),
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
