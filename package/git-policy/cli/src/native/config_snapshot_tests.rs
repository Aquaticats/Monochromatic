//! Controls for the configuration snapshot:
//!  the listing parser, each conversion, each lookup, and generated cases checked against real Git as the oracle.

use super::*;
use crate::maintenance_decision::parse_config_records;
use crate::test_support::{fixture, git, git_context, isolated_overlay, remove, repository};
use std::ffi::OsString;
use std::path::PathBuf;

/// Build a listing in `git config --list -z --show-scope` form from `(scope, record)` pairs.
fn listing(records: &[(&str, &str)]) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    for (scope, record) in records {
        bytes.extend_from_slice(scope.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(record.as_bytes());
        bytes.push(0);
    }
    return bytes;
}

/// Parse a listing that must be well formed.
fn snapshot_of(records: &[(&str, &str)]) -> ConfigSnapshot {
    return ConfigSnapshot::parse(listing(records).as_slice(), true).expect("a well-formed listing");
}

/// A key with a value, a valueless key and a value holding a newline keep their shape.
#[test]
fn parse_reads_values_valueless_keys_and_newlines_in_values() {
    let snapshot: ConfigSnapshot = snapshot_of(&[
        ("global", "a.b\nvalue"),
        ("local", "c.d"),
        ("local", "e.f\nline one\nline two"),
        ("command", "g.h\n"),
    ]);
    let entries: &[ConfigEntry] = snapshot.entries();
    assert_eq!(entries.len(), 4);
    assert_eq!(entries[0].scope, "global");
    assert_eq!(entries[0].value.as_deref(), Some(&b"value"[..]));
    assert_eq!(entries[1].value, None);
    assert_eq!(entries[2].value.as_deref(), Some(&b"line one\nline two"[..]));
    assert_eq!(entries[3].value.as_deref(), Some(&b""[..]));
}

/// An empty listing is an empty configuration, and any malformed listing is refused whole.
#[test]
fn parse_refuses_a_listing_that_is_not_scope_record_pairs() {
    assert_eq!(snapshot_of(&[]).entries().len(), 0);
    assert!(ConfigSnapshot::parse(b"local\0a.b\nc", true).is_none(), "no final NUL");
    assert!(ConfigSnapshot::parse(b"local\0a.b\nc\0local\0", true).is_none(), "a scope without its record");
    assert!(ConfigSnapshot::parse(b"local\0\nvalue\0", true).is_none(), "an empty key");
    assert!(ConfigSnapshot::parse(b"local\0\xff.b\nc\0", true).is_none(), "a key that is not UTF-8");
}

/// The last entry of a key wins, and a key nobody set is unset.
#[test]
fn the_last_entry_of_a_key_wins() {
    let snapshot: ConfigSnapshot = snapshot_of(&[
        ("global", "core.hookspath\n/first"),
        ("local", "core.hookspath\n/second"),
        ("command", "core.hookspath\n/third"),
    ]);
    assert_eq!(snapshot.path_value("core.hookspath"), Lookup::Found(String::from("/third")));
    assert_eq!(snapshot.path_value("core.other"), Lookup::Unset);
}

/// One entry the lookup cannot reproduce makes the lookup unhandled even when it is not the last.
#[test]
fn an_unhandled_earlier_entry_makes_the_whole_lookup_unhandled() {
    let snapshot: ConfigSnapshot = snapshot_of(&[
        ("global", "core.hookspath\n~/hooks"),
        ("local", "core.hookspath\n/plain"),
    ]);
    assert_eq!(snapshot.path_value("core.hookspath"), Lookup::Unhandled);
}

/// Every boolean spelling the lookup models converts as Git converts it.
#[test]
fn booleans_convert_as_git_converts_them() {
    for (value, expected) in [
        (Some(&b"true"[..]), Some(true)),
        (Some(&b"TRUE"[..]), Some(true)),
        (Some(&b"Yes"[..]), Some(true)),
        (Some(&b"on"[..]), Some(true)),
        (Some(&b"false"[..]), Some(false)),
        (Some(&b"No"[..]), Some(false)),
        (Some(&b"OFF"[..]), Some(false)),
        (Some(&b""[..]), Some(false)),
        (None, Some(true)),
        (Some(&b"0"[..]), Some(false)),
        (Some(&b"1"[..]), Some(true)),
        (Some(&b"42"[..]), Some(true)),
        (Some(&b"1k"[..]), None),
        (Some(&b"-1"[..]), None),
        (Some(&b"007"[..]), None),
        (Some(&b"0x10"[..]), None),
        (Some(&b" 1"[..]), None),
        (Some(&b"maybe"[..]), None),
        (Some(&b"\xff"[..]), None),
        (Some(&b"1234567890"[..]), None),
    ] {
        assert_eq!(canonical_bool(value), expected, "{value:?}");
    }
}

