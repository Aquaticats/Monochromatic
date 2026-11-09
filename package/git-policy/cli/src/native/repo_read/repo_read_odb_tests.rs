//! Differential controls for the in-process object database:
//!  every object of fixture repositories (loose,
//! packed with deltas,
//! SHA-256,
//! alternates) reads as `git cat-file` prints it,
//!  and every repository whose object names mean more than the files say is declined.

use super::*;
use crate::repo_read_object::{ObjectKind, from_hex};
use crate::test_support::{fixture, git, git_output, remove};

/// A repository with a large file edited many times (so a repack makes deltas),
/// a subtree,
/// a tag and a merge.
fn history(root: &Path, name: &str, format: &str) -> PathBuf {
    let repo = root.join(name);
    std::fs::create_dir(&repo).expect("dir");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main", &format!("--object-format={format}")]);
    let lines: Vec<String> = (0..600).map(|number| { return format!("line {number} of the large fixture file\n"); }).collect();
    std::fs::write(repo.join("big.txt"), lines.concat()).expect("big");
    std::fs::create_dir_all(repo.join("sub/deeper")).expect("dirs");
    std::fs::write(repo.join("sub/deeper/leaf.txt"), "leaf\n").expect("leaf");
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=first"]);
    for step in 0..12 {
        let mut edited = lines.clone();
        edited[step * 7] = format!("edited in step {step}\n");
        std::fs::write(repo.join("big.txt"), edited.concat()).expect("edit");
        git(repo.as_path(), &["commit", "--quiet", "--all", "--message", &format!("step {step}\n\nbody")]);
    }
    git(repo.as_path(), &["tag", "-a", "-m", "annotated", "v1"]);
    git(repo.as_path(), &["tag", "light"]);
    return repo;
}

/// Every object of the repository as `(name, kind word, bytes)` according to real Git.
fn git_objects(repo: &Path) -> Vec<(String, String, Vec<u8>)> {
    let listing = git_output(repo, &["cat-file", "--batch-all-objects", "--batch-check=%(objectname) %(objecttype)"]);
    let mut objects = Vec::new();
    for line in String::from_utf8(listing.stdout).expect("UTF-8").lines() {
        let (name, kind) = line.split_once(' ').expect("two words");
        let content = git_output(repo, &["cat-file", kind, name]);
        assert!(content.status.success());
        objects.push((name.to_string(), kind.to_string(), content.stdout));
    }
    return objects;
}

/// The format of the repository at `repo`.
fn format_of(repo: &Path) -> ObjectFormat {
    let named = String::from_utf8(git_output(repo, &["rev-parse", "--show-object-format"]).stdout).expect("UTF-8");
    return if named.trim() == "sha256" { ObjectFormat::Sha256 } else { ObjectFormat::Sha1 };
}

/// Require the database to return every object exactly as Git prints it;
/// returns how many were compared.
fn all_equal(repo: &Path, label: &str) -> usize {
    let format = format_of(repo);
    let database = ObjectDatabase::open(repo.join(".git").as_path(), None, format).expect("plain database");
    let objects = git_objects(repo);
    assert!(!objects.is_empty(), "{label}: the fixture has objects");
    for (name, kind, bytes) in &objects {
        let raw = database.read(&from_hex(name.as_bytes(), format).expect("name")).unwrap_or_else(|| panic!("{label}: {name} was not read"));
        assert_eq!(raw.kind.word(), kind, "{label}: kind of {name}");
        assert_eq!(&raw.bytes, bytes, "{label}: bytes of {name}");
    }
    return objects.len();
}

/// Loose objects of every kind read as Git prints them.
#[test]
fn loose_objects_are_read() {
    let root = fixture("repo-read-odb-loose");
    let repo = history(root.as_path(), "repo", "sha1");
    assert!(git_output(repo.as_path(), &["count-objects", "-v"]).stdout.starts_with(b"count: "));
    all_equal(repo.as_path(), "loose");
    remove(root.as_path());
}

