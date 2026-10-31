//! Unit tests of `verify-archive`:
//! the bootstrap check of a downloaded file against the lock.

use std::fs;

use super::error::ErrorKind;
use super::fixtures::sample_scope_on_disk;
use super::hash::sha256_hex;
use super::lock::render_lock;
use super::model::Artifact;
use super::verify_archive::run_verify_archive;

/// Writes a lock whose `node` row records the hash of `bytes`,
/// and returns a downloaded file holding `downloaded`.
fn scope_with_node_hash(bytes: &[u8], downloaded: &[u8]) -> (tempfile::TempDir, super::scope::Scope, std::path::PathBuf) {
    let (directory, scope, _env) = sample_scope_on_disk();
    let mut lock = super::fixtures::sample_lock();
    let node = lock.tools.iter_mut().find(|tool| return tool.declaration.id == "node").expect("node");
    node.artifacts = vec![("linux-x64".to_string(), Artifact { url: "https://example.invalid/node.tgz".to_string(), sha256: sha256_hex(bytes) })];
    fs::write(&scope.lock_file, render_lock(&lock)).expect("write the lock");
    let file = directory.path().join("downloaded.tgz");
    fs::write(&file, downloaded).expect("write the download");
    return (directory, scope, file);
}

#[test]
fn a_file_with_the_recorded_hash_is_verified() {
    let (_directory, scope, file) = scope_with_node_hash(b"archive", b"archive");
    let line = run_verify_archive(&scope, "linux-x64", "node", &file).expect("verified");
    assert!(line.starts_with("verified "), "{line}");
    assert!(line.contains(&sha256_hex(b"archive")), "{line}");
    assert!(line.contains("node 1.2.3 on linux-x64"), "{line}");
}

#[test]
fn a_tampered_file_is_refused_with_both_hashes() {
    let (_directory, scope, file) = scope_with_node_hash(b"archive", b"tampered");
    let error = run_verify_archive(&scope, "linux-x64", "node", &file).expect_err("the hash differs");
    assert_eq!(error.kind, ErrorKind::Failure);
    assert!(error.message.contains(&sha256_hex(b"archive")), "{error}");
    assert!(error.message.contains(&sha256_hex(b"tampered")), "{error}");
    assert!(error.message.contains("do not run it"), "{error}");
}

#[test]
fn an_unknown_tool_or_platform_is_a_configuration_error_naming_what_the_lock_has() {
    let (_directory, scope, file) = scope_with_node_hash(b"archive", b"archive");
    let tool = run_verify_archive(&scope, "linux-x64", "nope", &file).expect_err("no such tool");
    assert_eq!(tool.kind, ErrorKind::Config);
    let platform = run_verify_archive(&scope, "macos-arm64", "node", &file).expect_err("no such platform");
    assert!(platform.message.contains("it records: linux-x64"), "{platform}");
}

#[test]
fn a_missing_file_is_reported_and_a_hand_edited_lock_cannot_vouch() {
    let (_directory, scope, file) = scope_with_node_hash(b"archive", b"archive");
    fs::remove_file(&file).expect("remove");
    assert!(run_verify_archive(&scope, "linux-x64", "node", &file).expect_err("no file").message.contains("cannot open"));
    fs::write(&file, b"archive").expect("restore");
    let text = fs::read_to_string(&scope.lock_file).expect("read");
    fs::write(&scope.lock_file, text.replace(&sha256_hex(b"archive"), &sha256_hex(b"tampered"))).expect("hand edit");
    let refused = run_verify_archive(&scope, "linux-x64", "node", &file).expect_err("the digest fails first");
    assert!(refused.message.contains("edited by hand"), "{refused}");
}

#[test]
fn without_a_lock_there_is_nothing_to_verify_against() {
    let (_directory, scope, file) = scope_with_node_hash(b"archive", b"archive");
    fs::remove_file(&scope.lock_file).expect("remove the lock");
    let error = run_verify_archive(&scope, "linux-x64", "node", &file).expect_err("no lock");
    assert!(error.message.contains("no recorded SHA-256"), "{error}");
}
