//! Controls for the plan and the capped input.
//! Every bound is the module's named constant or a relation to the input.

use super::*;

/// Owned names from literals,
/// as Git's NUL-separated names give them.
fn names(items: &[&str]) -> Vec<ChangedPath> {
    return items.iter().map(|item| return ChangedPath::from_bytes(item.as_bytes().to_vec())).collect();
}

/// A commit whose patch has `patch_lines` lines (no final newline) and whose files are given.
fn commit_with(patch_lines: usize, files: &[(&str, &str)]) -> LandedCommit {
    let patch: Vec<u8> = (0..patch_lines).map(|number| return format!("patch {number}")).collect::<Vec<String>>().join("\n").into_bytes();
    return LandedCommit {
        sha: String::from("0123456789abcdef"),
        subject: String::from("subject"),
        patch,
        contents: files.iter().map(|(name, text)| return (ChangedPath::from_bytes(name.as_bytes().to_vec()), String::from(*text))).collect(),
    };
}

/// No names and only the log give no entry;
/// a code-free commit gets the skip entry;
/// one code name reviews.
#[test]
fn plan_follows_the_names() {
    assert_eq!(plan_for(&[]), Plan::Silent);
    assert_eq!(plan_for(&names(&[TRACKED_LOG_RELATIVE])), Plan::Silent);
    assert_eq!(plan_for(&names(&[TRACKED_LOG_RELATIVE, TRACKED_LOG_RELATIVE])), Plan::Silent);
    assert_eq!(plan_for(&names(&["doc/a.md", TRACKED_LOG_RELATIVE])), Plan::NoCode);
    assert_eq!(plan_for(&names(&["a.tsx.md", "b.TS", "c.json"])), Plan::NoCode);
    for suffix in CODE_SUFFIXES {
        let name: ChangedPath = ChangedPath::from_bytes(format!("src/x{suffix}").into_bytes());
        assert_eq!(plan_for(&[name.clone(), ChangedPath::from_bytes(b"README.md".to_vec())]), Plan::Review(vec![name]));
    }
}

/// The patch cap applies from one line past the cap,
/// and the whole patch's line count is what is reported.
#[test]
fn patch_cap_switches_exactly_at_the_cap() {
    let at_cap: ReviewInput = build_input(&commit_with(CAP_LINES, &[]), &[]);
    assert!(!at_cap.truncated);
    assert_eq!(at_cap.diff_line_count, CAP_LINES);
    let over: LandedCommit = commit_with(CAP_LINES + 1, &[]);
    let capped: ReviewInput = build_input(&over, &[]);
    assert!(capped.truncated);
    assert_eq!(capped.diff_line_count, CAP_LINES + 1);
    assert_eq!(capped.model_input.split('\n').count(), CAP_LINES + 2, "kept lines, then the blank separator and empty excerpt");
    assert!(!capped.model_input.contains(&format!("patch {CAP_LINES}")));
}

/// A patch that ends with a newline has one more,
/// empty,
/// line,
/// as `split('\n')` counts.
#[test]
fn a_final_newline_counts_as_a_line() {
    let mut commit: LandedCommit = commit_with(3, &[]);
    commit.patch.push(b'\n');
    assert_eq!(build_input(&commit, &[]).diff_line_count, 4);
}

/// The excerpt window is the changed span plus the context,
/// clamped to the file,
/// and its header says so.
#[test]
fn excerpt_window_is_span_plus_context() {
    let file: String = (1..=200).map(|number| return format!("l{number}")).collect::<Vec<String>>().join("\n");
    let mut commit: LandedCommit = commit_with(0, &[("src/a.ts", file.as_str())]);
    commit.patch = Vec::from("diff --git a/src/a.ts b/src/a.ts\n--- a/src/a.ts\n+++ b/src/a.ts\n@@ -100,2 +100,2 @@\n-l100\n-l101\n+x\n+y\n");
    let input: ReviewInput = build_input(&commit, &names(&["src/a.ts"]));
    // The two added lines are new lines 100 and 101.
    let from: i64 = 100 - EXCERPT_CONTEXT;
    let to: i64 = 101 + EXCERPT_CONTEXT;
    let total: usize = file.split('\n').count();
    assert!(input.model_input.contains(&format!("// candidate excerpt of src/a.ts, lines {from} to {to} of {total}")));
    assert!(input.model_input.contains(&format!("l{from}\n")));
    assert!(input.model_input.ends_with(&format!("l{to}")));
}

/// A change is excerpted at its new-side lines even when an earlier insertion made the old and new numbers differ.
#[test]
fn the_window_follows_new_side_lines() {
    let file: String = (1..=300).map(|number| return format!("l{number}")).collect::<Vec<String>>().join("\n");
    let mut commit: LandedCommit = commit_with(0, &[("a.ts", file.as_str())]);
    // Old lines 200 to 201 became new lines 210 to 211 after ten lines were inserted earlier.
    commit.patch = Vec::from("diff --git a/a.ts b/a.ts\n@@ -200,2 +210,2 @@\n-old\n-old\n+new\n+new\n");
    let input: ReviewInput = build_input(&commit, &names(&["a.ts"]));
    assert!(input.model_input.contains(&format!("lines {} to {} of", 210 - EXCERPT_CONTEXT, 211 + EXCERPT_CONTEXT)));
}

