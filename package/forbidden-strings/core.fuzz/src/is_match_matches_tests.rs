// What:  unit tests for the `RegexSet` invariant the `fuzz_is_match_matches` target
//        asserts: the boolean face (`is_match`, `is_match_batch`) and the attribution
//        face (`matches`, `line_matches`) must answer the same question the same way.
// Why:   a fuzz target only fails when libFuzzer happens to reach a disagreement, and it
//        cannot promise that a given case reached both verdicts. The fixed table here
//        pins one line per routing path on both outcomes, and the generated sweep varies
//        the rulesets; together they prove the check can see a match and its absence.

use arbitrary::{Arbitrary, Unstructured};
use forbidden_regex::RegexSet;

use super::generators::RulesetAndBuffer;

// One seeded rule, one line-start-anchored rule, and one literal-free rule, so the fixed
// table below drives every routing path of both faces.
const RULES: &[&str] = &["AKIA[A-Z2-7]{4}", "^deny-code:", "[0-9]{15,16}"];

// How many generated cases the sweep visits. Each case compiles a ruleset of at most four
// rules and matches at most eight short lines, so this stays a fraction of a second in a
// debug build while covering the generator's bounded depth many times over.
const GENERATED_CASES: usize = 64;

// Bytes fed to the structured generator per case. One case draws a rule count, up to four
// patterns of bounded size, a line count, up to eight lines, and up to `MAX_CONTENT` bytes
// per line, so a small budget leaves most lines empty and most patterns truncated. At this
// size the sweep keeps at least one rule in about half its cases and produces matching and
// unmatched lines in quantity, which is the breadth the fixed table cannot supply.
const CASE_BYTES: usize = 1024;

// What:  one line per routing path that must match, plus lines that must match nothing.
// Why:   the two tables are the positive and negative controls for every agreement check
//        below; without them a matcher that never matched anything would pass.
fn matching_lines() -> Vec<&'static [u8]> {
    return vec![b"AKIA2345", b"deny-code:7", b"x 4000000000000000 y"]
}

fn unmatched_lines() -> Vec<&'static [u8]> {
    return vec![b"", b"all clear", b"A K I A"]
}

// What:  asserts one line's verdict through both faces, and that both equal `expected`.
// Why:   a single comparison per line keeps the failure message naming the line and both
//        faces, which is what a future regression needs to be diagnosable.
fn assert_verdict(set: &RegexSet, line: &[u8], expected: bool) {
    let boolean = set.is_match(line);
    let attributed = set.matches(line).next().is_some();
    assert_eq!(boolean, attributed, "is_match and matches disagreed on line {line:?}");
    assert_eq!(boolean, expected, "is_match({line:?}) should report {expected}");
}

// What:  one reproducible generator input: byte `offset` of case `index` is
//        `index ^ offset ^ (offset >> 8)`.
// Why:   the sweep must be identical on every run. XORing counters needs no PRNG
//        dependency, no seeded state, and no tuning constant; folding in the offset's high
//        byte keeps the stream from repeating every 256 positions, so a case that consumes
//        the whole budget still reads fresh bytes, and varying the leading bytes (which
//        decide the drawn counts) separates the cases.
fn case_bytes(index: usize) -> Vec<u8> {
    return (0..CASE_BYTES)
        .map(|offset| return (index as u8) ^ (offset as u8) ^ ((offset >> 8) as u8))
        .collect()
}

#[test]
fn boolean_face_agrees_with_attribution_face() {
    let set = RegexSet::new(RULES).expect("the control ruleset must compile");
    for line in matching_lines() {
        assert_verdict(&set, line, true);
    }
    for line in unmatched_lines() {
        assert_verdict(&set, line, false);
    }

    let mut all_lines = matching_lines();
    all_lines.extend(unmatched_lines());
    let batch = set.is_match_batch(&all_lines);
    for (index, line) in all_lines.iter().enumerate() {
        assert_eq!(batch[index], set.is_match(line), "is_match_batch disagreed with is_match on line {index}: {line:?}");
    }
}

#[test]
fn generated_rulesets_agree_on_every_line() {
    let mut matched_lines = 0usize;
    let mut unmatched_lines_seen = 0usize;
    for index in 0..GENERATED_CASES {
        let bytes = case_bytes(index);
        let mut unstructured = Unstructured::new(&bytes);
        let Ok(input) = RulesetAndBuffer::arbitrary(&mut unstructured) else {
            continue;
        };
        let (set, kept) = RegexSet::compile_lenient(&input.patterns);
        let lines = input.line_contents();
        let batch = set.is_match_batch(&lines);
        for (line_index, line) in lines.iter().enumerate() {
            let boolean = set.is_match(line);
            let attributed = set.matches(line).next().is_some();
            assert_eq!(
                boolean,
                attributed,
                "case {index} ({} of {} rules kept) disagreed on line {line_index}: is_match={boolean}, matches().next().is_some()={attributed}, patterns={:?}, line={line:?}",
                kept.len(),
                input.patterns.len(),
                input.patterns,
            );
            assert_eq!(
                batch[line_index],
                boolean,
                "case {index} batch verdict disagreed on line {line_index}: batch={}, scalar={boolean}, patterns={:?}, line={line:?}",
                batch[line_index],
                input.patterns,
            );
            if boolean {
                matched_lines += 1;
            } else {
                unmatched_lines_seen += 1;
            }
        }

        for (line_index, rule) in set.line_matches(&input.buf, &input.starts) {
            let line = lines[line_index];
            assert!(set.is_match(line), "case {index} attributed rule {rule} on line {line_index} that is_match denies: {line:?}");
            assert!(
                set.matches(line).any(|matched| return matched == rule),
                "case {index} attributed rule {rule} on line {line_index} that matches does not name: {line:?}",
            );
        }
    }

    assert!(matched_lines > 0, "the sweep saw no matching line, so it could not catch a matcher that never matches");
    assert!(
        unmatched_lines_seen > 0,
        "the sweep saw no unmatched line, so it could not catch a matcher that always matches",
    );
}
