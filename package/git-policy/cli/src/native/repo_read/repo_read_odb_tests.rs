//! Differential controls for the in-process object database:
//!  every object of fixture repositories (loose,
//! packed with deltas,
//! SHA-256,
//! alternates) reads as `git cat-file` prints it,
//!  and every repository whose object names mean more than the files say is declined.

use super::*;
use crate::repo_read_object::{ObjectKind, from_hex};
use crate::test_support::{FIXTURE_GIT_BOUND, REAL_GIT, fixture, git, git_output, git_text, real_git_within, remove};
use std::ffi::OsString;

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

/// What:
///  A replace ref a reftable repository keeps in its tables declines the database.
/// Why:
///  `what_cannot_be_trusted_is_declined` looks at `refs/replace` and `packed-refs`,
/// and the reftable backend writes neither:
///  the ref is in `reftable/`,
/// so a replacement there would go unseen and the reader would answer the object Git replaces,
///  with the parents and the tree of the commit the replace ref stands for.
/// The control reads the same fixture with Git,
/// whose answer is the replacement,
///  and declines the database only when the ref store is one this reader cannot scan.
#[test]
fn a_reftable_repositorys_replace_ref_declines_the_database() {
    let root = fixture("repo-read-odb-reftable-replace");
    let repo = root.join("repo");
    std::fs::create_dir(&repo).expect("dir");
    git(repo.as_path(), &["init", "--quiet", "--ref-format=reftable", "--initial-branch=main"]);
    std::fs::write(repo.join("a.txt"), "one\n").expect("a");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=first"]);
    std::fs::write(repo.join("a.txt"), "two\n").expect("a");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=second"]);
    let older = String::from_utf8(git_output(repo.as_path(), &["rev-parse", "HEAD~1"]).stdout).expect("UTF-8");
    let head = String::from_utf8(git_output(repo.as_path(), &["rev-parse", "HEAD"]).stdout).expect("UTF-8");
    git(repo.as_path(), &["replace", older.trim(), head.trim()]);
    let git_dir = repo.join(".git");
    // Positive control: the two objects differ, so Git's answer for the replaced name is not the object the files hold.
    let replaced = git_output(repo.as_path(), &["cat-file", "commit", older.trim()]).stdout;
    let unreplaced = git_output(repo.as_path(), &["--no-replace-objects", "cat-file", "commit", older.trim()]).stdout;
    assert_ne!(replaced, unreplaced, "the fixture's replace ref changes what Git reads");
    // The ref store the reader cannot scan: tables instead of the replace dir and the packed refs it looks for.
    assert!(git_dir.join("reftable").is_dir(), "the repository keeps its refs in reftable tables");
    assert!(!git_dir.join("refs/replace").exists(), "no replace ref is a file");
    assert!(!git_dir.join("packed-refs").exists(), "no replace ref is packed");
    let format = ObjectFormat::Sha1;
    let name = from_hex(older.trim().as_bytes(), format).expect("name");
    let opened: Option<ObjectDatabase> = ObjectDatabase::open(git_dir.as_path(), None, format);
    let answered: Option<Vec<u8>> = opened.as_ref().and_then(|database| { return database.read(name.as_slice()).map(|object| { return object.bytes; }); });
    assert!(
        opened.is_none(),
        "a replace ref kept in reftable/ is outside what this reader scans, and it answered {answered:?} instead of declining"
    );
    remove(root.as_path());
}

