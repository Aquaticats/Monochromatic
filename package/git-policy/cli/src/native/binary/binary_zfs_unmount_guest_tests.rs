//! What:
//!  Plan step 9's staging mechanics in the throwaway guest:
//!  the record written before the bind,
//!  a bind made private,
//!  a mount moved onto it,
//!  the later sweep's unmount from the record's handle,
//!  the record of another boot read as void,
//!  and the wrapper's `rmdir`-only cleanup with an allowed and a rejected
//!  fixture.
//! Why:
//!  A bind,
//! a move and an unmount need mount authority,
//!  so these arms run only where `CLI_GIT_ZFS_UNMOUNT_GUEST` names a
//!  disposable guest;
//!  every other machine prints a skip line,
//!  so the gate says what it did not run.
//!  The fixture here is a tmpfs mount standing in for the clone,
//!  because the full ZFS layout,
//!  the built sweep's attach and the design's M1 to M9 remain owed and are
//!  recorded as owed in the plan's step 9 notes;
//!  what this arm proves is the staging and cleanup mechanism the ZFS arms
//!  will drive.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! writeRecord(cache, record); bind(folder); move(tmpfs, bind);
//! expect(finishAStagedUnmount(sweep).word).toBe('unmounted');
//! ```

use git_policy_cli::staged_folder_identity::StagedFolderIdentity;
use git_policy_cli::staged_folder_record::{write_record, StagedFolderRecord};
use git_policy_cli::staged_unmount::{finish_a_staged_unmount, StagedSweep};
use git_policy_cli::staging_calls::{LiveStagingCalls, StagingCalls};
use git_policy_cli::staging_claim::{write_claim, StagingClaim};
use git_policy_cli::staging_cleanup::{clean_up_staging_folder, CleanupOutcome, CleanupRequest};
use git_policy_cli::staging_folder_name::{staging_folder_name, staging_record_name};
use git_policy_cli::user_cache_pin::{cache_mount_facts, UserCachePin};
use git_policy_cli::worktree_clone_mount_protocol::DatasetGuid;
use std::ffi::CString;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

/// The variable the guest driver sets.
const GUEST_VARIABLE: &str = "CLI_GIT_ZFS_UNMOUNT_GUEST";

/// The GUID the arm's folder carries.
const GUID: &str = "9001";

/// Whether this run is inside the guest,
/// or a skip line on every other machine.
fn guest() -> bool {
    let Ok(name) = std::env::var(GUEST_VARIABLE) else {
        println!("SKIP worktree_clone_unmount::guest: {GUEST_VARIABLE} is unset, so this arm runs only in the throwaway mvm guest");
        return false;
    };
    let running: String = std::fs::read_to_string("/etc/hostname").map(|text| return text.trim().to_string()).unwrap_or_default();
    assert_eq!(running, name, "this arm runs only in the guest {name}");
    return true;
}

/// A fixture folder under `/root`.
fn folder(name: &str) -> PathBuf {
    let path: PathBuf = Path::new("/root").join(name);
    let _ = std::fs::remove_dir_all(path.as_path());
    std::fs::create_dir_all(path.as_path()).expect("the fixture folder is made");
    return path;
}

/// Mount a tmpfs at `target` and answer a descriptor on it.
fn mount_tmpfs(target: &Path) -> std::fs::File {
    let at: CString = CString::new(target.as_os_str().as_bytes()).expect("a path holds no NUL");
    let kind: CString = CString::new("tmpfs").expect("no NUL in a type");
    let data: CString = CString::new("size=64k,nr_inodes=16,mode=0700").expect("no NUL in an option");
    // SAFETY: every pointer is this call's own and outlives it.
    let mounted: libc::c_int = unsafe { libc::mount(kind.as_ptr(), at.as_ptr(), kind.as_ptr(), libc::MS_NOSUID | libc::MS_NODEV, data.as_ptr().cast::<libc::c_void>()) };
    assert_eq!(mounted, 0, "the fixture tmpfs mounts: {}", std::io::Error::last_os_error());
    return std::fs::File::open(target).expect("the mounted folder opens");
}

/// The record one arm writes before its bind.
fn record(guid: DatasetGuid, folder_path: &Path, _cache: &std::fs::File, boot: &str) -> StagedFolderRecord {
    let held: std::fs::File = std::fs::File::open(folder_path).expect("the staging folder opens");
    let identity: StagedFolderIdentity = StagedFolderIdentity::of(&held).expect("the staging folder's identity is read");
    let (handle_type, handle): (i32, Vec<u8>) = LiveStagingCalls.name_handle(&held).expect("the folder's handle is read");
    let (namespace_device, namespace_inode): (u64, u64) = LiveStagingCalls.namespace_pair().expect("the namespace pair is read");
    return StagedFolderRecord {
        component: staging_folder_name(guid),
        dataset: String::from("tmpfs"),
        guid,
        identity,
        handle_type,
        handle,
        nonce: String::from("00112233445566778899aabbccddeeff"),
        namespace_device,
        namespace_inode,
        boot: String::from(boot),
    };
}

