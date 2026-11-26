//! What:
//!  Controls of the staging folder under the account's cache and of its live
//!  bind's propagation.
//! Why:
//!  A staging folder another account owns,
//!  one at another mode,
//!  one holding content and one whose name no removal derived are each a
//!  state the design names,
//!  and a shared bind delivers a mount to another namespace,
//!  so each control changes one fact.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(check_staging_folder(withContent('monochromatic-git-policy-cli-staging-1234'))).toMatchObject({ field: 'content' });
//! ```

use super::{
    STAGING_FOLDER_MODE,
    StagedFolder,
    StagingBind,
    StagingFolderFacts,
    check_staging_bind_propagation,
    check_staging_folder,
    is_derived_folder_name,
    staging_binds,
};
use crate::fs_mount_options::parse_mount_table;
use crate::health::doctor_zfs_route::handler_checks::{
    PathState,
    ROOT_OWNER,
};
use crate::health::doctor_zfs_route::route_test_facts::{
    ACCOUNT,
    STAGING_PREFIX,
    path_facts,
};
use std::path::PathBuf;

/// A folder name a removal derives,
/// which carries a dataset GUID's digits.
const DERIVED: &str = "monochromatic-git-policy-cli-staging-1837462836473829";

/// One staging folder.
fn folder(
    name: &str,
    owner: u32,
    mode: u32,
    derived: bool,
    empty: bool,
) -> StagedFolder {
    return StagedFolder {
        name: String::from(name),
        state: PathState::Directory {
            owner,
            mode,
        },
        derived,
        empty,
    };
}

/// The healthy facts with one folder added.
fn with_folder(staged: StagedFolder) -> StagingFolderFacts {
    return StagingFolderFacts {
        cache: path_facts(
            "/home/user/.cache",
            PathState::Directory {
                owner: ACCOUNT,
                mode: 0o700,
            },
        ),
        account: ACCOUNT,
        folders: vec![staged],
    };
}

//region The folder

/// No staging folder passes.
#[test]
fn no_staging_folder_passes() {
    assert!(matches!(
        check_staging_folder(&crate::health::doctor_zfs_route::route_test_facts::healthy_staging()),
        super::CheckOutcome::Pass { .. }
    ));
}

/// The design's folder passes.
#[test]
fn the_designs_staging_folder_passes() {
    let outcome: super::CheckOutcome = check_staging_folder(&with_folder(folder(
        DERIVED,
        ACCOUNT,
        STAGING_FOLDER_MODE,
        true,
        true,
    )));
    let super::CheckOutcome::Pass {
        detail,
    } = outcome
    else {
        panic!("the design's folder passes: {outcome:?}");
    };
    assert!(detail.contains('1'), "{detail}");
}

/// A folder another account owns is a finding naming the owner.
#[test]
fn a_folder_another_account_owns_is_a_finding() {
    let super::CheckOutcome::Finding {
        field,
        detail,
    } = check_staging_folder(&with_folder(folder(
        DERIVED,
        ROOT_OWNER,
        STAGING_FOLDER_MODE,
        true,
        true,
    )))
    else {
        panic!("another owner is a finding");
    };
    assert_eq!(field, "owner");
    assert!(detail.contains(DERIVED), "{detail}");
}

/// A folder at another mode is a finding naming the mode.
#[test]
fn a_folder_at_another_mode_is_a_finding() {
    let super::CheckOutcome::Finding {
        field,
        ..
    } = check_staging_folder(&with_folder(folder(DERIVED, ACCOUNT, 0o755, true, true)))
    else {
        panic!("another mode is a finding");
    };
    assert_eq!(field, "mode");
}

/// A folder holding content is a finding naming the content.
#[test]
fn a_folder_holding_content_is_a_finding() {
    let super::CheckOutcome::Finding {
        field,
        detail,
    } = check_staging_folder(&with_folder(folder(
        DERIVED,
        ACCOUNT,
        STAGING_FOLDER_MODE,
        true,
        false,
    )))
    else {
        panic!("content is a finding");
    };
    assert_eq!(field, "content");
    assert!(detail.contains(DERIVED), "{detail}");
}

