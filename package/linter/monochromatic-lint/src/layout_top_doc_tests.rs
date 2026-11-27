//! What:
//!  Controls of the top doc reader.
//! Why:
//!  The file rule trusts this reader to find the list and nothing else.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! test('reads entries after the marker', () => expect(readTopDoc(text).entries.map(e => e.name)).toEqual([...]));
//! ```

use super::{
    LIST_MARKER,
    counted_work,
    read_top_doc,
};

/// Entries after the marker are read in order with their lines,
/// and free text before the marker and after the list is ignored.
#[test]
fn entries_after_the_marker_are_read_in_order() {
    let text: String = format!(
        "//! What: this folder saves files.\n//!\n//! {LIST_MARKER}\n//!\n//! - `first.rs`: writes a file.\n//!   More words about it.\n//! - `second.rs`: saves two.\n//!\n//! Closing words that end the list.\n//! - `ignored.rs`: after the list.\n\nmod first;\n"
    );
    let doc = read_top_doc(text.as_str());
    assert_eq!(doc.marker_line, Some(3));
    let seen: Vec<(&str, usize)> = doc
        .entries
        .iter()
        .map(|entry| return (entry.name.as_str(), entry.line))
        .collect::<Vec<(&str, usize)>>();
    assert_eq!(seen, [("first.rs", 5), ("second.rs", 7)]);
    assert_eq!(doc.first_line, 1);
}

/// A doc without the marker has no list even when it has bullet lines.
#[test]
fn bullets_without_the_marker_are_no_list() {
    let doc = read_top_doc("//! Words.\n//! - `first.rs`: something.\n");
    assert_eq!(doc.marker_line, None);
    assert!(doc.entries.is_empty());
    assert_eq!(doc.first_line, 1);
}

/// Only the leading doc lines count:
/// comments and inner attributes do not end the doc,
/// code does.
#[test]
fn only_the_leading_doc_block_counts() {
    let text: String = format!(
        "// plain\n#![allow(dead_code)]\n//! {LIST_MARKER}\n//! - `a.rs`: x\n\nfn code() {{}}\n//! {LIST_MARKER}\n//! - `late.rs`: y\n"
    );
    let doc = read_top_doc(text.as_str());
    assert_eq!(doc.marker_line, Some(3));
    assert_eq!(doc.entries.len(), 1);
    assert_eq!(doc.first_line, 3);
}

/// A file with no doc reports the first line,
/// and an entry line without backticks ends the list.
#[test]
fn no_doc_and_unquoted_entries() {
    assert_eq!(read_top_doc("fn code() {}\n").first_line, 1);
    let text: String = format!(
        "//! {LIST_MARKER}\n//! - `a.rs`: x\n//! - b.rs: unquoted\n//! - `c.rs`: not read\n"
    );
    let doc = read_top_doc(text.as_str());
    assert_eq!(doc.entries.len(), 1);
}

/// The marker may break after its comma,
/// as the Markdown rule demands,
/// and the line of the marker is the line where it starts.
#[test]
fn a_marker_wrapped_after_its_comma_is_found() {
    let text: String = String::from(
        "//! Files in this folder,\n//! each using only files listed before it:\n//!\n//! - `first.rs`: writes a file.\n",
    );
    let doc = read_top_doc(text.as_str());
    assert_eq!(doc.marker_line, Some(1));
    let seen: Vec<(&str, usize)> = doc
        .entries
        .iter()
        .map(|entry| return (entry.name.as_str(), entry.line))
        .collect::<Vec<(&str, usize)>>();
    assert_eq!(seen, [("first.rs", 4)]);
}

/// The marker may break wherever a sentence may,
/// including more than once and in the middle of a phrase.
#[test]
fn a_marker_wrapped_at_other_words_is_found() {
    let text: String = String::from(
        "//! Files in this\n//! folder, each using\n//! only files listed before it:\n//! - `first.rs`: writes a file.\n",
    );
    let doc = read_top_doc(text.as_str());
    assert_eq!(doc.marker_line, Some(1));
    assert_eq!(doc.entries.len(), 1);
    assert_eq!(doc.entries[0].line, 4);
}

