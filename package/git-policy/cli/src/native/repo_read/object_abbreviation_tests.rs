//! Controls for the abbreviated object name:
//! the setting,
//! the automatic length,
//! and agreement with real Git for loose objects,
//!  packs,
//! a multi-pack-index and alternates,
//! in a repository where four characters are often ambiguous.

use super::*;
use crate::test_support::{fixture, git, git_output, remove};

/// How many files the fixture commits:
/// enough that some four-character prefixes are shared by several names.
const FILES: usize = 1500;

/// The shortest length the fixture asks for,
/// so that many names must be lengthened.
const SHORT: usize = 4;

/// `core.abbrev` unset,
/// `auto` in any case,
/// and unusable values give the automatic length;
/// a number from four up gives
///  that length;
/// a false word or an empty value gives the whole name.
#[test]
fn the_setting_follows_what_git_reads_from_core_abbrev() {
    assert_eq!(setting_of(None), Setting::Auto);
    for auto in ["auto", "AUTO", " Auto "] {
        assert_eq!(setting_of(Some(auto.as_bytes())), Setting::Auto, "{auto}");
    }
    assert_eq!(setting_of(Some(b"12")), Setting::Length(12));
    assert_eq!(setting_of(Some(MINIMUM_ABBREVIATION.to_string().as_bytes())), Setting::Length(usize::try_from(MINIMUM_ABBREVIATION).expect("small")));
    for whole in ["", "no", "false", "off", "OFF"] {
        assert_eq!(setting_of(Some(whole.as_bytes())), Setting::Length(LONGEST_NAME), "{whole:?}");
    }
    for unusable in ["3", "-1", "twelve", "1.5"] {
        assert_eq!(setting_of(Some(unusable.as_bytes())), Setting::Auto, "{unusable}");
    }
}

/// The automatic length is half the bit length of the object count rounded up,
/// never below the fallback:
/// a repository of
///  about 380000 packed objects (as this one) gets ten characters.
#[test]
fn the_automatic_length_grows_with_the_object_count() {
    assert_eq!(automatic_length(0), FALLBACK_ABBREVIATION);
    assert_eq!(automatic_length(1), FALLBACK_ABBREVIATION);
    // 2^18 <= 379598 < 2^19: the bit length is nineteen, half of it rounded up is ten.
    assert_eq!(automatic_length(379_598), 10);
    // At the last count before the next bit the length is still the same; the next bit moves it by half a character.
    let bit_boundary: u64 = 1 << 19;
    assert_eq!(automatic_length(bit_boundary - 1), 10);
    assert_eq!(automatic_length(bit_boundary), 10);
    assert_eq!(automatic_length(bit_boundary << 1), 11);
    let mut previous: usize = 0;
    for bits in 0..40 {
        let length: usize = automatic_length(1_u64 << bits);
        assert!(length >= previous, "the length never shrinks as the count grows");
        previous = length;
    }
}

/// A repository whose last commit holds `FILES` blobs,
/// so that four characters are ambiguous for several of them.
fn crowded(root: &Path, name: &str) -> PathBuf {
    let repo: PathBuf = root.join(name);
    std::fs::create_dir(&repo).expect("dir");
    git(repo.as_path(), &["init", "--quiet", "--initial-branch=main"]);
    std::fs::create_dir(repo.join("many")).expect("many");
    for number in 0..FILES {
        std::fs::write(repo.join(format!("many/file-{number}.txt")), format!("content of file {number}\n")).expect("file");
    }
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=many"]);
    return repo;
}

/// The names of the blobs of `HEAD`,
/// whole,
/// then as Git abbreviates them with `argument`.
fn names_of(repo: &Path, argument: Option<&str>) -> Vec<String> {
    let mut arguments: Vec<&str> = vec!["ls-tree", "-r"];
    arguments.extend(argument);
    arguments.push("HEAD");
    let listing = git_output(repo, arguments.as_slice());
    assert!(listing.status.success(), "{}", String::from_utf8_lossy(&listing.stderr));
    return String::from_utf8(listing.stdout)
        .expect("UTF-8")
        .lines()
        .map(|line| { return line.split_once('\t').expect("a tab").0.split(' ').nth(2).expect("a name").to_string(); })
        .collect();
}

