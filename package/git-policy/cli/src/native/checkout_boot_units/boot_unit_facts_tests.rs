//! What:
//!  The facts' controls:
//!  the route's own values pass,
//!  and each unusable one is refused with a sentence naming it.
//! Why:
//!  the refusals are the only thing standing between a checkout path a person
//!  mistyped and a boot unit that runs another command,
//!  and each one is asserted on its own branch so a missing check is a failing
//!  control rather than a unit nobody read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(() => checked({ checkout: 'Monochromatic' })).toThrow(UnitTextRefusal);
//! ```

use crate::checkout_boot_units::boot_unit_facts::{
    CheckoutBootFacts,
    NestedDatasetMount,
    UnitTextRefusal,
    check_path,
    check_unit_name,
    check_word,
    path_text,
};
use crate::worktree_clone_mount_handler::handler_install_paths::ROOT_MODE_INSTALL_PATH;
use std::path::{
    Path,
    PathBuf,
};

/// The route's facts as the design states them on this machine.
fn facts() -> CheckoutBootFacts {
    return facts_of("/home/user/Monochromatic", ROOT_MODE_INSTALL_PATH);
}

/// Facts naming one checkout and one root-owned file,
/// which every control of this folder builds from instead of spelling them.
///
/// Why:
///  the four other controls of the folder each need a pair that differs from
///  the route's,
/// and a second spelling of the shape drifts from this one.
pub(crate) fn facts_of(
    checkout: &str,
    root_file: &str,
) -> CheckoutBootFacts {
    return CheckoutBootFacts {
        checkout: PathBuf::from(checkout),
        root_file: PathBuf::from(root_file),
        key_unit: None,
    };
}

/// One nested dataset as the design states it on this machine.
fn nested() -> NestedDatasetMount {
    return nested_of(
        "zpcachyos/nosnap/vm-builder-output",
        "/home/user/Monochromatic/package/vm-builder/cli/output",
        "output",
    );
}

/// One nested dataset with the three values a control states.
///
/// Why:
///  a control about the shape of the plan needs the plan's own nested dataset
///  and not this machine's,
/// and the three values are what the unit is built from.
pub(crate) fn nested_of(
    dataset: &str,
    mountpoint: &str,
    slug: &str,
) -> NestedDatasetMount {
    return NestedDatasetMount {
        dataset: String::from(dataset),
        mountpoint: PathBuf::from(mountpoint),
        slug: String::from(slug),
    };
}

#[test]
fn the_route_s_own_facts_pass_every_check() {
    assert_eq!(facts().checked(), Ok(()));
    assert_eq!(nested().checked(), Ok(()));
    let with_key: CheckoutBootFacts = CheckoutBootFacts {
        key_unit: Some(String::from(r"zfs-load-key@zfsstep19-home\x2dgit.service")),
        ..facts()
    };
    assert_eq!(
        with_key.checked(),
        Ok(()),
        "an escaped unit name carries backslashes and is still one dependency word"
    );
}

#[test]
fn a_missing_key_unit_is_admitted_and_an_empty_one_is_refused() {
    assert_eq!(
        CheckoutBootFacts {
            key_unit: None,
            ..facts()
        }
        .checked(),
        Ok(())
    );
    let empty: CheckoutBootFacts = CheckoutBootFacts {
        key_unit: Some(String::new()),
        ..facts()
    };
    assert!(
        matches!(empty.checked(), Err(UnitTextRefusal::WordNotOneWord { .. })),
        "an empty name would write `Requires=` with no unit"
    );
}

#[test]
fn a_relative_path_is_refused_and_names_itself() {
    let refusal: UnitTextRefusal = check_path("the checkout path", Path::new("Monochromatic"))
        .expect_err("a relative path resolves against a reader's directory");
    assert!(matches!(refusal, UnitTextRefusal::PathNotAbsolute { .. }));
    assert!(
        refusal.message().contains("Monochromatic"),
        "the refusal names the value: {}",
        refusal.message()
    );
    assert!(
        refusal.message().contains("Nothing was written"),
        "the refusal says nothing happened: {}",
        refusal.message()
    );
}

#[test]
fn a_path_that_is_not_text_or_carries_a_space_is_refused() {
    let spaced: PathBuf = PathBuf::from("/home/user/Mono chrome");
    let refusal: UnitTextRefusal =
        path_text("the checkout path", spaced.as_path()).expect_err("a space splits an Exec line");
    assert!(matches!(refusal, UnitTextRefusal::PathNotOneWord { .. }));
    assert!(
        refusal.message().contains("Mono chrome"),
        "the refusal names the value: {}",
        refusal.message()
    );
}

#[test]
fn a_word_that_is_two_words_is_refused() {
    assert_eq!(
        check_word("the nested dataset's name", "zpcachyos/nosnap/one"),
        Ok(())
    );
    let refusal: UnitTextRefusal =
        check_word("the nested dataset's name", "zpcachyos/nosnap/one two")
            .expect_err("a space splits the option's value");
    assert!(matches!(refusal, UnitTextRefusal::WordNotOneWord { .. }));
    assert!(
        refusal.message().contains("the nested dataset's name"),
        "the refusal names the value: {}",
        refusal.message()
    );
}

#[test]
fn a_unit_name_with_whitespace_is_refused_and_one_with_a_backslash_is_not() {
    assert_eq!(
        check_unit_name(
            "the key-loading unit's name",
            r"zfs-load-key@a\x2db.service"
        ),
        Ok(())
    );
    assert!(matches!(
        check_unit_name("the key-loading unit's name", "one.service two.service"),
        Err(UnitTextRefusal::WordNotOneWord { .. })
    ));
}

#[test]
fn a_slug_a_unit_name_does_not_take_is_refused() {
    for slug in ["", "-output", "Out put", "out_put", "output "] {
        let one: NestedDatasetMount = NestedDatasetMount {
            slug: String::from(slug),
            ..nested()
        };
        let refusal: UnitTextRefusal = one
            .checked()
            .expect_err("the slug becomes a unit name a person reads");
        assert!(
            matches!(refusal, UnitTextRefusal::Slug { .. }),
            "`{slug}` is refused as a slug"
        );
        assert!(
            refusal.message().contains("slug"),
            "the refusal names the value: {}",
            refusal.message()
        );
    }
}

#[test]
fn a_nested_dataset_with_a_missing_mountpoint_is_refused() {
    let relative: NestedDatasetMount = NestedDatasetMount {
        mountpoint: PathBuf::from("output"),
        ..nested()
    };
    assert!(matches!(
        relative.checked(),
        Err(UnitTextRefusal::PathNotAbsolute { .. })
    ));
}