/// The bool-or-int text is a word, or the decimal number itself.
#[test]
fn bool_or_int_prints_words_and_plain_numbers() {
    for (value, expected) in [
        (Some(&b"yes"[..]), Some("true")),
        (Some(&b"Off"[..]), Some("false")),
        (Some(&b""[..]), Some("false")),
        (None, Some("true")),
        (Some(&b"0"[..]), Some("0")),
        (Some(&b"6700"[..]), Some("6700")),
        (Some(&b"1k"[..]), None),
        (Some(&b"007"[..]), None),
        (Some(&b"x"[..]), None),
    ] {
        assert_eq!(canonical_bool_or_int(value), expected.map(String::from), "{value:?}");
    }
}

/// Paths Git would expand, an empty path and a valueless path are left to Git.
#[test]
fn paths_that_git_expands_or_refuses_are_left_to_git() {
    for (value, expected) in [
        (Some(&b"relative/hooks"[..]), Some("relative/hooks")),
        (Some(&b"/abs/hooks"[..]), Some("/abs/hooks")),
        (Some(&b"a~b"[..]), Some("a~b")),
        (Some(&b"~/hooks"[..]), None),
        (Some(&b"~user/hooks"[..]), None),
        (Some(&b"%(prefix)/hooks"[..]), None),
        (Some(&b":(optional)hooks"[..]), None),
        (Some(&b""[..]), None),
        (None, None),
        (Some(&b"\xff"[..]), None),
    ] {
        assert_eq!(plain_path(value), expected.map(String::from), "{value:?}");
    }
}

/// A named hook is any `hook.<name>.event` or `hook.<name>.command`, whatever the scope or the event.
#[test]
fn a_named_hook_is_any_event_or_command_key() {
    assert!(!snapshot_of(&[("local", "hook.pre-commit.enabled\nfalse"), ("local", "hook.jobs\n2")]).defines_named_hook());
    assert!(snapshot_of(&[("global", "hook.mine.event\npre-push")]).defines_named_hook());
    assert!(snapshot_of(&[("local", "hook.a.b.command\ntrue")]).defines_named_hook());
    assert!(!snapshot_of(&[("local", "core.hookspath\n/x")]).defines_named_hook());
}

/// The maintenance records hold the four keys' last values in Git's canonical text.
#[test]
fn maintenance_records_hold_the_last_value_of_each_key() {
    let snapshot: ConfigSnapshot = snapshot_of(&[
        ("global", "gc.auto\n6700"),
        ("local", "maintenance.auto\nno"),
        ("local", "maintenance.auto\nyes"),
        ("local", "gc.autodetach"),
        ("local", "other.key\nx"),
    ]);
    let records = snapshot.maintenance_records().expect("modelled values");
    assert_eq!(records.get("maintenance.auto").map(String::as_str), Some("true"));
    assert_eq!(records.get("gc.auto").map(String::as_str), Some("6700"));
    assert_eq!(records.get("gc.autodetach").map(String::as_str), Some("true"));
    assert_eq!(records.len(), 3);
    let unhandled: ConfigSnapshot = snapshot_of(&[("local", "gc.auto\n1k")]);
    assert_eq!(unhandled.maintenance_records(), None);
}

/// A listing read with global options never proves the ref storage, and one that names the key never does.
#[test]
fn ref_storage_is_proven_only_without_options_and_without_the_key() {
    assert!(snapshot_of(&[("local", "core.repositoryformatversion\n0")]).proves_files_ref_storage());
    assert!(!snapshot_of(&[("local", "extensions.refstorage\nfiles")]).proves_files_ref_storage());
    let with_options: ConfigSnapshot =
        ConfigSnapshot::parse(listing(&[("local", "core.bare\nfalse")]).as_slice(), false).expect("listing");
    assert!(!with_options.proves_files_ref_storage());
    assert!(!with_options.without_global_options());
}

