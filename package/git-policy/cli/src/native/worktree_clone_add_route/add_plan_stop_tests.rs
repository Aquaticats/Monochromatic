//! What:
//!  Controls for the stop a standing removal plan leaves the add,
//!  one for every state the plan's head word and the pool's answer make.
//! Why:
//!  A person who meets a standing plan must read the one command that ends it,
//!  so each state's sentence is asserted by the command words that state's
//!  facts give,
//!  and the cancel command is asserted beside it only where the plan records
//!  nothing destroyed
//!  (`doc/decision/cli-git-zfs-step-10-reach-plan-text-grants-and-migration-timing.md`,
//!  choice 2C;
//!  `doc/planning/cli-git-zfs-add-binding-record-design.md`, `V24`).
//!  The pool is scripted because no control on a person's machine may open
//!  `/dev/zfs`,
//!  and the plan is a fixture file under the clone's own GUID,
//!  which is the name the plan's grammar gives it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const stop = refuseWhileAPlanStands(route, probe); // throws with the command for this state
//! ```

use super::refuse_while_a_plan_stands;
use crate::worktree_clone_add_route::add_route_gate::RouteFacts;
use crate::worktree_clone_binding_record::plan_folder_read::PLAN_HEADER;
use crate::worktree_clone_binding_record::registration_home::plans_folder;
use crate::worktree_clone_mount_handler::sweep_dataset_reader::DatasetReader;
use crate::worktree_clone_record_settling::settling_fixtures::{ScriptedProbe, a_guid, an_identity};
use std::path::PathBuf;

/// The slot every fixture add fills.
const SLOT: &str = "topic";

/// The container the fixture clone is made in.
const CONTAINER: &str = "tank/worktree-clones";

/// The clone the fixture add would make,
/// which is the container's child named by the slot.
const CLONE: &str = "tank/worktree-clones/topic";

/// The GUID the fixture plan records for that clone.
const CLONE_GUID: u64 = 4_821_193_048_571_903_102;

/// The plan's first attempt line,
/// which is the line a plan's first write records for one snapshot.
const A_PLANNED_ATTEMPT: &str = "tuple=snapshot 0 planned 9021019283746501 41 - name=tank/worktree-clones/topic@origin";

/// An attempt line whose outcome says the snapshot was destroyed,
/// which is what a plan that witnesses a destroy holds.
const A_DESTROYED_ATTEMPT: &str = "tuple=snapshot 1 destroyed 9021019283746501 41 - name=tank/worktree-clones/topic@origin";

/// The words the settling command's line always carries.
const THE_SETTLING_WORDS: &str = "settle-worktree-clone-record";

/// The words the removal's line always carries.
const THE_REMOVAL_WORDS: &str = "resumes from the plan and ends it";

/// The words the finish pass's wait line always carries.
const THE_WAIT_WORDS: &str = "wait: the pool cannot prove the clone's absence";

/// The words the cancel command's clause carries.
const THE_CANCEL_WORDS: &str = "cancel-worktree-removal";

/// One add's route,
/// with its plan folder made under a fixture's common directory and no plan in it.
fn a_route(name: &str) -> RouteFacts {
    let root: PathBuf = crate::test_support::fixture(name);
    let common: PathBuf = root.join(".git");
    std::fs::create_dir_all(common.as_path()).expect("the common directory");
    std::fs::create_dir_all(plans_folder(common.as_path()).as_path()).expect("the plan folder");
    return RouteFacts {
        source_root: root.clone(),
        common_dir: common,
        git_dir: root.join(".git"),
        destination: root.join(".claude").join("worktrees").join(SLOT),
        slot: String::from(SLOT),
        checkout_dataset: String::from("tank/checkouts/checkout"),
        container_dataset: String::from(CONTAINER),
        pool: String::from("tank"),
    };
}

/// One plan written into a route's plan folder,
/// under the name the plan's grammar gives it,
/// which is the clone's canonical GUID.
fn write_plan(route: &RouteFacts, clone: &str, head: &str, attempts: &[&str]) -> PathBuf {
    let mut text: String = format!("{PLAN_HEADER}\ncloneDataset={clone}\ncloneGuid={CLONE_GUID}\nhead={head}\nattempts={}\n", attempts.len());
    for attempt in attempts {
        text.push_str(attempt);
        text.push('\n');
    }
    let path: PathBuf = plans_folder(route.common_dir.as_path()).join(a_guid(CLONE_GUID).file_name());
    std::fs::write(path.as_path(), text.as_bytes()).expect("the fixture plan");
    return path;
}

/// One add's route with a plan naming this add's clone already written.
fn a_route_with_a_plan(name: &str, head: &str, attempts: &[&str]) -> RouteFacts {
    let route: RouteFacts = a_route(name);
    write_plan(&route, CLONE, head, attempts);
    return route;
}

/// The sentence the add stops with,
/// which a control reads instead of the add's exit status.
fn the_stop(route: &RouteFacts, probe: &dyn DatasetReader) -> String {
    return match refuse_while_a_plan_stands(route, probe) {
        Ok(()) => panic!("the fixture's plan names this add's clone, so the add stops"),
        Err(refusal) => refusal.message,
    };
}

