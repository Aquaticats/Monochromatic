//! What:
//!  Controls of the sweep's two configuration names.
//! Why:
//!  A sweep that guessed a parent dataset or a checkout would act on another
//!  dataset's children,
//!  so every way the configuration can fail to name them is refused with the
//!  key a person must write.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(readConfiguration(fixture)).toEqual({ parentDataset: 'pool/clones', checkout: '/home/user/Monochromatic' });
//! ```

use super::{CLONE_MOUNTS_KEY, CHECKOUT_KEY, ConfigurationRefusal, PARENT_KEY, SweepConfiguration, parse_configuration, read_configuration};
use std::path::{Path, PathBuf};

/// The file every control parses as if it were the installed one.
const CONFIGURATION: &str = "/etc/monochromatic-git-policy-cli/git-worktree-clone-mounts.jsonc";

/// The parent the controls name.
const PARENT: &str = "zpcachyos/ROOT/cos/home/monochromatic-git-worktree-clone-datasets";

/// The checkout the controls name.
const CHECKOUT: &str = "/home/user/Monochromatic";

/// One whole configuration document.
fn document(parent: &str, checkout: &str) -> String {
    return format!(r#"{{ "{CLONE_MOUNTS_KEY}": {{ "{PARENT_KEY}": "{parent}", "{CHECKOUT_KEY}": "{checkout}" }} }}"#);
}

/// One fixture folder of this control's own,
/// removed however the control ends.
struct Fixture {
    /// The folder the fixture file is written in.
    path: PathBuf,
}

impl Fixture {
    /// A fresh folder under this process's own temporary root.
    fn new() -> Self {
        let path: PathBuf = std::env::temp_dir().join(format!("sweep-configuration-{}", std::process::id()));
        let _removed: std::io::Result<()> = std::fs::remove_dir_all(path.as_path());
        std::fs::create_dir_all(path.as_path()).expect("the fixture folder is made");
        return Fixture { path };
    }

    /// The configuration file inside the fixture folder,
    /// holding `document`.
    fn write(&self, document: &str) -> PathBuf {
        let file: PathBuf = self.path.join("git-worktree-clone-mounts.jsonc");
        std::fs::write(file.as_path(), document).expect("the fixture configuration is written");
        return file;
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _removed: std::io::Result<()> = std::fs::remove_dir_all(self.path.as_path());
    }
}

/// The document the install writes names both keys,
/// and a comment beside them changes nothing.
#[test]
fn a_configuration_that_names_both_keys_is_read() {
    let commented: String = format!("// written by the setup run\n{}\n", document(PARENT, CHECKOUT));
    let configuration: SweepConfiguration = parse_configuration(commented.as_str(), Path::new(CONFIGURATION)).expect("both keys are there");
    assert_eq!(configuration.parent_dataset, PARENT);
    assert_eq!(configuration.checkout, PathBuf::from(CHECKOUT));
    assert_eq!(configuration, SweepConfiguration { parent_dataset: String::from(PARENT), checkout: PathBuf::from(CHECKOUT) });
}

/// A file that is not there is its own refusal,
/// and the message names the two keys a person writes.
#[test]
fn a_missing_file_is_refused_with_the_keys_to_write() {
    let fixture: Fixture = Fixture::new();
    let absent: PathBuf = fixture.path.join("nothing.jsonc");
    let refusal: ConfigurationRefusal = read_configuration(absent.as_path()).expect_err("a missing file names nothing");
    assert!(matches!(refusal, ConfigurationRefusal::Missing { .. }), "{refusal:?}");
    let said: String = refusal.message();
    assert!(said.contains("is not there"), "the message says the file is missing: {said}");
    assert!(said.contains(&format!("{CLONE_MOUNTS_KEY}.{PARENT_KEY}")), "the message names the parent key: {said}");
    assert!(said.contains(&format!("{CLONE_MOUNTS_KEY}.{CHECKOUT_KEY}")), "the message names the checkout key: {said}");
    assert!(said.contains("attached nothing and recorded nothing"), "the message says nothing acted: {said}");
}

/// A file that is not JSONC is refused as such rather than as an absent key.
#[test]
fn a_document_that_does_not_parse_is_refused() {
    let refusal: ConfigurationRefusal = parse_configuration("{ \"cloneMounts\": ", Path::new(CONFIGURATION)).expect_err("a broken document names nothing");
    assert!(matches!(refusal, ConfigurationRefusal::NotJsonc { .. }), "{refusal:?}");
    assert!(refusal.message().contains("not one JSONC document"), "{}", refusal.message());
}

/// Each key is required,
/// and a key another part has not written names itself.
#[test]
fn a_missing_key_is_refused_by_name() {
    let without_parent: String = format!(r#"{{ "{CLONE_MOUNTS_KEY}": {{ "{CHECKOUT_KEY}": "{CHECKOUT}" }} }}"#);
    let parent_refusal: ConfigurationRefusal = parse_configuration(without_parent.as_str(), Path::new(CONFIGURATION)).expect_err("the parent key is required");
    assert_eq!(parent_refusal.key(), format!("{CLONE_MOUNTS_KEY}.{PARENT_KEY}"));
    let without_checkout: String = format!(r#"{{ "{CLONE_MOUNTS_KEY}": {{ "{PARENT_KEY}": "{PARENT}" }} }}"#);
    let checkout_refusal: ConfigurationRefusal = parse_configuration(without_checkout.as_str(), Path::new(CONFIGURATION)).expect_err("the checkout key is required");
    assert_eq!(checkout_refusal.key(), format!("{CLONE_MOUNTS_KEY}.{CHECKOUT_KEY}"));
    assert!(checkout_refusal.message().contains("nothing, attached nothing and recorded nothing"), "{}", checkout_refusal.message());
}

/// A value of the wrong type is refused as not text rather than as absent.
#[test]
fn a_value_of_another_type_is_refused_as_not_text() {
    let numbered: String = format!(r#"{{ "{CLONE_MOUNTS_KEY}": {{ "{PARENT_KEY}": 7, "{CHECKOUT_KEY}": "{CHECKOUT}" }} }}"#);
    let refusal: ConfigurationRefusal = parse_configuration(numbered.as_str(), Path::new(CONFIGURATION)).expect_err("a number is not text");
    assert!(matches!(refusal, ConfigurationRefusal::NotText { .. }), "{refusal:?}");
    assert!(refusal.message().contains("is not text"), "{}", refusal.message());
}

/// A parent that is no dataset name is refused by the kernel's own rule.
#[test]
fn a_parent_that_is_no_dataset_name_is_refused() {
    let refusal: ConfigurationRefusal = parse_configuration(document("zpcachyos/clones@snapshot", CHECKOUT).as_str(), Path::new(CONFIGURATION))
        .expect_err("a snapshot is no parent dataset");
    assert!(matches!(refusal, ConfigurationRefusal::ParentNameRefused { .. }), "{refusal:?}");
    assert!(refusal.message().contains("snapshot or a bookmark"), "the message names the cause: {}", refusal.message());
}

/// A relative checkout is refused,
/// because it would be resolved against the handler's own directory.
#[test]
fn a_relative_checkout_is_refused() {
    let refusal: ConfigurationRefusal = parse_configuration(document(PARENT, "Monochromatic").as_str(), Path::new(CONFIGURATION)).expect_err("a relative path is refused");
    assert!(matches!(refusal, ConfigurationRefusal::CheckoutNotAbsolute { .. }), "{refusal:?}");
    assert!(refusal.message().contains("not an absolute checkout path"), "{}", refusal.message());
}

/// The file is read from disk,
/// so the release path and a control take the same route to the same parse.
#[test]
fn the_file_is_read_and_parsed() {
    let fixture: Fixture = Fixture::new();
    let file: PathBuf = fixture.write(document(PARENT, CHECKOUT).as_str());
    let configuration: SweepConfiguration = read_configuration(file.as_path()).expect("the fixture file names both keys");
    assert_eq!(configuration.parent_dataset, PARENT);
}
