//! What:
//!  The ban on a sentence that shows a name another program chose without the one escape:
//!  no release source builds such a sentence from a raw name,
//!  and every sentence of this program that names a process calls
//!  `shown_so_it_cannot_rewrite_the_line` once per name for the wrapping that sentence uses.
//! Why:
//!  A process sets its own name (`/proc/<pid>/comm`,
//!  and the `comm` field of `/proc/<pid>/stat`)
//!  to any bytes it likes,
//!  and a command line is its own text too:
//!  an escape in it moves the cursor or erases the line,
//!  a line break starts a line this program never wrote,
//!  and a `)` or a backtick ends the wrapping the name is shown in,
//!  so the rest of the name reads as this program's own words.
//!  Alpha.49 escaped one waiting line and printed the same name raw three lines later
//!  (the review's finding M2 of 2026-11-27),
//!  and no module test could see it,
//!  because each sentence only ever met a name of printable ASCII.
//!  Two lists make the rule checkable:
//!  the sentences this program has,
//!  which each call the escape,
//!  and the accessors a name is read through,
//!  which no `format!` may interpolate raw.
//! Gotcha:
//!  The second list reads the accessors a name is reached through,
//!  not the value:
//!  a sentence that binds the name to a local of another spelling before it formats it is
//!  seen by the first list alone,
//!  which is why the first list names every sentence and fails when one stops calling the escape.
//!  A wider read of the locals was tried on 2026-11-27 and cannot be precise:
//!  the name propagates from an accessor into locals called `name`, `operation` and `relative`,
//!  and those words are also this program's own text in the same files,
//!  so the wider read reported eleven sentences that show no process name at all.
//!  What a reader of this file checks, because the scan cannot:
//!  every new sentence that shows a name is written into `THE_SENTENCES`,
//!  and no `format!` of such a sentence interpolates the name through a local,
//!  or through any other name of its own, instead of the escape.
//!  The control `a_sentence_that_binds_a_name_to_a_local_of_another_spelling_is_not_seen`
//!  plants that sentence and asserts the scan reports nothing,
//!  so the hole is recorded where a reader meets the rule.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(sentencesMissingTheEscape(THE_SENTENCES, releaseSources())).toEqual([]);
//! expect(formatCallsWithARawName(releaseSources())).toEqual([]);
//! ```

use crate::source_scan_tests::{
    CrateRoot,
    SourceFile,
    classified_sources,
    real_roots,
    release_root,
    source_root,
    write_tree,
};
use std::path::{
    Path,
    PathBuf,
};

/// The escape every sentence that names a process calls,
/// once for each name it shows.
const THE_ESCAPE: &str = "shown_so_it_cannot_rewrite_the_line(";

/// The one function that writes text another program chose into a line.
const THE_FILE: &str = "foundation/shown_so_it_cannot_rewrite_the_line.rs";

/// What:
///  The accessors a name another program chose is read through:
///  the `comm` of a process the scan found,
///  the name the kernel keeps for the process a pid file names,
///  the name of a program below the step of a commit,
///  the program and journal folder of a look at the command lines of this account,
///  and the name of a process whose command line could not be read.
/// Why:
///  A `format!` that holds one of them and not the escape writes that name raw,
///  which is the finding this ban exists for.
const A_NAME_IS_READ_THROUGH: &[&str] = &[
    ".command.as_deref()",
    ".command.clone()",
    ".command.as_ref()",
    "used.name",
    "the_held.name",
    "program.program",
    "program.journal_word",
    "unreadable.program",
];

/// What:
///  One sentence of this program that shows a name another program chose:
///  the source that builds it,
///  the function in it,
///  the wrapping that sentence puts around the name,
///  how many names the sentence shows,
///  and why it shows one.
/// Why:
///  A sentence is the unit the rule is about:
///  it is where the name reaches the person,
///  and one function holds every name of it,
///  so the count is what catches a second name added to a sentence the escape already serves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ASentence = { file: string; function: string; wrapping: string; names: number; why: string };
/// ```
struct ASentence {
    /// The source that builds the sentence,
    /// as its identity under `src/native`.
    file: &'static str,
    /// The function in that source,
    /// as the text after `fn ` up to its argument list.
    function: &'static str,
    /// The wrapping the sentence puts around the name:
    /// `Parentheses` or `Backticks`.
    wrapping: &'static str,
    /// How many names of processes the sentence shows.
    names: usize,
    /// Why it shows a name,
    /// and what a person reads it for.
    why: &'static str,
}

