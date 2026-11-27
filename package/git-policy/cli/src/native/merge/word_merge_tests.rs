//! Controls for the word-level three-way merge:
//! equality with Jujutsu 0.46.0's stored results on the corpus,
//! the cases the
//!  owner named (the README conflict of merge `14601b28d3` resolves to the committed blob,
//! both-append and a true overlap
//!  stay conflicts),
//! a file with one unresolvable region stays whole in conflict,
//! and nothing either side wrote is dropped.

use super::*;
use std::path::{
    Path,
    PathBuf,
};

/// Where the stored inputs and Jujutsu's outputs are.
fn corpus() -> PathBuf {
    return Path::new(env!("CARGO_MANIFEST_DIR")).join("src/native/merge/word_merge_corpus");
}

/// Where the inputs whose word merge must be refused are,
/// each as `.base`,
/// `.ours` and `.theirs`.
/// Why:
///  They are the shapes a word merge writes wrong text for,
///  so they are kept apart from the corpus the controls hold to Jujutsu's verdicts:
/// Jujutsu merges some of these the wrong way too.
fn refusals() -> PathBuf {
    return Path::new(env!("CARGO_MANIFEST_DIR")).join("src/native/merge/word_merge_refusals");
}

/// The bytes of one stored file of a refusal input.
fn refused(
    name: &str,
    extension: &str,
) -> Vec<u8> {
    return std::fs::read(refusals().join(format!("{name}.{extension}")))
        .expect("a stored refusal input");
}

/// The names of the refusal inputs,
/// in order.
fn refusal_names() -> Vec<String> {
    let mut found: Vec<String> = std::fs::read_dir(refusals())
        .expect("the refusal directory")
        .filter_map(|entry| {
            return entry
                .ok()?
                .file_name()
                .to_str()?
                .strip_suffix(".base")
                .map(str::to_owned);
        })
        .collect();
    found.sort();
    return found;
}

/// The bytes of one stored file of a corpus input.
fn stored(
    name: &str,
    extension: &str,
) -> Vec<u8> {
    return std::fs::read(corpus().join(format!("{name}.{extension}")))
        .expect("a stored corpus file");
}

/// The merge of one corpus input.
fn merged(name: &str) -> WordMerge {
    return merge_by_word(
        stored(name, "base").as_slice(),
        stored(name, "ours").as_slice(),
        stored(name, "theirs").as_slice(),
    );
}

/// The names of the corpus inputs,
/// in order.
fn names() -> Vec<String> {
    let mut found: Vec<String> = std::fs::read_dir(corpus())
        .expect("the corpus directory")
        .filter_map(|entry| {
            return entry
                .ok()?
                .file_name()
                .to_str()?
                .strip_suffix(".base")
                .map(str::to_owned);
        })
        .collect();
    found.sort();
    return found;
}

/// Jujutsu's verdict for an input:
/// whether its word-level merge resolved.
fn reference_resolved(name: &str) -> bool {
    return stored(name, "word.status").starts_with(b"resolved");
}

/// Every input gets Jujutsu's verdict,
/// and every resolved output is Jujutsu's bytes.
#[test]
fn the_corpus_equals_the_stored_results_of_jujutsu() {
    let mut resolved: usize = 0;
    let mut conflicted: usize = 0;
    for name in names() {
        match (merged(name.as_str()), reference_resolved(name.as_str())) {
            (
                WordMerge::Resolved {
                    merged: bytes,
                    ..
                },
                true,
            ) => {
                assert!(
                    bytes == stored(name.as_str(), "word.out"),
                    "{name}: resolved, but not to the bytes Jujutsu wrote"
                );
                resolved += 1;
            }
            (
                WordMerge::Conflict {
                    ..
                },
                false,
            ) => conflicted += 1,
            (
                WordMerge::Resolved {
                    ..
                },
                false,
            ) => panic!("{name}: resolved, Jujutsu left a conflict"),
            (
                WordMerge::Conflict {
                    ..
                },
                true,
            ) => panic!("{name}: left a conflict, Jujutsu resolved it"),
        }
    }
    // Positive controls: the corpus holds both verdicts, so an implementation that always says one of them fails here.
    assert!(resolved > 0, "no input resolves");
    assert!(conflicted > 0, "no input conflicts");
}