/// Packed objects,
/// with deltas,
/// read as Git prints them,
/// in a SHA-1 and a SHA-256 repository.
#[test]
fn packed_objects_with_deltas_are_read() {
    let root = fixture("repo-read-odb-packed");
    for format in ["sha1", "sha256"] {
        let repo = history(root.as_path(), format, format);
        git(repo.as_path(), &["repack", "-a", "-d", "-f", "--depth=50", "--window=50", "--quiet"]);
        git(repo.as_path(), &["prune-packed"]);
        let pack = std::fs::read_dir(repo.join(".git/objects/pack")).expect("pack dir").flatten().map(|entry| { return entry.path(); }).find(|path| { return path.extension().is_some_and(|ext| { return ext == "idx"; }); }).expect("an index");
        let verified = git_output(repo.as_path(), &["verify-pack", "-v", pack.to_str().expect("UTF-8")]);
        let table = String::from_utf8(verified.stdout).expect("UTF-8");
        // Positive control: an object stored as a delta has seven columns (the depth and the base follow the offset).
        assert!(table.lines().any(|line| { return line.split_whitespace().count() == 7; }), "{format}: the pack has deltas");
        let compared = all_equal(repo.as_path(), format);
        assert!(compared > 20, "{format}: a history this size has more than a few objects");
    }
    remove(root.as_path());
}

/// Objects of a repository that borrows another's objects through `info/alternates` are read.
#[test]
fn alternates_are_followed() {
    let root = fixture("repo-read-odb-alternates");
    let source = history(root.as_path(), "source", "sha1");
    git(source.as_path(), &["repack", "-a", "-d", "-f", "--quiet"]);
    let clone = root.join("clone");
    git(root.as_path(), &["clone", "--quiet", "--shared", source.to_str().expect("UTF-8"), clone.to_str().expect("UTF-8")]);
    assert!(clone.join(".git/objects/info/alternates").exists());
    let database = ObjectDatabase::open(clone.join(".git").as_path(), None, ObjectFormat::Sha1).expect("database");
    let head = String::from_utf8(git_output(clone.as_path(), &["rev-parse", "HEAD"]).stdout).expect("UTF-8");
    let raw = database.read(&from_hex(head.trim().as_bytes(), ObjectFormat::Sha1).expect("name")).expect("an object only the alternate holds");
    assert_eq!(raw.kind, ObjectKind::Commit);
    remove(root.as_path());
}

/// A pack whose own header is damaged and an alternates file that is a link to an endless device are declined.
#[test]
fn a_damaged_pack_and_a_hostile_alternates_file_are_declined() {
    let root = fixture("repo-read-odb-hostile");
    let repo = history(root.as_path(), "repo", "sha1");
    git(repo.as_path(), &["repack", "-a", "-d", "-f", "--quiet"]);
    git(repo.as_path(), &["prune-packed"]);
    let git_dir = repo.join(".git");
    assert!(ObjectDatabase::open(git_dir.as_path(), None, ObjectFormat::Sha1).is_some(), "the intact pack opens");
    let pack = std::fs::read_dir(git_dir.join("objects/pack")).expect("pack dir").flatten().map(|entry| { return entry.path(); }).find(|path| { return path.extension().is_some_and(|ext| { return ext == "pack"; }); }).expect("a pack");
    let original = std::fs::read(&pack).expect("pack bytes");
    std::fs::set_permissions(&pack, std::os::unix::fs::PermissionsExt::from_mode(0o644)).expect("writable");
    let mut damaged = original.clone();
    damaged[0] = b'X';
    std::fs::write(&pack, &damaged).expect("damage");
    assert!(ObjectDatabase::open(git_dir.as_path(), None, ObjectFormat::Sha1).is_none(), "a pack with a bad signature");
    std::fs::write(&pack, &original).expect("restore");
    std::fs::create_dir_all(git_dir.join("objects/info")).expect("info");
    std::os::unix::fs::symlink("/dev/zero", git_dir.join("objects/info/alternates")).expect("link");
    assert!(ObjectDatabase::open(git_dir.as_path(), None, ObjectFormat::Sha1).is_none(), "a link to an endless device");
    remove(root.as_path());
}