/// The whole staging and sweep arm,
/// plus the record's other-boot test and the cleanup's two fixtures.
#[test]
fn zfs_unmount_guest() {
    if !guest() {
        return;
    }
    let cache_path: PathBuf = folder("unmount-cache");
    let cache: std::fs::File = std::fs::File::open(cache_path.as_path()).expect("the fixture cache opens");
    let guid: DatasetGuid = DatasetGuid::parse(GUID).expect("the fixture GUID parses");
    let name: String = staging_folder_name(guid);
    let folder_path: PathBuf = cache_path.join(name.as_str());
    std::fs::create_dir_all(folder_path.as_path()).expect("the staging folder is made");
    let boot: String = LiveStagingCalls.boot().expect("this boot is read");
    let facts = cache_mount_facts(&cache).expect("the cache's mount is read");
    let pin: UserCachePin = UserCachePin { source: facts.source.clone(), root_inode: cache.metadata().expect("the cache's metadata is read").ino(), device: 7 };
    let written: StagedFolderRecord = record(guid, folder_path.as_path(), &cache, boot.as_str());
    write_record(&cache, &written).expect("the record is written before the bind");
    println!("UNMOUNT record-before-bind=true folder={name}");
    let staging: std::fs::File = std::fs::File::open(folder_path.as_path()).expect("the staging folder opens");
    let bind: std::fs::File = LiveStagingCalls.bind_the_folder(&staging, &cache, std::ffi::OsStr::new(name.as_str())).expect("the folder binds");
    LiveStagingCalls.make_private(&bind).expect("the bind is made private");
    let clone_path: PathBuf = folder("unmount-clone");
    let clone: std::fs::File = mount_tmpfs(clone_path.as_path());
    LiveStagingCalls.move_clone(&clone, &bind).expect("the clone moves onto the bind");
    println!("UNMOUNT bind-private=true moved=true");
    let sweep = StagedSweep { cache: &cache, cache_path: cache_path.as_path(), guid, boot: boot.as_str(), pin: &pin, calls: &LiveStagingCalls };
    let outcome = finish_a_staged_unmount(&sweep);
    assert_eq!(outcome.word(), "unmounted", "the sweep unmounts the staged mount from the record's handle: {outcome:?}");
    println!("UNMOUNT sweep=unmounted");
    let other = StagedFolderRecord { boot: String::from("an-older-boot"), ..record(guid, folder_path.as_path(), &cache, boot.as_str()) };
    write_record(&cache, &other).expect("the other-boot record is written");
    let other_outcome = finish_a_staged_unmount(&sweep);
    assert_eq!(other_outcome.word(), "not-current", "a record from another boot is void and unmounts nothing: {other_outcome:?}");
    println!("UNMOUNT other-boot=void");
    std::fs::remove_file(cache_path.join(staging_record_name(guid)).as_path()).expect("the other-boot record is removed");
    let claims_path: PathBuf = folder("unmount-claims");
    let claims: std::fs::File = std::fs::File::open(claims_path.as_path()).expect("the claim folder opens");
    let claim: StagingClaim = StagingClaim { guid, nonce: String::from("00112233445566778899aabbccddeeff"), owner: String::from("bogus"), identity: StagedFolderIdentity::of(&std::fs::File::open(folder_path.as_path()).expect("the folder opens")).expect("the identity is read") };
    write_claim(&claims, &claim).expect("the claim is written");
    let no_mounts = || return Vec::new();
    let claim_path: PathBuf = claims_path.join("9001.staged");
    let request = CleanupRequest { cache: &cache, cache_path: cache_path.as_path(), claim_directory: &claims, guid, owner_token: Some("bogus"), mounts: &no_mounts, claim_path: claim_path.as_path() };
    let removed: CleanupOutcome = clean_up_staging_folder(&request);
    assert_eq!(removed.word(), "removed", "an empty claimed folder is removed: {removed:?}");
    println!("UNMOUNT cleanup=removed");
    std::fs::create_dir_all(folder_path.as_path()).expect("the kept folder is made");
    std::fs::write(folder_path.join("sentinel").as_path(), b"keep").expect("the sentinel is written");
    write_claim(&claims, &claim).expect("the claim is written again");
    let kept: CleanupOutcome = clean_up_staging_folder(&request);
    assert_eq!(kept.word(), "kept", "a folder holding a file is kept: {kept:?}");
    println!("UNMOUNT cleanup=kept");
    std::fs::remove_dir_all(cache_path.as_path()).expect("the fixture cache is removed");
    std::fs::remove_dir_all(clone_path.as_path()).expect("the fixture clone is removed");
    std::fs::remove_dir_all(claims_path.as_path()).expect("the fixture claims are removed");
    println!("UNMOUNT done");
}