/// A name with the prefix and no derived component is a finding naming it.
#[test]
fn a_name_with_no_derived_component_is_a_finding() {
    let super::CheckOutcome::Finding {
        field,
        detail,
    } = check_staging_folder(&with_folder(folder(
        "monochromatic-git-policy-cli-staging-not-a-guid",
        ACCOUNT,
        STAGING_FOLDER_MODE,
        false,
        true,
    )))
    else {
        panic!("a stranger's name is a finding");
    };
    assert_eq!(field, "name");
    assert!(detail.contains("not-a-guid"), "{detail}");
}

/// A cache this account could not list leaves the answer unknown.
#[test]
fn an_unlistable_cache_is_unknown() {
    let facts: StagingFolderFacts = StagingFolderFacts {
        cache: path_facts(
            "/home/user/.cache",
            PathState::Unreadable {
                why: String::from("permission denied"),
            },
        ),
        account: ACCOUNT,
        folders: Vec::new(),
    };
    assert!(matches!(
        check_staging_folder(&facts),
        super::CheckOutcome::Unknown { .. }
    ));
}

/// An absent cache passes:
/// no staging folder can stand under one.
#[test]
fn an_absent_cache_passes() {
    let facts: StagingFolderFacts = StagingFolderFacts {
        cache: path_facts("/home/user/.cache", PathState::Absent),
        account: ACCOUNT,
        folders: Vec::new(),
    };
    assert!(matches!(
        check_staging_folder(&facts),
        super::CheckOutcome::Pass { .. }
    ));
}

/// The derivation rule follows the name's component.
#[test]
fn the_derivation_rule_follows_the_component() {
    assert!(is_derived_folder_name(DERIVED), "{}", STAGING_PREFIX);
    assert!(!is_derived_folder_name(
        "monochromatic-git-policy-cli-staging-not-a-guid"
    ));
    assert!(!is_derived_folder_name("some-other-folder"));
}

//endregion

//region The bind

/// One bindinfo row for a staging folder.
fn bind_row(
    id: u32,
    path: &str,
    source: &str,
    optional: &str,
) -> String {
    let optional_fields: String = if optional.is_empty() {
        String::new()
    } else {
        format!("{optional} ")
    };
    return format!("{id} 1 0:50 / {path} rw,relatime {optional_fields}- zfs {source} rw,xattr");
}

/// No live bind passes.
#[test]
fn no_live_bind_passes() {
    assert!(matches!(
        check_staging_bind_propagation(&[]),
        super::CheckOutcome::Pass { .. }
    ));
}

/// The design's private self-bind passes.
#[test]
fn the_designs_private_self_bind_passes() {
    let stage: String = format!("/home/user/.cache/{DERIVED}");
    let rows = parse_mount_table(bind_row(60, &stage, &stage, "").as_bytes());
    let binds: Vec<StagingBind> =
        staging_binds(&rows, PathBuf::from("/home/user/.cache").as_path());
    assert_eq!(binds.len(), 1, "{binds:?}");
    assert!(matches!(
        check_staging_bind_propagation(&binds),
        super::CheckOutcome::Pass { .. }
    ));
}

/// A shared bind is a finding that prints the command.
#[test]
fn a_shared_bind_is_a_finding_with_the_command() {
    let stage: String = format!("/home/user/.cache/{DERIVED}");
    let rows = parse_mount_table(bind_row(60, &stage, &stage, "shared:9").as_bytes());
    let binds: Vec<StagingBind> =
        staging_binds(&rows, PathBuf::from("/home/user/.cache").as_path());
    let super::CheckOutcome::Finding {
        field,
        detail,
    } = check_staging_bind_propagation(&binds)
    else {
        panic!("a shared bind is a finding");
    };
    assert_eq!(field, "propagation");
    assert!(detail.contains("mount --make-private"), "{detail}");
}

/// A row outside the staging prefix is no staging bind.
#[test]
fn a_row_outside_the_prefix_is_no_staging_bind() {
    let rows = parse_mount_table(
        bind_row(
            60,
            "/home/user/.cache/some-other-folder",
            "/home/user/.cache/some-other-folder",
            "",
        )
        .as_bytes(),
    );
    assert!(staging_binds(&rows, PathBuf::from("/home/user/.cache").as_path()).is_empty());
}

//endregion
