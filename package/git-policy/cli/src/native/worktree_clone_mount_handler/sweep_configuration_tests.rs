//! What:
//!  Controls of the sweep's configuration names:
//!  the three keys the setup run writes,
//!  the design's own names a file without the two dataset names is read
//!  through,
//!  and every refusal that names the key a person must write.
//! Why:
//!  A sweep that guessed a parent dataset,
//!  a checkout dataset or a checkout would act on another dataset's children,
//!  and the project decided the two dataset names fall back to the design's
//!  layout rather than being guessed
//!  (`doc/decision/cli-git-zfs-step-8-sweep-choices.md`,
//!  "The datasets the sweep acts on").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(parseConfiguration(fixture, path, mounts)).toEqual({ parentDataset: 'pool/…', checkout: '/home/user/Monochromatic', checkoutDataset: 'pool/…' });
//! ```

use super::{CHECKOUT_DATASET_KEY, CHECKOUT_KEY, CLONE_MOUNTS_KEY, ConfigurationRefusal, PARENT_KEY, PROBE_OBJECT_KEY, SweepConfiguration, parse_configuration, read_configuration};
use crate::worktree_clone_mount_handler::sweep_checkout_mount::{CheckoutMount, CheckoutMountRefusal, CheckoutMounts};
use std::path::{Path, PathBuf};

/// The file every control parses as if it were the installed one.
const CONFIGURATION: &str = "/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc";

/// The checkout dataset the design names.
const CHECKOUT_DATASET: &str = "zpcachyos/ROOT/cos/home/monochromatic-git-checkout";

/// The container the design places beside it.
const PARENT: &str = "zpcachyos/ROOT/cos/home/monochromatic-git-worktree-clone-datasets";

/// The checkout the controls name.
const CHECKOUT: &str = "/home/user/Monochromatic";

/// One scripted mount table.
struct ScriptedMounts {
    /// The dataset the checkout is on.
    dataset: String,
    /// The filesystem of the mount,
    /// which a control names to prove a checkout on no dataset.
    filesystem: String,
    /// The refusal a control hands in instead of a mount.
    refusal: Option<CheckoutMountRefusal>,
}

impl ScriptedMounts {
    /// The design's checkout dataset,
    /// mounted at the design's checkout path.
    fn designed() -> Self {
        return ScriptedMounts { dataset: String::from(CHECKOUT_DATASET), filesystem: String::from("zfs"), refusal: None };
    }
}

impl CheckoutMounts for ScriptedMounts {
    fn mount_holding(&self, _checkout: &Path) -> Result<CheckoutMount, CheckoutMountRefusal> {
        if let Some(refusal) = &self.refusal {
            return Err(refusal.clone());
        }
        return Ok(CheckoutMount { source: self.dataset.clone(), filesystem_type: self.filesystem.clone(), mount_point: PathBuf::from(CHECKOUT), shared: None });
    }
}

