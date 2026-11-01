//! Controls for transaction pack keeps on disposable object directories.

use super::*;
use crate::test_support::{fixture, remove};

/// Keep messages name the transaction.
#[test]
fn keep_messages_name_the_transaction() {
    assert_eq!(transaction_keep_message("0b6c"), "cli-git 0b6c");
}

/// Only keeps holding exactly this transaction's message are removed.
#[test]
fn only_owned_keeps_are_removed() {
    let root: PathBuf = fixture("pack-keep");
    let objects: PathBuf = root.join("objects");
    assert_eq!(
        remove_transaction_keeps(objects.as_path(), "0b6c").expect("no pack directory"),
        0
    );
    let pack: PathBuf = objects.join("pack");
    std::fs::create_dir_all(&pack).expect("pack directory");
    std::fs::write(pack.join("pack-a.keep"), b"cli-git 0b6c\n").expect("owned keep");
    std::fs::write(pack.join("pack-b.keep"), b"cli-git 0b6c\n").expect("second owned keep");
    std::fs::write(pack.join("pack-c.keep"), b"cli-git 0b6c").expect("no newline");
    std::fs::write(pack.join("pack-d.keep"), b"cli-git ffff\n").expect("other transaction");
    std::fs::write(pack.join("pack-e.keep"), b"").expect("empty keep");
    std::fs::write(pack.join("pack-f.pack"), b"cli-git 0b6c\n").expect("not a keep");
    std::fs::write(pack.join("pack-f.keep.tmp"), b"cli-git 0b6c\n").expect("not a keep either");
    assert_eq!(
        remove_transaction_keeps(objects.as_path(), "0b6c").expect("removed"),
        2
    );
    assert!(!pack.join("pack-a.keep").exists());
    assert!(!pack.join("pack-b.keep").exists());
    for kept in [
        "pack-c.keep",
        "pack-d.keep",
        "pack-e.keep",
        "pack-f.pack",
        "pack-f.keep.tmp",
    ] {
        assert!(pack.join(kept).exists(), "{kept}");
    }
    remove_pack_keep(objects.as_path(), "d").expect("named keep");
    assert!(!pack.join("pack-d.keep").exists());
    remove_pack_keep(objects.as_path(), "d").expect("absent keep");
    std::fs::create_dir(pack.join("pack-g.keep")).expect("directory keep");
    assert!(remove_transaction_keeps(objects.as_path(), "0b6c").is_err());
    assert!(remove_pack_keep(objects.as_path(), "g").is_err());
    std::fs::remove_dir(pack.join("pack-g.keep")).expect("remove directory keep");
    std::fs::remove_dir_all(&pack).expect("remove pack");
    std::fs::write(&pack, b"").expect("pack is a file");
    assert!(remove_transaction_keeps(objects.as_path(), "0b6c").is_err());
    remove(root.as_path());
}