/// A resolved output of a real file equals the resolution the owner committed.
#[test]
fn a_resolved_real_file_equals_the_committed_resolution() {
    let mut compared: usize = 0;
    for name in names().into_iter().filter(|name| {
        return corpus().join(format!("{name}.resolved")).exists();
    }) {
        if let WordMerge::Resolved {
            merged: bytes,
            ..
        } = merged(name.as_str())
        {
            assert!(
                bytes == stored(name.as_str(), "resolved"),
                "{name}: not the committed blob"
            );
            compared += 1;
        }
    }
    assert!(compared > 0, "no real file resolved");
}

/// The README conflict of merge `14601b28d3` resolves to the blob that merge committed,
/// and the part is reported.
#[test]
fn the_readme_conflict_resolves_to_the_committed_blob() {
    let WordMerge::Resolved {
        merged: bytes,
        parts,
    } = merged("real_m1_readme")
    else {
        panic!("the README conflict stays a conflict")
    };
    assert!(bytes == stored("real_m1_readme", "resolved"));
    assert!(!parts.is_empty(), "a part merged by word is reported");
    for part in &parts {
        assert!(!part.merged.is_empty());
        assert!(
            part.ours.first > 0 && part.theirs.first > 0 && part.base.first > 0,
            "line numbers count from one"
        );
    }
}

/// Both sides appending different text at one point stays a conflict:
/// the planning-document hunk of merge `78928da1f1`
///  (the headings of the two "Round 3" sections collide,
/// so a clean result would be wrong),
/// and the small section append.
#[test]
fn both_append_stays_a_conflict() {
    for name in ["real_m2_planning", "md2"] {
        assert!(matches!(merged(name), WordMerge::Conflict { .. }), "{name}");
    }
}

/// Two different edits of the same word stay a conflict.
#[test]
fn a_true_overlap_stays_a_conflict() {
    assert!(matches!(merged("md3"), WordMerge::Conflict { unresolved } if unresolved > 0));
    let outcome: WordMerge = merge_by_word(b"scale(1, 2)\n", b"scale(7, 2)\n", b"scale(8, 2)\n");
    assert!(matches!(outcome, WordMerge::Conflict { .. }));
}

/// Every stored refusal input keeps its conflict,
/// and the text a carried layer keeps holds both sides and no doubled word.
/// The real case is merge `ef43afe9f9`,
/// whose word-level pass shipped
/// `return return without_variable == *selected;`
/// in `package/git-policy/cli/src/native/policy/policy_forbidden_strings.rs`.
#[test]
fn the_stored_refusal_inputs_stay_conflicts() {
    let names: Vec<String> = refusal_names();
    assert!(
        names.len() >= 2,
        "the stored refusal inputs are gone: {names:?}"
    );
    for name in names {
        let base: Vec<u8> = refused(name.as_str(), "base");
        let ours: Vec<u8> = refused(name.as_str(), "ours");
        let theirs: Vec<u8> = refused(name.as_str(), "theirs");
        let outcome: WordMerge = merge_by_word(base.as_slice(), ours.as_slice(), theirs.as_slice());
        assert!(
            matches!(outcome, WordMerge::Conflict { .. }),
            "{name}: resolved, and the word pass writes text no side wrote for this input"
        );
        let kept: Vec<u8> = keep_both(
            base.as_slice(),
            ours.as_slice(),
            theirs.as_slice(),
            ("ours", "theirs"),
        );
        let text: String = String::from_utf8_lossy(kept.as_slice()).into_owned();
        assert!(
            text.contains("<<<<<<< ours\n"),
            "{name}: the file a carried layer keeps holds both sides"
        );
    }
}

