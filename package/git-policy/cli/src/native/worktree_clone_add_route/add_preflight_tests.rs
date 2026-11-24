//! Controls for the preflight's own shape:
//!  it makes the delegation read once,
//!  that read answers before the claim is taken,
//!  and a removal's plan that still names the clone refuses the preflight
//!  in the plan stop's own sentence.
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

use super::preflight_the_add;
use crate::test_support::{fixture, git_context, remove, repository};
use crate::worktree_clone_add_route::add_route_steps::StepRefusal;
use crate::worktree_add_plan::AddPlan;
use crate::worktree_clone_add_route::add_route_gate::RouteFacts;
use crate::worktree_clone_binding_record::plan_folder_read::{PLAN_HEADER, PlanStop, refuse_while_a_plan_names_the_clone};
use crate::worktree_clone_binding_record::registration_home::plans_folder;
use std::ffi::OsString;
use std::path::PathBuf;

/// The plan's own file name:
///  the clone's canonical numeric GUID,
///  which is the plan's key and the only name this reader reads.
const THE_PLANS_KEY: &str = "4821193048571903102";

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
    let folder: PathBuf = plans_folder(route.common_dir.as_path());
    std::fs::create_dir_all(folder.as_path()).expect("the plan folder");
    let plan_text: String = format!("{PLAN_HEADER}\ncloneDataset={}\n", route.clone_dataset());
    std::fs::write(folder.join(THE_PLANS_KEY), plan_text.as_bytes()).expect("the standing plan");
    let plan: AddPlan = AddPlan { destination: route.destination.clone(), insert_at: 2, quiet: false, target: Some(OsString::from("main")), checks_out_a_branch: true };
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