/// A file with nothing to anchor on gets the head of the file,
/// never a negative window.
#[test]
fn an_unanchored_file_gets_its_head() {
    let file: String = (1..=300).map(|number| return format!("l{number}")).collect::<Vec<String>>().join("\n");
    let commit: LandedCommit = commit_with(1, &[("a.ts", file.as_str())]);
    let input: ReviewInput = build_input(&commit, &names(&["a.ts"]));
    assert!(input.model_input.contains(&format!("lines 1 to {HEAD_WINDOW} of 300")));
    assert!(input.model_input.ends_with(&format!("l{HEAD_WINDOW}")));
}

/// Only the first files up to the file cap are excerpted,
/// and a file past the cap does not appear.
#[test]
fn excerpts_stop_at_the_file_cap() {
    let many: Vec<String> = (0..=MAX_EXCERPT_FILES).map(|number| return format!("f{number}.rs")).collect();
    let files: Vec<(&str, &str)> = many.iter().map(|name| return (name.as_str(), "x")).collect();
    let commit: LandedCommit = commit_with(1, files.as_slice());
    let paths: Vec<ChangedPath> = many.iter().map(|name| return ChangedPath::from_bytes(name.as_bytes().to_vec())).collect();
    let input: ReviewInput = build_input(&commit, paths.as_slice());
    assert_eq!(input.code_file_count, many.len());
    assert_eq!(input.model_input.matches("// candidate excerpt of").count(), MAX_EXCERPT_FILES);
    assert!(!input.model_input.contains(many[MAX_EXCERPT_FILES].as_str()));
}

/// Files stop getting excerpts once the line cap is reached,
/// and a deleted file gets none and costs no lines.
#[test]
fn excerpts_stop_at_the_line_cap_and_skip_deleted_files() {
    let wide: String = (0..EXCERPT_LINE_CAP).map(|_| return "w").collect::<Vec<&str>>().join("\n");
    let mut commit: LandedCommit = commit_with(1, &[("a.ts", wide.as_str()), ("c.ts", "late")]);
    // One hunk adding every line of the first file, so its window alone reaches the cap.
    commit.patch = format!("diff --git a/a.ts b/a.ts\n@@ -0,0 +1,{EXCERPT_LINE_CAP} @@\n{}", "+w\n".repeat(usize::try_from(EXCERPT_LINE_CAP).expect("small"))).into_bytes();
    let input: ReviewInput = build_input(&commit, &names(&["a.ts", "gone.ts", "c.ts"]));
    assert!(input.model_input.contains("excerpt of a.ts"));
    assert!(!input.model_input.contains("gone.ts"));
    assert!(!input.model_input.contains("excerpt of c.ts"), "the cap is reached after the first file");
    // The same commit with a small first file reaches the third file, so the cap is what shut it out.
    let small: LandedCommit = commit_with(1, &[("a.ts", "tiny"), ("c.ts", "late")]);
    let reached: ReviewInput = build_input(&small, &names(&["a.ts", "gone.ts", "c.ts"]));
    assert!(reached.model_input.contains("excerpt of c.ts"));
}

/// The input is cut by UTF-16 units,
/// the reported length is the cut length,
/// and a short input reports its own length.
#[test]
fn input_is_cut_by_utf16_units() {
    let mut commit: LandedCommit = commit_with(1, &[]);
    commit.patch = "\u{1f600}".repeat(INPUT_UNIT_CAP).into_bytes();
    let input: ReviewInput = build_input(&commit, &[]);
    assert_eq!(input.model_input_units, INPUT_UNIT_CAP);
    assert!(utf16_length(input.model_input.as_str()) <= INPUT_UNIT_CAP);
    assert!(input.model_input.chars().all(|character| return character == '\u{1f600}' || character == '\n'));
    let short: ReviewInput = build_input(&commit_with(2, &[]), &[]);
    assert_eq!(short.model_input_units, utf16_length(short.model_input.as_str()));
}

/// A section whose path Git quotes still anchors its excerpt on the changed line,
/// so the model reads that line's context and never the head of the file.
#[test]
fn a_quoted_path_section_anchors_its_excerpt() {
    let name: &str = "quote\".rs";
    let file: String = (1..=200).map(|number| return format!("l{number}")).collect::<Vec<String>>().join("\n");
    let changed: i64 = 123;
    let mut commit: LandedCommit = commit_with(0, &[(name, file.as_str())]);
    // The header Git writes for a quoted path, and a one-line replacement at `changed`.
    commit.patch = format!("diff --git \"a/quote\\\".rs\" \"b/quote\\\".rs\"\n--- \"a/quote\\\".rs\"\n+++ \"b/quote\\\".rs\"\n@@ -{changed},1 +{changed},1 @@\n-before\n+after\n").into_bytes();
    let input: ReviewInput = build_input(&commit, &names(&[name]));
    let (from, to) = (changed - EXCERPT_CONTEXT, changed + EXCERPT_CONTEXT);
    assert!(input.model_input.contains(&format!("// candidate excerpt of {name}, lines {from} to {to} of 200")), "{}", input.model_input);
}
