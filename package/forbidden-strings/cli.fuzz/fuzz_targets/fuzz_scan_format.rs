// What:  load a generated two-form ruleset and scan a generated multi-line buffer
//        through the real `scan_file`, then assert the columnless output contract: every
//        finding is exactly `PATH:LINE rule=<token>` (or the fail-closed
//        `PATH: engine error`), where the token is a numeric id or a tail-format section
//        name drawn from the strict name alphabet `[a-z0-9.-]`, the line index is
//        1-based and within the buffer, findings arrive line-ascending, and no content
//        byte leaks into the finding.
// Why:   the finding format is the externally visible contract and the leak surface. The
//        engine swap (#384) dropped the old `path:line:col_start..col_end rule=N` columns
//        for the columnless form, and the tail-format migration made the rule token a
//        section name for named rules (numeric ids remain for legacy sources, the only
//        kind this generator produces: its alphabets exclude `=`/`<`/`>`, so a generated
//        file can never open with a section header). The redaction argument is
//        structural (the formatter interpolates only the fixed path, the line integer,
//        and a token bounded by the name alphabet), and this target pins that a
//        regression cannot reintroduce columns or interpolate content.

#![no_main]

use libfuzzer_sys::fuzz_target;

use forbidden_strings::fuzz_api::{load_from_text, scan_file, scan_path_for_fuzzing};
use forbidden_strings_fuzz::generators::{redacted_fingerprint, RuleFileAndContent};
use std::sync::OnceLock;

// The fixed path handed to `scan_file`; findings must interpolate exactly this and two
// integers, nothing else.
const PATH: &str = "fuzz.txt";

// A fixed positive control proves the masked-path assertion can catch a real hit.
static NAME_CONTROL: OnceLock<forbidden_strings::fuzz_api::LoadedRules> = OnceLock::new();

// What:  parse one finding and assert the columnless format, returning its line index.
// Why:   the redaction check is the reconstruction at the end: a finding equal to the
//        fixed template plus its two parsed integers cannot carry any content byte.
fn check_format(hit: &str, content: &[u8], max_line: usize) -> usize {
    // Split at the trailing ` rule=` marker (paths never contain it because PATH is fixed).
    let Some(rule_pos) = hit.rfind(" rule=") else {
        panic!("finding missing ' rule=': {} ({})", hit.len(), redacted_fingerprint(content));
    };
    // The rule token is a numeric id (legacy rules; digits are inside the name
    // alphabet) or a tail-format section name over `[a-z0-9.-]`. Anything outside
    // that alphabet would mean interpolated content or format drift.
    let rule_str = &hit[rule_pos + " rule=".len()..];
    let token_ok = !rule_str.is_empty()
        && rule_str.bytes().all(|byte| {
            return byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || byte == b'.'
                || byte == b'-';
        });
    if !token_ok {
        panic!("finding rule token outside name alphabet ({})", redacted_fingerprint(content));
    }

    // The prefix is `PATH:LINE`; split at the last colon (PATH itself has no colon).
    let prefix = &hit[..rule_pos];
    let Some(colon_pos) = prefix.rfind(':') else {
        panic!("finding missing ':' ({})", redacted_fingerprint(content));
    };
    let path = &prefix[..colon_pos];
    assert_eq!(path, PATH, "finding path mismatch ({})", redacted_fingerprint(content));
    let line_str = &prefix[colon_pos + 1..];
    let Ok(line) = line_str.parse::<usize>() else {
        panic!("finding line not numeric ({})", redacted_fingerprint(content));
    };

    assert!(line >= 1, "line index not 1-based: {} ({})", line, redacted_fingerprint(content));
    assert!(
        line <= max_line,
        "line index {} exceeds {} lines ({})",
        line,
        max_line,
        redacted_fingerprint(content),
    );

    // Redaction: the whole finding is exactly the fixed template plus the parsed line
    // integer and the alphabet-bounded rule token. Any other byte would make this
    // reconstruction unequal, so equality proves nothing else was interpolated.
    let expected = format!("{PATH}:{line} rule={rule_str}");
    assert_eq!(hit, expected.as_str(), "finding carries unexpected bytes ({})", redacted_fingerprint(content));

    return line
}