/// Real Git's answer to `git config <type> --get <key>` in the same context.
fn oracle(context: &GitContext, repo: &Path, kind: &str, key: &str) -> (Option<i32>, String) {
    let arguments: [&str; 4] = ["config", kind, "--get", key];
    let output = run_git(context, &GitRequest::new(repo, &arguments)).expect("git config starts");
    let text: String = String::from_utf8(output.stdout).expect("UTF-8");
    return (output.code, String::from(text.strip_suffix('\n').unwrap_or(text.as_str())));
}

/// Require `lookup` to agree with Git's answer, or decline; return whether it answered.
fn agrees<Value: std::fmt::Debug + PartialEq>(
    lookup: &Lookup<Value>,
    (code, text): &(Option<i32>, String),
    render: impl Fn(&Value) -> String,
    label: &str,
) -> bool {
    match (code, lookup) {
        (Some(1), Lookup::Unset) => return true,
        (Some(0), Lookup::Found(value)) => {
            assert_eq!(&render(value), text, "{label}");
            return true;
        }
        (Some(0), Lookup::Unhandled) => return false,
        (Some(1), other) => panic!("{label}: Git says unset but the snapshot says {other:?}"),
        (Some(0), Lookup::Unset) => panic!("{label}: Git found {text:?} but the snapshot says unset"),
        (_, Lookup::Unhandled) => return false,
        (other_code, answer) => panic!("{label}: Git exited {other_code:?} ({text:?}) but the snapshot answered {answer:?}"),
    }
}

/// A small deterministic generator, so every run draws the same cases.
struct Draw(u64);

impl Draw {
    /// The next pseudo-random number below `bound`.
    fn below(&mut self, bound: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        return (self.0 % (bound as u64)) as usize;
    }
}

/// Values the generated cases draw from: boolean words, integers, bad booleans and paths Git expands.
const VALUES: [&str; 24] = [
    "", "true", "TRUE", "yes", "on", "off", "No", "false", "0", "1", "2", "007", "08", "-1", "1k", "0x10", "x",
    "~/h", "%(prefix)/p", ":(optional)p", "rel/p", "/abs/p", " 1", "a~b",
];

/// Keys the generated cases set, with the `--type` the consumer reads each with.
const KEYS: [(&str, &str); 4] = [
    ("core.hooksPath", "--type=path"),
    ("hook.pre-commit.enabled", "--type=bool"),
    ("gc.auto", "--type=bool-or-int"),
    ("maintenance.auto", "--type=bool-or-int"),
];

/// One generated config line for a random key and value, or a valueless key.
fn generated_line(draw: &mut Draw) -> String {
    let (key, _kind): (&str, &str) = KEYS[draw.below(KEYS.len())];
    let (section, variable): (&str, &str) = key.rsplit_once('.').expect("dotted key");
    let section_header: String = match section.split_once('.') {
        Some((name, subsection)) => format!("[{name} \"{subsection}\"]"),
        None => format!("[{section}]"),
    };
    let value: &str = VALUES[draw.below(VALUES.len())];
    if draw.below(8) == 0 {
        return format!("{section_header}\n\t{variable}\n");
    }
    return format!("{section_header}\n\t{variable} = \"{value}\"\n");
}

/// Several generated lines, 0 to 3 of them.
fn generated_block(draw: &mut Draw) -> String {
    let mut text: String = String::new();
    for _ in 0..draw.below(4) {
        text.push_str(generated_line(draw).as_str());
    }
    return text;
}

/// The counts a differential run reports, so a run that declines everything cannot pass.
#[derive(Default)]
struct Tally {
    /// Lookups that answered and matched Git.
    answered: usize,
    /// Lookups that declined.
    declined: usize,
}

