//! Controls for the route's chosen names, its request reading and its stop switch.

use super::{
    AddNames,
    STOP_AFTER_VARIABLE,
    STOP_STEPS,
    Stops,
    name_the_add,
    request_of,
};
use crate::worktree_clone_add_route::add_route_gate::RouteFacts;
use crate::worktree_clone_binding_record::binding_encoding::Request;
use crate::worktree_clone_binding_record::binding_words::RequestForm;
use crate::worktree_command_facts::AddPlan;
use std::ffi::OsString;
use std::path::PathBuf;

/// The facts a gate proved for one add at `topic`.
fn facts() -> RouteFacts {
    return RouteFacts {
        source_root: PathBuf::from("/home/user/Monochromatic"),
        common_dir: PathBuf::from("/home/user/Monochromatic/.git"),
        git_dir: PathBuf::from("/home/user/Monochromatic/.git"),
        destination: PathBuf::from("/home/user/Monochromatic/.claude/worktrees/topic"),
        slot: String::from("topic"),
        checkout_dataset: String::from("tank/ROOT/home/monochromatic-git-checkout"),
        container_dataset: String::from("tank/ROOT/home/monochromatic-git-worktree-clone-datasets"),
        pool: String::from("tank"),
    };
}

/// The plan of an add with these operands and these option words before them.
fn plan(target: Option<&str>) -> AddPlan {
    return AddPlan {
        destination: PathBuf::from("/home/user/Monochromatic/.claude/worktrees/topic"),
        insert_at: 2,
        quiet: false,
        target: target.map(OsString::from),
        checks_out_a_branch: true,
    };
}

/// The words an add was given: `worktree add <options> <destination> [<target>]`.
fn words(
    options: &[&str],
    target: Option<&str>,
) -> Vec<OsString> {
    let mut all: Vec<OsString> = vec![OsString::from("worktree"), OsString::from("add")];
    all.extend(options.iter().map(OsString::from));
    all.push(OsString::from(
        "/home/user/Monochromatic/.claude/worktrees/topic",
    ));
    if let Some(named) = target {
        all.push(OsString::from(named));
    }
    return all;
}

#[test]
fn the_names_carry_the_nonce_and_the_designs_forms() {
    let given: AddPlan = plan(None);
    let names: AddNames =
        name_the_add(&facts(), &given, words(&[], None).as_slice()).expect("the names are chosen");
    assert_eq!(names.nonce.as_str().len(), 32);
    assert_eq!(
        names.origin_snapshot,
        format!(
            "tank/ROOT/home/monochromatic-git-checkout@git-worktree-clone-origin-{}",
            names.nonce.as_str()
        )
    );
    assert_eq!(
        names.clone_dataset,
        "tank/ROOT/home/monochromatic-git-worktree-clone-datasets/topic"
    );
    assert_eq!(
        names.request,
        Request::with_argument(RequestForm::Branch, b"topic")
    );
}

#[test]
fn a_named_branch_is_the_request() {
    let given: AddPlan = plan(Some("main"));
    let request: Request = request_of(&given, words(&["-b", "topic-two"], Some("main")).as_slice())
        .expect("the request is read");
    assert_eq!(
        request,
        Request::with_argument(RequestForm::Branch, b"topic-two")
    );
}

#[test]
fn a_commit_ish_without_a_new_branch_is_the_commit_form() {
    let request: Request = request_of(
        &plan(Some("v2.56.1")),
        words(&[], Some("v2.56.1")).as_slice(),
    )
    .expect("the request is read");
    assert_eq!(
        request,
        Request::with_argument(RequestForm::Commit, b"v2.56.1")
    );
}

#[test]
fn a_detached_add_carries_its_target_or_nothing() {
    let with_target: Request = request_of(
        &plan(Some("main")),
        words(&["--detach"], Some("main")).as_slice(),
    )
    .expect("the request is read");
    assert_eq!(
        with_target,
        Request::with_argument(RequestForm::Detach, b"main")
    );
    let without: Request = request_of(&plan(None), words(&["--detach"], None).as_slice())
        .expect("the request is read");
    assert_eq!(without, Request::detached());
    let bundled: Request = request_of(
        &plan(Some("main")),
        words(&["-fd"], Some("main")).as_slice(),
    )
    .expect("a bundled short word is read");
    assert_eq!(
        bundled,
        Request::with_argument(RequestForm::Detach, b"main")
    );
}

#[test]
fn a_new_branch_with_no_value_refuses() {
    let last: Vec<OsString> = vec![
        OsString::from("worktree"),
        OsString::from("add"),
        OsString::from("-b"),
    ];
    let refusal: String =
        request_of(&plan(None), last.as_slice()).expect_err("no value follows -b");
    assert!(refusal.contains("names no branch"), "{refusal}");
}

#[test]
fn the_stop_switch_takes_a_closed_list_of_steps() {
    assert_eq!(
        Stops::named("clone").map(|stops| return stops.step),
        Ok(Some(String::from("clone")))
    );
    let refusal: String =
        Stops::named("clone-the-thing").expect_err("a word the add does not know");
    assert!(refusal.contains("names `clone-the-thing`"), "{refusal}");
    assert!(refusal.contains(STOP_AFTER_VARIABLE), "{refusal}");
    assert_eq!(Stops::none().step, None);
    // A switch for another step does nothing, and a run with no switch does not
    // die either. This is the control that a killed guest arm is caused by the
    // switch and not by the add itself.
    Stops::named("exchange")
        .expect("the step is known")
        .after("clone");
    Stops::none().after("clone");
    assert_eq!(STOP_AFTER_VARIABLE, "CLI_GIT_TEST_ONLY_ZFS_ADD_STOP_AFTER");
    assert!(
        STOP_STEPS.contains(&"exchange"),
        "the guest arm's exchange kill is a step the switch knows"
    );
}