/// A pool whose clone answers under its own name,
/// which is the state every live clone is in.
fn the_clone_stands() -> ScriptedProbe {
    return ScriptedProbe::answering(vec![(CLONE, an_identity(CLONE_GUID, 7))]);
}

/// A pool proving the absence,
/// which is what a container without the clone answers.
fn the_clone_is_proved_gone() -> ScriptedProbe {
    return ScriptedProbe::empty().listing_children(&[&format!("{CONTAINER}/other")]);
}

/// A pool that could not prove the absence,
/// which is what a refused listing answers.
fn the_absence_is_unproved() -> ScriptedProbe {
    return ScriptedProbe::empty().refusing_the_listing("the pool answered EIO");
}

/// The five head words a plan's grammar allows,
/// which every state control walks.
const EVERY_HEAD_WORD: [&str; 5] = ["present", "requested", "gone", "refused", "unknown"];

/// A clone that stands is the removal's state whatever the plan's head word says,
/// because the plan's clone is the same clone however the demand was recorded.
#[test]
fn a_clone_that_stands_names_the_removal_for_every_head_word() {
    for head in EVERY_HEAD_WORD {
        let route: RouteFacts = a_route_with_a_plan(format!("stop-stands-{head}").as_str(), head, &[A_PLANNED_ATTEMPT]);
        let sentence: String = the_stop(&route, &the_clone_stands());
        assert!(sentence.contains(THE_REMOVAL_WORDS), "a clone that stands names the removal beside head `{head}`: {sentence}");
        assert!(!sentence.contains(THE_SETTLING_WORDS), "and never the settling command: {sentence}");
    }
}

/// A proved absence names the settling command where the plan's head still
/// stands,
/// and the removal where the head was already gone.
#[test]
fn a_proved_absence_names_the_command_the_head_word_gives() {
    for head in EVERY_HEAD_WORD {
        let route: RouteFacts = a_route_with_a_plan(format!("stop-gone-{head}").as_str(), head, &[A_PLANNED_ATTEMPT]);
        let sentence: String = the_stop(&route, &the_clone_is_proved_gone());
        let settling: bool = matches!(head, "present" | "refused");
        assert_eq!(sentence.contains(THE_SETTLING_WORDS), settling, "head `{head}` names the settling command only where the head still stands: {sentence}");
        assert_eq!(sentence.contains(THE_REMOVAL_WORDS), !settling, "and the removal for the other three words: {sentence}");
    }
}

/// An absence the pool could not prove names the settling command for every head
/// word but `gone`,
/// whose route is the finish pass and which names no command to run.
#[test]
fn an_unproved_absence_names_the_settling_command_unless_the_head_is_gone() {
    for head in EVERY_HEAD_WORD {
        let route: RouteFacts = a_route_with_a_plan(format!("stop-unproved-{head}").as_str(), head, &[A_PLANNED_ATTEMPT]);
        let sentence: String = the_stop(&route, &the_absence_is_unproved());
        if head == "gone" {
            assert!(sentence.contains(THE_WAIT_WORDS), "a `gone` head the pool cannot prove is the finish pass's wait line: {sentence}");
            assert!(!sentence.contains(THE_SETTLING_WORDS) && !sentence.contains(THE_REMOVAL_WORDS), "and names no command to run: {sentence}");
            continue;
        }
        assert!(sentence.contains(THE_SETTLING_WORDS), "head `{head}` with an unproved absence names the settling command: {sentence}");
        assert!(sentence.contains("the pool answered EIO"), "and the pool's own words are the evidence: {sentence}");
    }
}

/// A recorded name that answers another clone is the state no command settles,
/// so the sentence names the records a person reads and no command.
#[test]
fn a_reused_name_names_the_records_and_no_command() {
    let route: RouteFacts = a_route_with_a_plan("stop-reused", "present", &[A_PLANNED_ATTEMPT]);
    let probe: ScriptedProbe = ScriptedProbe::answering(vec![(CLONE, an_identity(CLONE_GUID + 1, 7))]);
    let sentence: String = the_stop(&route, &probe);
    assert!(sentence.contains("no command of this release settles the state"), "{sentence}");
    assert!(sentence.contains("doctor"), "the reader's next step is named: {sentence}");
    assert!(sentence.contains(&(CLONE_GUID + 1).to_string()), "the GUID that took the name is named: {sentence}");
    for command in [THE_SETTLING_WORDS, THE_REMOVAL_WORDS, THE_WAIT_WORDS] {
        assert!(!sentence.contains(command), "no command is named beside a reused name (`{command}`): {sentence}");
    }
    assert!(!sentence.contains(THE_CANCEL_WORDS), "and no clause says a worktree the name does not hold can be kept: {sentence}");
}

