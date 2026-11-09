//! Controls for reading `core.fsync`:
//! every value is checked against the flushes real Git makes with that value,
//! counted
//!  from Git's own trace2 `fsync` counters,
//! so a reading that drifts from Git's fails here.

use super::*;
use crate::test_support::{fixture, git_context, remove, repository};
use crate::transaction_git::{GitRequest, run_git};
use std::path::{Path, PathBuf};

/// The values checked:
/// every word,
/// groupings,
/// removals that a later word undoes,
/// `none` in and out of the last place,
///  prefixes,
/// and words Git ignores.
const VALUES: [&str; 22] = [
    "", "none", "all", "added", "committed", "objects", "loose-object", "pack", "-pack", "-all", "all,-pack", "pack-metadata",
    "derived-metadata", "loose-object,none", "none,loose-object", "p", "reference,-reference", "bogus", "  pack ,\tnone", "-",
    "pack,-pack-metadata", "commit-graph,pack-metadata,none",
];

/// What:
///  The values of `VALUES` spelled by `spell`,
///  each with the value it was spelled from and the reading that value has,
/// and each resulting spelling asked once.
/// Why:
///   Every value is returned, so the round-trip assertion of the controls below runs for every one of them:
/// a
///  serializer that loses a component gives two values the same spelling,
/// and a value left out here would never reach the
///  assertion that sees it.
/// Asking Git once per spelling stays the caller's business (`ask_once`),
/// because the string is the question
///  and a second start of Git only repeats the answer (one took 0.9 to 2.8 s while the suite ran on 2026-11-09).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const spellingsOf = (spell) => VALUES.reduce((kept, value) => {
///     const spelled = spell(parse(value)).value();
///     return kept.some(([, seen]) => seen === spelled) ? kept : [...kept, [value, spelled, parse(value)]];
/// }, []);
/// ```
fn spellings_of(spell: fn(FsyncComponents) -> FsyncComponents) -> Vec<(&'static str, String, FsyncComponents)> {
    let mut kept: Vec<(&'static str, String, FsyncComponents)> = Vec::new();
    for value in VALUES {
        let set: FsyncComponents = FsyncComponents::parse(value);
        let spelled: String = spell(set).value();
        kept.push((value, spelled, set));
    }
    return kept;
}

/// What:
///  Whether this spelling is the first of its string,
///  so the control asks Git about it once and asserts the round trip for every value.
/// Why:
///   The round trip is a question about the serializer and has to be asked of every value;
/// the flush counts are a question about
///  Git and are the same for one string.
/// Returns false for a spelling already asked.
fn ask_once(asked: &mut Vec<String>, spelled: &str) -> bool {
    if asked.iter().any(|seen: &String| { return seen == spelled; }) {
        return false;
    }
    asked.push(String::from(spelled));
    return true;
}

/// How many flushes Git reports (trace2 `fsync` counters) for one start of `arguments` with `core.fsync` set to `value`,
/// or
///  not set at all for `None`.
fn flushes(repo: &Path, trace: &Path, value: Option<&str>, arguments: &[&str], input: Option<&[u8]>) -> u64 {
    let _gone = std::fs::remove_file(trace);
    let setting: String = value.map(|text| { return format!("core.fsync={text}"); }).unwrap_or_default();
    let mut full: Vec<&str> = Vec::new();
    if value.is_some() {
        full.extend(["-c", setting.as_str()]);
    }
    full.extend(arguments);
    let mut request: GitRequest = GitRequest::new(repo, full.as_slice());
    request.environment.push((std::ffi::OsString::from("GIT_TRACE2_EVENT"), trace.as_os_str().to_os_string()));
    request.input = input.map(<[u8]>::to_vec);
    let output = run_git(&git_context(), &request).expect("Git starts");
    assert!(output.succeeded(), "{full:?}: {}", output.error_text());
    let events: String = std::fs::read_to_string(trace).unwrap_or_default();
    return events
        .lines()
        .filter(|line| { return line.contains("\"event\":\"counter\"") && line.contains("\"category\":\"fsync\""); })
        .filter_map(|line| { return line.split("\"count\":").nth(1)?.trim_end_matches('}').parse::<u64>().ok(); })
        .sum();
}

/// The flushes of `index-pack` storing `pack`:
/// the pack and its index,
/// no reference.
fn pack_flushes(repo: &Path, trace: &Path, value: Option<&str>, pack: &[u8]) -> u64 {
    return flushes(repo, trace, value, &["index-pack", "--stdin", "--strict"], Some(pack));
}

/// A pack of the repository's history,
/// to give `index-pack`.
fn history_pack(repo: &Path) -> Vec<u8> {
    let mut request: GitRequest = GitRequest::new(repo, &["pack-objects", "--stdout", "--revs", "--quiet"]);
    request.input = Some(b"HEAD\n".to_vec());
    return run_git(&git_context(), &request).expect("pack-objects starts").stdout;
}

/// The flushes of the import step of a landing,
/// which also updates a reference:
/// `fast-import` with `fastimport.unpackLimit=0`,
///  which keeps even a small pack,
/// so the pack and its index are written.
/// The message and the file hold `name`,
/// so no earlier
///  probe wrote the same objects (an object that exists is not written again,
/// and a pack of none is not made).
fn import_flushes(repo: &Path, trace: &Path, value: Option<&str>, name: &str) -> u64 {
    let stream: String = format!("commit refs/probe/import-{name}\nmark :1\ncommitter T <t@example.invalid> 1793535889 +0000\ndata {length}\n{name}\nM 100644 inline f\ndata {length}\n{name}\n\ndone\n", length = name.len());
    return flushes(repo, trace, value, &["-c", "fastimport.unpackLimit=0", "fast-import", "--quiet", "--done"], Some(stream.as_bytes()));
}

/// The flushes of one loose object written by `hash-object`.
fn loose_flushes(repo: &Path, trace: &Path, value: Option<&str>, name: &str) -> u64 {
    return flushes(repo, trace, value, &["hash-object", "-w", "--stdin"], Some(format!("loose object {name}").as_bytes()));
}

/// The flushes of one reference written by `update-ref`.
fn reference_flushes(repo: &Path, trace: &Path, value: Option<&str>, name: &str) -> u64 {
    return flushes(repo, trace, value, &["update-ref", format!("refs/probe/{name}").as_str(), "HEAD"], None);
}

/// Each value agrees with Git:
/// a component is named exactly when Git flushes its writes.
/// The loop goes over the values of `VALUES` themselves:
/// a value whose reading drifted to another
///  reading's is exactly the value Git has to be asked about,
/// so no value may be left out for reading like one already asked.
#[test]
fn a_value_names_the_components_git_flushes() {
    let root: PathBuf = fixture("config-fsync-reading");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let trace: PathBuf = root.join("trace.json");
    let pack: Vec<u8> = history_pack(repo.as_path());
    for (position, value) in VALUES.iter().enumerate() {
        let set: FsyncComponents = FsyncComponents::parse(value);
        let name: String = format!("v{position}");
        assert_eq!(loose_flushes(repo.as_path(), trace.as_path(), Some(value), name.as_str()) > 0, set.names_loose_object(), "loose objects under {value:?}");
        assert_eq!(reference_flushes(repo.as_path(), trace.as_path(), Some(value), name.as_str()) > 0, set.names_reference(), "references under {value:?}");
        assert_eq!(pack_flushes(repo.as_path(), trace.as_path(), Some(value), pack.as_slice()) > 0, set.names_pack(), "packs under {value:?}");
        // The import also writes a reference, so its pack is only told apart where the reference is not flushed.
        if !set.names_reference() {
            assert_eq!(import_flushes(repo.as_path(), trace.as_path(), Some(value), name.as_str()) > 0, set.names_pack(), "the import's pack under {value:?}");
        }
    }
    remove(root.as_path());
}

/// The unset key is the default:
/// the pack is flushed and the loose object and the reference are not.
#[test]
fn an_unset_key_is_the_default_set() {
    let root: PathBuf = fixture("config-fsync-default");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let trace: PathBuf = root.join("trace.json");
    let pack: Vec<u8> = history_pack(repo.as_path());
    let set: FsyncComponents = FsyncComponents::default_set();
    assert_eq!(loose_flushes(repo.as_path(), trace.as_path(), None, "unset") > 0, set.names_loose_object());
    assert_eq!(reference_flushes(repo.as_path(), trace.as_path(), None, "unset") > 0, set.names_reference());
    assert_eq!(pack_flushes(repo.as_path(), trace.as_path(), None, pack.as_slice()) > 0, set.names_pack());
    assert!(set.names_pack() && !set.names_loose_object(), "the default flushes packs and not loose objects");
    remove(root.as_path());
}

/// The value that drops the packs keeps every other component and flushes no pack,
/// under every value checked:
/// the pack of
///  `fast-import` makes no flush,
/// the loose object and the reference are flushed exactly as under the person's own value.
#[test]
fn the_value_without_packs_keeps_the_other_components_and_flushes_no_pack() {
    let root: PathBuf = fixture("config-fsync-without-packs");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let trace: PathBuf = root.join("trace.json");
    let pack: Vec<u8> = history_pack(repo.as_path());
    let mut asked: Vec<String> = Vec::new();
    for (value, spelled, set) in spellings_of(FsyncComponents::without_packs) {
        assert_eq!(FsyncComponents::parse(spelled.as_str()).bits, set.without_packs().bits & !OBJECT_MAP, "{value:?} spelled as {spelled:?} reads back the same, but for the mapping between hash algorithms that no word names alone");
        if !ask_once(&mut asked, spelled.as_str()) {
            continue;
        }
        let name: String = format!("w{}", asked.len());
        assert_eq!(pack_flushes(repo.as_path(), trace.as_path(), Some(spelled.as_str()), pack.as_slice()), 0, "no pack flush under {spelled:?} (from {value:?})");
        if !set.names_reference() {
            assert_eq!(import_flushes(repo.as_path(), trace.as_path(), Some(spelled.as_str()), name.as_str()), 0, "the import flushes nothing under {spelled:?} (from {value:?})");
        }
        assert_eq!(
            loose_flushes(repo.as_path(), trace.as_path(), Some(spelled.as_str()), name.as_str()) > 0,
            set.names_loose_object(),
            "loose objects under {spelled:?} (from {value:?})"
        );
        assert_eq!(
            reference_flushes(repo.as_path(), trace.as_path(), Some(spelled.as_str()), name.as_str()) > 0,
            set.names_reference(),
            "references under {spelled:?} (from {value:?})"
        );
    }
    remove(root.as_path());
}

/// What:
///  Every value of `VALUES` reaches the round trip of the two transformed-value controls,
///  whatever its spelling shares with another.
/// Why:
///   A serializer that loses a component gives two different readings the same spelling.
/// A grouping by spelling kept the
///  first of them and dropped the second before its round trip ran,
/// so the serializer's error was never
///  asserted;
/// the dedupe now covers only the questions asked of Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(spellingsOf(withoutPacks)).toHaveLength(VALUES.length);
/// ```
#[test]
fn every_values_spelling_reaches_its_round_trip() {
    for spell in [FsyncComponents::without_packs, FsyncComponents::with_packs] {
        let spellings: Vec<(&'static str, String, FsyncComponents)> = spellings_of(spell);
        assert_eq!(spellings.len(), VALUES.len(), "every value is spelled, so every value's round trip runs");
        for (value, spelled, set) in spellings {
            assert_eq!(
                FsyncComponents::parse(spelled.as_str()).bits,
                spell(set).bits & !OBJECT_MAP,
                "{value:?} spelled as {spelled:?} reads back the same"
            );
        }
    }
}

/// The index is named by `index`,
/// `added` and `all` only.
#[test]
fn only_index_added_and_all_name_the_index() {
    for value in VALUES {
        let named: bool = ["index", "added", "all"].iter().any(|word| { return value.contains(word); });
        let set: FsyncComponents = FsyncComponents::parse(value);
        if !named {
            assert!(!set.names_index(), "{value:?}");
        }
    }
    assert!(FsyncComponents::parse("added,-index").names_index(), "an added component stays when it is also removed, as in Git");
    assert!(!FsyncComponents::parse("-index").names_index());
}

/// The value that adds the packs keeps every other component and flushes the pack and its index as well:
/// `index-pack` makes
/// as many flushes under it as under a value that names the packs alone,
/// and the loose object and the reference are flushed
/// exactly as under the person's own value.
#[test]
fn the_value_with_packs_flushes_the_pack_and_its_index_and_keeps_the_other_components() {
    let root: PathBuf = fixture("config-fsync-with-packs");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let trace: PathBuf = root.join("trace.json");
    let pack: Vec<u8> = history_pack(repo.as_path());
    let packed_by_name: u64 = pack_flushes(repo.as_path(), trace.as_path(), Some("pack,pack-metadata,none"), pack.as_slice());
    let mut asked: Vec<String> = Vec::new();
    for (value, spelled, set) in spellings_of(FsyncComponents::with_packs) {
        assert_eq!(FsyncComponents::parse(spelled.as_str()).bits, set.with_packs().bits & !OBJECT_MAP, "{value:?} spelled as {spelled:?}");
        if !ask_once(&mut asked, spelled.as_str()) {
            continue;
        }
        let name: String = format!("p{}", asked.len());
        assert!(set.with_packs().names_pack_and_metadata());
        // The same flushes as a value that names the packs alone make (the pack, its index and what Git writes beside it).
        assert_eq!(
            pack_flushes(repo.as_path(), trace.as_path(), Some(spelled.as_str()), pack.as_slice()),
            packed_by_name,
            "the pack and its index under {spelled:?} (from {value:?})"
        );
        assert_eq!(
            loose_flushes(repo.as_path(), trace.as_path(), Some(spelled.as_str()), name.as_str()) > 0,
            set.names_loose_object(),
            "loose objects under {spelled:?} (from {value:?})"
        );
        assert_eq!(
            reference_flushes(repo.as_path(), trace.as_path(), Some(spelled.as_str()), name.as_str()) > 0,
            set.names_reference(),
            "references under {spelled:?} (from {value:?})"
        );
    }
    remove(root.as_path());
}