/// The marker on one line still works,
/// with the entries following it directly.
#[test]
fn a_marker_on_one_line_still_works() {
    let text: String = format!("//! {LIST_MARKER}\n//! - `first.rs`: writes a file.\n");
    let doc = read_top_doc(text.as_str());
    assert_eq!(doc.marker_line, Some(1));
    assert_eq!(doc.entries.len(), 1);
    assert_eq!(doc.entries[0].line, 2);
}

/// An entry whose words wrap onto indented lines is one entry,
/// and the next entry still starts at its own `- ` line.
#[test]
fn entries_whose_words_wrap_onto_indented_lines_stay_one_entry() {
    let text: String = String::from(
        "//! Files in this folder,\n//! each using only files listed before it:\n//!\n//! - `first.rs`:\n//!   writes a file so that a crash\n//!   never leaves half of it.\n//! - `second.rs`:\n//!   saves several files,\n//!   using `first.rs` for each one.\n",
    );
    let doc = read_top_doc(text.as_str());
    let seen: Vec<(&str, usize)> = doc
        .entries
        .iter()
        .map(|entry| return (entry.name.as_str(), entry.line))
        .collect::<Vec<(&str, usize)>>();
    assert_eq!(seen, [("first.rs", 4), ("second.rs", 7)]);
}

/// Entries may follow the wrapped marker with no blank line between,
/// and a backticked name inside a continuation line is not an entry.
#[test]
fn an_entry_list_right_after_the_wrapped_marker_is_read() {
    let text: String = String::from(
        "//! Files in this folder,\n//! each using only files listed before it:\n//! - `first.rs`: uses\n//!   `not_an_entry.rs`.\n//! - `second.rs`: saves two.\n",
    );
    let doc = read_top_doc(text.as_str());
    assert_eq!(doc.marker_line, Some(1));
    let seen: Vec<&str> = doc
        .entries
        .iter()
        .map(|entry| return entry.name.as_str())
        .collect::<Vec<&str>>();
    assert_eq!(seen, ["first.rs", "second.rs"]);
}

/// A blank `//!` line ends a paragraph,
/// so a marker split by one is two unrelated halves and the doc has no list.
/// The words of those two halves are the marker's words,
/// so the doc also reports one near miss that names both lines.
#[test]
fn a_blank_doc_line_between_the_halves_is_no_list() {
    let text: String = String::from(
        "//! Files in this folder,\n//!\n//! each using only files listed before it:\n//! - `first.rs`: writes a file.\n",
    );
    let doc = read_top_doc(text.as_str());
    assert_eq!(doc.marker_line, None);
    assert!(doc.entries.is_empty());
    assert_eq!(doc.first_line, 1);
    assert_eq!(doc.near_misses.len(), 1, "{:?}", doc.near_misses);
    assert_eq!(doc.near_misses[0].line, 1);
    assert_eq!(doc.near_misses[0].continues_at, Some(3));
    assert_eq!(
        doc.near_misses[0].text,
        "Files in this folder, each using only files listed before it:"
    );
}

/// A plain comment between the halves ends the paragraph like a blank line does,
/// so it does not join them either,
/// and the two halves are a near miss named by both of their lines.
#[test]
fn a_plain_comment_between_the_halves_is_no_list() {
    let text: String = String::from(
        "//! Files in this folder,\n// a plain comment\n//! each using only files listed before it:\n//! - `first.rs`: writes a file.\n",
    );
    let doc = read_top_doc(text.as_str());
    assert_eq!(doc.marker_line, None);
    assert!(doc.entries.is_empty());
    assert_eq!(doc.near_misses.len(), 1, "{:?}", doc.near_misses);
    assert_eq!(doc.near_misses[0].line, 1);
    assert_eq!(doc.near_misses[0].continues_at, Some(3));
}

/// Text before the marker may wrap too,
/// whether it is a paragraph of its own
/// or the lines right above the marker with no blank line between.
#[test]
fn doc_text_before_the_wrapped_marker_may_wrap() {
    let apart: String = String::from(
        "//! What this folder is for,\n//! in words for someone\n//! who knows only `git commit`.\n//!\n//! Files in this folder,\n//! each using only files listed before it:\n//! - `first.rs`: writes a file.\n",
    );
    let apart_doc = read_top_doc(apart.as_str());
    assert_eq!(apart_doc.marker_line, Some(5));
    assert_eq!(apart_doc.entries.len(), 1);
    let touching: String = String::from(
        "//! What this folder is for,\n//! in words for someone.\n//! Files in this folder,\n//! each using only files listed before it:\n//! - `first.rs`: writes a file.\n",
    );
    let touching_doc = read_top_doc(touching.as_str());
    assert_eq!(touching_doc.marker_line, Some(3));
    assert_eq!(touching_doc.entries.len(), 1);
}