/// The doubled `return` of merge `ef43afe9f9` is the word the two sides inserted at one place,
/// and the merge of it is a conflict rather than the line that shipped.
#[test]
fn a_word_no_side_doubles_is_never_written_twice() {
    let base: &[u8] =
        b"    return configured.is_ok_and(|without_variable| without_variable == *selected);\n";
    let ours: &[u8] = b"    let Ok(without_variable) = configured else {\n        return false;\n    };\n    return without_variable == *selected;\n";
    let theirs: &[u8] = b"    return configured.is_ok_and(|without_variable| return without_variable == *selected);\n";
    let outcome: WordMerge = merge_by_word(base, ours, theirs);
    let text: String = match &outcome {
        WordMerge::Conflict {
            ..
        } => String::from("a conflict"),
        WordMerge::Resolved {
            merged: bytes,
            ..
        } => String::from_utf8_lossy(bytes.as_slice()).into_owned(),
    };
    assert!(
        matches!(outcome, WordMerge::Conflict { .. }),
        "the merge wrote {text}"
    );
    assert!(
        !text.contains("return return"),
        "the merge wrote a doubled return: {text}"
    );
}

/// A word a side doubles is that side's own text,
/// so a region that keeps it stays resolved.
#[test]
fn a_word_a_side_doubles_is_not_refused() {
    let outcome: WordMerge = merge_by_word(
        b"the very old value\n",
        b"the very very old value\n",
        b"the very old value here\n",
    );
    let WordMerge::Resolved {
        merged: bytes,
        parts,
    } = outcome
    else {
        panic!("a doubling one side wrote itself is its own text")
    };
    let text: String = String::from_utf8_lossy(bytes.as_slice()).into_owned();
    assert!(text.contains("very very"), "{text}");
    assert!(text.contains("here"), "{text}");
    assert_eq!(parts.len(), 1);
}

/// A region that spans lines keeps both sides when the merge can only write a line no side holds,
/// and a region whose sentences each come from one side still resolves.
/// The first is what merge `31f72ecf12` left in `doc/handover/questions-for-the-owner.md`:
/// one side joined two clauses and the other side added a word,
/// and the word pass wrote the clauses as one line neither side holds.
#[test]
fn a_line_no_side_holds_is_refused_across_lines() {
    let joined: WordMerge = merge_by_word(b"alpha\nbeta\n", b"alpha beta\n", b"alpha\nbeta two\n");
    let text: String = match &joined {
        WordMerge::Conflict {
            ..
        } => String::from("a conflict"),
        WordMerge::Resolved {
            merged: bytes,
            ..
        } => String::from_utf8_lossy(bytes.as_slice()).into_owned(),
    };
    assert!(
        matches!(joined, WordMerge::Conflict { .. }),
        "the merge wrote a line no side holds: {text}"
    );
    let separate: WordMerge = merge_by_word(
        b"alpha one\nbeta two\n",
        b"alpha uno\nbeta two\n",
        b"alpha one\nbeta zwei\n",
    );
    let WordMerge::Resolved {
        merged: bytes,
        ..
    } = separate
    else {
        panic!("two edits of two lines resolve")
    };
    assert_eq!(
        String::from_utf8_lossy(bytes.as_slice()),
        "alpha uno\nbeta zwei\n"
    );
}

/// The first lines of the text of the next two controls:
/// one region where the sides edit different words of a line.
const RESOLVABLE_BASE: &str = "alpha one\nkeep\nkeep again\nbeta two\n";

/// The first side edits the second word of the first line.
const RESOLVABLE_OURS: &str = "alpha uno\nkeep\nkeep again\nbeta two\n";

/// The second side edits the first word of the first line.
const RESOLVABLE_THEIRS: &str = "ALPHA one\nkeep\nkeep again\nbeta two\n";

/// The merge of the resolvable region reports exactly it,
/// with its lines in every text.
#[test]
fn a_region_of_different_words_resolves_and_is_reported_with_its_lines() {
    let WordMerge::Resolved {
        merged: bytes,
        parts,
    } = merge_by_word(
        RESOLVABLE_BASE.as_bytes(),
        RESOLVABLE_OURS.as_bytes(),
        RESOLVABLE_THEIRS.as_bytes(),
    )
    else {
        panic!("edits of different words of one line stay a conflict")
    };
    assert_eq!(
        String::from_utf8(bytes).expect("UTF-8"),
        "ALPHA uno\nkeep\nkeep again\nbeta two\n"
    );
    let span = LineSpan {
        first: 1,
        count: 1,
    };
    assert_eq!(
        parts,
        vec![MergedPart {
            base: span,
            ours: span,
            theirs: span,
            merged: b"ALPHA uno\n".to_vec()
        }]
    );
}