/// What:
///  Every sentence of this program that shows a name another program chose.
/// Why:
///  The list is the ban's subject:
///  a sentence that stops calling the escape fails here,
///  and a sentence that is not listed is found by the scan below when it interpolates a name raw.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const THE_SENTENCES: ASentence[] = [ ... ];
/// ```
const THE_SENTENCES: &[ASentence] = &[
    ASentence {
        file: "lock/index_lock_evidence_words.rs",
        function: "holder_name",
        wrapping: "Parentheses",
        names: 1,
        why: "the PID with its program in parentheses, which every waiting line repeats",
    },
    ASentence {
        file: "lock/index_lock_evidence_words.rs",
        function: "evidence_words",
        wrapping: "Parentheses",
        names: 2,
        why: "the sentence about a Git lock PID file, in the arm for a reused number and the arm for a live owner",
    },
    ASentence {
        file: "lock/index_lock_holder_words.rs",
        function: "holder_way_out",
        wrapping: "Parentheses",
        names: 1,
        why: "the way out for a holder that is stuck, which names the program to end",
    },
    ASentence {
        file: "lock/stale_ref_lock_judge.rs",
        function: "a_held_file_verdict",
        wrapping: "Parentheses",
        names: 1,
        why: "the verdict that a live process has the judged lock, or the judged staging file, open",
    },
    ASentence {
        file: "process/step_output_held_open_by_a_program_the_step_started.rs",
        function: "holder_named",
        wrapping: "Parentheses",
        names: 1,
        why: "the program a step started that still holds one of its streams",
    },
    ASentence {
        file: "commit/scratch_step_notice_line.rs",
        function: "notice_line",
        wrapping: "Backticks",
        names: 1,
        why: "the program running below the step of a commit while it waits",
    },
    ASentence {
        file: "commit/scratch_step_held_by_the_kernel.rs",
        function: "held_by_the_kernel_line",
        wrapping: "Backticks",
        names: 1,
        why: "the program the kernel holds in a disk wait instead of the step itself",
    },
    ASentence {
        file: "commit/scratch_step_held_by_the_kernel.rs",
        function: "a_stall_is_waited_for",
        wrapping: "Backticks",
        names: 1,
        why: "the program the kernel holds, in the line that says why the ceiling stood aside",
    },
    ASentence {
        file: "landing/earlier_service_journal_adopted.rs",
        function: "program_note",
        wrapping: "Backticks",
        names: 2,
        why: "the program behind a hold and the journal folder it names, in the service log",
    },
    ASentence {
        file: "invocation/management_service_settle_earlier_batches.rs",
        function: "unsure_program_sentences",
        wrapping: "Backticks",
        names: 2,
        why: "the program behind a hold and the journal folder it named, in the command's answer",
    },
    ASentence {
        file: "invocation/management_service_settle_earlier_batches.rs",
        function: "unreadable_process_sentences",
        wrapping: "Backticks",
        names: 1,
        why: "the process whose command line this check could not read",
    },
    ASentence {
        file: "landing/service_index_found.rs",
        function: "open_by_text",
        wrapping: "Parentheses",
        names: 1,
        why: "the processes a scan saw holding a lock open, as the `open_by` field",
    },
    ASentence {
        file: "landing/service_index_holder_look.rs",
        function: "holder_words",
        wrapping: "Parentheses",
        names: 1,
        why: "the holder a service's answer names",
    },
    ASentence {
        file: "subvolume/subvolume_migrate_words.rs",
        function: "a_process_left_in_the_old_tree_line",
        wrapping: "Parentheses",
        names: 1,
        why: "the process a migration leaves in the old tree",
    },
    ASentence {
        file: "worktree/worktree_convert_swap_and_removal.rs",
        function: "a_process_using_the_worktree_line",
        wrapping: "Parentheses",
        names: 1,
        why: "the process that stops a swap from happening",
    },
    ASentence {
        file: "worktree/worktree_convert_swap_and_removal.rs",
        function: "a_process_in_the_converted_tree_line",
        wrapping: "Parentheses",
        names: 1,
        why: "the process that keeps the converted tree from being removed",
    },
    ASentence {
        file: "worktree/worktree_convert_inspection.rs",
        function: "a_process_inside_the_tree_refusal",
        wrapping: "Parentheses",
        names: 1,
        why: "the process that makes a conversion refuse to start",
    },
    ASentence {
        file: "worktree/worktree_convert_abandon.rs",
        function: "a_process_inside_the_directory_line",
        wrapping: "Parentheses",
        names: 1,
        why: "the process that keeps an abandoned stage or tree",
    },
    ASentence {
        file: "worktree_clone_forced_removal/the_refusal_sentences.rs",
        function: "caller_holds_in_its_working_directory",
        wrapping: "Parentheses",
        names: 1,
        why: "the process that holds the clone through its working directory, in a forced removal's refusal",
    },
    ASentence {
        file: "worktree_clone_forced_removal/the_refusal_sentences.rs",
        function: "the_callers_namespace_holds",
        wrapping: "Parentheses",
        names: 1,
        why: "the process in the caller's session that a mount namespace's copy is held through",
    },
];