/// Extra spaces at a join or inside a line count as one space,
/// because the Markdown reader collapses them too.
#[test]
fn extra_spaces_at_a_join_count_as_one_space() {
    let text: String = String::from(
        "//!  Files   in this folder,   \n//!     each using  only files listed before it:  \n//! - `first.rs`: writes a file.\n",
    );
    let doc = read_top_doc(text.as_str());
    assert_eq!(doc.marker_line, Some(1));
    assert_eq!(doc.entries.len(), 1);
}

/// Words that differ from the marker,
/// or follow it on the same line,
/// are no marker.
/// One word put in place of another is a near miss;
/// two words changed, added or dropped are a different sentence and no near miss.
#[test]
fn a_changed_or_extended_marker_is_no_list() {
    let changed: String = String::from(
        "//! Files in this folder,\n//! each using only files listed after it:\n//! - `first.rs`: writes a file.\n",
    );
    assert_eq!(read_top_doc(changed.as_str()).marker_line, None);
    assert_eq!(read_top_doc(changed.as_str()).near_misses.len(), 1);
    let extended: String =
        format!("//! {LIST_MARKER} and more\n//! - `first.rs`: writes a file.\n");
    assert_eq!(read_top_doc(extended.as_str()).marker_line, None);
    assert_eq!(read_top_doc(extended.as_str()).near_misses.len(), 0);
    let cut: String = String::from("//! Files in this folder,\n//! - `first.rs`: writes a file.\n");
    assert_eq!(read_top_doc(cut.as_str()).marker_line, None);
    assert_eq!(read_top_doc(cut.as_str()).near_misses.len(), 0);
}

/// A marker with one word added, dropped or put in place of another is a near miss,
///  and each way of missing it names the words the doc holds and the line they start on.
#[test]
fn a_marker_one_word_off_is_a_near_miss() {
    for (text, expected) in [
        (
            "//! Files in this folder,\n//! each using only the files listed before it:\n",
            "Files in this folder, each using only the files listed before it:",
        ),
        (
            "//! Files in this folder,\n//! each using only files before it:\n",
            "Files in this folder, each using only files before it:",
        ),
        (
            "//! Files in this folder,\n//! each using only files listed after it:\n",
            "Files in this folder, each using only files listed after it:",
        ),
        (
            format!("//! Words about the folder.\n//!\n//! {LIST_MARKER} files\n").as_str(),
            "Files in this folder, each using only files listed before it: files",
        ),
    ] {
        let doc = read_top_doc(text);
        assert_eq!(doc.marker_line, None, "{text}");
        assert_eq!(doc.near_misses.len(), 1, "{text}: {:?}", doc.near_misses);
        let near = &doc.near_misses[0];
        assert_eq!(near.text, expected, "{text}");
        assert_eq!(near.line, if text.starts_with("//! Words") { 3 } else { 1 });
        assert_eq!(near.continues_at, None, "{text}");
    }
}

/// The same words in another case are a near miss,
///  because a reader sees one sentence and the marker reader sees no marker.
#[test]
fn a_marker_in_a_different_case_is_a_near_miss() {
    let doc = read_top_doc("//! files in this folder, Each using only files listed before it:\n");
    assert_eq!(doc.marker_line, None);
    assert_eq!(doc.near_misses.len(), 1, "{:?}", doc.near_misses);
    assert_eq!(
        doc.near_misses[0].text,
        "files in this folder, Each using only files listed before it:"
    );
}

/// A text two words away from the marker is a different sentence,
///  so no finding asks its author to change it,
///  and that holds at the marker's own length too,
///  where one word put in place of another would still be a near miss.
#[test]
fn a_marker_two_words_off_is_no_near_miss() {
    for text in [
        "//! Files in this module,\n//! each using only the files listed before it:\n",
        "//! Files in this folder,\n//! each using only files listed before it: and more\n",
        // What: the marker's own length with two of its words put in place of two others, `after one` for `before it`.
        // Why: an equal-length text one word away is a near miss, so the length alone must not decide this one.
        "//! Files in this folder,\n//! each using only files listed after one:\n",
        "//! Words about the folder.\n",
    ] {
        let doc = read_top_doc(text);
        assert_eq!(doc.marker_line, None, "{text}");
        assert_eq!(doc.near_misses.len(), 0, "{text}: {:?}", doc.near_misses);
    }
}

