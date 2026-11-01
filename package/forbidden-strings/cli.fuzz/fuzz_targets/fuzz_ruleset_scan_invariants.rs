// What:  drive the strict two-form loader (`load_from_text`) and the columnless scan
//        (`scan_file`) over a generated ruleset and buffer, asserting loader-contract and
//        scan invariants that hold regardless of which rules match: a rejected flag fails
//        the load closed, `m`/`x` are no-ops, a file with no rule line loads nothing, and
//        reversing the rule order renumbers ids but never changes which positions match.
// Why:   the engine swap (#384/#385) replaced the resharp/aho-corasick loader with the
//        strict two-form frx loader and the columnless output. Invariant-style testing
//        (rather than a slow reference scanner that would drift) pins those properties;
//        a fixed loader-contract battery pins the strict-flag policy on every process.

#![no_main]

use libfuzzer_sys::fuzz_target;

use forbidden_strings::fuzz_api::{load_from_text, scan_file};
use forbidden_strings_fuzz::generators::{redacted_fingerprint, RuleFileAndContent};

// The fixed path handed to `scan_file`.
const PATH: &str = "fuzz.txt";

// What:  pin the strict-loader flag policy once per process.
// Why:   the two-form contract is: bare literal and `/PATTERN/FLAGS` load, `m`/`x` are
//        no-ops, every other flag letter is a hard error, and a source with no rule line
//        is `NoRules`. A fixed battery exercises each branch on every campaign, including
//        a bounded smoke pass, independent of what the fuzzer happens to generate.
fn run_loader_contract_battery() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        // Accepted forms.
        assert!(load_from_text("abc").is_ok(), "a bare literal must load");
        assert!(load_from_text("/abc/").is_ok(), "an empty flag run must load");
        assert!(load_from_text("/abc/m").is_ok(), "'m' must be an accepted no-op");
        assert!(load_from_text("/abc/x").is_ok(), "'x' must be an accepted no-op");
        assert!(load_from_text("/abc/mx").is_ok(), "'m' and 'x' must be accepted no-ops");
        // Rejected forms: any other flag letter is a hard, fail-closed load error.
        assert!(load_from_text("/abc/i").is_err(), "'i' must be a hard load error");
        assert!(load_from_text("/abc/s").is_err(), "'s' must be a hard load error");
        assert!(load_from_text("/abc/g").is_err(), "'g' must be a hard load error");
        // No-rule forms.
        assert!(load_from_text("").is_err(), "an empty source loads no rules");
        assert!(load_from_text("# comment").is_err(), "a comment-only source loads no rules");
        assert!(load_from_text("   ").is_err(), "a whitespace-only source loads no rules");
    });
}

// What:  the position-only key of a finding: `PATH:LINE`, dropping the `rule=N` suffix.
// Why:   rule-order invariance renumbers rule ids but not positions, so positions (as a
//        multiset) must match across orderings. The fail-closed `engine error` line has
//        no position and is dropped.
fn position_key(hit: &str) -> Option<String> {
    return hit.rfind(" rule=").map(|pos| return hit[..pos].to_string())
}

fuzz_target!(|input: RuleFileAndContent| {
    run_loader_contract_battery();

    let source = input.rules.render();
    let load_result = load_from_text(&source);

    // Strict-loader prediction: a file carrying a rejected flag must fail closed, no
    // matter where in the file that line sits.
    if input.rules.has_bad_flag() {
        assert!(
            load_result.is_err(),
            "a ruleset with a rejected flag must fail closed",
        );
    }

    let loaded = match load_result {
        Ok(loaded) => loaded,
        Err(_) => return,
    };

    let content: &[u8] = &input.content;
    let hits = scan_file(PATH, content, &loaded);

    // Every finding's line index stays within the buffer; the format itself is pinned in
    // detail by `fuzz_scan_format`, so here we only bound the line index.
    let max_line = content.iter().filter(|&&byte| return byte == b'\n').count() + 1;
    let engine_error = format!("{PATH}: engine error");
    for hit in &hits {
        if hit == &engine_error {
            continue;
        }
        let Some(key) = position_key(hit) else {
            panic!("finding missing ' rule=' ({})", redacted_fingerprint(content));
        };
        let Some(colon_pos) = key.rfind(':') else {
            panic!("finding missing ':' ({})", redacted_fingerprint(content));
        };
        let Ok(line) = key[colon_pos + 1..].parse::<usize>() else {
            panic!("finding line not numeric ({})", redacted_fingerprint(content));
        };
        assert!(
            line >= 1 && line <= max_line,
            "line index {} out of range 1..={} ({})",
            line,
            max_line,
            redacted_fingerprint(content),
        );
    }

    // Rule-order invariance: reversing the rule lines gives every rule a new id but leaves
    // the set of patterns unchanged, so the multiset of matched positions is invariant.
    // The forward load succeeded, so no rejected flag is present and the reverse loads too.
    let reversed_source = input.rules.render_reversed();
    if let Ok(loaded_rev) = load_from_text(&reversed_source) {
        let hits_rev = scan_file(PATH, content, &loaded_rev);
        let mut keys: Vec<String> = hits.iter().filter_map(|hit| return position_key(hit)).collect();
        let mut keys_rev: Vec<String> =
            hits_rev.iter().filter_map(|hit| return position_key(hit)).collect();
        keys.sort();
        keys_rev.sort();
        assert_eq!(
            keys,
            keys_rev,
            "rule-order invariance violated ({})",
            redacted_fingerprint(content),
        );
    }
});
