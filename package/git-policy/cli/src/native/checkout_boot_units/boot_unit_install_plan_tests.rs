//! What:
//!  The install plan's controls:
//!  which files it holds,
//! in which order,
//!  the shape of every text,
//!  the gate line the propagation pair needs,
//!  the nested unit's place last,
//!  and that a refused fact writes nothing.
//! Why:
//!  a person's setup run writes exactly this list,
//!  so a file missing from it is a boot that does not happen,
//!  and the pair's position after the gate is the one thing its safety rests
//!  on.
//!  The whole-plan refusal is asserted because a plan that returned the files it
//!  built before a later refusal would leave a machine half set up.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const paths = installPlan(facts, [nested]).map(f => f.path);
//! expect(paths[0]).toBe('/etc/systemd/system/reconcile-git-worktree-clone-mounts.service');
//! ```

use crate::checkout_boot_units::boot_unit_facts::{CheckoutBootFacts, InstalledUnitFile, NestedDatasetMount, UnitTextRefusal};
use crate::checkout_boot_units::boot_unit_install_plan::install_plan;
use crate::checkout_boot_units::unit_text_sections::{assert_file_shape, carries};
use crate::worktree_clone_mount_handler::handler_install_paths::ROOT_MODE_INSTALL_PATH;
use std::path::PathBuf;

/// The route's facts.
fn facts() -> CheckoutBootFacts {
    return CheckoutBootFacts { checkout: PathBuf::from("/home/user/Monochromatic"), root_file: PathBuf::from(ROOT_MODE_INSTALL_PATH), key_unit: None };
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
fn the_plan_holds_every_file_in_the_order_a_setup_writes_them() {
    let plan: Vec<InstalledUnitFile> = install_plan(&facts(), &[nested()]).expect("the route's facts are text");
    let paths: Vec<String> = plan.iter().map(|file| return file.path.display().to_string()).collect();
    assert_eq!(
        paths,
        vec![
            String::from("/etc/systemd/system/reconcile-git-worktree-clone-mounts.service"),
            String::from("/etc/systemd/system/reconcile-git-worktree-clone-mounts.service.d/zz-checkout-stop-failure-mode.conf"),
            String::from("/etc/systemd/system/zfs-mount.service.d/reconcile-git-worktree-clone-mounts-after-checkout.conf"),
            String::from("/etc/systemd/system/checkout-propagation.service"),
            String::from("/etc/systemd/system/home-user-Monochromatic.mount.d/10-upholds-propagation.conf"),
            String::from("/etc/systemd/system/git-worktree-nested-output-mount.service"),
        ],
        "the gate comes first and the nested unit last: {paths:?}"
    );
}

#[test]
fn every_text_of_the_plan_has_the_one_shape_a_unit_file_must_have() {
    let plan: Vec<InstalledUnitFile> = install_plan(&facts(), &[nested()]).expect("the route's facts are text");
    assert!(!plan.is_empty());
    for file in &plan {
        assert_file_shape(&file.text);
    }
}

#[test]
fn the_gate_the_plan_writes_carries_the_reload_line_the_pair_needs() {
    let plan: Vec<InstalledUnitFile> = install_plan(&facts(), &[nested()]).expect("the route's facts are text");
    let gate: &InstalledUnitFile = plan.first().expect("the gate is the plan's first file");
    assert!(carries(&gate.text, "[Service]", "ExecReload="), "the propagation pair's request is a reload only because of this line: {}", gate.text);
}

#[test]
fn a_machine_with_no_nested_dataset_writes_five_files() {
    let plan: Vec<InstalledUnitFile> = install_plan(&facts(), &[]).expect("the route's facts are text");
    assert_eq!(plan.len(), 5, "no nested unit stands where no dataset does: {plan:?}");
}

#[test]
fn a_refused_fact_returns_no_file_at_all() {
    let relative: CheckoutBootFacts = CheckoutBootFacts { checkout: PathBuf::from("Monochromatic"), ..facts() };
    assert!(matches!(install_plan(&relative, &[nested()]), Err(UnitTextRefusal::PathNotAbsolute { .. })), "a plan is written or nothing is");

    let broken_nested: NestedDatasetMount = NestedDatasetMount { slug: String::from("out put"), ..nested() };
    let refusal: UnitTextRefusal = install_plan(&facts(), &[nested(), broken_nested]).expect_err("one bad nested dataset stops the plan");
    assert!(refusal.message().contains("slug"), "the refusal names the value: {}", refusal.message());
}

#[test]
fn two_nested_datasets_get_two_units_in_the_order_they_were_given() {
    let second: NestedDatasetMount = NestedDatasetMount {
        dataset: String::from("zpcachyos/nosnap/cargo-build"),
        mountpoint: PathBuf::from("/home/user/Monochromatic/.cargo/build"),
        slug: String::from("cargo-build"),
    };
    let plan: Vec<InstalledUnitFile> = install_plan(&facts(), &[nested(), second]).expect("the route's facts are text");
    let names: Vec<String> = plan.iter().skip(5).map(|file| return file.path.display().to_string()).collect();
    assert_eq!(
        names,
        vec![
            String::from("/etc/systemd/system/git-worktree-nested-output-mount.service"),
            String::from("/etc/systemd/system/git-worktree-nested-cargo-build-mount.service"),
        ]
    );
}