/// Compare every consumer lookup of one configured repository with real Git.
fn compare_case(context: &GitContext, repo: &Path, label: &str, tally: &mut Tally) {
    let snapshot: Option<ConfigSnapshot> = read_config_snapshot(context, repo);
    let Some(listing_snapshot) = snapshot else {
        // A configuration Git itself refuses must make every oracle read fail too.
        let (code, _text) = oracle(context, repo, "--type=bool", "gc.auto");
        assert_ne!(code, Some(0), "{label}: the snapshot failed but Git read the configuration");
        tally.declined += 1;
        return;
    };
    let path_answer: bool = agrees(
        &listing_snapshot.path_value("core.hookspath"),
        &oracle(context, repo, "--type=path", "core.hooksPath"),
        |value| { return value.clone(); },
        label,
    );
    let bool_answer: bool = agrees(
        &listing_snapshot.bool_value("hook.pre-commit.enabled"),
        &oracle(context, repo, "--type=bool", "hook.pre-commit.enabled"),
        |value| { return String::from(if *value { "true" } else { "false" }); },
        label,
    );
    for answered in [path_answer, bool_answer] {
        if answered {
            tally.answered += 1;
        } else {
            tally.declined += 1;
        }
    }
    let arguments: [&str; 5] = [
        "config",
        "--type=bool-or-int",
        "--null",
        "--get-regexp",
        r"^(maintenance\.auto|maintenance\.autodetach|gc\.auto|gc\.autodetach)$",
    ];
    let listing_output = run_git(context, &GitRequest::new(repo, &arguments)).expect("git config starts");
    match listing_snapshot.maintenance_records() {
        None => tally.declined += 1,
        Some(records) => {
            assert!(
                matches!(listing_output.code, Some(0 | 1)),
                "{label}: the snapshot answered but Git exited {:?}",
                listing_output.code
            );
            let expected = parse_config_records(listing_output.stdout.as_slice()).expect("UTF-8 records");
            assert_eq!(records, expected, "{label}");
            tally.answered += 1;
        }
    }
}

/// Generated configurations, over files, includes, conditional includes, `-c` and `GIT_CONFIG_*`, agree with Git.
#[test]
fn generated_configurations_agree_with_git() {
    let root: PathBuf = fixture("config-snapshot-differential");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let git_dir: PathBuf = repo.join(".git");
    let global: PathBuf = root.join("global-config");
    let mut draw: Draw = Draw(0x2545_f491_4f6c_dd1d);
    let mut tally: Tally = Tally::default();
    for case in 0..60 {
        let mut local: String = String::from("[core]\n\trepositoryformatversion = 0\n");
        local.push_str(generated_block(&mut draw).as_str());
        std::fs::write(git_dir.join("included"), generated_block(&mut draw)).expect("include file");
        std::fs::write(git_dir.join("conditional"), generated_block(&mut draw)).expect("conditional file");
        std::fs::write(global.as_path(), generated_block(&mut draw)).expect("global file");
        if draw.below(2) == 0 {
            local.push_str("[include]\n\tpath = included\n");
        }
        local.push_str(generated_block(&mut draw).as_str());
        if draw.below(2) == 0 {
            local.push_str(format!("[includeIf \"gitdir:{}\"]\n\tpath = conditional\n", git_dir.display()).as_str());
        }
        if draw.below(3) == 0 {
            local.push_str("[includeIf \"gitdir:/nowhere/\"]\n\tpath = conditional\n");
        }
        local.push_str(generated_block(&mut draw).as_str());
        std::fs::write(git_dir.join("config"), local).expect("local config");
        let mut overlay: Vec<(OsString, OsString)> = isolated_overlay();
        overlay.retain(|(name, _value)| { return name != "GIT_CONFIG_GLOBAL"; });
        overlay.push((OsString::from("GIT_CONFIG_GLOBAL"), global.clone().into_os_string()));
        if draw.below(3) == 0 {
            let (key, _kind) = KEYS[draw.below(KEYS.len())];
            overlay.push((OsString::from("GIT_CONFIG_COUNT"), OsString::from("1")));
            overlay.push((OsString::from("GIT_CONFIG_KEY_0"), OsString::from(key)));
            overlay.push((OsString::from("GIT_CONFIG_VALUE_0"), OsString::from(VALUES[draw.below(VALUES.len())])));
        }
        let mut prefix: Vec<OsString> = Vec::new();
        if draw.below(3) == 0 {
            let (key, _kind) = KEYS[draw.below(KEYS.len())];
            prefix.push(OsString::from("-c"));
            prefix.push(OsString::from(format!("{key}={}", VALUES[draw.below(VALUES.len())])));
        }
        let context: GitContext = GitContext {
            real_git: PathBuf::from(crate::test_support::REAL_GIT),
            overlay,
            global_prefix: prefix,
        };
        compare_case(&context, repo.as_path(), format!("case {case}").as_str(), &mut tally);
    }
    assert!(tally.answered > 60, "the lookups answered only {} times; they decline too much", tally.answered);
    assert!(tally.declined > 5, "only {} lookups declined; the bad shapes were not drawn", tally.declined);
    remove(root.as_path());
}

