//! What:
//!  Controls for the `user-cache` pin and the cache descriptor.
//! Why:
//!  The pin's source is the ordering check and its inode the root check,
//!  and each must fail alone;
//!  the descriptor must reach a fixture's `.cache` with no mount at all,
//!  which is the shape this host's tests can give it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! checkUserCacheMount(cache, pin, { source: 'pool/cache', root: '/' });
//! ```

use crate::user_cache_pin::{check_pin, open_cache, passwd_home_of, pin_from_text, read_pin, UserCachePin, PIN_KIND, PIN_VERSION};
use crate::fs_mount_options::MountFacts;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

/// The pin a fixture writes.
fn pin(root_inode: u64) -> UserCachePin {
    return UserCachePin { source: String::from("pool/cache"), root_inode, device: 9 };
}

/// One mount's facts,
/// with the fields the pin check compares.
fn facts(source: &str) -> MountFacts {
    return MountFacts {
        mount_id: 7,
        parent_id: 1,
        root: PathBuf::from("/"),
        optional_fields: Vec::new(),
        mount_point: PathBuf::from("/home/one/.cache"),
        filesystem_type: String::from("zfs"),
        source: String::from(source),
        mount_options: Vec::new(),
        super_options: Vec::new(),
    };
}

/// The pin text carries its source and never its device as a compared field.
#[test]
fn one_pin_survives_its_text() {
    let written: UserCachePin = pin(34);
    let text: String = written.encode();
    assert!(text.contains(PIN_VERSION));
    assert!(text.contains(PIN_KIND));
    assert_eq!(pin_from_text(text.as_str()), Ok(written));
}

/// A pin file that is absent is a refusal, not a silent default.
#[test]
fn an_absent_pin_is_refused() {
    let path: PathBuf = crate::testing::test_support::fixture_root().join("user-cache-pin-absent/pin");
    assert!(matches!(read_pin(path.as_path()), Err(crate::user_cache_pin::PinRefusal::Missing { .. })));
}

/// A pin of another shape is refused.
#[test]
fn a_foreign_pin_is_refused() {
    assert!(pin_from_text("{}").is_err());
    assert!(pin_from_text("{\"version\":\"v2\",\"kind\":\"user-cache-pin\",\"source\":\"a\",\"root-inode\":\"1\",\"device\":\"1\"}").is_err());
    assert!(pin_from_text("{\"version\":\"v1\",\"kind\":\"user-cache-pin\",\"source\":\"a\",\"root-inode\":\"x\",\"device\":\"1\"}").is_err());
}

/// The cache descriptor reaches a fixture's `.cache` and the pin passes on the fixture's own mount.
#[test]
fn the_cache_descriptor_and_the_pin_agree() {
    let home: PathBuf = crate::testing::test_support::fixture_root().join("user-cache-pin-home");
    let cache_path: PathBuf = home.join(".cache");
    std::fs::create_dir_all(cache_path.as_path()).expect("the fixture's cache is made");
    let cache: std::fs::File = open_cache(home.as_path()).expect("the fixture's cache opens");
    let inode: u64 = cache.metadata().expect("the cache's metadata is read").ino();
    check_pin(&cache, &pin(inode), &facts("pool/cache")).expect("the fixture's own mount passes");
}

/// A pin of another source fails on the source and names both.
#[test]
fn a_pin_of_another_source_is_refused() {
    let home: PathBuf = crate::testing::test_support::fixture_root().join("user-cache-pin-source");
    std::fs::create_dir_all(home.join(".cache").as_path()).expect("the fixture's cache is made");
    let cache: std::fs::File = open_cache(home.as_path()).expect("the fixture's cache opens");
    let inode: u64 = cache.metadata().expect("the cache's metadata is read").ino();
    let refusal = check_pin(&cache, &pin(inode), &facts("pool/home")).expect_err("another source is refused");
    assert!(matches!(refusal, crate::user_cache_pin::PinRefusal::Source { .. }));
}

/// A pin whose root inode is another folder fails on the inode.
#[test]
fn a_pin_of_another_inode_is_refused() {
    let home: PathBuf = crate::testing::test_support::fixture_root().join("user-cache-pin-inode");
    std::fs::create_dir_all(home.join(".cache").as_path()).expect("the fixture's cache is made");
    let cache: std::fs::File = open_cache(home.as_path()).expect("the fixture's cache opens");
    let inode: u64 = cache.metadata().expect("the cache's metadata is read").ino();
    let refusal = check_pin(&cache, &pin(inode + 1), &facts("pool/cache")).expect_err("another inode is refused");
    assert!(matches!(refusal, crate::user_cache_pin::PinRefusal::Inode { .. }));
}

/// The account's passwd home is absolute and exists for this run's own account.
#[test]
fn the_passwd_home_is_the_accounts() {
    // SAFETY: `getuid` reads this process's own real UID and touches no memory of ours.
    let uid: u32 = unsafe { libc::getuid() };
    let home: PathBuf = passwd_home_of(uid).expect("this run's own account has a passwd home");
    assert!(home.is_absolute());
    assert!(home.exists());
}
