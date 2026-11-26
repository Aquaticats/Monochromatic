//! What:
//!  Controls of the two names the probe acts on:
//!  the file's `startProbe` object,
//!  the design's own names the mount table fills in,
//!  and every refusal that names the word the record carries.
//! Why:
//!  The project answered on 2026-11-22 that a missing file no longer leaves the
//!  route blocked,
//!  so a probe with no file has to find its dataset by its design's name and its
//!  checkout as the sweep finds one
//!  (`doc/decision/cli-git-zfs-start-probe.md`,
//!  "Answer of 2026-11-22:
//! the probe needs no configuration file").
//!  Each control scripts the text of a mount table,
//!  so nothing on this machine is read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(sourceName(undefined, path, table, undefined).snapshotName()).toBe('pool/nosnap/git-worktree-probe@ready');
//! ```

use super::{
    CHECKOUT_KEY,
    DATASET_KEY,
    ProbeCheckout,
    START_PROBE_KEY,
    checkout_path,
    read_configuration,
    source_name,
};
use crate::start_probe::probe_configuration_refusal::{
    CheckoutRefusal,
    SourceRefusal,
};
use crate::start_probe::probe_source_name::ProbeSource;
use crate::testing::mounted_dataset_table::{
    ScriptedMountTable,
    row,
    the_designs_table,
};
use crate::zfs_mounted_dataset::mounted_dataset_instance::{
    MountHoldingAPath,
    MountedDatasetInstance,
    MountedDatasets,
};
use std::cell::Cell;
use std::path::{
    Path,
    PathBuf,
};

/// The configuration the controls read as if it were the installed one.
const CONFIGURATION: &str = "/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc";

/// The checkout dataset the design names,
/// in the fixture's pool.
const CHECKOUT_DATASET: &str = "zpcachyos/ROOT/cos/home/monochromatic-git-checkout";

/// The probe dataset the design names in that pool.
const PROBE_DATASET: &str = "zpcachyos/nosnap/git-worktree-probe";

/// The checkout the controls name.
const CHECKOUT: &str = "/home/user/Monochromatic";

/// A checkout the file named,
/// which carries no dataset because the file named a folder and not a table row.
fn named_checkout() -> ProbeCheckout {
    return ProbeCheckout {
        path: PathBuf::from(CHECKOUT),
        dataset: None,
    };
}

/// The design's own table.
fn designed() -> ScriptedMountTable {
    return ScriptedMountTable::text(&the_designs_table(CHECKOUT, CHECKOUT_DATASET));
}

/// One `startProbe` object holding both names.
fn both(
    dataset: &str,
    checkout: &str,
) -> String {
    return format!(
        r#"{{ "{START_PROBE_KEY}": {{ "{DATASET_KEY}": "{dataset}", "{CHECKOUT_KEY}": "{checkout}" }} }}"#
    );
}

/// With no file the probe takes the design's probe dataset in the pool the
/// mounted checkout dataset names,
/// and that dataset's mount point as its checkout.
#[test]
fn no_file_takes_the_designs_probe_dataset_and_the_mounted_checkout() {
    let table: ScriptedMountTable = designed();
    let checkout: ProbeCheckout =
        checkout_path(None, &table).expect("the table holds the checkout once");
    assert_eq!(checkout.path, PathBuf::from(CHECKOUT));
    let source: ProbeSource = source_name(None, Path::new(CONFIGURATION), &table, Some(&checkout))
        .expect("the design's probe dataset is named");
    assert_eq!(source.dataset, PROBE_DATASET);
    assert_eq!(
        source.snapshot_name(),
        "zpcachyos/nosnap/git-worktree-probe@ready"
    );
}