/// One region that cannot resolve keeps the whole file in conflict,
/// although another region resolves by word.
#[test]
fn a_file_with_one_unresolvable_region_stays_whole_in_conflict() {
    let ours: String = RESOLVABLE_OURS.replace("beta two", "beta 2");
    let theirs: String = RESOLVABLE_THEIRS.replace("beta two", "beta zwei");
    let outcome: WordMerge = merge_by_word(
        RESOLVABLE_BASE.as_bytes(),
        ours.as_bytes(),
        theirs.as_bytes(),
    );
    assert_eq!(
        outcome,
        WordMerge::Conflict {
            unresolved: 1
        }
    );
}

/// The seed of the generated edits,
/// so a failure repeats.
const SEED: u64 = 20_261_103;

/// How many generated pairs of edits the property runs.
const GENERATED_PAIRS: usize = 400;

/// How many lines the generated base has.
const GENERATED_LINES: usize = 12;

/// The multiplier and increment of the linear congruential generator (Numerical Recipes).
const MULTIPLIER: u64 = 1_664_525;
const INCREMENT: u64 = 1_013_904_223;

/// The next number below `bound`.
fn next(
    state: &mut u64,
    bound: usize,
) -> usize {
    *state = (state.wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT)) & u64::from(u32::MAX);
    return usize::try_from(*state >> 8).expect("fits") % bound;
}

/// The generated base:
/// numbered lines of words.
fn generated_base() -> Vec<String> {
    return (0..GENERATED_LINES)
        .map(|line| {
            return format!("word{line} alpha{line} beta{line}");
        })
        .collect();
}

/// A side:
/// some lines get a word inserted at a random place,
/// tagged with the side so it can be found again.
fn with_insertions(
    base: &[String],
    tag: &str,
    state: &mut u64,
) -> (Vec<String>, Vec<String>) {
    let mut lines: Vec<String> = base.to_vec();
    let mut inserted: Vec<String> = Vec::new();
    for _ in 0..=next(state, 3) {
        let line: usize = next(state, lines.len());
        let mut words: Vec<String> = lines[line].split(' ').map(str::to_owned).collect();
        let marker: String = format!("{tag}{}", inserted.len());
        words.insert(next(state, words.len() + 1), marker.clone());
        lines[line] = words.join(" ");
        inserted.push(marker);
    }
    return (lines, inserted);
}

/// Whatever resolves keeps every word either side inserted and every word of the base,
/// and some pairs resolve while others do not.
#[test]
fn nothing_either_side_wrote_is_dropped() {
    let mut state: u64 = SEED;
    let base: Vec<String> = generated_base();
    let base_text: String = base.join("\n") + "\n";
    let (mut resolved, mut conflicted) = (0_usize, 0_usize);
    for _ in 0..GENERATED_PAIRS {
        let (ours, ours_words) = with_insertions(&base, "ours", &mut state);
        let (theirs, theirs_words) = with_insertions(&base, "theirs", &mut state);
        let outcome: WordMerge = merge_by_word(
            base_text.as_bytes(),
            (ours.join("\n") + "\n").as_bytes(),
            (theirs.join("\n") + "\n").as_bytes(),
        );
        let WordMerge::Resolved {
            merged: bytes,
            ..
        } = outcome
        else {
            conflicted += 1;
            continue;
        };
        resolved += 1;
        let text: String = String::from_utf8(bytes).expect("UTF-8");
        for word in ours_words.iter().chain(theirs_words.iter()).chain(
            base.iter()
                .flat_map(|line| {
                    return line.split(' ');
                })
                .map(str::to_owned)
                .collect::<Vec<String>>()
                .iter(),
        ) {
            assert!(
                text.contains(word.as_str()),
                "{word} was dropped from {text}"
            );
        }
    }
    assert!(
        resolved > 0 && conflicted > 0,
        "the generated pairs must exercise both verdicts: {resolved} resolved, {conflicted} conflicted"
    );
}

