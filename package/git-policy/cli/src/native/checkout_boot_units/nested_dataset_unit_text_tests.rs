//! What:
//!  One nested dataset's unit controls:
//!  its requirement on the gate,
//!  the condition that tolerates an already-mounted dataset,
//!  the mode line that places the mount,
//!  the install section,
//!  and the values it refuses.
//! Why:
//!  the unit's whole job is the order and the condition:
//!  without the requirement it mounts under whatever mount came first,
//!  and without the condition a re-run exits one with `filesystem already
//!  mounted` and fails a consumer that requires it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(linesOf(nestedDatasetUnit(nested, facts), '[Unit]')).toContain('Requires=reconcile-…service');
//! ```

use crate::checkout_boot_units::boot_unit_facts::{
    CheckoutBootFacts,
    NestedDatasetMount,
    UnitTextRefusal,
};
use crate::checkout_boot_units::boot_unit_names::{
    BOOT_GATE_UNIT_NAME,
    root_words,
};
use crate::checkout_boot_units::nested_dataset_unit_text::nested_dataset_unit;
use crate::checkout_boot_units::nested_dataset_unit_text::{
    DATASET_OPTION,
    MOUNT_PATH_DESCENDANT_MODE_WORD,
};
use crate::checkout_boot_units::unit_text_sections::{
    assert_file_shape,
    carries,
    holds,
};
use crate::worktree_clone_mount_handler::handler_install_paths::ROOT_MODE_INSTALL_PATH;
use std::path::PathBuf;

/// The route's facts.
fn facts() -> CheckoutBootFacts {
    return CheckoutBootFacts {
        checkout: PathBuf::from("/home/user/Monochromatic"),
        root_file: PathBuf::from(ROOT_MODE_INSTALL_PATH),
        key_unit: None,
    };
}

/// The one nested dataset this machine has.
fn nested() -> NestedDatasetMount {
    return NestedDatasetMount {
        dataset: String::from("zpcachyos/nosnap/vm-builder-output"),
        mountpoint: PathBuf::from("/home/user/Monochromatic/package/vm-builder/cli/output"),
        slug: String::from("output"),
    };
}

#[test]
fn the_unit_requires_and_orders_after_the_gate() {
    let text: String =
        nested_dataset_unit(&nested(), &facts()).expect("the route's values are text");
    assert_file_shape(&text);
    assert!(
        holds(
            &text,
            "[Unit]",
            format!("Requires={BOOT_GATE_UNIT_NAME}").as_str()
        ),
        "a failed gate authorizes no mount: {text}"
    );
    assert!(
        holds(
            &text,
            "[Unit]",
            format!("After={BOOT_GATE_UNIT_NAME}").as_str()
        ),
        "and the mount waits for it: {text}"
    );
    assert!(
        !carries(&text, "[Unit]", "Before=systemd-user-sessions.service"),
        "V8a keeps logins out of this order too: {text}"
    );
}

#[test]
fn the_condition_skips_an_already_mounted_dataset() {
    let text: String =
        nested_dataset_unit(&nested(), &facts()).expect("the route's values are text");
    assert!(
        holds(
            &text,
            "[Unit]",
            "ConditionPathIsMountPoint=!/home/user/Monochromatic/package/vm-builder/cli/output"
        ),
        "a skipped unit counts as success where `zfs mount` would exit one: {text}"
    );
}

#[test]
fn the_mode_line_names_the_dataset_and_nothing_a_caller_composed() {
    let text: String =
        nested_dataset_unit(&nested(), &facts()).expect("the route's values are text");
    let command: String = format!(
        "{ROOT_MODE_INSTALL_PATH} {} {MOUNT_PATH_DESCENDANT_MODE_WORD} {DATASET_OPTION} zpcachyos/nosnap/vm-builder-output",
        root_words()
    );
    assert!(
        holds(&text, "[Service]", format!("ExecStart={command}").as_str()),
        "the mode takes a dataset and no path: {text}"
    );
    assert!(holds(&text, "[Service]", "Type=oneshot"));
    assert!(holds(&text, "[Service]", "RemainAfterExit=yes"));
    assert!(holds(&text, "[Install]", "WantedBy=multi-user.target"));
}

#[test]
fn the_unit_file_name_and_its_comment_agree() {
    let text: String =
        nested_dataset_unit(&nested(), &facts()).expect("the route's values are text");
    assert!(
        text.contains("/etc/systemd/system/git-worktree-nested-output-mount.service"),
        "the comment names the file: {text}"
    );
}

#[test]
fn a_dataset_name_with_a_space_is_refused() {
    let spaced: NestedDatasetMount = NestedDatasetMount {
        dataset: String::from("zpcachyos/nosnap/two words"),
        ..nested()
    };
    let refusal: UnitTextRefusal =
        nested_dataset_unit(&spaced, &facts()).expect_err("a space splits the option");
    assert!(
        refusal.message().contains("the nested dataset's name"),
        "the refusal names the value: {}",
        refusal.message()
    );
}

#[test]
fn a_mountpoint_with_a_space_is_refused() {
    let spaced: NestedDatasetMount = NestedDatasetMount {
        mountpoint: PathBuf::from("/home/user/Monochromatic/out put"),
        ..nested()
    };
    assert!(matches!(
        nested_dataset_unit(&spaced, &facts()),
        Err(UnitTextRefusal::PathNotOneWord { .. })
    ));
}
