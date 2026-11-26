//! What:
//!  Controls of the fingerprint's reads:
//!  a complete set of fixture files,
//!  a module that carries no build ID,
//!  a missing kernel release and a missing boot id.
//! Why:
//!  A record whose fingerprint cannot be read must not be written as a record
//!  that judges nothing,
//!  and an absent module attribute weakens the record rather than refusing it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const print = ProbeFingerprint.read(paths); expect(print.moduleBuildId).toBeUndefined();
//! ```

use super::{
    FingerprintPaths,
    FingerprintRefusal,
    ProbeFingerprint,
    binary_identity,
    hexadecimal,
};
use crate::testing::test_support::fixture;
use std::path::{
    Path,
    PathBuf,
};

/// One fixture set,
/// every file written,
/// and one attribute left absent.
fn paths(root: &Path) -> FingerprintPaths {
    let write = |name: &str, text: &str| -> PathBuf {
        let path: PathBuf = root.join(name);
        std::fs::write(&path, text).expect("the fixture file is written");
        return path;
    };
    return FingerprintPaths {
        kernel_release: write("osrelease", "6.8.0-147-generic\n"),
        module_version: write("version", "2.4.4-1\n"),
        module_srcversion: write("srcversion", "A3553C308C29FD4B6595A72\n"),
        module_build_id: root.join("no-build-id-note"),
        boot_id: write("boot_id", "6f0f1a2b-3c4d-4e5f-8a9b-0c1d2e3f4a5b\n"),
        handler: root.join("no-handler"),
    };
}

/// The kernel release loses its newline,
/// the module's release and source version are kept,
/// and the absent build ID and handler are absent.
#[test]
fn a_complete_set_reads_with_its_absences_absent() {
    let root: PathBuf = fixture("start-probe-fingerprint");
    let print: ProbeFingerprint =
        ProbeFingerprint::read(&paths(root.as_path())).expect("the fingerprint is read");
    assert_eq!(print.kernel_release, "6.8.0-147-generic");
    assert_eq!(print.module_version.as_deref(), Some("2.4.4-1"));
    assert_eq!(print.boot_id, "6f0f1a2b-3c4d-4e5f-8a9b-0c1d2e3f4a5b");
    assert_eq!(print.module_build_id, None);
    assert_eq!(print.handler, None);
}

/// A missing kernel release refuses the whole fingerprint,
/// and so does a missing boot id.
#[test]
fn a_missing_required_attribute_refuses() {
    let root: PathBuf = fixture("start-probe-fingerprint-missing");
    let mut without_release: FingerprintPaths = paths(root.as_path());
    without_release.kernel_release = root.join("no-such-release");
    assert!(matches!(
        ProbeFingerprint::read(&without_release),
        Err(FingerprintRefusal::KernelRelease { .. })
    ));
    let mut without_boot: FingerprintPaths = paths(root.as_path());
    without_boot.boot_id = root.join("no-such-boot");
    assert!(matches!(
        ProbeFingerprint::read(&without_boot),
        Err(FingerprintRefusal::BootId { .. })
    ));
}

/// A build ID note reads as lowercase hexadecimal,
/// and a handler file's identity names its own path.
#[test]
fn bytes_are_hexadecimal_and_a_file_has_an_identity() {
    assert_eq!(hexadecimal(&[0x00, 0xff, 0x1a]), "00ff1a");
    let root: PathBuf = fixture("start-probe-fingerprint-identity");
    let handler: PathBuf = root.join("handler");
    std::fs::write(&handler, b"text").expect("the fixture file is written");
    let identity: String = binary_identity(handler.as_path()).expect("the file has an identity");
    assert!(
        identity.starts_with(
            root.canonicalize()
                .expect("canonical root")
                .join("handler")
                .to_str()
                .expect("text path")
        )
    );
}
