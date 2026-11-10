// What:  for a generated ruleset and a generated multi-line buffer, assert that the
//        boolean face of `RegexSet` (`is_match`, `is_match_batch`) agrees with its
//        attribution face (`matches`, `line_matches`) on every line.
// Why:   `fuzz_line_matches` compares two attribution paths against each other and
//        `fuzz_from_bytes` calls the boolean and attribution queries without comparing
//        their verdicts, so a defect confined to one face would go uncaught; this target
//        is the only one that pins `is_match(line) == matches(line).next().is_some()`
//        (TCV). Its fixed positive and negative controls live in
//        `src/is_match_matches_tests.rs`, because a generated ruleset may compile to an
//        empty set and then assert nothing.

#![no_main]

use forbidden_regex_fuzz::generators::RulesetAndBuffer;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: RulesetAndBuffer| {
    // What:  build the compiled matcher leniently, like the real scanner does.
    // Why:   some generated patterns are rejected by this dialect; `compile_lenient`
    //        drops those and keeps the rest, so the set is never a hard failure by
    //        construction alone.
    let (set, _kept) = forbidden_regex::RegexSet::compile_lenient(&input.patterns);

    // What:  the per-line slices, empty lines included.
    // Why:   an empty line is a legitimate input to both faces (both must report no
    //        match, since a rule matching the empty line is rejected at compile), and
    //        `line_matches` skips it, so the boolean invariant has to be checked here.
    let lines: Vec<&[u8]> = input.line_contents();

    // What:  the core invariant: the boolean face says exactly what the attribution
    //        face reports.
    // Why:   a coverage regression in one face (the gate, the line-start path, or the
    //        seedless groups) that the other face does not share shows up only here.
    for (index, line) in lines.iter().enumerate() {
        let boolean = set.is_match(line);
        let attributed = set.matches(line).next().is_some();
        if boolean != attributed {
            panic!(
                "is_match disagreed with matches on line {index}: is_match={boolean} matches().next().is_some()={attributed}\npatterns = {:?}\nline = {line:?}\nbuf = {:?}\nstarts = {:?}",
                input.patterns, input.buf, input.starts,
            );
        }
    }

    // What:  the batch boolean face must report, per line, what the scalar face reports.
    // Why:   `is_match_batch` is the scanner's hot path; a future batched kernel that
    //        skipped a line would otherwise disagree with `is_match` unnoticed.
    let batch = set.is_match_batch(&lines);
    for (index, line) in lines.iter().enumerate() {
        if batch[index] != set.is_match(line) {
            panic!(
                "is_match_batch disagreed with is_match on line {index}: batch={} scalar={}\npatterns = {:?}\nline = {line:?}\nbuf = {:?}\nstarts = {:?}",
                batch[index],
                set.is_match(line),
                input.patterns,
                input.buf,
                input.starts,
            );
        }
    }

    // What:  the batch attribution face must agree with the boolean face: an attributed
    //        rule implies a match on that line, and that rule is one `matches` names.
    // Why:   `line_matches` is the batched twin of `matches`; if its per-line resolution
    //        drifted from the gate the two would attribute different rules to the same
    //        bytes, and a false positive here would make the scanner report a hit that
    //        `is_match` denies.
    for (index, rule) in set.line_matches(&input.buf, &input.starts) {
        let line = lines[index];
        if !set.is_match(line) {
            panic!(
                "line_matches attributed rule {rule} on line {index} but is_match says no match\npatterns = {:?}\nline = {line:?}\nbuf = {:?}\nstarts = {:?}",
                input.patterns, input.buf, input.starts,
            );
        }
        if !set.matches(line).any(|matched| return matched == rule) {
            panic!(
                "line_matches attributed rule {rule} on line {index} but matches does not name it\npatterns = {:?}\nline = {line:?}\nbuf = {:?}\nstarts = {:?}",
                input.patterns, input.buf, input.starts,
            );
        }
    }
});
