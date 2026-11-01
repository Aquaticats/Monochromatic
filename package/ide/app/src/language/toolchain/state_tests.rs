//! The notice text and the lookup facts of every state.

use super::{Failed, Provided, ToolchainState};
use std::path::{Path, PathBuf};

fn provided() -> ToolchainState {
    return ToolchainState::Provided(Provided {
        directories: vec![PathBuf::from("/tools/node/bin")],
        warnings: "the lookup warns about the machine".to_string(),
    });
}

#[test]
fn only_a_project_with_a_toolchain_is_served() {
    assert!(provided().serves_project());
    assert!(
        ToolchainState::Failed(Failed {
            message: "x".to_string()
        })
        .serves_project()
    );
    assert!(!ToolchainState::Disabled.serves_project());
    assert!(!ToolchainState::Unused("no node".to_string()).serves_project());
}

#[test]
fn directories_and_warnings_exist_only_when_the_toolchain_provided_them() {
    assert_eq!(provided().directories(), [PathBuf::from("/tools/node/bin")]);
    assert_eq!(provided().warnings(), "the lookup warns about the machine");
    for state in [
        ToolchainState::Disabled,
        ToolchainState::Unused("x".to_string()),
        ToolchainState::Failed(Failed {
            message: "x".to_string(),
        }),
    ] {
        assert!(state.directories().is_empty());
        assert_eq!(state.warnings(), "");
    }
}

#[test]
fn a_failure_is_named_quoted_and_left_to_the_user_to_fix() {
    let state = ToolchainState::Failed(Failed {
        message: "Error: toolchain.lock.json is unreadable".to_string(),
    });
    let paragraphs = state.advisory(Path::new("/work/project one"));
    assert_eq!(paragraphs.len(), 3);
    assert!(
        paragraphs[0].contains("/work/project one"),
        "{paragraphs:?}"
    );
    assert!(
        paragraphs[0].contains("looked up on PATH only"),
        "{paragraphs:?}"
    );
    assert!(
        paragraphs[1].contains("mpm-shadow bin --json"),
        "{paragraphs:?}"
    );
    assert!(
        paragraphs[1].contains("at the worktree root"),
        "{paragraphs:?}"
    );
    assert!(
        !paragraphs[1].contains("the IDE never"),
        "the notice promised an IDE-side fix: {paragraphs:?}"
    );
    assert_eq!(
        paragraphs[2],
        "the toolchain lookup said: Error: toolchain.lock.json is unreadable"
    );
}

#[test]
fn a_failure_that_stopped_the_run_points_at_the_run() {
    let state = ToolchainState::Failed(Failed {
        message: "`mpm-shadow bin --json` did not finish within 10 s and was stopped"
            .to_string(),
    });
    let paragraphs = state.advisory(Path::new("/work/project"));
    assert_eq!(paragraphs.len(), 3);
    assert!(
        paragraphs[2].contains("did not finish within 10 s"),
        "{paragraphs:?}"
    );
    assert!(paragraphs[1].contains("bin --json"), "{paragraphs:?}");
}

#[test]
fn a_sandbox_that_stopped_before_the_lookup_is_not_blamed_on_the_toolchain() {
    let state = ToolchainState::Failed(Failed {
        message: "bwrap: Can't bind mount /oldroot/cache/proj-1/tmp on /newroot/tmp: Unable to mount source on destination: No such file or directory"
            .to_string(),
    });
    let paragraphs = state.advisory(Path::new("/work/project"));
    assert_eq!(paragraphs.len(), 3);
    assert!(paragraphs[0].contains("/work/project"), "{paragraphs:?}");
    assert!(
        paragraphs[0].contains("sandbox that runs the toolchain lookup"),
        "the notice blamed the toolchain for a sandbox that stopped: {paragraphs:?}"
    );
    assert!(
        paragraphs[0].contains("not the cause"),
        "the notice blamed the toolchain for a sandbox that stopped: {paragraphs:?}"
    );
    assert!(
        !paragraphs
            .iter()
            .any(|paragraph| return paragraph.contains("bin --json")
                || paragraph.contains("toolchain lookup said")),
        "the notice blamed the toolchain for a sandbox that stopped: {paragraphs:?}"
    );
    assert!(paragraphs[1].contains("bwrap --version"), "{paragraphs:?}");
    assert!(
        paragraphs[1].contains("monochromatic-ide/language"),
        "{paragraphs:?}"
    );
    assert!(
        paragraphs[2].starts_with("bubblewrap said: bwrap: Can't bind mount"),
        "{paragraphs:?}"
    );
}

#[test]
fn only_a_message_that_begins_with_bubblewraps_prefix_is_the_sandbox() {
    for (message, sandbox) in [
        ("bwrap: No permissions to create new namespace", true),
        ("  bwrap: leading blanks", true),
        (
            "node:internal/modules/cjs/loader: Cannot find module",
            false,
        ),
        (
            "`mpm-shadow bin --json` did not finish within 10 s and was stopped",
            false,
        ),
        ("node: the word bwrap: appears later", false),
    ] {
        let failed = Failed {
            message: message.to_string(),
        };
        assert_eq!(failed.stopped_by_sandbox(), sandbox, "{message}");
    }
}

#[test]
fn no_notice_is_given_when_the_lookup_worked_or_did_not_apply() {
    for state in [
        provided(),
        ToolchainState::Disabled,
        ToolchainState::Unused("the project has no toolchain plan".to_string()),
    ] {
        assert!(state.advisory(Path::new("/work/project")).is_empty());
    }
}
