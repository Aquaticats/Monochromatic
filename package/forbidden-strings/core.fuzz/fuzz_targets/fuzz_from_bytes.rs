// What:  feed arbitrary bytes to the serialized-matcher decoders and, when they accept,
//        run the decoded automaton. Nothing may panic or read out of bounds.
// Why:   the deployed scanner loads a pre-serialized RegexSet, and bytes on disk may be
//        corrupt or hostile. `from_bytes` validates the decoded graph BEFORE first use;
//        this target tries to find bytes that pass validation yet still crash at match
//        time, which would be an out-of-bounds read on attacker-influenced input.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // What:  decode a single-pattern matcher; on success, exercise it.
    // Why:   a decoded `Regex` that validate() accepted must match safely.
    if let Ok(regex) = forbidden_regex::Regex::from_bytes(data) {
        let _ = regex.is_match(b"");
        let _ = regex.is_match(data);
        let _ = regex.is_match(b"A3TabcAKIA0123456789ABCDEF probe line");
    }

    // What:  decode a whole ruleset matcher; on success, exercise both queries.
    // Why:   `RegexSet::from_bytes` validates every per-rule engine; `is_match` and
    //        `matches` walk all of them, so both must stay in bounds.
    if let Ok(set) = forbidden_regex::RegexSet::from_bytes(data) {
        let _ = set.is_match(data);
        let _ = set.is_match(b"");
        let _ = set.matches(data).count();
    }
});
