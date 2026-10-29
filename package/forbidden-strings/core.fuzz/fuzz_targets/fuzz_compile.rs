// What:  feed arbitrary bytes as a pattern string and a ruleset, asserting that
//        compilation never panics and always terminates (it returns Result).
// Why:   `compile` / `RegexSet::new` run on attacker-influenced rule files; a panic or
//        non-termination in the parser, desugarer, or DFA/counting builder would be a
//        denial-of-service or a crash, so the contract is "reject, never panic".

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // What:  only valid UTF-8 is a pattern; non-UTF-8 is out of contract.
    // Why:   patterns are `&str`; the byte path is covered by fuzz_from_bytes.
    let Ok(text) = std::str::from_utf8(data) else {
        return;
    };

    // What:  single-pattern compile; ignore the verdict, assert no panic.
    // Why:   the parser and back-end selector must reject cleanly, never crash.
    if let Ok(regex) = forbidden_regex::compile(text) {
        // What:  exercise matching and the serialize path on the compiled engine.
        // Why:   a pattern that compiled must also match and serialize without panic.
        let _ = regex.is_match(data);
        let _ = regex.to_bytes();
    }

    // What:  compile the bytes as a whole ruleset, one rule per line, leniently.
    // Why:   the real scanner builds a RegexSet from a multi-rule file, dropping rules
    //        this dialect cannot express; that lenient path must also never panic.
    let lines: Vec<&str> = text.lines().collect();
    let (set, _kept) = forbidden_regex::RegexSet::compile_lenient(&lines);
    let _ = set.is_match(data);
});
