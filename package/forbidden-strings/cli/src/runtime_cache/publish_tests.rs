//! Atomic artifact publication tests over disposable directories.

use super::{publish_artifact, read_artifact, ArtifactReadError};
use crate::runtime_cache::path::{cache_location, source_digest};
use std::path::PathBuf;

/// Creates fresh process-scoped test directory.
fn test_directory(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "forbidden-strings-cache-publish-{label}-{}",
        std::process::id(),
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("create test directory");
    return path
}

/// Missing artifact receives distinct read category.
#[test]
fn missing_artifact_is_distinct() {
    let directory = test_directory("missing");
    assert_eq!(
        read_artifact(&directory.join("missing.bin"), 1024),
        Err(ArtifactReadError::Missing),
    );
    let _ = std::fs::remove_dir_all(directory);
}

/// Oversized artifact is rejected before complete read allocation.
#[test]
fn oversized_artifact_is_unreadable() {
    let directory = test_directory("oversized");
    let artifact = directory.join("artifact.bin");
    std::fs::write(&artifact, b"12345").expect("write fixture");
    assert_eq!(read_artifact(&artifact, 4), Err(ArtifactReadError::Unreadable));
    let _ = std::fs::remove_dir_all(directory);
}

/// Publication writes complete bytes and replaces prior artifact.
#[test]
fn publication_replaces_complete_artifact() {
    let root = test_directory("replace");
    let digest = source_digest(b"rules").expect("digest");
    let location = cache_location(&root, digest);

    publish_artifact(&location, b"first").expect("first publication");
    assert_eq!(read_artifact(&location.artifact_path, 1024).expect("read"), b"first");

    publish_artifact(&location, b"second").expect("replacement");
    assert_eq!(read_artifact(&location.artifact_path, 1024).expect("read"), b"second");
    let _ = std::fs::remove_dir_all(root);
}

/// Unix publication enforces owner-only directory and artifact modes.
#[cfg(unix)]
#[test]
fn publication_enforces_private_modes() {
    use std::os::unix::fs::PermissionsExt;

    let root = test_directory("permissions");
    let digest = source_digest(b"rules").expect("digest");
    let location = cache_location(&root, digest);
    publish_artifact(&location, b"private").expect("publication");

    for directory in &location.protected_directories {
        let mode = std::fs::metadata(directory).expect("directory metadata").permissions().mode();
        assert_eq!(mode & 0o777, 0o700);
    }
    let file_mode = std::fs::metadata(&location.artifact_path)
        .expect("artifact metadata")
        .permissions()
        .mode();
    assert_eq!(file_mode & 0o777, 0o600);
    let _ = std::fs::remove_dir_all(root);
}