/// The records of a file that holds one record per line,
/// as the two sides of issue 6529's merge delete them:
/// this side deletes the `beta` records,
/// the other side the `gamma` records that stand right after them.
const DELETED_RECORDS_BASE: &str =
    "alpha one\nalpha two\nbeta one\nbeta two\ngamma one\ngamma two\ndelta one\ndelta two\n";

/// The first side's file: the `beta` records are gone.
const DELETED_RECORDS_OURS: &str =
    "alpha one\nalpha two\ngamma one\ngamma two\ndelta one\ndelta two\n";

/// The second side's file: the `gamma` records are gone.
const DELETED_RECORDS_THEIRS: &str =
    "alpha one\nalpha two\nbeta one\nbeta two\ndelta one\ndelta two\n";

/// Two sides that each delete different adjacent records of a file that holds one record per line
/// resolve to the union of the deletions,
/// and the region is reported as one part.
/// The shape is the one merge `b928c5fa6999` of this repository's history left wrong:
/// its word pass kept the `direct_fix_install` records `main` had deleted
/// (issue 6529,
/// corrected by hand in `24f8c1947b`).
#[test]
fn two_sides_deleting_adjacent_records_resolve_to_the_union() {
    let outcome: WordMerge = merge_by_word(
        DELETED_RECORDS_BASE.as_bytes(),
        DELETED_RECORDS_OURS.as_bytes(),
        DELETED_RECORDS_THEIRS.as_bytes(),
    );
    let WordMerge::Resolved {
        merged: bytes,
        parts,
    } = outcome
    else {
        panic!("two sides that only delete records resolve")
    };
    assert_eq!(
        String::from_utf8(bytes).expect("UTF-8"),
        "alpha one\nalpha two\ndelta one\ndelta two\n"
    );
    assert_eq!(
        parts.len(),
        1,
        "the region of deletions is reported as one part"
    );
    assert!(
        parts[0].merged.is_empty(),
        "the part's own text is what the union of the deletions left: nothing"
    );
}

/// The smallest region of the shape:
/// each side deletes the one record the other side keeps,
/// so the union of the deletions is the whole region.
#[test]
fn one_record_deleted_by_each_side_leaves_nothing() {
    let outcome: WordMerge =
        merge_by_word(b"alpha\nbeta\ngamma\n", b"alpha\ngamma\n", b"alpha\nbeta\n");
    let WordMerge::Resolved {
        merged: bytes,
        ..
    } = outcome
    else {
        panic!("one deletion from each side resolves")
    };
    assert_eq!(String::from_utf8(bytes).expect("UTF-8"), "alpha\n");
}

/// The smaller region of the same shape:
/// both sides insert the same record at the top,
/// and each deletes the record the other side keeps,
/// so the word pass would pair the inserted record with a kept one and write the kept one back.
/// It is refused instead,
/// so the person resolves the region.
#[test]
fn a_region_with_the_same_insertion_on_both_sides_keeps_a_deleted_record_out() {
    let outcome: WordMerge = merge_by_word(
        b"alpha one\nbeta two\n",
        b"zed wild\nalpha one\n",
        b"zed wild\nbeta two\n",
    );
    assert!(
        matches!(outcome, WordMerge::Conflict { .. }),
        "the region keeps both sides: {outcome:?}"
    );
}

/// A resolved file never writes back a whole line the base held that one side deleted and the other
/// side kept,
/// which is the failure issue 6529 reports.
#[test]
fn no_record_one_side_deleted_is_written_back() {
    let WordMerge::Resolved {
        merged: bytes,
        ..
    } = merge_by_word(
        DELETED_RECORDS_BASE.as_bytes(),
        DELETED_RECORDS_OURS.as_bytes(),
        DELETED_RECORDS_THEIRS.as_bytes(),
    )
    else {
        panic!("two sides that only delete records resolve")
    };
    let text: String = String::from_utf8(bytes).expect("UTF-8");
    for line in DELETED_RECORDS_BASE.lines() {
        if DELETED_RECORDS_OURS.lines().any(|kept| return kept == line)
            && DELETED_RECORDS_THEIRS
                .lines()
                .any(|kept| return kept == line)
        {
            continue;
        }
        assert!(!text.contains(line), "{line} came back: {text}");
    }
}