/// Requires the in-process answer to equal Git's for every blob,
/// with the asked minimum and with the automatic length;
///  returns how many names were lengthened beyond the minimum,
/// which shows the uniqueness search had work to do.
fn agrees_with_git(repo: &Path, objects: &Path, label: &str) -> usize {
    let whole: Vec<String> = names_of(repo, None);
    let asked: Vec<String> = names_of(repo, Some(format!("--abbrev={SHORT}").as_str()));
    let automatic: Vec<String> = names_of(repo, Some("--abbrev"));
    assert!(whole.len() >= FILES, "{label}");
    let mut lengthened: usize = 0;
    for (index, name) in whole.iter().enumerate() {
        let short: String = abbreviate(objects, Setting::Length(SHORT), name).unwrap_or_else(|| panic!("{label}: {name} was not abbreviated"));
        assert_eq!(short, asked[index], "{label}: asked length for {name}");
        if short.len() > SHORT {
            lengthened += 1;
        }
        let by_count: String = abbreviate(objects, Setting::Auto, name).unwrap_or_else(|| panic!("{label}: {name} was not abbreviated"));
        assert_eq!(by_count, automatic[index], "{label}: automatic length for {name}");
    }
    return lengthened;
}

/// Loose objects:
/// a name is lengthened exactly when another loose object shares its start.
#[test]
fn loose_objects_agree_with_git() {
    let root: PathBuf = fixture("abbrev-loose");
    let repo: PathBuf = crowded(root.as_path(), "repo");
    let lengthened: usize = agrees_with_git(repo.as_path(), repo.join(".git/objects").as_path(), "loose");
    assert!(lengthened > 0, "the fixture must make four characters ambiguous, or the control proves nothing");
    remove(root.as_path());
}

/// Packed objects:
/// the neighbours in the index decide,
/// as they do for Git.
#[test]
fn packed_objects_agree_with_git() {
    let root: PathBuf = fixture("abbrev-packed");
    let repo: PathBuf = crowded(root.as_path(), "repo");
    git(repo.as_path(), &["repack", "--quiet", "-a", "-d"]);
    let lengthened: usize = agrees_with_git(repo.as_path(), repo.join(".git/objects").as_path(), "packed");
    assert!(lengthened > 0);
    remove(root.as_path());
}

/// Requires every name `open_index` reads from the one pack index of `repo` to be the name Git lists at that position;
/// the
///  listing of every object is in name order,
/// which is the order of a pack index's names.
fn index_names_equal_git(repo: &Path, version: u8) {
    let pack: PathBuf = repo.join(".git/objects/pack");
    let indexes: Vec<PathBuf> = std::fs::read_dir(&pack)
        .expect("pack directory")
        .map(|entry| { return entry.expect("entry").path(); })
        .filter(|path| { return path.extension().is_some_and(|extension| { return extension == "idx"; }); })
        .collect();
    assert_eq!(indexes.len(), 1, "one pack after the repack");
    let magic: Vec<u8> = std::fs::read(&indexes[0]).expect("index").into_iter().take(8).collect();
    assert_eq!(magic.as_slice() == b"\xfftOc\0\0\0\x02", version == 2, "the control needs a version {version} index");
    let listing = git_output(repo, &["cat-file", "--batch-all-objects", "--batch-check=%(objectname)"]);
    assert!(listing.status.success(), "{}", String::from_utf8_lossy(&listing.stderr));
    let listed: Vec<String> = String::from_utf8(listing.stdout).expect("UTF-8").lines().map(str::to_string).collect();
    let names: SortedNames = open_index(&indexes[0], 20).expect("a plain index");
    assert_eq!(names.count, listed.len() as u64, "version {version}: the count");
    for (position, expected) in listed.iter().enumerate() {
        let read: Vec<u8> = names.name(position as u64).expect("a name");
        let hex: String = read.iter().map(|byte| { return format!("{byte:02x}"); }).collect();
        assert_eq!(&hex, expected, "version {version}: name {position}");
    }
}

/// Every name read from a version 2 pack index is the one Git lists at that position,
/// from the first to the last;
/// agreement
///  on the abbreviations alone passes with a shifted name table,
/// because only the neighbours of the names that collide decide.
#[test]
fn every_name_of_a_version_2_pack_index_is_read_where_git_has_it() {
    let root: PathBuf = fixture("abbrev-index-v2");
    let repo: PathBuf = crowded(root.as_path(), "repo");
    git(repo.as_path(), &["-c", "pack.indexVersion=2", "repack", "--quiet", "-a", "-d"]);
    index_names_equal_git(repo.as_path(), 2);
    remove(root.as_path());
}