/// A file that holds no `startProbe` object does what no file does.
#[test]
fn a_file_holding_no_start_probe_object_takes_the_designs_names() {
    let table: ScriptedMountTable = designed();
    let document: &str = r#"{"cloneMounts":{"checkout":"/home/user/Monochromatic"}}"#;
    let checkout: ProbeCheckout =
        checkout_path(Some(document), &table).expect("the table holds the checkout once");
    assert_eq!(checkout.path, PathBuf::from(CHECKOUT));
    let source: ProbeSource = source_name(
        Some(document),
        Path::new(CONFIGURATION),
        &table,
        Some(&checkout),
    )
    .expect("the design's names fill in");
    assert_eq!(source.dataset, PROBE_DATASET);
}

/// A file that names both uses both,
/// and a dataset of its own wins over the design's name.
#[test]
fn a_file_naming_both_uses_both() {
    let table: ScriptedMountTable = designed();
    let document: String = both(
        "zpcachyos/nosnap/another-probe",
        "/home/user/another-checkout",
    );
    let checkout: ProbeCheckout =
        checkout_path(Some(document.as_str()), &table).expect("the file names the checkout");
    assert_eq!(checkout.path, PathBuf::from("/home/user/another-checkout"));
    let source: ProbeSource = source_name(
        Some(document.as_str()),
        Path::new(CONFIGURATION),
        &table,
        Some(&checkout),
    )
    .expect("the file names the dataset");
    assert_eq!(source.dataset, "zpcachyos/nosnap/another-probe");
}

