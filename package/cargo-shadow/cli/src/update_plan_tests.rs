//! The unit tests of the `cargo update` to `pnpm update` mapping.

use std::ffi::OsString;

use crate::update_plan::{UpdatePlan, plan_update};

fn plan(words: &[&str]) -> Result<crate::update_plan::Planned, String> {
    let arguments: Vec<OsString> = words.iter().map(OsString::from).collect();
    return plan_update(&arguments);
}

#[test]
fn no_names_is_pnpm_update_of_everything() {
    assert_eq!(plan(&["update"]).expect("plans").plan, UpdatePlan::Full { dry_run: false });
    assert_eq!(plan(&["+nightly", "update", "--dry-run"]).expect("plans").plan, UpdatePlan::Full { dry_run: true });
    assert_eq!(plan(&["update", "-n"]).expect("plans").plan, UpdatePlan::Full { dry_run: true });
}

#[test]
fn a_named_package_is_pnpm_update_of_that_name_run_as_given() {
    let planned = plan(&["update", "serde", "-p", "tokio@1.2.3"]).expect("plans");

    assert_eq!(
        planned.plan,
        UpdatePlan::Targeted {
            dry_run: false,
            cargo_args: ["update", "serde", "-p", "tokio@1.2.3"].iter().map(|word| { return word.to_string(); }).collect(),
            names: vec!["serde".to_string(), "tokio".to_string()],
            precise: false,
        }
    );
}

#[test]
fn precise_is_pnpm_update_name_at_version_and_needs_exactly_one_name() {
    let planned = plan(&["update", "serde", "--precise", "1.0.200"]).expect("plans");
    match planned.plan {
        UpdatePlan::Targeted { precise, cargo_args, .. } => {
            assert!(precise);
            assert!(cargo_args.contains(&"--precise".to_string()) && cargo_args.contains(&"1.0.200".to_string()));
        }
        other => panic!("expected targeted, got {other:?}"),
    }
    assert!(plan(&["update", "--precise", "1.0.0"]).is_err());
    assert!(plan(&["update", "a", "b", "--precise", "1.0.0"]).is_err());
}

#[test]
fn breaking_is_pnpm_update_latest_and_runs_cargos_own_breaking_update() {
    match plan(&["update", "--breaking"]).expect("plans").plan {
        UpdatePlan::Targeted { cargo_args, .. } => assert!(cargo_args.contains(&"--breaking".to_string())),
        other => panic!("expected targeted, got {other:?}"),
    }
}

#[test]
fn workspace_is_the_install_step_and_locked_or_frozen_is_the_frozen_check() {
    assert_eq!(plan(&["update", "--workspace"]).expect("plans").plan, UpdatePlan::Install { dry_run: false });
    assert_eq!(plan(&["update", "-w"]).expect("plans").plan, UpdatePlan::Install { dry_run: false });
    assert_eq!(plan(&["update", "--locked"]).expect("plans").plan, UpdatePlan::FrozenCheck);
    assert_eq!(plan(&["update", "serde", "--frozen"]).expect("plans").plan, UpdatePlan::FrozenCheck);
}

#[test]
fn generate_lockfile_is_pnpm_install_lockfile_only() {
    assert_eq!(plan(&["generate-lockfile"]).expect("plans").plan, UpdatePlan::Install { dry_run: false });
}

#[test]
fn cargo_only_options_are_forwarded_with_their_values_and_the_manifest_path_is_dropped() {
    let planned = plan(&["update", "--offline", "--color", "never", "--config", "net.retry=1", "-Zunstable-options", "-m", "a/Cargo.toml", "-q"])
        .expect("plans");

    assert_eq!(
        planned.forward,
        ["--offline", "--color", "never", "--config", "net.retry=1", "-Zunstable-options", "-q"].iter().map(|word| { return word.to_string(); }).collect::<Vec<String>>()
    );
    assert_eq!(planned.plan, UpdatePlan::Full { dry_run: false });
}

fn names_of(words: &[&str]) -> Vec<String> {
    match plan(words).expect("plans").plan {
        UpdatePlan::Targeted { names, .. } => return names,
        other => panic!("expected targeted, got {other:?}"),
    }
}

/// Review finding (`--package=foo`): the attached spelling names the package like `--package foo` does, and the plan must not
/// widen into a full update.
#[test]
fn the_attached_and_short_spellings_of_package_name_the_package() {
    assert_eq!(names_of(&["update", "--package=foo"]), vec!["foo".to_string()]);
    assert_eq!(names_of(&["update", "--package=foo@1.2.3"]), vec!["foo".to_string()]);
    assert_eq!(names_of(&["update", "-pfoo"]), vec!["foo".to_string()]);
}

/// Review finding (global options before the verb): `--locked` before the verb asks for the frozen check, as after it.
#[test]
fn a_locked_or_frozen_before_the_verb_is_the_frozen_check() {
    assert_eq!(plan(&["--locked", "update"]).expect("plans").plan, UpdatePlan::FrozenCheck);
    assert_eq!(plan(&["+nightly", "--frozen", "update"]).expect("plans").plan, UpdatePlan::FrozenCheck);
}

/// Review finding: cargo-only options before the verb are forwarded to every cargo call the procedure makes.
#[test]
fn cargo_only_options_before_the_verb_are_forwarded_too() {
    let planned = plan(&["--offline", "--color", "never", "update"]).expect("plans");

    assert_eq!(planned.forward, vec!["--offline".to_string(), "--color".to_string(), "never".to_string()]);
}

/// Review finding (reject, do not broaden): an option the shadow cannot map to pnpm update is refused by name.
#[test]
fn an_option_the_shadow_cannot_answer_is_refused_by_name_and_not_dropped() {
    let message: String = plan(&["update", "--bogus"]).expect_err("an unknown option must be refused");

    assert!(message.contains("--bogus"), "{message}");
    assert!(plan(&["update", "serde", "--lockfile-path", "x"]).is_err());
}

/// The options of `cargo update --help` stay accepted.
#[test]
fn every_option_of_cargo_update_help_is_still_accepted() {
    for words in [
        vec!["update", "--recursive", "serde"],
        vec!["update", "-vv"],
        vec!["update", "--ignore-rust-version"],
        vec!["update", "--manifest-path=a/Cargo.toml"],
        vec!["-C", "dir", "update"],
    ] {
        assert!(plan(&words).is_ok(), "{words:?}");
    }
}
