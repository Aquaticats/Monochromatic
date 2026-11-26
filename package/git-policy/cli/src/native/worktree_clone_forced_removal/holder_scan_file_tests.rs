//! Controls for the second root-owned file beside the receipt:
//!  a result written by the scan's own writer reads back and answers its request,
//!  a missing file is a refusal and never a clear answer,
//!  a result for another dataset or another nonce is refused by name,
//!  and a file another account owns or of another mode is refused before its bytes are read.

use crate::clone_holder_scan::holder_scan::HolderScan;
use crate::worktree_clone_forced_removal::holder_scan_file::*;
use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

/// The GUID and the nonce a control's result answers.
fn a_request() -> (DatasetGuid, RequestNonce) {
    let dataset: DatasetGuid =
        DatasetGuid::parse("11709359031163289601").expect("a canonical GUID");
    let nonce: RequestNonce =
        RequestNonce::parse("0123456789abcdef0123456789abcdef").expect("a nonce");
    return (dataset, nonce);
}

/// An empty scan under one request,
/// which is the result its own writer makes.
fn a_clear_scan(
    dataset: DatasetGuid,
    nonce: RequestNonce,
) -> HolderScan {
    return HolderScan {
        dataset,
        source: String::from("tank/one"),
        nonce,
        own_namespace: None,
        namespaces_scanned: 3,
        own_mounts: Vec::new(),
        holders: Vec::new(),
        incomplete: Vec::new(),
        time: String::from("2026-11-25T12:00:00-05:00"),
    };
}

/// A folder a control owns,
/// with the mode the design gives the results folder,
/// under the crate's own fixture root and removed by the control that made it.
fn a_folder(name: &str) -> (PathBuf, ScanFile) {
    let tree: PathBuf = crate::testing::test_support::fixture_root().join(name);
    let _removed: std::io::Result<()> = std::fs::remove_dir_all(&tree);
    std::fs::create_dir_all(&tree).expect("the fixture folder");
    std::fs::set_permissions(&tree, std::fs::Permissions::from_mode(0o755))
        .expect("the folder's mode");
    let scan_file: ScanFile = ScanFile::at(tree.clone(), libc_uid());
    return (tree, scan_file);
}

/// The account this control runs as,
/// which is the owner a fixture folder has.
fn libc_uid() -> u32 {
    // SAFETY: `getuid` takes no arguments and writes no memory.
    return unsafe { libc::getuid() };
}

/// A written result reads back,
/// answers the request it was written under,
/// and the file's name carries the GUID and the suffix.
#[test]
fn a_written_result_reads_back() {
    let (tree, scan_file) = a_folder("scan-file-reads");
    let (dataset, nonce) = a_request();
    let scan: HolderScan = a_clear_scan(dataset, nonce.clone());
    let path: PathBuf = tree.join(scan_file.entry_name(dataset));
    std::fs::write(&path, scan.text()).expect("the result is written");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644))
        .expect("the file's mode");
    assert_eq!(
        scan_file.entry_name(dataset),
        format!("{}.holder-scan", dataset.file_name())
    );
    let read: HolderScan = scan_file
        .read_the_scan_file(dataset, &nonce)
        .expect("the result reads");
    assert_eq!(read, scan);
    assert!(read.is_clear());
}

/// A missing file is a refusal,
/// so a receipt beside it is never read as a clear scan.
#[test]
fn a_missing_file_is_a_refusal() {
    let (tree, scan_file) = a_folder("scan-file-missing");
    let (dataset, nonce) = a_request();
    let path: PathBuf = tree.join(scan_file.entry_name(dataset));
    match scan_file.read_the_scan_file(dataset, &nonce) {
        Err(ScanFileRefusal::Absent {
            path: found,
        }) => assert_eq!(found, path),
        other => panic!("a missing file is a refusal, not {other:?}"),
    }
}

/// A result for another request of the same dataset,
/// and one for another dataset,
/// are refused by name.
#[test]
fn a_result_for_another_request_is_stale() {
    let (tree, scan_file) = a_folder("scan-file-stale");
    let (dataset, nonce) = a_request();
    let scan: HolderScan = a_clear_scan(dataset, nonce.clone());
    let path: PathBuf = tree.join(scan_file.entry_name(dataset));
    std::fs::write(&path, scan.text()).expect("the result is written");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644))
        .expect("the file's mode");
    let other: RequestNonce =
        RequestNonce::parse("fedcba9876543210fedcba9876543210").expect("a nonce");
    assert!(matches!(
        scan_file.read_the_scan_file(dataset, &other),
        Err(ScanFileRefusal::Stale { .. })
    ));
    let elsewhere: DatasetGuid = DatasetGuid::parse("11709359031163289602").expect("a GUID");
    assert!(matches!(
        scan_file.read_the_scan_file(elsewhere, &nonce),
        Err(ScanFileRefusal::Absent { .. })
    ));
}

/// A file of another mode,
/// and a text that is not a result,
/// are refused before anything in it is used.
#[test]
fn a_wrong_mode_or_a_foreign_text_is_refused() {
    let (tree, scan_file) = a_folder("scan-file-modes");
    let (dataset, nonce) = a_request();
    let path: PathBuf = tree.join(scan_file.entry_name(dataset));
    std::fs::write(&path, a_clear_scan(dataset, nonce.clone()).text())
        .expect("the result is written");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
        .expect("a mode the design does not give");
    assert!(matches!(
        scan_file.read_the_scan_file(dataset, &nonce),
        Err(ScanFileRefusal::Mode { .. })
    ));
    std::fs::write(&path, "not a scan result\n").expect("a foreign text is written");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644))
        .expect("the file's mode");
    assert!(matches!(
        scan_file.read_the_scan_file(dataset, &nonce),
        Err(ScanFileRefusal::NotAResult { .. })
    ));
    // A folder that is not there at all is a folder refusal and not an absent file.
    let gone: PathBuf = tree.join("no-such-folder");
    let elsewhere: ScanFile = ScanFile::at(gone, libc_uid());
    assert!(matches!(
        elsewhere.read_the_scan_file(dataset, &nonce),
        Err(ScanFileRefusal::Folder { .. })
    ));
}
