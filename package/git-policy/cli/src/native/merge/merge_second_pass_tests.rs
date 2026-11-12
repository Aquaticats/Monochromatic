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
    // The part names the line of the file it came from, not the line of Git's conflict region:
    // the fixture edits line 4 and the other side inserts after it.
    assert_eq!(
        file.parts[0],
        MergedPart {
            base: LineSpan { first: 4, count: 1 },
            ours: LineSpan { first: 4, count: 1 },
            theirs: LineSpan { first: 4, count: 2 },
            merged: b"Flags are optional, see below.\nExit code is zero on success.\n".to_vec(),
        }
    );
    remove(world.root.as_path());
}

/// Two conflicts in one file,
/// each an edit of a different word of its line,
/// merge whole,
/// with the text Git settled between them kept as Git wrote it,
/// and both parts named by their line in the file.
#[test]
fn two_resolvable_regions_of_one_file_merge_with_the_text_between_them() {
    let world: World = world(
        "second-pass-two-regions",
        &[("a.md", b"alpha one\nkeep\nbeta two\n")],
        &[("a.md", Change::Write(b"alpha uno\nkeep\nBETA two\n"))],
        &[("a.md", Change::Write(b"ALPHA one\nkeep\nbeta zwei\n"))],
    );
    let verdicts: Vec<Verdict> = verdicts_of(&world);
    let [Verdict::Merged(file)] = verdicts.as_slice() else { panic!("expected one merged file: {verdicts:?}") };
    assert_eq!(String::from_utf8_lossy(file.bytes.as_slice()), "ALPHA uno\nkeep\nBETA zwei\n");
    let spans: Vec<(LineSpan, LineSpan, LineSpan)> = file.parts.iter().map(|part| { return (part.base, part.ours, part.theirs); }).collect();
    let one: LineSpan = LineSpan { first: 1, count: 1 };
    let three: LineSpan = LineSpan { first: 3, count: 1 };
    assert_eq!(spans, vec![(one, one, one), (three, three, three)]);
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

/// What:
///  A line merge whose answer is larger than the bytes one captured step keeps reaches the pass whole,
///  so the file it merges keeps its beginning.
/// Why:
///  The pass re-merges a stage at its own limit,
///  and Git's answer for a conflicted one is larger than that stage:
///  the same text,
///  the other two sides of the region and four marker lines
///  (`STEP_OUTPUT_LIMIT` in `src/native/process/output_capture.rs` keeps the last 8 MiB of a step that is not ended).
///  A pass that reads only the tail cuts its answer in the middle of the file,
///  finds the one region that tail holds,
///  and stages the file without its start:
///  the person's bytes are gone from the file the index records
///  (cross-vendor review of round 133,
///  finding 1).
#[test]
fn a_line_merge_larger_than_the_captured_tail_keeps_the_start_of_the_file() {
    let limit: usize = LARGEST_BLOB_BYTES;
    let start: &[u8] = b"THE START OF THIS FILE MUST SURVIVE THE SECOND PASS\n";
    let last: &[u8] = b"alpha one\n";
    // One long line of filler, then the line both sides changed. The answer is the stage's own bytes, the other two
    // sides of that line and four marker lines, which is more than the bytes a captured step keeps.
    let mut base: Vec<u8> = Vec::with_capacity(limit);
    base.extend_from_slice(start);
    base.resize(limit - last.len() - 1, b'x');
    base.push(b'\n');
    base.extend_from_slice(last);
    assert_eq!(base.len(), limit, "the fixture is exactly the largest stage the pass takes");
    let mut ours: Vec<u8> = base[..base.len() - last.len()].to_vec();
    ours.extend_from_slice(b"alpha uno\n");
    let mut theirs: Vec<u8> = base[..base.len() - last.len()].to_vec();
    theirs.extend_from_slice(b"ALPHA one\n");
    let world: World = world(
        "second-pass-at-the-limit",
        &[("a.md", base.as_slice())],
        &[("a.md", Change::Write(ours.as_slice()))],
        &[("a.md", Change::Write(theirs.as_slice()))],
    );
    let verdicts: Vec<Verdict> = verdicts_of(&world);
    let summary: Vec<String> = verdicts
        .iter()
        .map(|verdict| {
            return match verdict {
                Verdict::Merged(file) => format!("merged, {} bytes", file.bytes.len()),
                Verdict::Left { why, .. } => format!("left: {why:?}"),
            };
        })
        .collect();
    let [Verdict::Merged(file)] = verdicts.as_slice() else { panic!("expected one merged file: {summary:?}") };
    assert!(
        file.bytes.starts_with(start),
        "the start of the file is kept: {} bytes beginning {:?}",
        file.bytes.len(),
        String::from_utf8_lossy(&file.bytes[..file.bytes.len().min(start.len() + 16)])
    );
    assert_eq!(file.bytes.len(), limit, "the merged file is as long as the stage, not the tail of Git's answer: {} bytes", file.bytes.len());
    remove(world.root.as_path());
}

/// What:
///  Marker-shaped text the person's file already held is kept,
///  even when Git reports the conflict count its exit status saturates at.
/// Why:
///  Cutting Git's answer cannot tell Git's delimiters from identical lines the file already held,
///  so the region count against Git's exit status is what catches a wrong cut,
///  and that status is the count truncated to 127:
///  at 127 the comparison was skipped,
///  a literal `zdiff3` example block at the start of the file became one more region,
///  and the word pass replaced it,
///  so the file was staged resolved without the person's text
///  (cross-vendor review of round 133,
///  finding 2).
#[test]
fn marker_shaped_text_of_the_person_is_kept_at_the_truncated_conflict_count() {
    // Git's exit status is the number of conflicts, truncated to this value.
    const GIT_CONFLICT_STATUS_LIMIT: usize = 127;
    let example: &[u8] = b"<<<<<<< documented-example\nalpha uno\n||||||| documented-example\nalpha one\n=======\nALPHA one\n>>>>>>> documented-example\n";
    let mut base_lines: Vec<String> = String::from_utf8(example.to_vec()).expect("text").lines().map(str::to_owned).collect();
    for index in 0..GIT_CONFLICT_STATUS_LIMIT {
        base_lines.push(format!("shared {index} before"));
        base_lines.push(format!("alpha one {index}"));
        base_lines.push(format!("shared {index} after"));
    }
    // Neither side may touch the example block:
    // it is the file's own text, and only the numbered lines are a choice.
    let as_ours = |line: &str| -> String {
        return match line.strip_prefix("alpha one ") {
            Some(rest) => format!("alpha uno {rest}"),
            None => line.to_owned(),
        };
    };
    let as_theirs = |line: &str| -> String {
        return match line.strip_prefix("alpha one ") {
            Some(rest) => format!("ALPHA one {rest}"),
            None => line.to_owned(),
        };
    };
    let ours: Vec<String> = base_lines.iter().map(|line| { return as_ours(line.as_str()); }).collect();
    let theirs: Vec<String> = base_lines.iter().map(|line| { return as_theirs(line.as_str()); }).collect();
    let as_bytes = |lines: &[String]| -> Vec<u8> { return format!("{}\n", lines.join("\n")).into_bytes(); };
    let world: World = world(
        "second-pass-marker-shaped-text",
        &[("a.md", as_bytes(base_lines.as_slice()).as_slice())],
        &[("a.md", Change::Write(as_bytes(ours.as_slice()).as_slice()))],
        &[("a.md", Change::Write(as_bytes(theirs.as_slice()).as_slice()))],
    );
    // The fixture must reach the status at which Git stops counting, or it proves nothing about it.
    let (_, conflicts) = merge_tree_of(&world);
    let stage = |side: Option<&Stage>| -> String {
        return match side {
            Some(found) => found.oid.clone(),
            None => String::from("none"),
        };
    };
    let [conflict] = conflicts.as_slice() else { panic!("expected one conflicted path: {conflicts:?}") };
    let oracle = git_output(
        world.repo.as_path(),
        &[
            "merge-file",
            "--stdout",
            "--object-id",
            "--zdiff3",
            "-L",
            "ours",
            "-L",
            "base",
            "-L",
            "theirs",
            stage(conflict.ours.as_ref()).as_str(),
            stage(conflict.base.as_ref()).as_str(),
            stage(conflict.theirs.as_ref()).as_str(),
        ],
    );
    assert_eq!(
        oracle.status.code(),
        Some(i32::try_from(GIT_CONFLICT_STATUS_LIMIT).expect("a conflict count fits the exit status")),
        "the fixture reaches the count Git's status saturates at"
    );
    let verdicts: Vec<Verdict> = verdicts_of(&world);
    let summary: Vec<String> = verdicts
        .iter()
        .map(|verdict| {
            return match verdict {
                Verdict::Merged(file) => format!("merged, {} bytes", file.bytes.len()),
                Verdict::Left { why, .. } => format!("left: {why:?}"),
            };
        })
        .collect();
    // Every one of the 127 conflicts is word-resolvable, so the file is merged and the person's block is not the only
    // thing that has to survive: a fix that refused at the status ceiling would leave the file for the person and fail here.
    let [Verdict::Merged(file)] = verdicts.as_slice() else { panic!("expected one merged file: {summary:?}") };
    assert!(
        file.bytes.starts_with(example),
        "the literal block the file already held is kept: the merged file begins {:?}",
        String::from_utf8_lossy(&file.bytes[..file.bytes.len().min(example.len() + 16)])
    );
    remove(world.root.as_path());
}

/// What:
///  A split-marker-shaped line one side inserted is that side's text,
///  and the file is merged by word.
/// Why:
///   Git writes its split marker as seven equals signs with no label,
///   so the inserted line read as the split,
///   Git's common ancestor marker then arrived behind it,
///   and the whole answer was refused as out of order although every side of it is what the stages hold
///   (cross-vendor review of round 198,
///   finding 1).
#[test]
fn a_split_marker_shaped_line_of_a_side_is_merged_as_text() {
    let world: World = world(
        "second-pass-literal-split-marker",
        &[("f.txt", b"alpha one\nbeta one\n")],
        &[("f.txt", Change::Write(b"alpha uno\n=======\nbeta one\n"))],
        &[("f.txt", Change::Write(b"alpha one\nbeta uno\n"))],
    );
    // The fixture is a real conflict Git leaves, and the second pass is what settles it.
    let (_, conflicts) = merge_tree_of(&world);
    assert_eq!(conflicts.len(), 1, "the fixture conflicts in one file");
    let verdicts: Vec<Verdict> = verdicts_of(&world);
    let summary: Vec<String> = verdicts
        .iter()
        .map(|verdict| {
            return match verdict {
                Verdict::Merged(file) => format!("merged, {} bytes: {:?}", file.bytes.len(), String::from_utf8_lossy(file.bytes.as_slice())),
                Verdict::Left { why, .. } => format!("left: {why:?}"),
            };
        })
        .collect();
    let [Verdict::Merged(file)] = verdicts.as_slice() else { panic!("expected one merged file: {summary:?}") };
    // Both sides edited a different line, so the merge is the first side's line, its inserted text, and the second
    // side's line.
    assert_eq!(file.bytes, b"alpha uno\n=======\nbeta uno\n".to_vec(), "the inserted line is the first side's text, and every other line comes from one side");
    remove(world.root.as_path());
}

/// What:
///  A conflicted file whose lines end with a carriage return and a line feed is merged by word.
/// Why:
///  Git writes the markers of such a file with that ending too;
///  a cut that accepts only a lone line feed reads no region,
///  the region count then refuses the answer,
///  and the file stays conflicted where the pass used to resolve it.
#[test]
fn a_file_whose_lines_end_with_both_bytes_is_merged() {
    let world: World = world(
        "second-pass-crlf",
        &[("a.md", b"head\r\nalpha one\r\n")],
        &[("a.md", Change::Write(b"head\r\nalpha uno\r\n"))],
        &[("a.md", Change::Write(b"head\r\nALPHA one\r\n"))],
    );
    let verdicts: Vec<Verdict> = verdicts_of(&world);
    let summary: Vec<String> = verdicts
        .iter()
        .map(|verdict| {
            return match verdict {
                Verdict::Merged(file) => format!("merged, {} bytes: {:?}", file.bytes.len(), String::from_utf8_lossy(file.bytes.as_slice())),
                Verdict::Left { why, .. } => format!("left: {why:?}"),
            };
        })
        .collect();
    let [Verdict::Merged(file)] = verdicts.as_slice() else { panic!("expected one merged file: {summary:?}") };
    assert_eq!(file.bytes, b"head\r\nALPHA uno\r\n".to_vec(), "the word merge of the two sides keeps the file's own line ending");
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

/// What:
///  A merge of many one-line regions is word-merged and every part is numbered in the line of the file a person opens.
/// Why:
///  The report moved every part into its file by counting the lines from the top of the stage,
///  so a file of many parts cost its stage once per part:
///  the pass over a hundred and three thousand parts against stages of 7.6 MB took about seven hundred seconds
///  of a push that took 889 seconds,
///  whose merge-tree took about 190
///  (measured 2026-11-11 on this host with the release build).
/// The count is now carried from region to region (`lines_between`),
/// and these numbers are what tells a count carried wrongly from one that is not.
#[test]
fn a_merge_of_many_regions_numbers_every_part_in_its_file() {
    const REGIONS: usize = 2_000;
    let padding: String = "b".repeat(30);
    let (mut base, mut ours, mut theirs) = (String::new(), String::new(), String::new());
    for index in 0..REGIONS {
        if index > 0 {
            let separator: String = format!("s{index}\n");
            base.push_str(separator.as_str());
            ours.push_str(separator.as_str());
            theirs.push_str(separator.as_str());
        }
        base.push_str(format!("alpha one {index} {padding}\n").as_str());
        ours.push_str(format!("alpha uno {index} {padding}\n").as_str());
        theirs.push_str(format!("ALPHA one {index} {padding}\n").as_str());
    }
    let root = fixture("second-pass-many-regions");
    let repo = repository(root.as_path(), "repo");
    for (name, text) in [("base", base.as_str()), ("ours", ours.as_str()), ("theirs", theirs.as_str())] {
        std::fs::write(repo.join(name), text).expect("a stage");
    }
    let hashed = git(&repo, &["hash-object", "-w", "base", "ours", "theirs"]);
    let oids: Vec<String> = String::from_utf8_lossy(&hashed.stdout).trim().split('\n').map(str::to_owned).collect();
    let stage = |oid: &String| { return Some(Stage { mode: 0o100_644, oid: oid.clone() }); };
    let conflict = Conflict { path: b"many.txt".to_vec(), base: stage(&oids[0]), ours: stage(&oids[1]), theirs: stage(&oids[2]) };
    let context = git_context();
    let runner = runner(&context, repo.as_path());
    let sides = [base.as_bytes(), ours.as_bytes(), theirs.as_bytes()];
    let outcome = await_it(crate::merge::merge_second_pass_git_lines::merge_with_git_lines(&runner, &conflict, sides));
    let WordMerge::Resolved { parts, .. } = outcome.expect("the pass runs") else { panic!("every region is the word merge of two edits") };
    assert_eq!(parts.len(), REGIONS, "one part for each region");
    // Each region is one line on each side, and a shared line sits between two of them, so the parts' lines are the odd
    // ones of every stage: the first from line 1.
    for (index, part) in parts.iter().enumerate() {
        let expected: usize = 2 * index + 1;
        assert_eq!((part.base.first, part.ours.first, part.theirs.first), (expected, expected, expected), "the lines of part {}", index + 1);
    }
    remove(root.as_path());
}
