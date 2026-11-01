//! Controls for the macOS resolver, which the Linux build does not compile.

use super::*;

/// The directory holding the package resolves to a non-empty identity that names its source,
///  by volume UUID or by device number.
#[test]
fn a_real_directory_resolves_to_a_prefixed_identity() {
    let directory: PathBuf = std::fs::canonicalize(env!("CARGO_MANIFEST_DIR")).expect("the package directory exists");
    let identity: String = darwin_fs_id(directory.as_path()).expect("macOS resolves an identity for an existing directory");
    let volume: &str = source_prefix(FsIdSource::VolumeUuid);
    let device: &str = source_prefix(FsIdSource::DeviceNumber);
    let payload: &str = identity
        .strip_prefix(volume)
        .or_else(|| { return identity.strip_prefix(device); })
        .unwrap_or_else(|| { panic!("{identity:?} names neither {volume:?} nor {device:?}"); });
    assert!(!payload.is_empty(), "{identity:?}");
    assert_eq!(filesystem_identity(directory.as_path()), Some(identity));
}