/// Replace refs,
/// a shallow file,
/// grafts,
/// a missing object and a damaged loose file are declined.
#[test]
fn what_cannot_be_trusted_is_declined() {
    let root = fixture("repo-read-odb-declined");
    let repo = history(root.as_path(), "repo", "sha1");
    let git_dir = repo.join(".git");
    let missing = vec![0x11_u8; 20];
    let database = ObjectDatabase::open(git_dir.as_path(), None, ObjectFormat::Sha1).expect("database");
    assert!(database.read(missing.as_slice()).is_none(), "an absent object is never answered");
    assert!(database.read(&missing[..10]).is_none(), "a name of the wrong length");
    let head = String::from_utf8(git_output(repo.as_path(), &["rev-parse", "HEAD"]).stdout).expect("UTF-8");
    let name = from_hex(head.trim().as_bytes(), ObjectFormat::Sha1).expect("name");
    let hex = head.trim();
    let loose = git_dir.join("objects").join(&hex[..2]).join(&hex[2..]);
    let original = std::fs::read(&loose).expect("loose");
    // Git writes loose objects read-only.
    std::fs::set_permissions(&loose, std::os::unix::fs::PermissionsExt::from_mode(0o644)).expect("writable");
    std::fs::write(&loose, &original[..original.len() / 2]).expect("truncate");
    assert!(database.read(name.as_slice()).is_none(), "a truncated loose object");
    std::fs::write(&loose, &original).expect("restore");
    assert!(database.read(name.as_slice()).is_some());
    let older = String::from_utf8(git_output(repo.as_path(), &["rev-parse", "HEAD~1"]).stdout).expect("UTF-8");
    git(repo.as_path(), &["replace", older.trim(), hex]);
    assert!(ObjectDatabase::open(git_dir.as_path(), None, ObjectFormat::Sha1).is_none(), "a replace ref");
    git(repo.as_path(), &["replace", "--delete", older.trim()]);
    std::fs::write(git_dir.join("shallow"), format!("{hex}\n")).expect("shallow");
    assert!(ObjectDatabase::open(git_dir.as_path(), None, ObjectFormat::Sha1).is_none(), "a shallow file");
    std::fs::remove_file(git_dir.join("shallow")).expect("remove");
    std::fs::create_dir_all(git_dir.join("info")).expect("info");
    std::fs::write(git_dir.join("info/grafts"), "").expect("grafts");
    assert!(ObjectDatabase::open(git_dir.as_path(), None, ObjectFormat::Sha1).is_none(), "a graft file");
    remove(root.as_path());
}

/// Reading every version of a file kept as one long delta chain decodes each stored entry about once:
/// a base that one read
/// decoded is not decoded again for the next version.
/// Without that,
/// n versions cost about n squared entry decodes,
/// which is
/// what made the in-process listing of a large repository's trees slower than starting Git.
#[test]
fn a_delta_chain_is_decoded_about_once_for_all_its_versions() {
    let root = fixture("repo-read-odb-chain");
    let repo = root.join("chain");
    std::fs::create_dir(&repo).expect("dir");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    let lines: Vec<String> = (0..400).map(|number| { return format!("line {number} of the chained file\n"); }).collect();
    let versions: usize = 40;
    for step in 0..versions {
        let mut edited = lines.clone();
        edited[step * 3] = format!("edited in step {step}\n");
        std::fs::write(repo.join("chain.txt"), edited.concat()).expect("edit");
        git(repo.as_path(), &["add", "chain.txt"]);
        git(repo.as_path(), &["commit", "--quiet", "--message", &format!("step {step}")]);
    }
    git(repo.as_path(), &["repack", "-a", "-d", "-f", "--quiet", "--window=100", "--depth=100"]);
    git(repo.as_path(), &["prune-packed"]);
    let listing = git_output(repo.as_path(), &["rev-list", "--objects", "--all"]);
    let blobs: Vec<String> = String::from_utf8(listing.stdout)
        .expect("UTF-8")
        .lines()
        .filter(|line| { return line.ends_with("chain.txt"); })
        .map(|line| { return line.split(' ').next().expect("name").to_string(); })
        .collect();
    assert_eq!(blobs.len(), versions);
    let format = format_of(repo.as_path());
    let database = ObjectDatabase::open(repo.join(".git").as_path(), None, format).expect("plain database");
    for blob in &blobs {
        let name = from_hex(blob.as_bytes(), format).expect("name");
        let expected = git_output(repo.as_path(), &["cat-file", "blob", blob]).stdout;
        assert_eq!(database.read(&name).expect("read").bytes, expected, "{blob}");
    }
    let decoded = database.decoded_entries();
    // One decode per stored entry would be the versions plus a few; allow twice that for the order the versions are read in.
    assert!(decoded <= 2 * versions as u64 + 8, "{decoded} entry decodes for {versions} versions of one chain");
    remove(root.as_path());
}