/// A doc that holds the marker reads its list,
///  so a paragraph beside it that is nearly the marker is no finding at all.
#[test]
fn a_doc_with_the_marker_reports_no_near_miss() {
    let text: String = String::from(
        "//! Files in this folder, each using only the files listed before it:\n//!\n//! Files in this folder,\n//! each using only files listed before it:\n//! - `first.rs`: writes a file.\n",
    );
    let doc = read_top_doc(text.as_str());
    assert_eq!(doc.marker_line, Some(3));
    assert_eq!(doc.entries.len(), 1);
    assert_eq!(doc.near_misses.len(), 0, "{:?}", doc.near_misses);
}

/// One doc that nearly writes the marker twice reports both places,
///  in the order the doc writes them,
///  so each has its own line and its own key.
#[test]
fn every_near_miss_of_one_doc_is_reported_in_document_order() {
    let text: String = String::from(
        "//! Files in this folder, each using only the files listed before it:\n//!\n//! Files in this folder, each using only files listed after it:\n//! - `first.rs`: writes a file.\n",
    );
    let doc = read_top_doc(text.as_str());
    let lines: Vec<usize> = doc
        .near_misses
        .iter()
        .map(|near| return near.line)
        .collect::<Vec<usize>>();
    assert_eq!(lines, [1, 3], "{:?}", doc.near_misses);
}

/// The doc lines the near-miss search read for a doc of one head line,
///  `lines` filler lines and a `mod` line, which holds no marker and no near miss.
fn reads_of(
    head: &str,
    filler: &str,
    lines: usize,
) -> usize {
    let text: String = format!("{head}{}mod one;\n", filler.repeat(lines));
    counted_work::reset();
    let doc = read_top_doc(text.as_str());
    assert_eq!(doc.marker_line, None, "the fixture doc holds no marker");
    assert_eq!(
        doc.near_misses.len(),
        0,
        "the fixture doc is no near miss: {:?}",
        doc.near_misses
    );
    return counted_work::lines_read();
}

/// The near-miss search reads each doc line a bounded number of times,
///  so a run of blank, plain-comment or one-word doc lines costs its own length
///  and never the square of it.
/// Why this is a cost the rule must keep:
///  a top doc that holds a long licence block, a generated table or a commented-out block
///  after its first doc line made the read of one file cost the run a whole CI minute.
#[test]
fn the_near_miss_search_reads_each_doc_line_a_bounded_number_of_times() {
    // What: the most doc lines the search may read for each line the doc holds, and the two run lengths each filler is tried at, the second twice the first.
    // Why: the blank and plain-comment fillers are tried in the thousands because that is the shape the read measured,
    // and the one-word doc filler in the hundreds because an edit that keeps reading it squares the read of every line;
    // 64 sits above the 30 a line of one-word doc lines costs (measured 2026-11-27 in this control), a run of blank lines costing 3;
    // a doc that doubles its length may at most triple what the search reads of it, and a square quadruples it.
    const READS_PER_LINE: usize = 64;
    const DOUBLING_MAY_MULTIPLY: usize = 3;
    let arms: [(&str, &str, [usize; 2]); 3] = [
        ("blank lines", "\n", [4000, 8000]),
        ("plain comment lines", "//\n", [4000, 8000]),
        ("one-word doc lines", "//! a\n", [400, 800]),
    ];
    for (what, filler, [small, large]) in arms {
        // What: what the search read of a doc of each of the two runs.
        // Why: one figure alone cannot tell a read that grows with the doc from one that grows with its square.
        let small_read: usize = reads_of("//! Notes for this folder.\n", filler, small);
        let large_read: usize = reads_of("//! Notes for this folder.\n", filler, large);
        assert!(
            large_read <= small_read * DOUBLING_MAY_MULTIPLY,
            "{what}: {small} lines were read in {small_read} but {large} in {large_read}, which is not a linear read"
        );
        assert!(
            large_read <= READS_PER_LINE * large,
            "{what}: {large_read} lines read of {large} is more than {READS_PER_LINE} for each line"
        );
    }
}