/// Every name read from a version 1 pack index (no header;
/// each entry is an offset,
/// then the name) is the one Git lists at
///  that position.
#[test]
fn every_name_of_a_version_1_pack_index_is_read_where_git_has_it() {
    let root: PathBuf = fixture("abbrev-index-v1");
    let repo: PathBuf = crowded(root.as_path(), "repo");
    git(repo.as_path(), &["-c", "pack.indexVersion=1", "repack", "--quiet", "-a", "-d"]);
    index_names_equal_git(repo.as_path(), 1);
    remove(root.as_path());
}

/// Two packs under a multi-pack-index,
/// plus loose objects made afterwards.
#[test]
fn a_multi_pack_index_and_loose_objects_agree_with_git() {
    let root: PathBuf = fixture("abbrev-midx");
    let repo: PathBuf = crowded(root.as_path(), "repo");
    git(repo.as_path(), &["repack", "--quiet", "-a", "-d"]);
    std::fs::write(repo.join("second.txt"), "second pack\n").expect("second");
    git(repo.as_path(), &["add", "second.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=second"]);
    git(repo.as_path(), &["repack", "--quiet", "-d"]);
    git(repo.as_path(), &["multi-pack-index", "write"]);
    assert!(repo.join(".git/objects/pack/multi-pack-index").exists(), "the control needs the index");
    // Objects made after the index was written are loose and are searched with it.
    std::fs::write(repo.join("third.txt"), "loose after the index\n").expect("third");
    git(repo.as_path(), &["add", "third.txt"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=third"]);
    let lengthened: usize = agrees_with_git(repo.as_path(), repo.join(".git/objects").as_path(), "midx");
    assert!(lengthened > 0);
    remove(root.as_path());
}

/// Objects that live in an alternate count and are searched too.
#[test]
fn objects_of_an_alternate_agree_with_git() {
    let root: PathBuf = fixture("abbrev-alternate");
    let origin: PathBuf = crowded(root.as_path(), "origin");
    git(origin.as_path(), &["repack", "--quiet", "-a", "-d"]);
    let clone: PathBuf = root.join("clone");
    git(root.as_path(), &["clone", "--quiet", "--shared", origin.to_str().expect("UTF-8"), clone.to_str().expect("UTF-8")]);
    assert!(clone.join(".git/objects/info/alternates").exists(), "the control needs the alternate");
    let lengthened: usize = agrees_with_git(clone.as_path(), clone.join(".git/objects").as_path(), "alternate");
    assert!(lengthened > 0);
    remove(root.as_path());
}

/// An incremental multi-pack-index chain is not modelled,
/// so nothing is answered and the caller asks Git.
#[test]
fn an_incremental_multi_pack_index_chain_is_declined() {
    let root: PathBuf = fixture("abbrev-chain");
    let repo: PathBuf = crowded(root.as_path(), "repo");
    git(repo.as_path(), &["repack", "--quiet", "-a", "-d"]);
    std::fs::create_dir(repo.join(".git/objects/pack/multi-pack-index.d")).expect("chain directory");
    let name: String = names_of(repo.as_path(), None).remove(0);
    assert_eq!(abbreviate(repo.join(".git/objects").as_path(), Setting::Auto, name.as_str()), None);
    remove(root.as_path());
}

/// A name that is not lower-case hexadecimal,
/// or an object directory that does not exist,
/// is not answered.
#[test]
fn names_and_directories_that_cannot_be_read_are_not_answered() {
    let root: PathBuf = fixture("abbrev-unreadable");
    let repo: PathBuf = crowded(root.as_path(), "repo");
    let objects: PathBuf = repo.join(".git/objects");
    assert_eq!(abbreviate(objects.as_path(), Setting::Auto, "not a name"), None);
    assert_eq!(abbreviate(objects.as_path(), Setting::Auto, "ABCDEF0123456789ABCDEF0123456789ABCDEF01"), None);
    assert_eq!(abbreviate(root.join("missing").as_path(), Setting::Auto, "0123456789abcdef0123456789abcdef01234567"), None);
    remove(root.as_path());
}

/// A setting that names the whole name or more returns the whole name without reading any object.
#[test]
fn a_length_of_the_whole_name_returns_it() {
    let name: &str = "0123456789abcdef0123456789abcdef01234567";
    let nowhere: PathBuf = PathBuf::from("/nonexistent-objects");
    assert_eq!(abbreviate(nowhere.as_path(), Setting::Length(LONGEST_NAME), name), None, "an unreadable directory is declined before the setting is looked at");
    let root: PathBuf = fixture("abbrev-whole");
    let repo: PathBuf = crowded(root.as_path(), "repo");
    assert_eq!(abbreviate(repo.join(".git/objects").as_path(), Setting::Length(LONGEST_NAME), name).as_deref(), Some(name));
    remove(root.as_path());
}
