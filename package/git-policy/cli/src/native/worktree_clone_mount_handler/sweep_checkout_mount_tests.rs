//! What:
//!  Controls of the checkout's mount as the handler reads it:
//!  the dataset a mount names,
//!  its propagation and the refusal of a path whose mount cannot be read.
//! Why:
//!  The origin rule and the private-parent check both rest on this read,
//!  and the fixture answers every shape the design names without this
//!  machine's mount table.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(mount.dataset()).toBe('zpcachyos/ROOT/cos/home/monochromatic-git-checkout');
//! ```

use super::{CheckoutMount, CheckoutMountRefusal, CheckoutMounts, LiveCheckoutMounts, SHARED_FIELD_PREFIX};
use crate::fs_mount_options::parse_mount_table;
use std::path::{Path, PathBuf};

/// One mount a scripted reader answers.
struct ScriptedMounts {
    /// What it answers.
    answer: Result<CheckoutMount, CheckoutMountRefusal>,
}

impl CheckoutMounts for ScriptedMounts {
    fn mount_holding(&self, _checkout: &Path) -> Result<CheckoutMount, CheckoutMountRefusal> {
        return self.answer.clone();
    }
}

/// One mount,
/// with the design's checkout dataset under it.
fn a_checkout_mount(shared: Option<&str>) -> CheckoutMount {
    return CheckoutMount {
        source: String::from("zpcachyos/ROOT/cos/home/monochromatic-git-checkout"),
        filesystem_type: String::from("zfs"),
        mount_point: PathBuf::from("/home/user/Monochromatic"),
        shared: shared.map(String::from),
    };
}

/// A dataset mount answers its dataset,
/// and a mount carrying a `shared:` tag answers that it is shared.
#[test]
fn a_dataset_mount_answers_its_dataset_and_its_shared_tag() {
    let quiet: CheckoutMount = a_checkout_mount(None);
    assert_eq!(quiet.dataset(), Some("zpcachyos/ROOT/cos/home/monochromatic-git-checkout"));
    assert!(!quiet.is_shared());
    let shared: CheckoutMount = a_checkout_mount(Some("shared:217"));
    assert_eq!(shared.dataset(), Some("zpcachyos/ROOT/cos/home/monochromatic-git-checkout"));
    assert!(shared.is_shared(), "a `shared:` tag is a shared mount: {shared:?}");
}

/// A mount of another filesystem is not a dataset mount,
/// so no design name finds a dataset through it.
#[test]
fn another_filesystem_names_no_dataset() {
    let mount: CheckoutMount = CheckoutMount { filesystem_type: String::from("tmpfs"), ..a_checkout_mount(None) };
    assert_eq!(mount.dataset(), None);
}

/// The live reader finds the mount holding a fixture folder,
/// and answers its mount point,
/// its source and its propagation.
#[test]
fn the_live_reader_finds_the_mount_holding_a_fixture_folder() {
    let folder: PathBuf = crate::test_support::fixture("sweep-checkout-mount-live");
    std::fs::create_dir_all(folder.as_path()).expect("the fixture folder is made");
    let mount: CheckoutMount = LiveCheckoutMounts.mount_holding(folder.as_path()).expect("this process's mount table holds the fixture");
    assert!(!mount.source.is_empty(), "the mount names a source: {mount:?}");
    assert!(folder.starts_with(mount.mount_point.as_path()), "the mount's point is an ancestor of the fixture: {mount:?}");
}

/// A path this process cannot resolve is a refusal rather than a guess.
#[test]
fn a_path_that_is_not_there_cannot_be_read() {
    let absent: PathBuf = PathBuf::from("/fixture/there-is-no-such-checkout");
    let refusal: CheckoutMountRefusal = LiveCheckoutMounts.mount_holding(absent.as_path()).expect_err("a path that is not there has no mount");
    assert!(matches!(refusal, CheckoutMountRefusal::Unreadable { .. }), "{refusal:?}");
    assert!(refusal.message().contains("/fixture/there-is-no-such-checkout"), "the refusal names the path: {}", refusal.message());
}

/// The optional fields are the kernel's own,
/// and the propagation is read from them rather than from the option lists.
#[test]
fn the_optional_fields_are_the_kernels_own() {
    let line: &str = "36 25 0:32 / /home/user rw,nosuid shared:217 master:3 - zfs zpcachyos/ROOT/cos/home rw,xattr,posixacl";
    let table = parse_mount_table(line.as_bytes());
    assert_eq!(table.len(), 1, "the line parses: {line}");
    let mount = &table[0];
    assert_eq!(mount.optional_fields, vec![String::from("shared:217"), String::from("master:3")]);
    assert_eq!(mount.optional_field(SHARED_FIELD_PREFIX), Some("shared:217"));
    assert!(!mount.has_option("shared"), "propagation is no mount option");
    assert_eq!(mount.mount_point, PathBuf::from("/home/user"));
    assert_eq!(mount.source, String::from("zpcachyos/ROOT/cos/home"));
}

/// A mount with no optional field answers none,
/// which is the private case the design requires.
#[test]
fn a_mount_with_no_optional_field_answers_none() {
    let line: &str = "36 25 0:32 / /home/user rw,nosuid - zfs zpcachyos/ROOT/cos/home rw,xattr";
    let table = parse_mount_table(line.as_bytes());
    assert_eq!(table.len(), 1);
    assert!(table[0].optional_fields.is_empty());
    assert_eq!(table[0].optional_field(SHARED_FIELD_PREFIX), None);
}

/// The scripted reader answers exactly what a control gives it,
/// which is how the shared and unshared cases elsewhere are driven.
#[test]
fn a_scripted_reader_answers_its_own_mount() {
    let scripted: ScriptedMounts = ScriptedMounts { answer: Ok(a_checkout_mount(Some("shared:1"))) };
    let mount: CheckoutMount = scripted.mount_holding(Path::new("/fixture")).expect("the scripted answer is a mount");
    assert!(mount.is_shared());
}
