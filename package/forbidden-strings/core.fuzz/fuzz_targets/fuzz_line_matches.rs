// What:  for a generated ruleset and a generated multi-line buffer (lines joined with
//        a mix of `\n`, `\r\n`, and an unterminated final line), assert
//        `RegexSet::line_matches` agrees with calling `RegexSet::matches` on each line
//        sliced out by hand per the documented contract.
// Why:   `line_matches`'s own rustdoc names it a deliberately naive per-line
//        delegation that a future single-sweep fast path is validated against; this
//        target pins that invariant by recomputing the naive slicing independently,
//        so any drift between the doc contract, the implementation, and a future fast
//        path shows up as a panic instead of shipping silently.

#![no_main]

use forbidden_regex_fuzz::generators::RulesetAndBuffer;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|input: RulesetAndBuffer| {
    // What:  build the compiled matcher leniently, like the real scanner does.
    // Why:   some generated patterns are rejected by this dialect (e.g.
    //        empty-matchable); `compile_lenient` drops those and keeps the rest, so
    //        the set is never a hard failure by construction alone.
    let (set, _kept) = forbidden_regex::RegexSet::compile_lenient(&input.patterns);

    // What:  the naive per-line reference: slice each line by the documented
    //        contract (next start, or buffer end for the last line; drop a trailing
    //        `\n` then one trailing `\r`; skip an empty line) and call `matches()`
    //        on the slice.
    // Why:   this mirrors `line_matches`'s own rustdoc exactly; recomputing it here,
    //        independently of the method under test, is what catches drift.
    let mut expected: Vec<(usize, usize)> = Vec::new();
    for index in 0..input.starts.len() {
        let start = input.starts[index];
        let mut end = input.starts.get(index + 1).copied().unwrap_or(input.buf.len());
        if end > start && input.buf[end - 1] == b'\n' {
            end -= 1;
        }
        if end > start && input.buf[end - 1] == b'\r' {
            end -= 1;
        }
        if end == start {
            continue;
        }
        for rule in set.matches(&input.buf[start..end]) {
            expected.push((index, rule));
        }
    }

    let actual = set.line_matches(&input.buf, &input.starts);
    if actual != expected {
        panic!(
            "line_matches disagreed with per-line matches(): actual={actual:?} expected={expected:?}\npatterns = {:?}\nbuf = {:?}\nstarts = {:?}",
            input.patterns, input.buf, input.starts,
        );
    }
});