/// A repeated key, a valueless key and a bad boolean each give the answer Git gives, or are declined.
#[test]
fn repeated_valueless_and_bad_values_agree_with_git() {
    let root: PathBuf = fixture("config-snapshot-shapes");
    let repo: PathBuf = repository(root.as_path(), "repo");
    let context: GitContext = git_context();
    let mut tally: Tally = Tally::default();
    for (label, text) in [
        ("repeated", "[hook \"pre-commit\"]\n\tenabled = false\n\tenabled = true\n"),
        ("valueless", "[hook \"pre-commit\"]\n\tenabled\n"),
        ("bad bool", "[hook \"pre-commit\"]\n\tenabled = maybe\n"),
        ("bad then good", "[hook \"pre-commit\"]\n\tenabled = maybe\n\tenabled = true\n"),
        ("empty", "[hook \"pre-commit\"]\n\tenabled =\n"),
        ("case", "[Hook \"pre-commit\"]\n\tEnabled = OFF\n"),
        ("old subsection form", "[hook.pre-commit]\n\tenabled = no\n"),
        ("other subsection case", "[hook \"Pre-Commit\"]\n\tenabled = false\n"),
        ("path valueless", "[core]\n\thooksPath\n"),
        ("path empty", "[core]\n\thooksPath =\n"),
        ("path tilde", "[core]\n\thooksPath = ~/hooks\n"),
        ("path repeated", "[core]\n\thooksPath = a\n\thooksPath = b\n"),
    ] {
        std::fs::write(repo.join(".git/config"), format!("[core]\n\trepositoryformatversion = 0\n{text}")).expect("config");
        compare_case(&context, repo.as_path(), label, &mut tally);
    }
    assert!(tally.answered > 0 && tally.declined > 0, "answered {} declined {}", tally.answered, tally.declined);
    remove(root.as_path());
}

/// The files backend is proven for an ordinary repository, and a reftable repository is not.
#[test]
fn ref_storage_proof_agrees_with_rev_parse() {
    let root: PathBuf = fixture("config-snapshot-ref-storage");
    let files: PathBuf = repository(root.as_path(), "files");
    let reftable: PathBuf = root.join("reftable");
    std::fs::create_dir(&reftable).expect("directory");
    git(reftable.as_path(), &["init", "--quiet", "--ref-format=reftable"]);
    let context: GitContext = git_context();
    for (name, repo, expected_format) in [("files", &files, "files"), ("reftable", &reftable, "reftable")] {
        let format = oracle(&context, repo.as_path(), "--type=bool", "core.bare");
        assert_eq!(format.0, Some(0), "{name}: the repository is readable");
        let reported = run_git(&context, &GitRequest::new(repo.as_path(), &["rev-parse", "--show-ref-format"]))
            .expect("rev-parse starts");
        assert_eq!(String::from_utf8_lossy(reported.stdout.as_slice()).trim(), expected_format);
        let snapshot: ConfigSnapshot = read_config_snapshot(&context, repo.as_path()).expect("snapshot");
        assert_eq!(snapshot.proves_files_ref_storage(), expected_format == "files", "{name}");
    }
    remove(root.as_path());
}

/// A listing that Git refuses (a syntax error in the configuration) gives no snapshot.
#[test]
fn a_configuration_git_refuses_gives_no_snapshot() {
    let root: PathBuf = fixture("config-snapshot-refused");
    let repo: PathBuf = repository(root.as_path(), "repo");
    std::fs::write(repo.join(".git/config"), "[core\n").expect("broken config");
    assert!(read_config_snapshot(&git_context(), repo.as_path()).is_none());
    remove(root.as_path());
}