/// What:
///  The sources the scan reads:
///  every file the module tree reaches,
///  so a file this walk missed is a file the ban does not cover.
/// Why:
///  A sentence in a file the scan never opened would pass every control here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sourcesIn(root: string, roots: CrateRoot[]): SourceFile[] { return classifiedSources(root, roots); }
/// ```
fn sources_in(
    root: &Path,
    roots: &[CrateRoot],
) -> Result<Vec<SourceFile>, String> {
    return classified_sources(root, roots);
}

/// What:
///  The body of `function` in `text`:
///  from the line that declares it to the first line that is one `}` at column 0.
/// Why:
///  Every function of this crate closes at column 0,
///  so the body is what the counts below are taken over,
///  and a sentence that moved into a sibling file is not found at all
///  (the entry then names no body,
///  which fails).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function bodyOf(text: string, fn: string): string | undefined;
/// ```
fn body_of(
    text: &str,
    function: &str,
) -> Option<String> {
    let declaration: String = format!("fn {function}(");
    let mut body: Vec<&str> = Vec::new();
    let mut inside: bool = false;
    for line in text.lines() {
        if !inside {
            inside = line.contains(declaration.as_str());
            if inside {
                body.push(line);
            }
            continue;
        }
        if line == "}" {
            return Some(body.join("\n"));
        }
        body.push(line);
    }
    return None;
}

/// What:
///  Every place the sentences list does not hold to:
///  a file that is no release source,
///  a function the file no longer holds,
///  and a body whose calls to the escape are not one per name for that sentence's wrapping.
/// Why:
///  Each finding names the sentence so a reader opens it,
///  and a sentence that no longer exists cannot go on passing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sentencesMissingTheEscape(sources, sentences): string[];
/// ```
fn sentences_missing_the_escape(
    texts: &[(String, String)],
    sentences: &[ASentence],
) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for sentence in sentences {
        let Some((_path, text)) = texts
            .iter()
            .find(|(path, _text)| return path == sentence.file)
        else {
            found.push(format!(
                "{}: no release source of this crate, so the sentence `{}` ({}) is not read at all",
                sentence.file, sentence.function, sentence.why
            ));
            continue;
        };
        let Some(body) = body_of(text.as_str(), sentence.function) else {
            found.push(format!(
                "{}: no `fn {}` any more, so the sentence that shows {} is gone or renamed",
                sentence.file, sentence.function, sentence.why
            ));
            continue;
        };
        let calls: usize = body.matches(THE_ESCAPE).count();
        let wrapping: String = format!("TheWrappingAroundTheName::{}", sentence.wrapping);
        let stated: usize = body.matches(wrapping.as_str()).count();
        if calls != sentence.names || stated != sentence.names {
            found.push(format!(
                "{}: `{}` shows {} name(s) of another program ({}) and calls the escape {calls} time(s) for `{}` {stated} time(s)",
                sentence.file, sentence.function, sentence.names, sentence.why, sentence.wrapping
            ));
        }
    }
    return found;
}

