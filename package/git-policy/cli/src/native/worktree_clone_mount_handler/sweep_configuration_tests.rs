//! What:
//!  Controls of the sweep's three names:
//!  the file's entries,
//!  the design's own names the mount table fills in,
//!  the order the two are read in,
//!  and every refusal that names what was found and what a person writes.
//! Why:
//!  A sweep that guessed a parent dataset,
//!  a checkout dataset or a checkout would act on another dataset's children,
//!  and the project answered on 2026-11-22 that everything works by default with
//!  no configuration file
//!  (`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
//!  "With no configuration file").
//!  Each control scripts the text of a mount table,
//!  so the rule that reads rows is the release rule and nothing on this machine
//!  is read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(readConfiguration(absentFile, scriptedTable)).toEqual({ parentDataset: 'pool/…', checkout: '/home/user/Monochromatic', checkoutDataset: 'pool/…' });
//! ```

use super::{
    CHECKOUT_DATASET_KEY, CHECKOUT_KEY, CLONE_MOUNTS_KEY, ConfigurationRefusal, PARENT_KEY, PROBE_OBJECT_KEY, SweepConfiguration, parse_configuration,
    read_configuration,
};
use crate::testing::mounted_dataset_table::{ScriptedMountTable, row, the_designs_table};
use std::path::{Path, PathBuf};

/// The file every control parses as if it were the installed one.
const CONFIGURATION: &str = "/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc";

/// The checkout dataset the design names.
const CHECKOUT_DATASET: &str = "zpcachyos/ROOT/cos/home/monochromatic-git-checkout";

/// The container the design places beside it.
const PARENT: &str = "zpcachyos/ROOT/cos/home/monochromatic-git-worktree-clone-datasets";

/// The checkout the controls name.
const CHECKOUT: &str = "/home/user/Monochromatic";

