//! What:
//!  Controls for the invocations that must stop before any Git process is chosen or trusted.
//! Why:
//!  A wrapper selected as real Git,
//!  a missing real Git and uninterpretable identity
//!      output must each stop with their evidence instead of forwarding or guessing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(planInvocation(['status'], { CLI_GIT_NATIVE_FORWARD_TARGET: self }, inputs).kind).toBe('exit');
//! ```
#![cfg(unix)]

/// Import the decision under test,
///  the fixtures and the helpers shared with the forwarding controls.
use super::plan_invocation;
use super::tests::{forward, inputs, stopped, text};
use crate::child_environment::FORWARD_TARGET_VARIABLE;
use crate::real_git::ResolutionInputs;
use crate::test_support::{REAL_GIT, executable, fixture, remove};
use std::ffi::OsString;
use std::path::PathBuf;

/// A wrapper named as the forward target by another wrapper refuses to forward;
///  other targets do not trigger.
#[test]
fn forward_target_naming_this_executable_stops() {
    let root: PathBuf = fixture("entry-handshake");
    let resolution: ResolutionInputs = inputs(root.as_path());
    let link: PathBuf = root.join("link-to-own");
    std::os::unix::fs::symlink(&resolution.own_executable, &link).expect("symlink");
    for target in [resolution.own_executable.clone(), link] {
        let environment: Vec<(OsString, OsString)> = vec![(
            OsString::from(FORWARD_TARGET_VARIABLE),
            target.into_os_string(),
        )];
        // Every kind of invocation stops, including the management namespace.
        for arguments in [
            vec!["--version"],
            vec!["add", "x"],
            vec!["monochromatic-git-policy-cli", "--help"],
        ] {
            assert_eq!(
                stopped(plan_invocation(
                    text(arguments.as_slice()).as_slice(),
                    environment.as_slice(),
                    &resolution
                )),
                format!(
                    "monochromatic-git-policy-cli: another monochromatic-git-policy-cli wrapper selected this monochromatic-git-policy-cli executable ({}) as \
                     real Git. Forwarding again would never reach Git. Remove the extra monochromatic-git-policy-cli \
                     from PATH or place the real Git executable on PATH.\n",
                    resolution.own_executable.display()
                ),
                "{arguments:?}"
            );
        }
    }
    // A marker naming real Git (inherited by a hook), a missing path or an empty value is not this executable.
    for target in [REAL_GIT, "/nonexistent/git", ""] {
        let environment: Vec<(OsString, OsString)> = vec![(
            OsString::from(FORWARD_TARGET_VARIABLE),
            OsString::from(target),
        )];
        assert_eq!(
            plan_invocation(
                text(&["--html-path"]).as_slice(),
                environment.as_slice(),
                &resolution
            ),
            forward(text(&["--html-path"])),
            "{target:?}"
        );
    }
    remove(root.as_path());
}

/// Without a usable real Git the invocation stops with the resolver's evidence.
#[test]
fn missing_real_git_stops_with_resolution_evidence() {
    let root: PathBuf = fixture("entry-no-git");
    let mut resolution: ResolutionInputs = inputs(root.as_path());
    std::fs::create_dir(root.join("only-wrapper")).expect("directory");
    std::os::unix::fs::symlink(&resolution.own_executable, root.join("only-wrapper/git"))
        .expect("symlink");
    resolution.path = OsString::from("only-wrapper:missing");
    for arguments in [vec!["status"], vec!["add", "x"]] {
        let said: String = stopped(plan_invocation(
            text(arguments.as_slice()).as_slice(),
            &[],
            &resolution,
        ));
        assert!(
            said.starts_with(
                "monochromatic-git-policy-cli: Could not find a real Git executable after examining 2 PATH candidates and \
                 skipping 1 monochromatic-git-policy-cli wrappers, so no Git command was run. PATH is only-wrapper:missing."
            ),
            "{arguments:?}: {said}"
        );
        // Every candidate is listed with its verdict, and the fix names the PATH of the program that ran the command.
        assert!(said.contains(root.join("only-wrapper/git").to_string_lossy().as_ref()), "{said}");
        assert!(said.contains("leads back to monochromatic-git-policy-cli"), "{said}");
        assert!(said.contains("is missing, not an executable file, or unreadable"), "{said}");
        assert!(said.contains("To fix it, install Git"), "{said}");
    }
    remove(root.as_path());
}

/// A "Git" whose identity output cannot be interpreted stops the command instead of guessing.
#[test]
fn uninterpretable_identity_output_stops() {
    let root: PathBuf = fixture("entry-bad-identity");
    let mut resolution: ResolutionInputs = inputs(root.as_path());
    std::fs::create_dir(root.join("fake")).expect("directory");
    executable(
        root.join("fake/git").as_path(),
        b"#!/bin/sh\nprintf 'false\\n/only-two-lines\\n'\nexit 0\n",
    );
    resolution.path = OsString::from("fake");
    let message: String = stopped(plan_invocation(
        text(&["commit"]).as_slice(),
        &[],
        &resolution,
    ));
    assert!(
        message.starts_with("monochromatic-git-policy-cli: monochromatic-git-policy-cli could not classify the Git worktree"),
        "{message}"
    );
    remove(root.as_path());
}