/// What:
///  Whether `text` names an accessor of `A_NAME_IS_READ_THROUGH` as a name of its own.
/// Why:
///  `refused.namespace` ends in the letters of `used.name`,
///  and a scan that read that as a process name would report a sentence that names no process at all,
///  so an accessor written without a leading dot must start a word,
///  which is what tells `used.name` from the tail of `refused.namespace`.
///  An accessor that starts with its dot already starts at one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const namesAProcess = (text: string): boolean => A_NAME_IS_READ_THROUGH.some((accessor) => {
///   for (let at = text.indexOf(accessor); at >= 0; at = text.indexOf(accessor, at + 1)) {
///     if (accessor.startsWith('.') || !/[A-Za-z0-9_]/.test(text[at - 1] ?? '')) return true;
///   }
///   return false;
/// });
/// ```
fn names_a_process(text: &str) -> bool {
    for accessor in A_NAME_IS_READ_THROUGH {
        let mut from: usize = 0;
        while let Some(found) = text
            .get(from..)
            .and_then(|rest| return rest.find(*accessor))
        {
            let at: usize = from + found;
            let before: Option<char> = text
                .get(..at)
                .and_then(|before| return before.chars().next_back());
            let starts_a_word: bool = !accessor.starts_with('.');
            let stands_alone: bool = !starts_a_word
                || before.is_none_or(|character| {
                    return !character.is_alphanumeric() && character != '_';
                });
            if stands_alone {
                return true;
            }
            from = at + accessor.len();
        }
    }
    return false;
}

/// What:
///  Every `format!` call in `text`,
///  each as the call's own text and where it starts.
/// Why:
///  A format call may run over many lines,
///  and a name is interpolated inside the call,
///  so the unit the scan judges is the whole call and not one line of it.
///  A string literal of the call is skipped whole,
///  because the parenthesis a sentence writes in its own words is no part of the call's shape.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function formatCalls(text: string): { call: string; at: number }[];
/// ```
fn format_calls(text: &str) -> Vec<(String, usize)> {
    let mut calls: Vec<(String, usize)> = Vec::new();
    let bytes: &[u8] = text.as_bytes();
    let mut at: usize = 0;
    while let Some(found) = text.get(at..).and_then(|rest| return rest.find("format!(")) {
        let start_index: usize = at + found;
        let open: usize = start_index + "format!".len();
        let (end, _closed): (usize, bool) = balanced_end(bytes, open);
        calls.push((
            text.get(start_index..end).unwrap_or_default().to_owned(),
            start_index,
        ));
        at = end.max(start_index + 1);
    }
    return calls;
}

/// What:
///  Where the parentheses opened at `open` close,
///  and whether they closed at all before the text ended.
/// Why:
///  A call's own string literals hold parentheses of their own,
///  and a parenthesis inside a literal would end the call early if it were counted:
///  so a literal is skipped whole,
///  with the backslash that escapes its quote inside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function balancedEnd(bytes: Uint8Array, open: number): [number, boolean];
/// ```
fn balanced_end(
    bytes: &[u8],
    open: usize,
) -> (usize, bool) {
    let mut depth: usize = 0;
    let mut in_a_literal: bool = false;
    let mut escaped: bool = false;
    for (offset, byte) in bytes.get(open..).unwrap_or_default().iter().enumerate() {
        if in_a_literal {
            if escaped {
                escaped = false;
            } else if *byte == b'\\' {
                escaped = true;
            } else if *byte == b'"' {
                in_a_literal = false;
            }
            continue;
        }
        match byte {
            b'"' => in_a_literal = true,
            b'(' => depth += 1,
            b')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return (open + offset + 1, true);
                }
            }
            _ => {}
        }
    }
    return (bytes.len(), false);
}

/// What:
///  Every `file:line` of a release source whose `format!` interpolates a name another program
///  chose without the escape.
/// Why:
///  The list is the finding list the ban's control asserts empty,
///  and the planted fixture below shows it fills.
///  A call that starts on a comment line is no finding:
///  the crate explains these sentences in its docs far more often than it writes them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function findingsIn(root, roots): string[];
/// ```
fn findings_in(
    root: &Path,
    roots: &[CrateRoot],
) -> Result<Vec<String>, String> {
    let sources: Vec<SourceFile> = sources_in(root, roots)?;
    let mut found: Vec<String> = Vec::new();
    for file in &sources {
        if file.test_only {
            continue;
        }
        let text: String = std::fs::read_to_string(root.join(file.path.as_str()))
            .map_err(|error| return format!("cannot read {}: {error}", file.path))?;
        for (call, start) in format_calls(text.as_str()) {
            if call.contains(THE_ESCAPE) || !names_a_process(call.as_str()) {
                continue;
            }
            let line: usize = text.get(..start).unwrap_or_default().matches('\n').count() + 1;
            let written: &str = text.lines().nth(line - 1).unwrap_or_default();
            if written.trim_start().starts_with("//") {
                continue;
            }
            found.push(format!("{}:{line}", file.path));
        }
    }
    return Ok(found);
}