/// A file that names the dataset and no checkout takes the mounted one.
#[test]
fn a_start_probe_object_without_the_checkout_takes_the_mounted_one() {
    let table: ScriptedMountTable = designed();
    let document: String =
        format!(r#"{{ "{START_PROBE_KEY}": {{ "{DATASET_KEY}": "{PROBE_DATASET}" }} }}"#);
    let checkout: ProbeCheckout =
        checkout_path(Some(document.as_str()), &table).expect("the table holds the checkout once");
    assert_eq!(checkout.path, PathBuf::from(CHECKOUT));
}

/// The probe reads the clone-container key as well as its own object,
/// so a clone row inside the container this run uses is no second instance.
#[test]
fn a_clone_row_in_the_container_the_file_names_is_ignored() {
    let named: &str = "zpcachyos/ROOT/cos/home/my-clones";
    let clone: String = format!("{named}/monochromatic-git-checkout");
    let rows: Vec<String> = vec![
        row(40, 1, "/", CHECKOUT, "zfs", CHECKOUT_DATASET),
        row(
            41,
            1,
            "/",
            "/home/user/Monochromatic/.claude/worktrees/clone",
            "zfs",
            clone.as_str(),
        ),
    ];
    let table: ScriptedMountTable = ScriptedMountTable::holding(&rows);
    let document: String = format!(r#"{{ "cloneMounts": {{ "parent": "{named}" }} }}"#);
    let checkout: ProbeCheckout = checkout_path(Some(document.as_str()), &table)
        .expect("a clone in the named container is no second instance");
    assert_eq!(checkout.path, PathBuf::from(CHECKOUT));
}

/// Two mounted instances of the design's leaf name are refused,
/// and the reason word says several.
#[test]
fn two_mounted_instances_are_refused_with_the_several_word() {
    let rows: Vec<String> = vec![
        row(40, 1, "/", CHECKOUT, "zfs", CHECKOUT_DATASET),
        row(
            50,
            1,
            "/",
            "/home/user/second",
            "zfs",
            "otherpool/ROOT/cos/home/monochromatic-git-checkout",
        ),
    ];
    let table: ScriptedMountTable = ScriptedMountTable::holding(&rows);
    let refusal: CheckoutRefusal =
        checkout_path(None, &table).expect_err("two instances name no single checkout");
    assert_eq!(refusal.word(), "several-mounted-checkouts");
    assert!(
        refusal
            .message(Path::new(CONFIGURATION))
            .contains("otherpool"),
        "{}",
        refusal.message(Path::new(CONFIGURATION))
    );
}

/// No mounted instance leaves the checkout untold with its own word,
/// and a named probe dataset still lets the source be told.
#[test]
fn no_mounted_instance_leaves_the_checkout_untold() {
    let table: ScriptedMountTable =
        ScriptedMountTable::holding(&[row(1, 0, "/", "/", "zfs", "zpcachyos/ROOT/cos/root")]);
    let refusal: CheckoutRefusal =
        checkout_path(None, &table).expect_err("no instance names no checkout");
    assert_eq!(refusal.word(), "no-mounted-checkout");
    let document: String = both(PROBE_DATASET, CHECKOUT);
    let source: ProbeSource = source_name(
        Some(document.as_str()),
        Path::new(CONFIGURATION),
        &table,
        None,
    )
    .expect("a named dataset needs no checkout");
    assert_eq!(source.dataset, PROBE_DATASET);
}

/// A file that names no dataset and this run told no checkout has no pool,
/// so the source is untold with the word that says so.
#[test]
fn no_checkout_and_no_named_dataset_leaves_the_source_untold() {
    let table: ScriptedMountTable =
        ScriptedMountTable::holding(&[row(1, 0, "/", "/", "zfs", "zpcachyos/ROOT/cos/root")]);
    let refusal: SourceRefusal =
        source_name(None, Path::new(CONFIGURATION), &table, None).expect_err("no pool can be told");
    assert_eq!(refusal.word(), "no-mounted-checkout");
    assert!(
        refusal
            .message(Path::new(CONFIGURATION))
            .contains(&format!("{START_PROBE_KEY}.{DATASET_KEY}")),
        "{}",
        refusal.message(Path::new(CONFIGURATION))
    );
}

/// A checkout path that is not absolute is refused,
/// because a relative one would be resolved against the worker's directory.
#[test]
fn a_relative_checkout_is_refused() {
    let document: String = both(PROBE_DATASET, "Monochromatic");
    let refusal: CheckoutRefusal = checkout_path(Some(document.as_str()), &designed())
        .expect_err("a relative path is refused");
    assert_eq!(refusal.word(), "name-refused");
}

/// A dataset name the kernel refuses is refused by its own rule.
#[test]
fn a_dataset_name_the_kernel_refuses_is_refused() {
    let document: String = both("zpcachyos/nosnap/probe@ready", CHECKOUT);
    let refusal: SourceRefusal = source_name(
        Some(document.as_str()),
        Path::new(CONFIGURATION),
        &designed(),
        Some(&named_checkout()),
    )
    .expect_err("a snapshot is no dataset");
    assert_eq!(refusal.word(), "name-refused");
    assert!(
        refusal
            .message(Path::new(CONFIGURATION))
            .contains("snapshot or a bookmark"),
        "{}",
        refusal.message(Path::new(CONFIGURATION))
    );
}

/// A value of the wrong type is refused rather than read as an entry the file
/// left out.
#[test]
fn a_value_of_another_type_is_refused() {
    let numbered: String = format!(r#"{{ "{START_PROBE_KEY}": {{ "{DATASET_KEY}": 7 }} }}"#);
    let refusal: SourceRefusal = source_name(
        Some(numbered.as_str()),
        Path::new(CONFIGURATION),
        &designed(),
        Some(&named_checkout()),
    )
    .expect_err("a number is not text");
    assert_eq!(refusal.word(), "name-not-text");
    let numbered_checkout: String =
        format!(r#"{{ "{START_PROBE_KEY}": {{ "{CHECKOUT_KEY}": 7 }} }}"#);
    let checkout_refusal: CheckoutRefusal =
        checkout_path(Some(numbered_checkout.as_str()), &designed())
            .expect_err("a number is not text");
    assert_eq!(checkout_refusal.word(), "name-not-text");
}

/// A document that does not parse is refused as unparsable rather than as a
/// file naming nothing.
#[test]
fn a_document_that_does_not_parse_is_refused() {
    let refusal: SourceRefusal = source_name(
        Some("{ \"startProbe\": "),
        Path::new(CONFIGURATION),
        &designed(),
        None,
    )
    .expect_err("a broken document names nothing");
    assert_eq!(refusal.word(), "configuration-unparsable");
    let checkout_refusal: CheckoutRefusal = checkout_path(Some("{ \"startProbe\": "), &designed())
        .expect_err("a broken document names nothing");
    assert_eq!(checkout_refusal.word(), "configuration-unparsable");
}

/// A file that is not there is no longer a refusal,
/// and a file that is there and unreadable still is.
#[test]
fn a_missing_file_is_no_refusal_and_an_unreadable_one_is() {
    let folder: PathBuf = crate::test_support::fixture("probe-configuration-unreadable");
    let absent: PathBuf = folder.join("nothing.jsonc");
    assert_eq!(
        read_configuration(absent.as_path()).expect("a missing file is no refusal"),
        None
    );
    let directory: PathBuf = folder.join("a-directory.jsonc");
    std::fs::create_dir_all(directory.as_path()).expect("the fixture folder is made");
    let refusal: SourceRefusal =
        read_configuration(directory.as_path()).expect_err("a folder is not a file");
    assert_eq!(refusal.word(), "configuration-unreadable");
    assert!(
        refusal
            .message(directory.as_path())
            .contains("could not be read"),
        "{}",
        refusal.message(directory.as_path())
    );
}

/// What:
///  A configuration file another account could write is refused with its own
///  word,
///  and the same file with no such bit is read,
///  so the refusal is the rule and not a reader that never reads.
/// Why:
///  The record must carry a closed word for this cause like every other file
///  refusal,
///  and the file names the datasets the probe acts on.
#[test]
fn a_configuration_another_account_could_write_is_refused_with_its_own_word() {
    use std::os::unix::fs::PermissionsExt;
    let folder: PathBuf = crate::test_support::fixture("probe-configuration-writable-by-others");
    let file: PathBuf = folder.join("git-worktree-clone-mounts.jsonc");
    std::fs::write(file.as_path(), both(PROBE_DATASET, CHECKOUT).as_str())
        .expect("the fixture configuration is written");
    assert!(
        read_configuration(file.as_path()).is_ok(),
        "a file this account alone may write is the positive control"
    );
    std::fs::set_permissions(file.as_path(), std::fs::Permissions::from_mode(0o666))
        .expect("the fixture mode is set");
    let refusal: SourceRefusal =
        read_configuration(file.as_path()).expect_err("a write bit for another account is refused");
    assert_eq!(refusal.word(), "configuration-not-under-root-control");
    let said: String = refusal.message(file.as_path());
    assert!(
        said.contains("0666"),
        "the refusal names the mode it found: {said}"
    );
    assert!(
        said.contains("chmod go-w"),
        "the refusal names what to run: {said}"
    );
}

/// A file that is there and holds nothing is refused,
/// and the message says an empty file is not the same as no file.
#[test]
fn an_empty_file_is_refused_and_says_it_is_not_the_same_as_no_file() {
    let folder: PathBuf = crate::test_support::fixture("probe-configuration-empty");
    let file: PathBuf = folder.join("git-worktree-clone-mounts.jsonc");
    std::fs::write(file.as_path(), "").expect("the empty fixture configuration is written");
    let refusal: SourceRefusal =
        read_configuration(file.as_path()).expect_err("an empty file is refused");
    assert_eq!(refusal.word(), "configuration-empty");
    let said: String = refusal.message(file.as_path());
    assert!(
        said.contains("is empty"),
        "the message says the file is empty: {said}"
    );
    assert!(
        said.contains("an empty file is not the same as no file"),
        "the message says the difference: {said}"
    );
    let checkout_refusal: CheckoutRefusal =
        checkout_path(Some(""), &designed()).expect_err("an empty document tells no checkout");
    assert_eq!(checkout_refusal.word(), "configuration-empty");
}

/// The file is read from disk and both names come from the same bytes.
#[test]
fn the_file_is_read_once_and_both_names_come_from_it() {
    let folder: PathBuf = crate::test_support::fixture("probe-configuration-read");
    let file: PathBuf = folder.join("git-worktree-clone-mounts.jsonc");
    std::fs::write(file.as_path(), both(PROBE_DATASET, CHECKOUT).as_str())
        .expect("the fixture configuration is written");
    let document: Option<String> =
        read_configuration(file.as_path()).expect("the fixture file is readable");
    let checkout: ProbeCheckout =
        checkout_path(document.as_deref(), &designed()).expect("the file names the checkout");
    assert_eq!(checkout.path, PathBuf::from(CHECKOUT));
    let source: ProbeSource = source_name(
        document.as_deref(),
        file.as_path(),
        &designed(),
        Some(&checkout),
    )
    .expect("the file names the dataset");
    assert_eq!(source.dataset, PROBE_DATASET);
}

/// A mount table that counts the two questions asked of it,
/// so a control can prove how many reads of system state one look-up costs.
struct CountingMountTable {
    /// The table both questions are answered from.
    table: ScriptedMountTable,
    /// How many times every row was asked for.
    table_reads: Cell<usize>,
    /// How many times the mount a path is on was asked for.
    path_reads: Cell<usize>,
}

impl CountingMountTable {
    /// The design's table,
    /// counting its questions.
    fn over(table: ScriptedMountTable) -> Self {
        return CountingMountTable {
            table,
            table_reads: Cell::new(0),
            path_reads: Cell::new(0),
        };
    }
}

impl MountedDatasets for CountingMountTable {
    fn mounted_datasets(&self) -> Result<Vec<MountedDatasetInstance>, String> {
        self.table_reads.set(self.table_reads.get() + 1);
        return self.table.mounted_datasets();
    }

    fn mount_holding(
        &self,
        path: &Path,
    ) -> Result<Option<MountHoldingAPath>, String> {
        self.path_reads.set(self.path_reads.get() + 1);
        return self.table.mount_holding(path);
    }
}

/// What:
///  With no file the look-up asks the mount table once,
///  and takes the probe dataset's pool from the row that carried the checkout.
/// Why:
///  The row answers both what the walk opens and which dataset the checkout is on
///  (`ProbeCheckout`),
///  so asking the table a second time for the answer the first read held is work a
///  person waits for at boot and nothing else.
///  Without the change the source arm read the whole table again,
///  which this control counts as a second read and a path question.
#[test]
fn no_file_asks_the_mount_table_once() {
    let table: CountingMountTable = CountingMountTable::over(designed());
    let checkout: ProbeCheckout =
        checkout_path(None, &table).expect("the table holds the checkout once");
    assert_eq!(checkout.dataset.as_deref(), Some(CHECKOUT_DATASET));
    let source: ProbeSource = source_name(None, Path::new(CONFIGURATION), &table, Some(&checkout))
        .expect("the design's probe dataset is named");
    assert_eq!(source.dataset, PROBE_DATASET);
    assert_eq!((table.table_reads.get(), table.path_reads.get()), (1, 0));
}

/// What:
///  A file that names the checkout and no dataset asks the table for the one
///  answer only the table holds.
/// Why:
///  The file named a folder and not a row,
///  so the dataset the pool comes from is a question the path can only answer
///  through the table,
///  and this states the one read such a file costs.
#[test]
fn a_file_naming_the_checkout_reads_the_table_once_for_its_dataset() {
    let table: CountingMountTable = CountingMountTable::over(designed());
    let document: String =
        format!(r#"{{ "{START_PROBE_KEY}": {{ "{CHECKOUT_KEY}": "{CHECKOUT}" }} }}"#);
    let checkout: ProbeCheckout =
        checkout_path(Some(document.as_str()), &table).expect("the file names the checkout");
    assert_eq!(checkout.path, PathBuf::from(CHECKOUT));
    assert_eq!(checkout.dataset, None);
    assert_eq!((table.table_reads.get(), table.path_reads.get()), (0, 0));
    let source: ProbeSource = source_name(
        Some(document.as_str()),
        Path::new(CONFIGURATION),
        &table,
        Some(&checkout),
    )
    .expect("the design's probe dataset is named");
    assert_eq!(source.dataset, PROBE_DATASET);
    assert_eq!((table.table_reads.get(), table.path_reads.get()), (0, 1));
}