/// The records a generated region is built from:
/// one record per line,
/// every record different from the others.
const GENERATED_RECORDS: [&str; 4] = ["alpha one\n", "beta two\n", "gamma three\n", "delta four\n"];

/// The records a generated side may insert,
/// which no record of the base holds.
const GENERATED_INSERTIONS: [&str; 2] = ["zed wild\n", "zed tame\n"];

/// What:
///  Every text a generated side of one base run can hold:
/// any run of the base's records,
/// any of those with one inserted record at any place,
/// and any of those with one record's last word replaced.
/// Why:
///   A region of one-record-per-line files is the whole of the space this rule governs:
/// sides that delete records,
/// sides that also write a record,
/// and sides that edit one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const variants = (base: string[]): string[] => /* every run, each with one insertion and each with one edit */ [];
/// ```
fn record_variants(base: &[&str]) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for mask in 0usize..(1usize << base.len()) {
        let mut kept: Vec<&str> = Vec::new();
        for (index, record) in base.iter().enumerate() {
            if mask & (1usize << index) == 0 {
                continue;
            }
            kept.push(*record);
        }
        found.push(kept.concat());
        for at in 0..=kept.len() {
            for inserted in GENERATED_INSERTIONS {
                let mut grown: Vec<&str> = kept.clone();
                grown.insert(at, inserted);
                found.push(grown.concat());
            }
        }
        for at in 0..kept.len() {
            let mut edited: Vec<String> = kept
                .iter()
                .map(|record| return (*record).to_owned())
                .collect();
            let record: &String = &edited[at];
            let head: &str = record.rsplit_once(' ').expect("a record of two words").0;
            edited[at] = format!("{head} zed\n");
            found.push(edited.concat());
        }
    }
    return found;
}

/// What:
///  A region that resolves never writes back a record the base held that one side deleted,
/// over every generated region of one record per line.
/// Why:
///   That is the failure issue 6529 reports:
/// a merge finished by itself with a record neither side kept,
/// here the records a side deleted while the other side left them.
/// A record of the output that one side's file does not hold at all came back from the base,
///  so it is text the merge had to invent.
/// The generator's sides delete records,
/// insert records and edit one record,
/// so a rule that only covers deletions meets the rest here too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const base of runs) for (const ours of variants(base)) for (const theirs of variants(base)) {
///   const merged = mergeByWord(base, ours, theirs);
///   if (merged.kind !== 'resolved') continue;
///   for (const record of lines(base)) expect(merged.bytes).not.toContainDeletedOnly(record, ours, theirs);
/// }
/// ```
#[test]
fn no_generated_region_writes_back_a_deleted_record() {
    let mut resolved: usize = 0;
    let mut conflicted: usize = 0;
    for size in [2_usize, 3, 4] {
        for start in 0..=(GENERATED_RECORDS.len() - size) {
            let base: &[&str] = &GENERATED_RECORDS[start..start + size];
            let base_text: String = base.concat();
            let variants: Vec<String> = record_variants(base);
            for ours in &variants {
                for theirs in &variants {
                    let WordMerge::Resolved {
                        merged: bytes,
                        ..
                    } = merge_by_word(base_text.as_bytes(), ours.as_bytes(), theirs.as_bytes())
                    else {
                        conflicted += 1;
                        continue;
                    };
                    resolved += 1;
                    let text: String = String::from_utf8_lossy(bytes.as_slice()).into_owned();
                    for record in &base_text.lines().collect::<Vec<&str>>() {
                        if !text.lines().any(|written| return written == *record) {
                            continue;
                        }
                        if ours.lines().any(|held| return held == *record)
                            && theirs.lines().any(|held| return held == *record)
                        {
                            continue;
                        }
                        panic!(
                            "the record {record:?} came back from base {base_text:?}, ours {ours:?}, theirs {theirs:?} as {text:?}"
                        );
                    }
                }
            }
        }
    }
    assert!(
        resolved > 0 && conflicted > 0,
        "the generated regions exercise both verdicts: {resolved} resolved, {conflicted} conflicted"
    );
}