/// One whole configuration document naming all three keys.
fn document(parent: &str, checkout: &str, checkout_dataset: &str) -> String {
    return format!(r#"{{ "{CLONE_MOUNTS_KEY}": {{ "{PARENT_KEY}": "{parent}", "{CHECKOUT_KEY}": "{checkout}", "{CHECKOUT_DATASET_KEY}": "{checkout_dataset}" }} }}"#);
}

/// One fixture folder of this control's own.
struct Fixture {
    /// The folder the fixture file is written in.
    path: PathBuf,
}

impl Fixture {
    /// A fresh folder of this control's own,
    /// named so two controls of one process never share it.
    fn new(name: &str) -> Self {
        return Fixture { path: crate::test_support::fixture(format!("sweep-configuration-{name}").as_str()) };
    }

    /// The configuration file inside the fixture folder,
    /// holding `document`.
    fn write(&self, document: &str) -> PathBuf {
        let file: PathBuf = self.path.join("git-worktree-clone-mounts.jsonc");
        std::fs::write(file.as_path(), document).expect("the fixture configuration is written");
        return file;
    }

    /// The path of a file this fixture never wrote.
    fn absent(&self) -> PathBuf {
        return self.path.join("nothing.jsonc");
    }
}

/// No file at all finds the checkout as the one mounted instance of the design's
/// leaf name,
/// and takes the container beside it.
#[test]
fn no_file_finds_the_mounted_checkout_and_both_design_names() {
    let fixture: Fixture = Fixture::new("no-file");
    let refusal_free: SweepConfiguration = read_configuration(fixture.absent().as_path(), &ScriptedMountTable::text(&the_designs_table(CHECKOUT, CHECKOUT_DATASET))).expect("the table holds the design's checkout once");
    assert_eq!(refusal_free.checkout, PathBuf::from(CHECKOUT));
    assert_eq!(refusal_free.checkout_dataset, CHECKOUT_DATASET);
    assert_eq!(refusal_free.parent_dataset, PARENT);
}

/// An empty object names nothing and finds both datasets and the checkout the
/// same way a missing file does.
#[test]
fn an_empty_object_finds_them_too() {
    let configuration: SweepConfiguration = parse_configuration("{}", Path::new(CONFIGURATION), &ScriptedMountTable::text(&the_designs_table(CHECKOUT, CHECKOUT_DATASET))).expect("an empty object names nothing");
    assert_eq!(configuration.checkout, PathBuf::from(CHECKOUT));
    assert_eq!(configuration.checkout_dataset, CHECKOUT_DATASET);
    assert_eq!(configuration.parent_dataset, PARENT);
}

/// A file holding only the start probe's object finds the checkout and both
/// datasets as a missing file does,
/// and its checkout entry has to agree with the one the table gives.
#[test]
fn a_file_holding_only_the_start_probe_object_finds_the_checkout() {
    let only_probe: String = format!(r#"{{ "{PROBE_OBJECT_KEY}": {{ "{CHECKOUT_KEY}": "{CHECKOUT}", "dataset": "zpcachyos/nosnap/git-worktree-probe" }} }}"#);
    let configuration: SweepConfiguration =
        parse_configuration(only_probe.as_str(), Path::new(CONFIGURATION), &ScriptedMountTable::text(&the_designs_table(CHECKOUT, CHECKOUT_DATASET))).expect("the probe's object names no clone mount");
    assert_eq!(configuration.checkout, PathBuf::from(CHECKOUT));
    assert_eq!(configuration.checkout_dataset, CHECKOUT_DATASET);
    assert_eq!(configuration.parent_dataset, PARENT);
    let elsewhere: String = format!(r#"{{ "{PROBE_OBJECT_KEY}": {{ "{CHECKOUT_KEY}": "/home/user/another" }} }}"#);
    let refusal: ConfigurationRefusal =
        parse_configuration(elsewhere.as_str(), Path::new(CONFIGURATION), &ScriptedMountTable::text(&the_designs_table(CHECKOUT, CHECKOUT_DATASET))).expect_err("the probe's checkout is another checkout");
    assert!(matches!(refusal, ConfigurationRefusal::CheckoutsDiffer { .. }), "{refusal:?}");
}

/// A file naming both datasets and no checkout takes the checkout path from the
/// one mount of the dataset it names.
#[test]
fn a_file_naming_both_datasets_and_no_checkout_takes_the_path_from_its_mount() {
    let without_checkout: String = format!(r#"{{ "{CLONE_MOUNTS_KEY}": {{ "{PARENT_KEY}": "{PARENT}", "{CHECKOUT_DATASET_KEY}": "{CHECKOUT_DATASET}" }} }}"#);
    let configuration: SweepConfiguration =
        parse_configuration(without_checkout.as_str(), Path::new(CONFIGURATION), &ScriptedMountTable::text(&the_designs_table(CHECKOUT, CHECKOUT_DATASET))).expect("the dataset is mounted once");
    assert_eq!(configuration.checkout, PathBuf::from(CHECKOUT));
    assert_eq!(configuration.checkout_dataset, CHECKOUT_DATASET);
    assert_eq!(configuration.parent_dataset, PARENT);
}

/// A dataset the file names and the table mounts nowhere is refused,
/// and the refusal names the entry that settles it.
#[test]
fn a_named_dataset_that_is_mounted_nowhere_is_refused() {
    let without_checkout: String = format!(r#"{{ "{CLONE_MOUNTS_KEY}": {{ "{CHECKOUT_DATASET_KEY}": "zpcachyos/ROOT/cos/home/unmounted" }} }}"#);
    let refusal: ConfigurationRefusal =
        parse_configuration(without_checkout.as_str(), Path::new(CONFIGURATION), &ScriptedMountTable::text(&the_designs_table(CHECKOUT, CHECKOUT_DATASET))).expect_err("nothing mounts that dataset");
    assert!(matches!(refusal, ConfigurationRefusal::Discovery { refusal: crate::zfs_mounted_dataset::mounted_dataset_refusal::MountedDatasetRefusal::DatasetNotMounted { .. }, .. }), "{refusal:?}");
    assert!(refusal.message().contains("unmounted"), "the refusal names the dataset: {}", refusal.message());
    assert!(refusal.message().contains(&format!("{CLONE_MOUNTS_KEY}.{PARENT_KEY}")) || refusal.message().contains(&format!("{CLONE_MOUNTS_KEY}.{CHECKOUT_KEY}")), "{}", refusal.message());
}

/// Two pools that hold the design's leaf name are several instances,
/// and the sweep refuses rather than picking one.
#[test]
fn two_pools_holding_the_leaf_name_are_several_and_are_listed() {
    let two_pools: ScriptedMountTable = ScriptedMountTable::holding(&[
        row(1, 0, "/", "/", "zfs", "zpcachyos/ROOT/cos/root"),
        row(40, 1, "/", CHECKOUT, "zfs", CHECKOUT_DATASET),
        row(50, 1, "/", "/home/user/second", "zfs", "otherpool/ROOT/cos/home/monochromatic-git-checkout"),
    ]);
    let refusal: ConfigurationRefusal = parse_configuration("{}", Path::new(CONFIGURATION), &two_pools).expect_err("two pools are several instances");
    let ConfigurationRefusal::Discovery { refusal: found, .. } = &refusal else {
        panic!("the refusal is a finding: {refusal:?}");
    };
    assert!(found.word() == "several-mounted-checkouts", "{}", refusal.message());
    let said: String = refusal.message();
    assert!(said.contains(CHECKOUT_DATASET), "the refusal lists the first instance: {said}");
    assert!(said.contains("otherpool/ROOT/cos/home/monochromatic-git-checkout"), "the refusal lists the second: {said}");
    assert!(said.contains("/home/user/second"), "the refusal names the second's path: {said}");
    assert!(said.contains("attached or recorded"), "the refusal says nothing acted: {said}");
}

/// One dataset mounted at two paths is several instances of one dataset,
/// and the sweep refuses rather than picking one.
#[test]
fn one_dataset_mounted_at_two_paths_is_several() {
    let twice: ScriptedMountTable = ScriptedMountTable::holding(&[
        row(40, 1, "/", CHECKOUT, "zfs", CHECKOUT_DATASET),
        row(41, 1, "/", "/home/user/monochromatic-again", "zfs", CHECKOUT_DATASET),
    ]);
    let refusal: ConfigurationRefusal = parse_configuration("{}", Path::new(CONFIGURATION), &twice).expect_err("one dataset twice is several instances");
    assert!(refusal.message().contains("/home/user/monochromatic-again"), "{}", refusal.message());
    assert_eq!(refusal.key(), Some(format!("{CLONE_MOUNTS_KEY}.{CHECKOUT_DATASET_KEY}")));
}

/// A bind mount of the dataset's own root elsewhere is a second instance of it,
/// because the mount table shows one row per mount.
#[test]
fn a_bind_mount_of_the_datasets_own_root_elsewhere_is_several() {
    let bound: ScriptedMountTable = ScriptedMountTable::holding(&[
        row(40, 1, "/", CHECKOUT, "zfs", CHECKOUT_DATASET),
        row(41, 1, "/", "/home/user/monochromatic-bound", "zfs", CHECKOUT_DATASET),
    ]);
    let refusal: ConfigurationRefusal = parse_configuration("{}", Path::new(CONFIGURATION), &bound).expect_err("the bind mount is a second row");
    assert!(refusal.message().contains("/home/user/monochromatic-bound"), "{}", refusal.message());
}

/// A bind mount of a subtree shows a deeper root,
/// so it is no instance of the dataset and one mounted checkout stays one.
#[test]
fn a_bind_mount_of_a_subtree_is_no_instance_of_the_dataset() {
    let subtree: ScriptedMountTable = ScriptedMountTable::holding(&[
        row(40, 1, "/", CHECKOUT, "zfs", CHECKOUT_DATASET),
        row(41, 1, "/.claude", "/home/user/monochromatic-subtree", "zfs", CHECKOUT_DATASET),
    ]);
    let configuration: SweepConfiguration = parse_configuration("{}", Path::new(CONFIGURATION), &subtree).expect("a subtree bind mount is no instance");
    assert_eq!(configuration.checkout, PathBuf::from(CHECKOUT));
    assert_eq!(configuration.checkout_dataset, CHECKOUT_DATASET);
}

/// A table with no instance of the design's leaf name is refused with the leaf
/// name it looked for and the rows that carried it.
#[test]
fn no_instance_is_refused_with_the_leaf_name_and_the_rows_that_carried_it() {
    let absent: ScriptedMountTable = ScriptedMountTable::holding(&[
        row(1, 0, "/", "/", "zfs", "zpcachyos/ROOT/cos/root"),
        row(41, 1, "/", "/home/user/other", "zfs", "zpcachyos/ROOT/cos/home/another-checkout"),
    ]);
    let refusal: ConfigurationRefusal = parse_configuration("{}", Path::new(CONFIGURATION), &absent).expect_err("no row is the design's dataset");
    let said: String = refusal.message();
    assert!(said.contains("`monochromatic-git-checkout`"), "the refusal names the leaf it looked for: {said}");
    assert!(said.contains("none, so no row's source carried it"), "the refusal says the table held none under that leaf: {said}");
    assert!(said.contains("plan step 19"), "the refusal names the setup step: {said}");
    assert_eq!(refusal.key(), Some(format!("{CLONE_MOUNTS_KEY}.{CHECKOUT_DATASET_KEY}")));
}

/// A leaf name that only a bind mount of a subtree carries is refused,
/// and the refusal lists that row so a person sees what the table held.
#[test]
fn a_leaf_name_only_a_subtree_bind_mount_carries_is_refused_and_listed() {
    let subtree_only: ScriptedMountTable = ScriptedMountTable::holding(&[row(41, 1, "/sub", "/home/user/bound", "zfs", CHECKOUT_DATASET)]);
    let refusal: ConfigurationRefusal = parse_configuration("{}", Path::new(CONFIGURATION), &subtree_only).expect_err("a subtree bind mount is no instance");
    let said: String = refusal.message();
    assert!(said.contains("/home/user/bound"), "the refusal lists the row it found: {said}");
    assert!(said.contains("with root /sub"), "the refusal says the row's root is deeper: {said}");
}

/// A mount table this process cannot read is refused rather than read as a
/// machine with no checkout.
#[test]
fn a_table_that_cannot_be_read_is_refused() {
    let unreadable: ScriptedMountTable = ScriptedMountTable::unreadable("permission denied");
    let refusal: ConfigurationRefusal = parse_configuration("{}", Path::new(CONFIGURATION), &unreadable).expect_err("an unread table names nothing");
    assert!(refusal.message().contains("could not be read"), "{}", refusal.message());
    assert!(refusal.message().contains("permission denied"), "{}", refusal.message());
}

/// A checkout that is on another dataset of another leaf name is refused,
/// because the origin a clone may name is that other dataset's snapshot.
#[test]
fn a_checkout_dataset_with_another_name_is_refused() {
    let other: String = format!(r#"{{ "{CLONE_MOUNTS_KEY}": {{ "{CHECKOUT_KEY}": "{CHECKOUT}" }} }}"#);
    let elsewhere: ScriptedMountTable = ScriptedMountTable::holding(&[row(40, 1, "/", CHECKOUT, "zfs", "zpcachyos/ROOT/cos/home/another-checkout")]);
    let refusal: ConfigurationRefusal = parse_configuration(other.as_str(), Path::new(CONFIGURATION), &elsewhere).expect_err("another dataset is not the design's name");
    assert_eq!(refusal.key(), Some(format!("{CLONE_MOUNTS_KEY}.{CHECKOUT_DATASET_KEY}")));
    assert!(refusal.message().contains("another-checkout"), "the refusal names the dataset it found: {}", refusal.message());
}

/// A checkout on no dataset is refused,
/// because the design's own name only finds a dataset.
#[test]
fn a_checkout_on_no_dataset_is_refused() {
    let other: String = format!(r#"{{ "{CLONE_MOUNTS_KEY}": {{ "{CHECKOUT_KEY}": "{CHECKOUT}" }} }}"#);
    let tmpfs: ScriptedMountTable = ScriptedMountTable::holding(&[row(40, 1, "/", CHECKOUT, "tmpfs", "tmpfs")]);
    let refusal: ConfigurationRefusal = parse_configuration(other.as_str(), Path::new(CONFIGURATION), &tmpfs).expect_err("a tmpfs checkout names no dataset");
    assert!(refusal.message().contains("rather than on a ZFS dataset"), "{}", refusal.message());
}

/// An entry the file holds wins over what the table would say,
/// entry by entry:
/// two mounted instances are no refusal once the file names the one to use.
#[test]
fn an_entry_the_file_holds_wins_over_what_the_table_would_say() {
    let two_pools: ScriptedMountTable = ScriptedMountTable::holding(&[
        row(40, 1, "/", CHECKOUT, "zfs", CHECKOUT_DATASET),
        row(50, 1, "/", "/home/user/second", "zfs", "otherpool/ROOT/cos/home/monochromatic-git-checkout"),
    ]);
    let named: String = document(PARENT, CHECKOUT, CHECKOUT_DATASET);
    let configuration: SweepConfiguration = parse_configuration(named.as_str(), Path::new(CONFIGURATION), &two_pools).expect("the file names the instance");
    assert_eq!(configuration.checkout, PathBuf::from(CHECKOUT));
    assert_eq!(configuration.checkout_dataset, CHECKOUT_DATASET);
    let container_named: String = format!(r#"{{ "{CLONE_MOUNTS_KEY}": {{ "{CHECKOUT_KEY}": "{CHECKOUT}", "{CHECKOUT_DATASET_KEY}": "{CHECKOUT_DATASET}", "{PARENT_KEY}": "zpcachyos/ROOT/cos/home/another-container" }} }}"#);
    let chosen: SweepConfiguration = parse_configuration(container_named.as_str(), Path::new(CONFIGURATION), &two_pools).expect("the file names the container");
    assert_eq!(chosen.parent_dataset, "zpcachyos/ROOT/cos/home/another-container");
}

/// The clone container this run uses is the one the file names,
/// so a clone row inside that container is no second instance of the checkout,
/// and the pass's stoppage does not happen.
#[test]
fn a_file_naming_another_container_ignores_its_clone_row() {
    let named: &str = "zpcachyos/ROOT/cos/home/my-clones";
    let clone: String = format!("{named}/monochromatic-git-checkout");
    let rows: Vec<String> = vec![row(40, 1, "/", CHECKOUT, "zfs", CHECKOUT_DATASET), row(41, 1, "/", "/home/user/Monochromatic/.claude/worktrees/monochromatic-git-checkout", "zfs", clone.as_str())];
    let table: ScriptedMountTable = ScriptedMountTable::holding(&rows);
    let named_parent: String = format!(r#"{{ "{CLONE_MOUNTS_KEY}": {{ "{PARENT_KEY}": "{named}" }} }}"#);
    let configuration: SweepConfiguration = parse_configuration(named_parent.as_str(), Path::new(CONFIGURATION), &table).expect("the clone is in the container this run uses");
    assert_eq!(configuration.parent_dataset, named);
    assert_eq!(configuration.checkout, PathBuf::from(CHECKOUT));
    assert_eq!(configuration.checkout_dataset, CHECKOUT_DATASET);
}

/// A clone row inside the designed container counts once the file names another
/// container,
/// which this run does not use,
/// and the sweep refuses on several as the project's first answer accepts.
#[test]
fn a_row_in_a_container_this_run_does_not_use_is_several() {
    let named: &str = "zpcachyos/ROOT/cos/home/my-clones";
    let designed_clone: String = format!("{PARENT}/monochromatic-git-checkout");
    let rows: Vec<String> = vec![row(40, 1, "/", CHECKOUT, "zfs", CHECKOUT_DATASET), row(41, 1, "/", "/home/user/clone", "zfs", designed_clone.as_str())];
    let table: ScriptedMountTable = ScriptedMountTable::holding(&rows);
    let named_parent: String = format!(r#"{{ "{CLONE_MOUNTS_KEY}": {{ "{PARENT_KEY}": "{named}" }} }}"#);
    let refusal: ConfigurationRefusal = parse_configuration(named_parent.as_str(), Path::new(CONFIGURATION), &table).expect_err("a row in another container is a second instance");
    assert!(refusal.message().contains("will not choose between them"), "{}", refusal.message());
}

/// The success of a file that names only the checkout path on a real file,
/// which the integration pass found the branch had lost.
#[test]
fn a_file_naming_only_the_checkout_path_is_read_through_the_table() {
    let fixture: Fixture = Fixture::new("path-only");
    let file: PathBuf = fixture.write(format!(r#"{{ "{CLONE_MOUNTS_KEY}": {{ "{CHECKOUT_KEY}": "{CHECKOUT}" }} }}"#).as_str());
    let configuration: SweepConfiguration =
        read_configuration(file.as_path(), &ScriptedMountTable::text(&the_designs_table(CHECKOUT, CHECKOUT_DATASET))).expect("the table names the dataset the path is on");
    assert_eq!(configuration.checkout, PathBuf::from(CHECKOUT));
    assert_eq!(configuration.checkout_dataset, CHECKOUT_DATASET);
    assert_eq!(configuration.parent_dataset, PARENT);
}

/// A commented document holding a trailing comma answers both probe keys,
/// which the integration pass found the branch had lost as a host control.
#[test]
fn a_commented_document_with_a_trailing_comma_answers_both_probe_keys() {
    let commented: String = format!(
        "// written by the setup run\n{{\n  \"{PROBE_OBJECT_KEY}\": {{ \"{CHECKOUT_KEY}\": \"{CHECKOUT}\", \"dataset\": \"zpcachyos/nosnap/git-worktree-probe\", }},\n}}\n"
    );
    let configuration: SweepConfiguration = parse_configuration(commented.as_str(), Path::new(CONFIGURATION), &ScriptedMountTable::text(&the_designs_table(CHECKOUT, CHECKOUT_DATASET)))
        .expect("a commented document with a trailing comma parses");
    assert_eq!(configuration.checkout, PathBuf::from(CHECKOUT));
}

/// A file that names a dataset the table mounts once wins over the design's own
/// leaf name the table also holds.
#[test]
fn a_named_dataset_wins_over_the_leaf_name_the_table_holds() {
    let two_datasets: ScriptedMountTable = ScriptedMountTable::holding(&[
        row(40, 1, "/", CHECKOUT, "zfs", CHECKOUT_DATASET),
        row(50, 1, "/", "/home/user/encrypted-mount", "zfs", "zpcachyos/ROOT/cos/home/encrypted/monochromatic-git-checkout"),
    ]);
    let named: String = document(PARENT, "/home/user/encrypted-mount", "zpcachyos/ROOT/cos/home/encrypted/monochromatic-git-checkout");
    let configuration: SweepConfiguration = parse_configuration(named.as_str(), Path::new(CONFIGURATION), &two_datasets).expect("the file names every entry");
    assert_eq!(configuration.checkout, PathBuf::from("/home/user/encrypted-mount"));
    assert_eq!(configuration.checkout_dataset, "zpcachyos/ROOT/cos/home/encrypted/monochromatic-git-checkout");
}

/// A dataset name of a pool alone has no sibling to name,
/// so the container's own entry is what a person writes.
#[test]
fn a_container_that_cannot_be_named_beside_a_pool_name_is_refused() {
    let named: String = format!(r#"{{ "{CLONE_MOUNTS_KEY}": {{ "{CHECKOUT_KEY}": "{CHECKOUT}", "{CHECKOUT_DATASET_KEY}": "monochromatic-git-checkout" }} }}"#);
    let refusal: ConfigurationRefusal = parse_configuration(named.as_str(), Path::new(CONFIGURATION), &ScriptedMountTable::text(&the_designs_table(CHECKOUT, CHECKOUT_DATASET))).expect_err("a pool-name dataset has no sibling");
    assert_eq!(refusal.key(), Some(format!("{CLONE_MOUNTS_KEY}.{PARENT_KEY}")));
    assert!(refusal.message().contains("has no parent component"), "{}", refusal.message());
}

/// A file that is not JSONC is refused as such rather than as a file that names
/// nothing.
#[test]
fn a_document_that_does_not_parse_is_refused() {
    let refusal: ConfigurationRefusal = parse_configuration("{ \"cloneMounts\": ", Path::new(CONFIGURATION), &ScriptedMountTable::text(&the_designs_table(CHECKOUT, CHECKOUT_DATASET))).expect_err("a broken document names nothing");
    assert!(matches!(refusal, ConfigurationRefusal::NotJsonc { .. }), "{refusal:?}");
    assert!(refusal.message().contains("not one JSONC document"), "{}", refusal.message());
}

/// A value of the wrong type is refused as not text rather than read as an entry
/// the file left out.
#[test]
fn a_value_of_another_type_is_refused_as_not_text() {
    let numbered: String = format!(r#"{{ "{CLONE_MOUNTS_KEY}": {{ "{PARENT_KEY}": 7 }} }}"#);
    let refusal: ConfigurationRefusal = parse_configuration(numbered.as_str(), Path::new(CONFIGURATION), &ScriptedMountTable::text(&the_designs_table(CHECKOUT, CHECKOUT_DATASET))).expect_err("a number is not text");
    assert!(matches!(refusal, ConfigurationRefusal::NotText { .. }), "{refusal:?}");
    assert!(refusal.message().contains("is not text"), "{}", refusal.message());
}

/// A container that is no dataset name is refused by the kernel's own rule.
#[test]
fn a_parent_that_is_no_dataset_name_is_refused() {
    let refusal: ConfigurationRefusal = parse_configuration(document("zpcachyos/clones@snapshot", CHECKOUT, CHECKOUT_DATASET).as_str(), Path::new(CONFIGURATION), &ScriptedMountTable::text(&the_designs_table(CHECKOUT, CHECKOUT_DATASET)))
        .expect_err("a snapshot is no parent dataset");
    assert!(matches!(refusal, ConfigurationRefusal::NameRefused { .. }), "{refusal:?}");
    assert_eq!(refusal.key(), Some(format!("{CLONE_MOUNTS_KEY}.{PARENT_KEY}")));
    assert!(refusal.message().contains("snapshot or a bookmark"), "the message names the cause: {}", refusal.message());
}

/// A relative checkout is refused,
/// because it would be resolved against the handler's own directory.
#[test]
fn a_relative_checkout_is_refused() {
    let refusal: ConfigurationRefusal = parse_configuration(document(PARENT, "Monochromatic", CHECKOUT_DATASET).as_str(), Path::new(CONFIGURATION), &ScriptedMountTable::text(&the_designs_table(CHECKOUT, CHECKOUT_DATASET)))
        .expect_err("a relative path is refused");
    assert!(matches!(refusal, ConfigurationRefusal::CheckoutNotAbsolute { .. }), "{refusal:?}");
    assert!(refusal.message().contains("not an absolute checkout path"), "{}", refusal.message());
}

/// Two keys naming two checkouts are still refused,
/// because one probe's pass says nothing about a walk that starts elsewhere.
#[test]
fn two_checkouts_are_refused() {
    let differing: String = format!(
        r#"{{ "{PROBE_OBJECT_KEY}": {{ "{CHECKOUT_KEY}": "/home/user/another" }}, "{CLONE_MOUNTS_KEY}": {{ "{PARENT_KEY}": "{PARENT}", "{CHECKOUT_KEY}": "{CHECKOUT}", "{CHECKOUT_DATASET_KEY}": "{CHECKOUT_DATASET}" }} }}"#
    );
    let refusal: ConfigurationRefusal = parse_configuration(differing.as_str(), Path::new(CONFIGURATION), &ScriptedMountTable::text(&the_designs_table(CHECKOUT, CHECKOUT_DATASET))).expect_err("two checkouts are refused");
    assert!(matches!(refusal, ConfigurationRefusal::CheckoutsDiffer { .. }), "{refusal:?}");
    assert!(refusal.message().contains("/home/user/another"), "the refusal names the probe's checkout: {}", refusal.message());
}

/// The same checkout named by both objects is accepted.
#[test]
fn one_checkout_named_twice_is_accepted() {
    let agreeing: String = format!(
        r#"{{ "{PROBE_OBJECT_KEY}": {{ "{CHECKOUT_KEY}": "{CHECKOUT}" }}, "{CLONE_MOUNTS_KEY}": {{ "{PARENT_KEY}": "{PARENT}", "{CHECKOUT_KEY}": "{CHECKOUT}", "{CHECKOUT_DATASET_KEY}": "{CHECKOUT_DATASET}" }} }}"#
    );
    let configuration: SweepConfiguration = parse_configuration(agreeing.as_str(), Path::new(CONFIGURATION), &ScriptedMountTable::text(&the_designs_table(CHECKOUT, CHECKOUT_DATASET))).expect("one checkout is accepted");
    assert_eq!(configuration.checkout, PathBuf::from(CHECKOUT));
}

/// A file that is there and cannot be read is refused,
/// because it won a precedence it cannot fill.
#[test]
fn a_file_that_is_there_and_unreadable_is_refused() {
    let fixture: Fixture = Fixture::new("unreadable");
    let directory: PathBuf = fixture.path.join("a-directory.jsonc");
    std::fs::create_dir_all(directory.as_path()).expect("the fixture folder is made");
    let refusal: ConfigurationRefusal = read_configuration(directory.as_path(), &ScriptedMountTable::text(&the_designs_table(CHECKOUT, CHECKOUT_DATASET))).expect_err("a folder is not a file");
    assert!(matches!(refusal, ConfigurationRefusal::Unreadable { .. }), "{refusal:?}");
    assert!(refusal.message().contains("could not be read"), "{}", refusal.message());
}

/// A file that is there and holds nothing is refused,
/// and the message says an empty file is not the same as no file.
#[test]
fn an_empty_file_is_refused_and_says_it_is_not_the_same_as_no_file() {
    let fixture: Fixture = Fixture::new("empty");
    let file: PathBuf = fixture.write("");
    let refusal: ConfigurationRefusal = read_configuration(file.as_path(), &ScriptedMountTable::text(&the_designs_table(CHECKOUT, CHECKOUT_DATASET)))
        .expect_err("an empty file is refused rather than read as a file that names nothing");
    assert!(matches!(refusal, ConfigurationRefusal::Empty { .. }), "{refusal:?}");
    let said: String = refusal.message();
    assert!(said.contains("is empty"), "the message says the file is empty: {said}");
    assert!(said.contains("an empty file is not the same as no file"), "the message says the difference: {said}");
    assert!(said.contains("remove the file") || said.contains("is removed"), "the message names the way out: {said}");
    assert!(!said.contains("not one JSONC document"), "an empty file is not worded as a broken one: {said}");
    let whitespace: PathBuf = fixture.write("\n  \t\n");
    let blank_refusal: ConfigurationRefusal = read_configuration(whitespace.as_path(), &ScriptedMountTable::text(&the_designs_table(CHECKOUT, CHECKOUT_DATASET)))
        .expect_err("a file of spaces is as empty as an empty one");
    assert!(matches!(blank_refusal, ConfigurationRefusal::Empty { .. }), "{blank_refusal:?}");
}

/// The file is read from disk,
/// so the release path and a control take the same route to the same parse.
#[test]
fn the_file_is_read_and_parsed() {
    let fixture: Fixture = Fixture::new("read");
    let file: PathBuf = fixture.write(document(PARENT, CHECKOUT, CHECKOUT_DATASET).as_str());
    let configuration: SweepConfiguration = read_configuration(file.as_path(), &ScriptedMountTable::text(&the_designs_table(CHECKOUT, CHECKOUT_DATASET))).expect("the fixture file names every key");
    assert_eq!(configuration.parent_dataset, PARENT);
    assert_eq!(configuration.checkout_dataset, CHECKOUT_DATASET);
    assert_eq!(configuration.checkout, PathBuf::from(CHECKOUT));
}