fuzz_target!(|input: RuleFileAndContent| {
    let source = input.rules.render();
    // The strict-loader contract is `fuzz_ruleset_scan_invariants`' job; here we need a
    // loaded set, so a load failure just ends the iteration.
    let loaded = match load_from_text(&source) {
        Ok(loaded) => loaded,
        Err(_) => return,
    };

    let content: &[u8] = &input.content;
    let hits = scan_file(PATH, content, &loaded);

    // Upper bound on any 1-based line index: one more than the newline count.
    let max_line = content.iter().filter(|&&byte| return byte == b'\n').count() + 1;
    let engine_error = format!("{PATH}: engine error");

    let mut prev_line = 0usize;
    for hit in &hits {
        // The documented fail-closed shape from a caught engine panic; redacted already.
        if hit == &engine_error {
            continue;
        }
        let line = check_format(hit, content, max_line);
        // A single loaded set yields findings in line-ascending order; pin it.
        assert!(
            line >= prev_line,
            "findings not line-ascending: {} then {} ({})",
            prev_line,
            line,
            redacted_fingerprint(content),
        );
        prev_line = line;
    }

    // Apply the same compiled rules to a generated pathname with actual segment
    // separators and adversarial content bytes; all name findings must use the
    // shared masked label and carry a valid one-based component position.
    let control = NAME_CONTROL.get_or_init(|| return load_from_text("VAULTTOKEN_LONG\n")
        .expect("compile name positive control"));
    let (masked, positive_hits) = scan_path_for_fuzzing("prefix/VAULTTOKEN_LONG/tail", control);
    assert_eq!(masked, "prefix/[REDACTED]/tail", "positive control failed");
    assert_eq!(positive_hits, vec!["prefix/[REDACTED]/tail:name:2 rule=0"], "positive control missed hit");

    let generated_name = format!("prefix/{}/tail", String::from_utf8_lossy(content));
    let (display, name_hits) = scan_path_for_fuzzing(&generated_name, &loaded);
    assert!(!display.contains('\n'), "pathname injected newline ({})", redacted_fingerprint(content));
    if generated_name.contains('\n') || generated_name.contains('\r') {
        // A name with a line break cannot satisfy the engine's single-line
        // contract; the scanner masks its entire path and fails closed.
        assert_eq!(display, "[REDACTED]", "line-break name was not masked");
        assert_eq!(name_hits, vec!["[REDACTED]: unsupported pathname line break"]);
    } else {
        let segment_count = generated_name.split('/').filter(|name| return !name.is_empty() && *name != "." && *name != "..").count();
        for hit in &name_hits {
            if hit == "[REDACTED]: engine error" {
                assert_eq!(display, "[REDACTED]", "engine failure path was not masked");
                continue;
            }
            let Some((reported, suffix)) = hit.rsplit_once(":name:") else {
                panic!("missing name locator ({})", redacted_fingerprint(content));
            };
            assert_eq!(reported, display, "name finding used wrong display path ({})", redacted_fingerprint(content));
            let Some((number, token)) = suffix.split_once(" rule=") else {
                panic!("missing name rule ({})", redacted_fingerprint(content));
            };
            let index = number.parse::<usize>().expect("numeric name position");
            assert!(index > 0 && index <= segment_count, "invalid segment position ({})", redacted_fingerprint(content));
            assert!(!token.is_empty(), "empty rule identity ({})", redacted_fingerprint(content));
        }
    }
    for hit in scan_file(&display, content, &loaded) {
        assert!(hit.starts_with(&display), "content finding used raw path ({})", redacted_fingerprint(content));
    }
});