/// What:
///  A ref store the repository's configuration moves out of the common directory declines the database,
///  and a configuration that names the files backend leaves it open.
/// Why:
///  Git 2.56 reads `extensions.refStorage` as `<format>://<payload>`
/// (`git-config(1)`,
///  entry `refStorage`),
/// and the tables of such a store stand wherever the value names,
/// so a store moved that way leaves no `reftable` folder behind:
///  a refusal that keyed on the folder opened the database and answered the object Git replaces,
/// which is the wrong parent issue 6340 measured.
/// The files backend is the value that changes nothing,
///  so the second arm shows the configuration is read and not merely present.
#[test]
fn a_ref_store_the_configuration_moves_away_declines_the_database() {
    let root = fixture("repo-read-odb-refstorage");
    let repo = root.join("repo");
    std::fs::create_dir(&repo).expect("dir");
    git(repo.as_path(), &["init", "--quiet", "--ref-format=reftable", "--initial-branch=main"]);
    std::fs::write(repo.join("a.txt"), "one\n").expect("a");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=first"]);
    std::fs::write(repo.join("a.txt"), "two\n").expect("a");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=second"]);
    let older = String::from_utf8(git_output(repo.as_path(), &["rev-parse", "HEAD~1"]).stdout).expect("UTF-8");
    let head = String::from_utf8(git_output(repo.as_path(), &["rev-parse", "HEAD"]).stdout).expect("UTF-8");
    git(repo.as_path(), &["replace", older.trim(), head.trim()]);
    let git_dir = repo.join(".git");
    // The tables move out of the common directory and the configuration names the new place.
    let store = root.join("store");
    std::fs::create_dir(&store).expect("store");
    std::fs::rename(git_dir.join("reftable"), store.join("reftable")).expect("move the tables");
    let named = format!("reftable://{}", store.display());
    git(repo.as_path(), &["config", "--local", "extensions.refstorage", named.as_str()]);
    assert!(!git_dir.join("reftable").exists(), "no reftable folder stands in the common directory");
    // Positive control: Git still resolves the store the configuration names, replacement included.
    let replaced = git_output(repo.as_path(), &["cat-file", "commit", older.trim()]).stdout;
    let unreplaced = git_output(repo.as_path(), &["--no-replace-objects", "cat-file", "commit", older.trim()]).stdout;
    assert_ne!(replaced, unreplaced, "the moved store still holds the replace ref Git reads");
    let format = ObjectFormat::Sha1;
    assert!(
        ObjectDatabase::open(git_dir.as_path(), None, format).is_none(),
        "a ref store the configuration names outside the common directory is outside what this reader scans"
    );
    // Git reads the version as an integer, so `01` is the same repository format and the store is still Git's.
    git(repo.as_path(), &["config", "--local", "core.repositoryformatversion", "01"]);
    assert_eq!(
        git_output(repo.as_path(), &["rev-parse", "--show-ref-format"]).stdout,
        b"reftable\n",
        "the control: Git reads the moved store as the reftable backend under this spelling of the version"
    );
    assert!(
        ObjectDatabase::open(git_dir.as_path(), None, format).is_none(),
        "a version Git spells `01` names the same store, so the reader declines it too"
    );
    // The files backend, named the same way, is the store this reader does scan.
    let files = root.join("files");
    std::fs::create_dir(&files).expect("dir");
    git(files.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(files.join("a.txt"), "one\n").expect("a");
    git(files.as_path(), &["add", "--", "a.txt"]);
    git(files.as_path(), &["commit", "--quiet", "--message=first"]);
    git(files.as_path(), &["config", "--local", "core.repositoryformatversion", "1"]);
    git(files.as_path(), &["config", "--local", "extensions.refstorage", "files"]);
    assert!(
        ObjectDatabase::open(files.join(".git").as_path(), None, format).is_some(),
        "a configuration that names the files backend is the store this reader scans"
    );
    remove(root.as_path());
}

/// What:
///  A files store the configuration moves out of the common directory declines the database,
///  while Git still reads the replacement there.
/// Why:
///  Git 2.56 reads `extensions.refStorage` as `<format>://<payload>`,
/// and a files payload puts the refs and their replace refs in another directory,
///  so a refusal that looked only under the common directory would answer the object Git replaces.
/// The files backend named without a payload is the store this reader scans,
///  which the sibling control above covers.
#[test]
fn a_files_store_the_configuration_moves_away_declines_the_database() {
    let root: PathBuf = fixture("repo-read-odb-files-payload");
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(&repo).expect("dir");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(repo.join("a.txt"), "one\n").expect("a");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=first"]);
    std::fs::write(repo.join("a.txt"), "two\n").expect("a");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=second"]);
    let older: String = git_text(repo.as_path(), &["rev-parse", "HEAD~1"]);
    let head: String = git_text(repo.as_path(), &["rev-parse", "HEAD"]);
    let git_dir: PathBuf = repo.join(".git");
    // Every ref of the store moves into the payload directory the configuration names.
    let store: PathBuf = root.join("store");
    std::fs::create_dir(&store).expect("store");
    std::fs::rename(git_dir.join("refs"), store.join("refs")).expect("move the refs");
    // The files payload is the whole reference directory, HEAD included; the Git directory keeps the stubs its own
    //  initializer writes, so Git still recognizes it as a Git directory.
    std::fs::rename(git_dir.join("HEAD"), store.join("HEAD")).expect("move HEAD");
    std::fs::write(git_dir.join("HEAD"), "ref: refs/heads/.invalid\n").expect("HEAD stub");
    std::fs::create_dir_all(git_dir.join("refs")).expect("refs stub");
    std::fs::write(git_dir.join("refs/heads"), "repository uses alternate refs storage").expect("heads mark");
    let named: String = format!("files://{}", store.display());
    git(repo.as_path(), &["config", "--local", "core.repositoryformatversion", "1"]);
    git(repo.as_path(), &["config", "--local", "extensions.refstorage", named.as_str()]);
    // Positive control: Git resolves the history from the moved store, before a replacement could shorten it.
    let read_back = git_output(repo.as_path(), &["rev-parse", "HEAD~1"]);
    assert!(read_back.status.success(), "Git reads the history from the store the configuration names: {}", String::from_utf8_lossy(read_back.stderr.as_slice()));
    assert_eq!(read_back.stdout, format!("{older}\n").into_bytes(), "Git names the older commit from the moved store");
    // The replacement is written into the same store, and Git reads it there.
    git(repo.as_path(), &["replace", older.as_str(), head.as_str()]);
    let replaced: Vec<u8> = git_output(repo.as_path(), &["cat-file", "commit", older.as_str()]).stdout;
    let unreplaced: Vec<u8> = git_output(repo.as_path(), &["--no-replace-objects", "cat-file", "commit", older.as_str()]).stdout;
    assert_eq!(replaced, git_output(repo.as_path(), &["cat-file", "commit", head.as_str()]).stdout, "Git reads the replacement from the moved store");
    assert_ne!(replaced, unreplaced, "the moved store holds the replace ref Git reads");
    let opened: Option<ObjectDatabase> = ObjectDatabase::open(git_dir.as_path(), None, ObjectFormat::Sha1);
    let answered: Option<Vec<u8>> = opened.as_ref().and_then(|database| { return database.read(from_hex(older.as_bytes(), ObjectFormat::Sha1).expect("name").as_slice()).map(|object| { return object.bytes; }); });
    assert!(
        opened.is_none(),
        "a files store the configuration names outside the common directory is outside what this reader scans, and it answered {answered:?} instead of declining"
    );
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

/// The variable Git reads as the ref storage format of one command,
/// which the reader also reads.
const REFERENCE_BACKEND_ENVIRONMENT: &str = "GIT_REFERENCE_BACKEND";

/// The marker that tells the child of the environment control which body it runs.
const REFERENCE_BACKEND_CHILD: &str = "CLI_GIT_READER_REFERENCE_BACKEND_CHILD";

/// The store the environment names in that control,
/// relative to the common directory as Git resolves a payload.
const NAMED_STORE: &str = "files://refstore";

/// A files repository with two commits,
/// and the commit names of the older and the newer.
fn two_commits_for_the_environment_control(root: &Path) -> (PathBuf, String, String) {
    let repo: PathBuf = root.join("repo");
    std::fs::create_dir(&repo).expect("dir");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::write(repo.join("a.txt"), "one\n").expect("a");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=first"]);
    std::fs::write(repo.join("a.txt"), "two\n").expect("a");
    git(repo.as_path(), &["add", "--", "a.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=second"]);
    let older: String = String::from_utf8(git_output(repo.as_path(), &["rev-parse", "HEAD~1"]).stdout).expect("UTF-8");
    let head: String = String::from_utf8(git_output(repo.as_path(), &["rev-parse", "HEAD"]).stdout).expect("UTF-8");
    return (repo, String::from(older.trim()), String::from(head.trim()));
}

/// The environment a fixture Git runs with,
/// and the reference backend when one is named.
fn environment_of(repo: &Path, backend: Option<&str>) -> Vec<(std::ffi::OsString, std::ffi::OsString)> {
    let mut environment: Vec<(std::ffi::OsString, std::ffi::OsString)> = vec![
        (std::ffi::OsString::from("PATH"), std::ffi::OsString::from("/usr/bin:/bin")),
        (std::ffi::OsString::from("GIT_CONFIG_NOSYSTEM"), std::ffi::OsString::from("1")),
        (std::ffi::OsString::from("GIT_CONFIG_GLOBAL"), std::ffi::OsString::from("/nonexistent-global-config")),
        (std::ffi::OsString::from("HOME"), repo.join("no-home").into_os_string()),
    ];
    if let Some(value) = backend {
        environment.push((std::ffi::OsString::from(REFERENCE_BACKEND_ENVIRONMENT), std::ffi::OsString::from(value)));
    }
    return environment;
}

/// An environment that names another ref store declines the database,
/// because Git reads that store's replace refs and this reader scans only the files under the common directory.
/// Why:
///  `GIT_REFERENCE_BACKEND` overrides `extensions.refStorage` on every command (Git 2.54 and later),
/// and a value other than exactly `files` puts the ref store somewhere this reader does not look.
#[test]
fn an_environment_that_moves_the_ref_store_declines_the_database() {
    if std::env::var_os(REFERENCE_BACKEND_CHILD).is_some() {
        let root: PathBuf = fixture("object-questions-reference-backend");
        let (repo, older, head) = two_commits_for_the_environment_control(root.as_path());
        let git_dir: PathBuf = repo.join(".git");
        // Git resolves a relative payload against the common directory, so the store stands beside `config`.
        let store: PathBuf = git_dir.join("refstore");
        std::fs::create_dir_all(store.join("refs/replace")).expect("store");
        std::fs::write(store.join("refs/replace").join(older.as_str()), format!("{head}\n")).expect("the replacement");
        // Positive control: Git reads the replacement from the store the environment names.
        let environment: Vec<(OsString, OsString)> = environment_of(repo.as_path(), Some(NAMED_STORE));
        let words: [OsString; 3] = [OsString::from("cat-file"), OsString::from("commit"), OsString::from(older.as_str())];
        let answer = real_git_within(Path::new(REAL_GIT), Some(repo.as_path()), environment.as_slice(), &words, None, FIXTURE_GIT_BOUND);
        assert!(answer.output.status.success(), "Git reads the named store: {}", String::from_utf8_lossy(answer.output.stderr.as_slice()));
        assert_eq!(answer.output.stdout, git_output(repo.as_path(), &["cat-file", "commit", head.as_str()]).stdout, "the environment's answer is the replacement");
        assert!(
            ObjectDatabase::open(git_dir.as_path(), None, ObjectFormat::Sha1).is_none(),
            "a ref store the environment names outside the files backend is outside what this reader scans"
        );
        remove(root.as_path());
        return;
    }
    // The variable belongs to one process, so the body runs in a child and the parent asserts its report.
    // `passed` is the whole assertion: the child reports that the reader declined the database, and a child that
    //  failed or ran nothing fails this control with what it said.
    let environment: [(&str, &std::ffi::OsStr); 2] = [(REFERENCE_BACKEND_CHILD, std::ffi::OsStr::new("1")), (REFERENCE_BACKEND_ENVIRONMENT, std::ffi::OsStr::new(NAMED_STORE))];
    crate::child_test_process::passed(crate::child_test_process::ChildTest {
        exact_test: "repo_read::repo_read_odb::tests::an_environment_that_moves_the_ref_store_declines_the_database",
        environment: environment.as_slice(),
        ..crate::child_test_process::ChildTest::default()
    });
}

/// The marker that tells the child of the replacement-base control which body it runs.
const REPLACE_BASE_CHILD: &str = "CLI_GIT_READER_REPLACE_BASE_CHILD";

/// The namespace replacement refs are read from in that control:
/// a path under the common directory,
/// as Git resolves it.
const NAMED_REPLACE_BASE: &str = "refs/probe/";

/// The marker that tells the child of the object-store control which body it runs.
const OBJECT_STORE_CHILD: &str = "CLI_GIT_READER_OBJECT_STORE_CHILD";

/// The object store the environment names in that control,
/// relative to the repository as Git resolves it.
const NAMED_OBJECT_STORE: &str = "relocated-objects";

/// The marker that tells the child of the alternate-store control which body it runs.
const ALTERNATE_STORE_CHILD: &str = "CLI_GIT_READER_ALTERNATE_STORE_CHILD";

/// The environment a fixture Git runs with,
/// and the variables of one control's arm.
fn environment_with(repo: &Path, variables: &[(&str, &str)]) -> Vec<(OsString, OsString)> {
    let mut environment: Vec<(OsString, OsString)> = environment_of(repo, None);
    for (name, value) in variables {
        environment.push((OsString::from(*name), OsString::from(*value)));
    }
    return environment;
}

/// What one real Git run in a fixture answers,
/// with `variables` set for it and the fixture's own hermetic environment.
fn git_with(repo: &Path, variables: &[(&str, &str)], arguments: &[&str]) -> std::process::Output {
    let environment: Vec<(OsString, OsString)> = environment_with(repo, variables);
    let words: Vec<OsString> = arguments.iter().map(|argument| { return OsString::from(*argument); }).collect();
    return real_git_within(Path::new(REAL_GIT), Some(repo), environment.as_slice(), words.as_slice(), None, FIXTURE_GIT_BOUND).output;
}

/// What:
///  The base of the replacement refs the environment names declines the database,
///  because Git reads replacements from there and this reader scans only `refs/replace`.
/// Why:
///  `GIT_REPLACE_REF_BASE` moves every replacement read of the command,
///  and the service's landing callers are not behind a check of the history variables.
#[test]
fn an_environment_that_moves_the_replacement_base_declines_the_database() {
    if std::env::var_os(REPLACE_BASE_CHILD).is_some() {
        let root: PathBuf = fixture("object-questions-replace-base");
        let (repo, older, head) = two_commits_for_the_environment_control(root.as_path());
        let git_dir: PathBuf = repo.join(".git");
        std::fs::create_dir_all(git_dir.join(NAMED_REPLACE_BASE)).expect("the named base");
        std::fs::write(git_dir.join(NAMED_REPLACE_BASE).join(older.as_str()), format!("{head}\n")).expect("the replacement");
        // Positive control: Git reads the replacement from the base the environment names.
        let environment: &[(&str, &str)] = &[("GIT_REPLACE_REF_BASE", NAMED_REPLACE_BASE)];
        let answer = git_with(repo.as_path(), environment, &["cat-file", "commit", older.as_str()]);
        assert!(answer.status.success(), "Git reads the named base: {}", String::from_utf8_lossy(answer.stderr.as_slice()));
        assert_eq!(answer.stdout, git_output(repo.as_path(), &["cat-file", "commit", head.as_str()]).stdout, "the environment's answer is the replacement");
        assert!(
            ObjectDatabase::open(git_dir.as_path(), None, ObjectFormat::Sha1).is_none(),
            "a replacement base the environment names outside refs/replace is outside what this reader scans"
        );
        remove(root.as_path());
        return;
    }
    // `passed` is the whole assertion: the child reports that the reader declined the database, and a child that
    //  failed or ran nothing fails this control with what it said.
    let environment: [(&str, &std::ffi::OsStr); 2] = [(REPLACE_BASE_CHILD, std::ffi::OsStr::new("1")), ("GIT_REPLACE_REF_BASE", std::ffi::OsStr::new(NAMED_REPLACE_BASE))];
    crate::child_test_process::passed(crate::child_test_process::ChildTest {
        exact_test: "repo_read::repo_read_odb::tests::an_environment_that_moves_the_replacement_base_declines_the_database",
        environment: environment.as_slice(),
        ..crate::child_test_process::ChildTest::default()
    });
}

/// What:
///  An object store the environment names declines the database,
///  because Git reads the objects from there and this reader reads the repository's own store.
/// Why:
///  `GIT_OBJECT_DIRECTORY` replaces the objects directory of every read of the command,
///  and a commit Git finds there and this reader does not is a wrong answer by another route than a replace ref.
#[test]
fn an_environment_that_names_another_object_store_declines_the_database() {
    if std::env::var_os(OBJECT_STORE_CHILD).is_some() {
        let root: PathBuf = fixture("object-questions-object-store");
        let (repo, older, head) = two_commits_for_the_environment_control(root.as_path());
        let git_dir: PathBuf = repo.join(".git");
        // The named store holds the newer commit alone, in the loose layout Git reads.
        let store: PathBuf = repo.join(NAMED_OBJECT_STORE);
        std::fs::create_dir_all(store.join(&head[..2])).expect("the named store");
        std::fs::copy(git_dir.join("objects").join(&head[..2]).join(&head[2..]), store.join(&head[..2]).join(&head[2..])).expect("the copy");
        // Positive control: with the variable Git finds the commit in the named store and the older one nowhere.
        let environment: &[(&str, &str)] = &[("GIT_OBJECT_DIRECTORY", NAMED_OBJECT_STORE)];
        assert!(git_with(repo.as_path(), environment, &["cat-file", "-e", head.as_str()]).status.success(), "Git reads the named store");
        assert_eq!(git_with(repo.as_path(), environment, &["cat-file", "-e", older.as_str()]).status.code(), Some(1), "the older commit is not in the named store");
        assert_eq!(git_output(repo.as_path(), &["cat-file", "-e", older.as_str()]).status.code(), Some(0), "the repository's own store holds it");
        assert!(
            ObjectDatabase::open(git_dir.as_path(), None, ObjectFormat::Sha1).is_none(),
            "an object store the environment names is outside what this reader scans"
        );
        remove(root.as_path());
        return;
    }
    // `passed` is the whole assertion: the child reports that the reader declined the database, and a child that
    //  failed or ran nothing fails this control with what it said.
    let environment: [(&str, &std::ffi::OsStr); 2] = [(OBJECT_STORE_CHILD, std::ffi::OsStr::new("1")), ("GIT_OBJECT_DIRECTORY", std::ffi::OsStr::new(NAMED_OBJECT_STORE))];
    crate::child_test_process::passed(crate::child_test_process::ChildTest {
        exact_test: "repo_read::repo_read_odb::tests::an_environment_that_names_another_object_store_declines_the_database",
        environment: environment.as_slice(),
        ..crate::child_test_process::ChildTest::default()
    });
}

/// What:
///  Alternate object directories the environment names decline the database,
///  because Git finds objects there that this reader does not search.
/// Why:
///  `GIT_ALTERNATE_OBJECT_DIRECTORIES` is searched beside the repository's own store by every read of the command,
///  so a commit only the named directory holds is one Git reads and this reader does not.
#[test]
fn an_environment_that_names_alternate_object_directories_declines_the_database() {
    if std::env::var_os(ALTERNATE_STORE_CHILD).is_some() {
        let root: PathBuf = fixture("object-questions-alternate-store");
        let (repo, _older, _head) = two_commits_for_the_environment_control(root.as_path());
        let git_dir: PathBuf = repo.join(".git");
        let tree: String = git_text(repo.as_path(), &["rev-parse", "HEAD^{tree}"]);
        let elsewhere: String = git_text(repo.as_path(), &["commit-tree", tree.as_str(), "-m", "elsewhere"]);
        // The commit is moved out of the repository's store, so only the named directory holds it.
        let store: PathBuf = repo.join(NAMED_OBJECT_STORE);
        std::fs::create_dir_all(store.join(&elsewhere[..2])).expect("the named store");
        std::fs::rename(git_dir.join("objects").join(&elsewhere[..2]).join(&elsewhere[2..]), store.join(&elsewhere[..2]).join(&elsewhere[2..])).expect("the move");
        let environment: &[(&str, &str)] = &[("GIT_ALTERNATE_OBJECT_DIRECTORIES", NAMED_OBJECT_STORE)];
        // Positive control: Git finds the commit only in the directory the environment names.
        assert_eq!(git_output(repo.as_path(), &["cat-file", "-e", elsewhere.as_str()]).status.code(), Some(1), "the repository alone does not hold it");
        assert!(git_with(repo.as_path(), environment, &["cat-file", "-e", elsewhere.as_str()]).status.success(), "Git reads the alternate directory");
        assert!(
            ObjectDatabase::open(git_dir.as_path(), None, ObjectFormat::Sha1).is_none(),
            "alternate object directories the environment names are outside what this reader scans"
        );
        remove(root.as_path());
        return;
    }
    // `passed` is the whole assertion: the child reports that the reader declined the database, and a child that
    //  failed or ran nothing fails this control with what it said.
    let environment: [(&str, &std::ffi::OsStr); 2] = [(ALTERNATE_STORE_CHILD, std::ffi::OsStr::new("1")), ("GIT_ALTERNATE_OBJECT_DIRECTORIES", std::ffi::OsStr::new(NAMED_OBJECT_STORE))];
    crate::child_test_process::passed(crate::child_test_process::ChildTest {
        exact_test: "repo_read::repo_read_odb::tests::an_environment_that_names_alternate_object_directories_declines_the_database",
        environment: environment.as_slice(),
        ..crate::child_test_process::ChildTest::default()
    });
}