/// An object that a pack holds is found in the pack without a look at the loose path first:
/// a commit reads about two thousand
/// objects of its base tree,
/// and each loose look is a system call that fails for a packed object (Git searches packs first,
/// then
/// the loose files).
/// Positive control:
/// an object that is only loose is still read,
/// after the packs were searched.
#[test]
fn a_packed_object_is_read_without_looking_for_a_loose_file() {
    let root = fixture("repo-read-odb-pack-first");
    let repo = history(root.as_path(), "repo", "sha1");
    let packed_names: Vec<(String, Vec<u8>)> = git_objects(repo.as_path()).into_iter().map(|(name, _, bytes)| { return (name, bytes); }).collect();
    git(repo.as_path(), &["repack", "-a", "-d", "-f", "--quiet"]);
    git(repo.as_path(), &["prune-packed"]);
    let format = format_of(repo.as_path());
    let database = ObjectDatabase::open(repo.join(".git").as_path(), None, format).expect("plain database");
    let before: u64 = loose_looks();
    for (name, bytes) in &packed_names {
        let raw = database.read(&from_hex(name.as_bytes(), format).expect("name")).expect("read");
        assert_eq!(&raw.bytes, bytes, "{name}");
    }
    let looks: u64 = loose_looks() - before;
    assert_eq!(looks, 0, "every object of {} was in a pack, so no loose path was looked at", packed_names.len());
    // An object written loose after the repack is found once the packs were searched.
    std::fs::write(repo.join("late.txt"), "late\n").expect("late");
    let late = git_output(repo.as_path(), &["hash-object", "-w", "late.txt"]);
    let late_name: String = String::from_utf8(late.stdout).expect("UTF-8").trim().to_string();
    let loose = ObjectDatabase::open(repo.join(".git").as_path(), None, format).expect("database with a loose object");
    let before_late: u64 = loose_looks();
    let raw = loose.read(&from_hex(late_name.as_bytes(), format).expect("name")).expect("the loose object is read");
    assert_eq!(raw.bytes, b"late\n");
    assert!(loose_looks() > before_late, "the loose file was looked at after the packs");
    remove(root.as_path());
}

/// `contains` finds a packed object without a look at its loose path,
/// which is what a ref listing spends most of its time on,
/// and still finds an object that is only loose.
#[test]
fn contains_asks_the_packs_before_the_loose_paths() {
    let root = fixture("repo-read-odb-contains-pack-first");
    let repo = history(root.as_path(), "repo", "sha1");
    let packed_names: Vec<String> = git_objects(repo.as_path()).into_iter().map(|(name, _, _)| { return name; }).collect();
    git(repo.as_path(), &["repack", "-a", "-d", "-f", "--quiet"]);
    git(repo.as_path(), &["prune-packed"]);
    let format = format_of(repo.as_path());
    let database = ObjectDatabase::open(repo.join(".git").as_path(), None, format).expect("plain database");
    let before: u64 = loose_looks();
    for name in &packed_names {
        assert!(database.contains(from_hex(name.as_bytes(), format).expect("name").as_slice()), "{name} is in a pack");
    }
    assert_eq!(loose_looks() - before, 0, "no loose path was looked at for {} packed objects", packed_names.len());
    std::fs::write(repo.join("late.txt"), "late\n").expect("late");
    let late = git_output(repo.as_path(), &["hash-object", "-w", "late.txt"]);
    let late_name: String = String::from_utf8(late.stdout).expect("UTF-8").trim().to_string();
    let loose = ObjectDatabase::open(repo.join(".git").as_path(), None, format).expect("database with a loose object");
    assert!(loose.contains(from_hex(late_name.as_bytes(), format).expect("name").as_slice()), "the loose object is found");
    assert!(loose_looks() > before, "the loose file was looked at after the packs");
    assert!(!loose.contains(from_hex("1".repeat(40).as_bytes(), format).expect("name").as_slice()), "an object nowhere is not found");
    remove(root.as_path());
}