/// The sources of this crate's release build,
/// each read once.
fn release_texts(
    root: &Path,
    roots: &[CrateRoot],
) -> Result<Vec<(String, String)>, String> {
    let sources: Vec<SourceFile> = sources_in(root, roots)?;
    let mut texts: Vec<(String, String)> = Vec::new();
    for file in &sources {
        if file.test_only {
            continue;
        }
        let text: String = std::fs::read_to_string(root.join(file.path.as_str()))
            .map_err(|error| return format!("cannot read {}: {error}", file.path))?;
        texts.push((file.path.clone(), text));
    }
    return Ok(texts);
}

/// What:
///  Every sentence this program has that shows a name another program chose calls the one escape,
///  once per name,
///  for the wrapping that sentence uses.
/// Why:
///  A sentence that stopped calling it would print the name as the program spelled it,
///  which is how alpha.49 printed one name escaped and the same name raw in another line of the
///  same wait (finding M2 of the 2026-11-27 review).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(sentencesMissingTheEscape(releaseTexts(sourceRoot(), realRoots()), THE_SENTENCES)).toEqual([]);
/// ```
#[test]
fn every_sentence_that_names_a_process_shows_it_for_the_wrapping_it_uses() {
    let root: PathBuf = source_root();
    let roots: Vec<CrateRoot> = real_roots();
    let texts: Vec<(String, String)> =
        release_texts(root.as_path(), roots.as_slice()).expect("the crate's own sources");
    let read: Vec<&str> = texts
        .iter()
        .map(|(path, _text)| return path.as_str())
        .collect();
    assert!(
        read.contains(&THE_FILE),
        "the scan read {} files of this crate and not `{THE_FILE}`, so it read nothing it is about",
        read.len()
    );
    let found: Vec<String> = sentences_missing_the_escape(texts.as_slice(), THE_SENTENCES);
    assert!(
        found.is_empty(),
        "every sentence that shows a name another program chose must call `{THE_ESCAPE}..)` once per name, \
         for the wrapping it puts around the name (`AGENTS.md`, rule `SYB`); the files read are {} of this crate:\n{}",
        read.len(),
        found.join("\n")
    );
}

/// What:
///  No release source interpolates a name another program chose into a `format!` without the escape.
/// Why:
///  This is the half of the ban that sees a sentence nobody listed:
///  a new sentence that reads `comm` or a command line and writes it into a line
///  is the finding,
///  whatever file it is in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(findingsIn(sourceRoot(), realRoots())).toEqual([]);
/// ```
#[test]
fn no_release_sentence_interpolates_a_name_another_program_chose_without_the_escape() {
    let found: Vec<String> =
        findings_in(&source_root(), &real_roots()).expect("the crate's sources");
    assert!(
        found.is_empty(),
        "a name another program chose is read through a `format!` without `{THE_ESCAPE}..)`, so the \
         program that chose it writes part of the line (finding M2 of the 2026-11-27 review); \
         take the name through `crate::shown_so_it_cannot_rewrite_the_line`, or add the sentence to \
         `THE_SENTENCES` in this file:\n{}",
        found.join("\n")
    );
}