/// A plan whose every attempt line still says `planned` and whose head left the
/// clone standing is the one state that names the cancel command too.
#[test]
fn a_plan_witnessing_no_destroy_names_the_cancel_command() {
    for head in ["present", "refused"] {
        let route: RouteFacts = a_route_with_a_plan(format!("stop-cancel-{head}").as_str(), head, &[A_PLANNED_ATTEMPT, A_PLANNED_ATTEMPT]);
        let sentence: String = the_stop(&route, &the_clone_is_proved_gone());
        assert!(sentence.contains(THE_CANCEL_WORDS), "head `{head}` over planned attempts names the cancel command: {sentence}");
        assert!(sentence.contains(SLOT), "the cancel command names the slot: {sentence}");
    }
}

/// A plan holding one attempt that left `planned` witnesses a destroy,
/// so the cancel command is not named,
/// and neither is it beside a head word that gave up the clone or could not be
/// attributed.
#[test]
fn a_plan_witnessing_a_destroy_names_no_cancel_command() {
    let destroyed: RouteFacts = a_route_with_a_plan("stop-cancel-destroyed", "present", &[A_PLANNED_ATTEMPT, A_DESTROYED_ATTEMPT]);
    let sentence: String = the_stop(&destroyed, &the_clone_is_proved_gone());
    assert!(!sentence.contains(THE_CANCEL_WORDS), "an attempt past `planned` names no cancel command: {sentence}");
    for head in ["requested", "gone", "unknown"] {
        let route: RouteFacts = a_route_with_a_plan(format!("stop-cancel-head-{head}").as_str(), head, &[A_PLANNED_ATTEMPT]);
        let sentence: String = the_stop(&route, &the_clone_stands());
        assert!(!sentence.contains(THE_CANCEL_WORDS), "head `{head}` names no cancel command: {sentence}");
    }
}

/// A plan this add cannot read keeps the refusal it always gave,
/// which names no command at all,
/// because nothing about such a plan is proved.
#[test]
fn a_plan_the_add_cannot_read_names_no_command() {
    let route: RouteFacts = a_route("stop-unreadable");
    let plan: PathBuf = write_plan(&route, CLONE, "present", &[A_PLANNED_ATTEMPT]);
    std::fs::write(plan.as_path(), format!("{PLAN_HEADER}\ncloneDataset={CLONE}\ncloneGuid={CLONE_GUID}\n").as_bytes()).expect("the plan without a head line");
    let sentence: String = the_stop(&route, &the_clone_stands());
    assert!(sentence.contains("was not read as a plan"), "the refusal names what the plan failed to say: {sentence}");
    assert!(sentence.contains("head="), "and names the field it could not read: {sentence}");
    for command in [THE_SETTLING_WORDS, THE_REMOVAL_WORDS, THE_CANCEL_WORDS, THE_WAIT_WORDS] {
        assert!(!sentence.contains(command), "an unread plan names no command (`{command}`): {sentence}");
    }
}

/// A head word outside the plan design's five is a plan this add cannot read,
/// which is the same refusal rather than a guess at the command.
#[test]
fn a_head_word_outside_the_five_refuses_the_plan() {
    let route: RouteFacts = a_route_with_a_plan("stop-head-word", "finished", &[A_PLANNED_ATTEMPT]);
    let sentence: String = the_stop(&route, &the_clone_stands());
    assert!(sentence.contains("was not read as a plan"), "{sentence}");
    assert!(sentence.contains("`finished`"), "the word the plan holds is named: {sentence}");
}

/// A folder holding no plan naming this clone lets the add go on,
/// which is every add that meets no removal.
#[test]
fn a_folder_with_no_plan_for_this_clone_lets_the_add_go_on() {
    let empty: RouteFacts = a_route("stop-no-plan");
    assert!(refuse_while_a_plan_stands(&empty, &the_clone_stands()).is_ok(), "no plan naming the clone stops nothing");
    let other: RouteFacts = a_route("stop-other-plan");
    write_plan(&other, "tank/worktree-clones/other", "present", &[A_PLANNED_ATTEMPT]);
    assert!(refuse_while_a_plan_stands(&other, &the_clone_stands()).is_ok(), "another clone's plan stops nothing");
}

/// The sentence names the plan,
/// the head word,
/// the GUID and the pool's answer,
/// so a person can tell two standing plans apart.
#[test]
fn the_sentence_names_the_plan_the_head_and_the_pool_answer() {
    let route: RouteFacts = a_route_with_a_plan("stop-sentence", "requested", &[A_PLANNED_ATTEMPT]);
    let sentence: String = the_stop(&route, &the_clone_is_proved_gone());
    let plan: PathBuf = plans_folder(route.common_dir.as_path()).join(a_guid(CLONE_GUID).file_name());
    assert!(sentence.contains(plan.display().to_string().as_str()), "the plan's own path is named: {sentence}");
    assert!(sentence.contains("`requested`"), "the head word is named: {sentence}");
    assert!(sentence.contains(CLONE_GUID.to_string().as_str()), "the plan's clone GUID is named: {sentence}");
    assert!(sentence.contains("no child of the clone container carries that GUID"), "the pool's answer is named: {sentence}");
}
