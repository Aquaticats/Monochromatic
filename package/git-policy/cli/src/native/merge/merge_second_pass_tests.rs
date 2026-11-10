//! Controls for the second pass over real merge-tree answers:
//! the stages parse,
//! a plain text file whose regions all resolve
//!  is merged,
//! and every kind of path the pass must leave alone (both-append,
//! binary,
//! added on both sides,
//! deleted on one,
//!  a link,
//! large,
//! claimed by a merge driver) is left,
//! with the reason.

use super::*;
use crate::transaction_git::GitContext;
use crate::word_merge::LineSpan;
use std::ffi::OsString;
use crate::test_support::{fixture, git, git_context, git_output, remove, repository};
use std::path::PathBuf;

/// A stored corpus file of the word-level merge.
pub(crate) fn corpus(name: &str, extension: &str) -> Vec<u8> {
    return std::fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/native/merge/word_merge_corpus").join(format!("{name}.{extension}")))
        .expect("a corpus file");
}

/// A repository with a base commit and the branches `ours` and `theirs` on top of it,
/// for the files given.
pub(crate) struct World {
    /// The fixture directory.
    pub(crate) root: PathBuf,
    /// The repository.
    pub(crate) repo: PathBuf,
}

/// What a side does to a path.
pub(crate) enum Change<'a> {
    /// The path holds these bytes.
    Write(&'a [u8]),
    /// The path is removed.
    Remove,
    /// The path becomes a symbolic link to this target.
    Link(&'a str),
    /// The path holds these bytes and is executable.
    Executable(&'a [u8]),
}

/// Build the world:
/// `base` files first,
/// then the side changes on the two branches.
pub(crate) fn world(name: &str, base: &[(&str, &[u8])], ours: &[(&str, Change<'_>)], theirs: &[(&str, Change<'_>)]) -> World {
    let root: PathBuf = fixture(name);
    let repo: PathBuf = repository(root.as_path(), "repo");
    for (path, bytes) in base {
        if let Some(parent) = repo.join(path).parent() {
            std::fs::create_dir_all(parent).expect("directories");
        }
        std::fs::write(repo.join(path), bytes).expect("base file");
    }
    git(repo.as_path(), &["add", "--all"]);
    git(repo.as_path(), &["commit", "--quiet", "--message=base"]);
    for (branch, changes) in [("theirs", theirs), ("ours", ours)] {
        git(repo.as_path(), &["switch", "--quiet", "--create", branch, "main"]);
        for (path, change) in changes {
            let full: PathBuf = repo.join(path);
            if let Some(parent) = full.parent() {
                std::fs::create_dir_all(parent).expect("directories");
            }
            match change {
                Change::Write(bytes) => std::fs::write(&full, bytes).expect("a side file"),
                Change::Remove => std::fs::remove_file(&full).expect("remove"),
                Change::Executable(bytes) => {
                    use std::os::unix::fs::PermissionsExt;
                    std::fs::write(&full, bytes).expect("a side file");
                    std::fs::set_permissions(&full, std::fs::Permissions::from_mode(0o755)).expect("mode");
                }
                Change::Link(target) => {
                    std::fs::remove_file(&full).ok();
                    std::os::unix::fs::symlink(target, &full).expect("link");
                }
            }
        }
        git(repo.as_path(), &["add", "--all"]);
        git(repo.as_path(), &["commit", "--quiet", format!("--message={branch}").as_str()]);
    }
    return World { root, repo };
}

/// The runner over the fixture repository.
pub(crate) fn runner<'a>(context: &'a GitContext, repo: &'a Path) -> ContextRunner<'a> {
    return ContextRunner { context, cwd: repo, without_prefix: false, repository: Vec::new(), object_directory: None, environment: Vec::new() };
}

/// The tree and the conflicts of `git merge-tree` over the two branches.
pub(crate) fn merge_tree_of(world: &World) -> (String, Vec<Conflict>) {
    let output = git_output(world.repo.as_path(), &["merge-tree", "--write-tree", "-z", "--no-messages", "ours", "theirs"]);
    assert!(output.status.code() == Some(0) || output.status.code() == Some(1), "{}", String::from_utf8_lossy(&output.stderr));
    return conflicts_of_merge_tree(output.stdout.as_slice());
}

/// The awaited faces run as a task of the runtime reaches them.
/// Why:
///   Every start of the pass is awaited,
/// so a task of the runtime is the caller these controls must prove the folder serves.
pub(crate) fn await_it<Output>(work: impl std::future::Future<Output = Output>) -> Output {
    return crate::lazy_runtime::run_on_runtime(work).expect("the runtime runs the work");
}

/// The verdicts of the second pass over the world's merge.
fn verdicts_of(world: &World) -> Vec<Verdict> {
    let (_, conflicts) = merge_tree_of(world);
    let context: GitContext = git_context();
    return await_it(async { return second_pass(&runner(&context, world.repo.as_path()), conflicts.as_slice(), AttributeSource::Invocation).await; }).expect("the pass runs");
}

/// The merge-tree stages parse into one conflict per path with all three sides,
/// in the order Git printed them.
#[test]
fn the_stages_of_a_merge_tree_answer_parse_per_path() {
    let adjacent: Vec<u8> = corpus("md1b", "base");
    let world: World = world(
        "second-pass-parse",
        &[("b.md", adjacent.as_slice()), ("a.md", adjacent.as_slice())],
        &[("b.md", Change::Write(corpus("md1b", "ours").as_slice())), ("a.md", Change::Write(corpus("md1b", "ours").as_slice()))],
        &[("b.md", Change::Write(corpus("md1b", "theirs").as_slice())), ("a.md", Change::Write(corpus("md1b", "theirs").as_slice()))],
    );
    let (tree, conflicts) = merge_tree_of(&world);
    assert!(tree.len() >= 40 && tree.chars().all(|character| { return character.is_ascii_hexdigit(); }), "{tree}");
    let paths: Vec<&[u8]> = conflicts.iter().map(|conflict| { return conflict.path.as_slice(); }).collect();
    assert_eq!(paths, vec![b"a.md".as_slice(), b"b.md".as_slice()]);
    for conflict in &conflicts {
        assert!(conflict.base.is_some() && conflict.ours.is_some() && conflict.theirs.is_some(), "{conflict:?}");
        assert_eq!(conflict.ours.as_ref().map(|stage| { return stage.mode; }), Some(0o100_644));
    }
    remove(world.root.as_path());
}

/// An edit and an insertion right after it merge to Jujutsu's bytes,
/// with the part reported.
#[test]
fn an_adjacent_edit_and_insertion_are_merged_with_their_part() {
    let world: World = world(
        "second-pass-merged",
        &[("a.md", corpus("md1b", "base").as_slice())],
        &[("a.md", Change::Write(corpus("md1b", "ours").as_slice()))],
        &[("a.md", Change::Write(corpus("md1b", "theirs").as_slice()))],
    );
    let verdicts: Vec<Verdict> = verdicts_of(&world);
    let [Verdict::Merged(file)] = verdicts.as_slice() else { panic!("expected one merged file: {verdicts:?}") };
    assert!(file.bytes == corpus("md1b", "word.out"));
    assert_eq!((file.path.as_slice(), file.mode), (b"a.md".as_slice(), 0o100_644));
    assert!(!file.parts.is_empty());
    remove(world.root.as_path());
}

/// Each kind of path the pass must not touch is left,
/// with the reason that belongs to it.
#[test]
fn paths_the_pass_must_not_touch_are_left_with_their_reason() {
    let text: &[u8] = b"one\ntwo\nthree\n";
    let large_line: String = "same line of a large file\n".repeat(LARGEST_BLOB_BYTES / 20);
    let large_edit = |first: &str| -> Vec<u8> { return format!("{first}\n{large_line}").into_bytes(); };
    let world: World = world(
        "second-pass-left",
        &[
            ("append.md", corpus("md2", "base").as_slice()),
            ("binary.bin", b"a\0one\n"),
            ("both.md", b"x\n"),
            ("gone.md", text),
            ("link.md", text),
            ("mode.md", text),
            ("large.md", large_edit("base").as_slice()),
            ("driven.md", corpus("md1b", "base").as_slice()),
            (".gitattributes", b"driven.md merge=binary\n"),
        ],
        &[
            ("append.md", Change::Write(corpus("md2", "ours").as_slice())),
            ("binary.bin", Change::Write(b"a\0two\n")),
            ("both.md", Change::Write(b"x\nours\n")),
            ("gone.md", Change::Remove),
            ("link.md", Change::Link("target")),
            ("mode.md", Change::Executable(b"one\nTWO\nthree\n")),
            ("large.md", Change::Write(large_edit("ours").as_slice())),
            ("driven.md", Change::Write(corpus("md1b", "ours").as_slice())),
        ],
        &[
            ("append.md", Change::Write(corpus("md2", "theirs").as_slice())),
            ("binary.bin", Change::Write(b"a\0three\n")),
            ("both.md", Change::Write(b"x\ntheirs\n")),
            ("gone.md", Change::Write(b"one\nTWO\nthree\n")),
            ("link.md", Change::Write(b"one\nTWO\nthree\n")),
            ("mode.md", Change::Write(b"one\ntwo\nTHREE\n")),
            ("large.md", Change::Write(large_edit("theirs").as_slice())),
            ("driven.md", Change::Write(corpus("md1b", "theirs").as_slice())),
        ],
    );
    let verdicts: Vec<Verdict> = verdicts_of(&world);
    let reasons: BTreeMap<String, Left> = verdicts
        .iter()
        .filter_map(|verdict| {
            return match verdict {
                Verdict::Left { path, why } => Some((path_message_text(path.as_slice()), why.clone())),
                Verdict::Merged(_) => None,
            };
        })
        .collect();
    assert!(matches!(reasons.get("append.md"), Some(Left::Overlap(count)) if *count > 0), "{reasons:?}");
    assert_eq!(reasons.get("binary.bin"), Some(&Left::Binary), "{reasons:?}");
    assert_eq!(reasons.get("gone.md"), Some(&Left::MissingStage), "{reasons:?}");
    // A link against an edit is a file/link conflict: Git keeps the sides under separate names, so no path has all three stages.
    assert_eq!(reasons.get("link.md"), Some(&Left::MissingStage), "{reasons:?}");
    // The same file with a different mode on the two sides has all three stages, and their modes differ.
    assert_eq!(reasons.get("mode.md"), Some(&Left::NotAPlainFile), "{reasons:?}");
    assert_eq!(reasons.get("large.md"), Some(&Left::TooLarge), "{reasons:?}");
    assert_eq!(reasons.get("driven.md"), Some(&Left::Driver(String::from("binary"))), "{reasons:?}");
    // Both sides inserting different text at one point is an overlap, not a choice of order.
    assert!(matches!(reasons.get("both.md"), Some(Left::Overlap(_))), "{reasons:?}");
    remove(world.root.as_path());
}

/// A path both sides added has no common ancestor stage and is left.
#[test]
fn a_path_both_sides_added_is_left_for_the_person() {
    let world: World = world("second-pass-add-add", &[("keep.md", b"keep\n")], &[("new.md", Change::Write(b"ours\n"))], &[("new.md", Change::Write(b"theirs\n"))]);
    let verdicts: Vec<Verdict> = verdicts_of(&world);
    assert_eq!(verdicts, vec![Verdict::Left { path: b"new.md".to_vec(), why: Left::MissingStage }]);
    remove(world.root.as_path());
}

/// The report names the file,
/// numbers the parts,
/// shows what each became,
/// names the three inputs as blobs,
/// and keeps a control character out of the terminal.
#[test]
fn the_report_lines_name_the_file_and_number_the_parts_and_escape_the_text() {
    let span = LineSpan { first: 3, count: 2 };
    let part = |merged: &[u8]| { return MergedPart { base: LineSpan { first: 3, count: 0 }, ours: span, theirs: LineSpan { first: 4, count: 1 }, merged: merged.to_vec() }; };
    let stage = |oid: &str| { return Some(Stage { mode: 0o100_644, oid: String::from(oid) }); };
    let stages = Conflict { path: b"dir/a.md".to_vec(), base: stage("aa12"), ours: stage("bb34"), theirs: stage("cc56") };
    let merged = Merged { path: b"dir/a.md".to_vec(), mode: 0o100_644, bytes: Vec::new(), parts: vec![part(b"plain\n"), part(b"\x1b[31mred")], stages };
    let lines: Vec<String> = report_lines("monochromatic-git-policy-cli: ", &merged);
    assert_eq!(lines.len(), 2);
    for (at, line) in lines.iter().enumerate() {
        assert!(line.starts_with(format!("monochromatic-git-policy-cli: {REPORT_MARK}: dir/a.md: part {} of 2: ", at + 1).as_str()), "{line}");
        assert!(line.contains("lines 3 to 4") && line.contains("line 4") && line.contains("no line (before line 3)"), "{line}");
        assert!(line.ends_with("from base blob aa12, ours blob bb34, theirs blob cc56"), "the three inputs are named by object: {line}");
        assert!(!line.contains('\x1b') && !line.contains('\n'), "{line:?}");
    }
    assert!(lines[0].contains("\"plain\\n\""), "{}", lines[0]);
    let long = Merged { parts: vec![part("w".repeat(EXCERPT_CHARACTERS * 2).as_bytes())], ..merged.clone() };
    assert!(report_lines("", &long)[0].contains("...\""), "a long part is cut");
    // A merge the pass recorded with no stages of its own names no blob rather than a wrong one.
    let unrecorded = Merged { stages: Conflict { path: b"dir/a.md".to_vec(), base: None, ours: None, theirs: None }, ..merged.clone() };
    assert!(report_lines("", &unrecorded)[0].ends_with("from base no blob, ours no blob, theirs no blob"), "{}", report_lines("", &unrecorded)[0]);
    let none = Merged { parts: Vec::new(), ..merged };
    assert!(report_lines("", &none)[0].contains("resolved without a word pass"));
}

/// An `update-index --index-info` record carries the mode in octal,
/// the object,
/// a tab,
/// the raw path bytes and a NUL.
#[test]
fn a_resolved_record_is_the_index_info_form() {
    assert_eq!(resolved_record(0o100_755, "abc", b"d/\xff.sh"), b"100755 abc\td/\xff.sh\0".to_vec());
}

/// What:
///  Only the configuration options of the global options remain,
/// in order,
/// so the second pass keeps the person's settings and no option that would move it away from the top of the
///  worktree.
/// Why:
///   The attribute tree is a configuration value whoever sets it:
/// a `-c`,
/// a `--config-env` in either of its
///  forms,
/// or a `--attr-source` in either of its.
/// Dropping them made `git check-attr` read another source than the one
///  the person chose,
/// so a file the person's rules exclude was word-merged and committed
/// (cross-vendor review round 3,
/// finding 3:
/// the pass named no tree over the person's,
/// and then read no tree at all).
#[test]
fn only_the_configuration_options_of_the_prefix_are_kept() {
    let prefix: Vec<OsString> = [
        "-C",
        "sub",
        "-c",
        "core.autocrlf=true",
        "--git-dir=x",
        "-c",
        "a.b=c",
        "--no-pager",
        "--config-env=attr.tree=RULES",
        "--config-env",
        "attr.tree=RULES2",
        "--attr-source=rules",
        "--attr-source",
        "rules2",
        "-c",
    ]
    .iter()
    .map(OsString::from)
    .collect();
    let kept: Vec<OsString> = config_options(prefix.as_slice());
    let expected: Vec<OsString> = [
        "-c",
        "core.autocrlf=true",
        "-c",
        "a.b=c",
        "--config-env=attr.tree=RULES",
        "--config-env",
        "attr.tree=RULES2",
        "--attr-source=rules",
        "--attr-source",
        "rules2",
    ]
    .iter()
    .map(OsString::from)
    .collect();
    assert_eq!(kept, expected);
    assert!(config_options(&[]).is_empty());
}

/// The journal entry names the file,
/// the operation,
/// the number of parts and the three blobs,
/// and carries every part in its message.
#[test]
fn the_journal_entry_names_the_file_the_operation_and_the_parts() {
    let span = LineSpan { first: 1, count: 1 };
    let part = MergedPart { base: span, ours: span, theirs: span, merged: b"joined\n".to_vec() };
    let stage = |oid: &str| { return Some(Stage { mode: 0o100_644, oid: String::from(oid) }); };
    let stages = Conflict { path: b"a.md".to_vec(), base: stage("aa12"), ours: stage("bb34"), theirs: stage("cc56") };
    let merged = Merged { path: b"a.md".to_vec(), mode: 0o100_644, bytes: Vec::new(), parts: vec![part.clone(), part], stages };
    let (message, fields) = journal_entry("rebase", &merged);
    assert!(message.contains("a.md") && message.contains("part 2 of 2"), "{message}");
    assert!(message.contains("from base blob aa12, ours blob bb34, theirs blob cc56"), "{message}");
    assert_eq!(
        fields,
        vec![
            (FILE_FIELD, String::from("a.md")),
            (OPERATION_FIELD, String::from("rebase")),
            (PARTS_FIELD, String::from("2")),
            (BLOBS_FIELD, String::from("base=aa12,ours=bb34,theirs=cc56")),
        ]
    );
    assert!(fields.iter().all(|(name, _)| { return crate::journald_native::valid_field_name(name); }));
}
