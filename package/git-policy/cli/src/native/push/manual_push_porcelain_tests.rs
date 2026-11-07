//! Controls for the porcelain dry-run parser,
//! on output Git 2.56.0 gave.

use super::*;

/// A full object name made of one repeated digit.
fn oid(digit: char) -> String {
    return digit.to_string().repeat(40);
}

/// A new branch,
/// an update,
/// a forced update,
/// a deletion and a refusal read as the report says.
#[test]
fn every_kind_of_line_reads_as_the_report_says() {
    let (old, new) = (oid('a'), oid('b'));
    let output: String = format!(
        "To /srv/origin.git\n*\trefs/heads/topic:refs/heads/topic\t[new branch]\n\
         *\trefs/tags/v1:refs/tags/v1\t[new tag]\n \trefs/heads/main:refs/heads/main\t{old}..{new}\n\
         +\trefs/heads/x:refs/heads/x\t{new}...{old} (forced update)\n-\t:refs/heads/gone\t[deleted]\n\
         !\trefs/heads/y:refs/heads/y\t[rejected] (non-fast-forward)\nDone\n"
    );
    let blocks: Vec<PorcelainBlock> = parse_porcelain(output.as_str()).expect("parses");
    assert_eq!(blocks.len(), 1);
    let updates: &[PorcelainUpdate] = blocks[0].updates.as_slice();
    let kinds: Vec<(&str, &RemoteBefore, Option<&str>)> = updates
        .iter()
        .map(|update| {
            return (update.to.as_str(), &update.before, update.after.as_deref());
        })
        .collect();
    assert_eq!(kinds[0], ("refs/heads/topic", &RemoteBefore::Absent, None));
    assert_eq!(kinds[1], ("refs/tags/v1", &RemoteBefore::Absent, None));
    assert_eq!(kinds[2], ("refs/heads/main", &RemoteBefore::Known(old.clone()), Some(new.as_str())));
    assert_eq!(kinds[3], ("refs/heads/x", &RemoteBefore::Known(new.clone()), Some(old.as_str())));
    assert_eq!(kinds[4], ("refs/heads/gone", &RemoteBefore::Unknown, None));
    assert_eq!(updates[4].from, "", "a deletion has no local side");
    assert_eq!(kinds[5], ("refs/heads/y", &RemoteBefore::Unknown, None));
}

/// Up-to-date refs are left out,
/// and each destination gets its own block.
#[test]
fn up_to_date_refs_are_left_out_and_destinations_are_separate() {
    let output: &str = "To one\n=\trefs/heads/main:refs/heads/main\t[up to date]\nTo two\n*\ta:refs/heads/a\t[new branch]\nDone\n";
    let blocks: Vec<PorcelainBlock> = parse_porcelain(output).expect("parses");
    assert_eq!(blocks.len(), 2);
    assert!(blocks[0].updates.is_empty());
    assert_eq!(blocks[1].updates.len(), 1);
}

/// Text that is not in the format is an error naming the line,
/// never a guess.
#[test]
fn text_outside_the_format_is_refused() {
    let refusal: String = parse_porcelain("no tab here\n").expect_err("a line outside the format");
    assert!(refusal.contains("no tab here"), "the message names the line: {refusal}");
    assert!(parse_porcelain("To x\n*\tno-colon\t[new branch]\n").is_err());
    assert!(parse_porcelain("*\ta:b\t[new branch]\n").is_err(), "a line before any destination");
    assert_eq!(parse_porcelain("").expect("empty output"), Vec::new());
}

/// A source that holds a colon is split from its destination at the last colon.
#[test]
fn a_source_with_a_colon_keeps_it() {
    let output: &str = "To /srv/origin.git\n*\t:/unique message:refs/heads/new\t[new branch]\nDone\n";
    let blocks: Vec<PorcelainBlock> = parse_porcelain(output).expect("parses");
    assert_eq!((blocks[0].updates[0].from.as_str(), blocks[0].updates[0].to.as_str()), (":/unique message", "refs/heads/new"));
}

/// The line Git adds when the push would set an upstream is skipped,
/// and the real updates around it still read.
#[test]
fn the_set_upstream_line_is_skipped() {
    let output: &str = "To /srv/origin.git\n*\trefs/heads/main:refs/heads/main\t[new branch]\n\
                        Would set upstream of 'main' to 'main' of '/srv/origin.git'\nDone\n";
    let blocks: Vec<PorcelainBlock> = parse_porcelain(output).expect("parses");
    assert_eq!(blocks[0].updates.len(), 1);
    assert_eq!(blocks[0].shown, "/srv/origin.git");
}