/// What:
///  The scan fires on a sentence that interpolates a name raw,
///  and not on one that takes the name through the escape.
/// Why:
///  A check that stopped seeing its own needle would pass every tree,
///  and the shape a passing sentence takes is the one a later round writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(findingsIn(fixtureRoot, [releaseRoot('lib.rs')])).toEqual(['reader.rs:1']);
/// ```
#[test]
fn the_scan_finds_a_planted_sentence_that_does_not_show_its_name() {
    let root: PathBuf = crate::test_support::fixture("source-guard-process-names");
    write_tree(
        root.as_path(),
        &[
            ("lib.rs", "mod reader;\nmod words;\n"),
            (
                "reader.rs",
                "pub fn raw(user: &crate::process_use::ProcessUser) -> String {\n\
                 \x20   format!(\"process {} ({}) has it\", user.pid, user.command.as_deref().unwrap_or(\"\"))\n\
                 }\n",
            ),
            (
                "words.rs",
                "use crate::shown_so_it_cannot_rewrite_the_line::{TheWrappingAroundTheName, shown_so_it_cannot_rewrite_the_line};\n\
                 pub fn shown(command: &str) -> String {\n\
                 \x20   format!(\"({})\", shown_so_it_cannot_rewrite_the_line(command, TheWrappingAroundTheName::Parentheses))\n\
                 }\n",
            ),
        ],
    );
    let found: Vec<String> =
        findings_in(root.as_path(), &[release_root("lib.rs")]).expect("the fixture sources");
    assert_eq!(
        found,
        vec![String::from("reader.rs:2")],
        "the planted sentence that interpolates a name raw, and not the one that takes it through the escape"
    );
    crate::test_support::remove(root.as_path());
}

/// What:
///  A sentence that binds the name to a local of another spelling
///  (`let Some(ref seen) = process.command` and `{seen}`) is not seen by the scan,
///  and the same fixture's sentence that interpolates the accessor itself is seen.
/// Why:
///  The accessor list reads spellings,
///  and neither `{seen}` nor the line that binds it holds one,
///  so this is the hole the top doc names and the sentence list closes.
///  The second file is the positive control:
///  a null over a fixture the scan cannot read at all would prove nothing,
///  and this one shows the same run reports the raw interpolation beside it.
///  A round that teaches the scan the local should turn this control into one that
///  expects `reader.rs:3` too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(findingsIn(fixtureRoot, [releaseRoot('lib.rs')])).toEqual(['raw.rs:2']);
/// ```
#[test]
fn a_sentence_that_binds_a_name_to_a_local_of_another_spelling_is_not_seen() {
    let root: PathBuf = crate::test_support::fixture("source-guard-process-name-locals");
    write_tree(
        root.as_path(),
        &[
            ("lib.rs", "mod raw;\nmod reader;\n"),
            (
                "reader.rs",
                "pub fn through_a_local(user: &crate::process_use::ProcessUser) -> String {\n\
                 \x20   let Some(ref seen) = user.command else {\n\
                 \x20       return String::new();\n\
                 \x20   };\n\
                 \x20   format!(\"process {} ({seen}) has it\", user.pid)\n\
                 }\n",
            ),
            (
                "raw.rs",
                "pub fn raw(user: &crate::process_use::ProcessUser) -> String {\n\
                 \x20   format!(\"process {} ({}) has it\", user.pid, user.command.as_deref().unwrap_or(\"\"))\n\
                 }\n",
            ),
        ],
    );
    let found: Vec<String> =
        findings_in(root.as_path(), &[release_root("lib.rs")]).expect("the fixture sources");
    assert_eq!(
        found,
        vec![String::from("raw.rs:2")],
        "the local spelling is not seen, and the raw interpolation beside it is"
    );
    crate::test_support::remove(root.as_path());
}

/// What:
///  The list of sentences names a function its source still holds,
///  and the file it names is still a release source.
/// Why:
///  A sentence that moved into a sibling file,
///  or that was renamed,
///  would otherwise pass as a sentence that no longer exists,
///  and a split moves a call into a sibling file without changing what the sentence says.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// expect(sentencesMissingTheEscape(releaseTexts(), THE_SENTENCES).filter(namesNoBody)).toEqual([]);
/// ```
#[test]
fn every_listed_sentence_is_still_a_function_of_the_file_it_names() {
    let root: PathBuf = source_root();
    let roots: Vec<CrateRoot> = real_roots();
    let texts: Vec<(String, String)> =
        release_texts(root.as_path(), roots.as_slice()).expect("the crate's own sources");
    let mut missing: Vec<String> = Vec::new();
    for sentence in THE_SENTENCES {
        let body: Option<String> = texts
            .iter()
            .find(|(path, _text)| return path == sentence.file)
            .and_then(|(_path, text)| return body_of(text.as_str(), sentence.function));
        if body.is_none() {
            missing.push(format!("{}.rs: {}", sentence.file, sentence.function));
        }
    }
    assert!(
        missing.is_empty(),
        "these sentences name no function of the file they name, so move the entry to the file the \
         sentence lives in now:\n{}",
        missing.join("\n")
    );
}