/// One whole configuration document naming all three keys.
fn document(parent: &str, checkout: &str, checkout_dataset: &str) -> String {
    return format!(r#"{{ "{CLONE_MOUNTS_KEY}": {{ "{PARENT_KEY}": "{parent}", "{CHECKOUT_KEY}": "{checkout}", "{CHECKOUT_DATASET_KEY}": "{checkout_dataset}" }} }}"#);
}

/// A document naming only the checkout,
/// which is the file that is read through the design's names.
fn checkout_only(checkout: &str) -> String {
    return format!(r#"{{ "{CLONE_MOUNTS_KEY}": {{ "{CHECKOUT_KEY}": "{checkout}" }} }}"#);
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
}

/// The document the setup run writes names all three keys,
/// and a comment beside them changes nothing.
#[test]
fn a_configuration_that_names_every_key_is_read() {
    let commented: String = format!("// written by the setup run\n{}\n", document(PARENT, CHECKOUT, CHECKOUT_DATASET));
    let mounts: ScriptedMounts = ScriptedMounts::designed();
    let configuration: SweepConfiguration = parse_configuration(commented.as_str(), Path::new(CONFIGURATION), &mounts).expect("every key is there");
    assert_eq!(configuration.parent_dataset, PARENT);
    assert_eq!(configuration.checkout, PathBuf::from(CHECKOUT));
    assert_eq!(configuration.checkout_dataset, CHECKOUT_DATASET);
}

/// A file holding no dataset names is read through the design's own names:
/// the dataset the checkout is on,
/// and its sibling container.
#[test]
fn a_file_with_no_dataset_names_is_read_through_the_designs_names() {
    let mounts: ScriptedMounts = ScriptedMounts::designed();
    let configuration: SweepConfiguration = parse_configuration(checkout_only(CHECKOUT).as_str(), Path::new(CONFIGURATION), &mounts).expect("the design's names are found");
    assert_eq!(configuration.checkout_dataset, CHECKOUT_DATASET);
    assert_eq!(configuration.parent_dataset, PARENT);
    assert_eq!(configuration.checkout, PathBuf::from(CHECKOUT));
}

/// A checkout on a dataset that does not carry the design's name is refused
/// with the key a person writes,
/// rather than read against the wrong origin.
#[test]
fn a_checkout_dataset_with_another_name_is_refused() {
    let mounts: ScriptedMounts = ScriptedMounts { dataset: String::from("zpcachyos/ROOT/cos/home/another-checkout"), ..ScriptedMounts::designed() };
    let refusal: ConfigurationRefusal =
        parse_configuration(checkout_only(CHECKOUT).as_str(), Path::new(CONFIGURATION), &mounts).expect_err("another dataset is not the design's name");
    assert_eq!(refusal.key(), format!("{CLONE_MOUNTS_KEY}.{CHECKOUT_DATASET_KEY}"));
    assert!(refusal.message().contains("another-checkout"), "the refusal names the dataset it found: {}", refusal.message());
}

/// A checkout on no dataset is refused,
/// because the design's own name only finds a dataset.
#[test]
fn a_checkout_on_no_dataset_is_refused() {
    let mounts: ScriptedMounts = ScriptedMounts { filesystem: String::from("tmpfs"), ..ScriptedMounts::designed() };
    let refusal: ConfigurationRefusal =
        parse_configuration(checkout_only(CHECKOUT).as_str(), Path::new(CONFIGURATION), &mounts).expect_err("a tmpfs checkout names no dataset");
    assert!(matches!(refusal, ConfigurationRefusal::CheckoutNotADataset { .. }), "{refusal:?}");
    assert!(refusal.message().contains("rather than on a dataset"), "{}", refusal.message());
}

/// A mount that cannot be read is refused rather than guessed at.
#[test]
fn a_mount_that_cannot_be_read_is_refused() {
    let mounts: ScriptedMounts = ScriptedMounts {
        refusal: Some(CheckoutMountRefusal::Unreadable { checkout: PathBuf::from(CHECKOUT), reason: String::from("the table did not read") }),
        ..ScriptedMounts::designed()
    };
    let refusal: ConfigurationRefusal =
        parse_configuration(checkout_only(CHECKOUT).as_str(), Path::new(CONFIGURATION), &mounts).expect_err("an unread mount names nothing");
    assert!(matches!(refusal, ConfigurationRefusal::CheckoutMount { .. }), "{refusal:?}");
}

/// A dataset named without a parent component has no sibling to name,
/// so the fallback refuses rather than inventing one.
#[test]
fn a_container_that_cannot_be_named_beside_a_pool_name_is_refused() {
    let mounts: ScriptedMounts = ScriptedMounts { dataset: String::from("monochromatic-git-checkout"), ..ScriptedMounts::designed() };
    let refusal: ConfigurationRefusal =
        parse_configuration(checkout_only(CHECKOUT).as_str(), Path::new(CONFIGURATION), &mounts).expect_err("a pool-name dataset has no sibling");
    assert!(matches!(refusal, ConfigurationRefusal::NoParentForTheDesignName { .. }), "{refusal:?}");
    assert_eq!(refusal.key(), format!("{CLONE_MOUNTS_KEY}.{PARENT_KEY}"));
}

/// A file that is not there is its own refusal,
/// and the message names the keys a person writes.
#[test]
fn a_missing_file_is_refused_with_the_keys_to_write() {
    let fixture: Fixture = Fixture::new("missing");
    let mounts: ScriptedMounts = ScriptedMounts::designed();
    let absent: PathBuf = fixture.path.join("nothing.jsonc");
    let refusal: ConfigurationRefusal = read_configuration(absent.as_path(), &mounts).expect_err("a missing file names nothing");
    assert!(matches!(refusal, ConfigurationRefusal::Missing { .. }), "{refusal:?}");
    let said: String = refusal.message();
    assert!(said.contains("is not there"), "the message says the file is missing: {said}");
    assert!(said.contains(&format!("{CLONE_MOUNTS_KEY}.{PARENT_KEY}")), "the message names the parent key: {said}");
    assert!(said.contains(&format!("{CLONE_MOUNTS_KEY}.{CHECKOUT_KEY}")), "the message names the checkout key: {said}");
    assert!(said.contains(&format!("{CLONE_MOUNTS_KEY}.{CHECKOUT_DATASET_KEY}")), "the message names the checkout dataset key: {said}");
    assert!(said.contains("attached nothing and recorded nothing"), "the message says nothing acted: {said}");
}

/// A file that is not JSONC is refused as such rather than as an absent key.
#[test]
fn a_document_that_does_not_parse_is_refused() {
    let mounts: ScriptedMounts = ScriptedMounts::designed();
    let refusal: ConfigurationRefusal = parse_configuration("{ \"cloneMounts\": ", Path::new(CONFIGURATION), &mounts).expect_err("a broken document names nothing");
    assert!(matches!(refusal, ConfigurationRefusal::NotJsonc { .. }), "{refusal:?}");
    assert!(refusal.message().contains("not one JSONC document"), "{}", refusal.message());
}

/// The checkout key is required,
/// and a key another part has not written names itself.
#[test]
fn a_missing_checkout_key_is_refused_by_name() {
    let mounts: ScriptedMounts = ScriptedMounts::designed();
    let without_checkout: String = format!(r#"{{ "{CLONE_MOUNTS_KEY}": {{ "{PARENT_KEY}": "{PARENT}" }} }}"#);
    let checkout_refusal: ConfigurationRefusal =
        parse_configuration(without_checkout.as_str(), Path::new(CONFIGURATION), &mounts).expect_err("the checkout key is required");
    assert_eq!(checkout_refusal.key(), format!("{CLONE_MOUNTS_KEY}.{CHECKOUT_KEY}"));
    assert!(checkout_refusal.message().contains("nothing, attached nothing and recorded nothing"), "{}", checkout_refusal.message());
}

/// A value of the wrong type is refused as not text rather than as absent.
#[test]
fn a_value_of_another_type_is_refused_as_not_text() {
    let mounts: ScriptedMounts = ScriptedMounts::designed();
    let numbered: String = format!(r#"{{ "{CLONE_MOUNTS_KEY}": {{ "{PARENT_KEY}": 7, "{CHECKOUT_KEY}": "{CHECKOUT}" }} }}"#);
    let refusal: ConfigurationRefusal = parse_configuration(numbered.as_str(), Path::new(CONFIGURATION), &mounts).expect_err("a number is not text");
    assert!(matches!(refusal, ConfigurationRefusal::NotText { .. }), "{refusal:?}");
    assert!(refusal.message().contains("is not text"), "{}", refusal.message());
}

/// A container that is no dataset name is refused by the kernel's own rule.
#[test]
fn a_parent_that_is_no_dataset_name_is_refused() {
    let mounts: ScriptedMounts = ScriptedMounts::designed();
    let refusal: ConfigurationRefusal = parse_configuration(document("zpcachyos/clones@snapshot", CHECKOUT, CHECKOUT_DATASET).as_str(), Path::new(CONFIGURATION), &mounts)
        .expect_err("a snapshot is no parent dataset");
    assert!(matches!(refusal, ConfigurationRefusal::NameRefused { .. }), "{refusal:?}");
    assert_eq!(refusal.key(), format!("{CLONE_MOUNTS_KEY}.{PARENT_KEY}"));
    assert!(refusal.message().contains("snapshot or a bookmark"), "the message names the cause: {}", refusal.message());
}

/// A relative checkout is refused,
/// because it would be resolved against the handler's own directory.
#[test]
fn a_relative_checkout_is_refused() {
    let mounts: ScriptedMounts = ScriptedMounts::designed();
    let refusal: ConfigurationRefusal = parse_configuration(document(PARENT, "Monochromatic", CHECKOUT_DATASET).as_str(), Path::new(CONFIGURATION), &mounts)
        .expect_err("a relative path is refused");
    assert!(matches!(refusal, ConfigurationRefusal::CheckoutNotAbsolute { .. }), "{refusal:?}");
    assert!(refusal.message().contains("not an absolute checkout path"), "{}", refusal.message());
}

/// Two keys naming two checkouts are refused,
/// because one probe's pass says nothing about a walk that starts elsewhere.
#[test]
fn two_checkouts_are_refused() {
    let mounts: ScriptedMounts = ScriptedMounts::designed();
    let differing: String = format!(
        r#"{{ "{PROBE_OBJECT_KEY}": {{ "{CHECKOUT_KEY}": "/home/user/another" }}, "{CLONE_MOUNTS_KEY}": {{ "{PARENT_KEY}": "{PARENT}", "{CHECKOUT_KEY}": "{CHECKOUT}", "{CHECKOUT_DATASET_KEY}": "{CHECKOUT_DATASET}" }} }}"#
    );
    let refusal: ConfigurationRefusal = parse_configuration(differing.as_str(), Path::new(CONFIGURATION), &mounts).expect_err("two checkouts are refused");
    assert!(matches!(refusal, ConfigurationRefusal::CheckoutsDiffer { .. }), "{refusal:?}");
    assert!(refusal.message().contains("/home/user/another"), "the refusal names the probe's checkout: {}", refusal.message());
}

/// The same checkout named by both objects is accepted.
#[test]
fn one_checkout_named_twice_is_accepted() {
    let mounts: ScriptedMounts = ScriptedMounts::designed();
    let agreeing: String = format!(
        r#"{{ "{PROBE_OBJECT_KEY}": {{ "{CHECKOUT_KEY}": "{CHECKOUT}" }}, "{CLONE_MOUNTS_KEY}": {{ "{PARENT_KEY}": "{PARENT}", "{CHECKOUT_KEY}": "{CHECKOUT}", "{CHECKOUT_DATASET_KEY}": "{CHECKOUT_DATASET}" }} }}"#
    );
    let configuration: SweepConfiguration = parse_configuration(agreeing.as_str(), Path::new(CONFIGURATION), &mounts).expect("one checkout is accepted");
    assert_eq!(configuration.checkout, PathBuf::from(CHECKOUT));
}

/// The file is read from disk,
/// so the release path and a control take the same route to the same parse.
#[test]
fn the_file_is_read_and_parsed() {
    let fixture: Fixture = Fixture::new("read");
    let mounts: ScriptedMounts = ScriptedMounts::designed();
    let file: PathBuf = fixture.write(document(PARENT, CHECKOUT, CHECKOUT_DATASET).as_str());
    let configuration: SweepConfiguration = read_configuration(file.as_path(), &mounts).expect("the fixture file names every key");
    assert_eq!(configuration.parent_dataset, PARENT);
    assert_eq!(configuration.checkout_dataset, CHECKOUT_DATASET);
}
