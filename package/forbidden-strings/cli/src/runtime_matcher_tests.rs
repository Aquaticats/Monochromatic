//! Hybrid runtime matcher semantic parity tests.

use super::RuntimeRules;

/// Returns line starts for small test buffer.
fn starts(buf: &[u8]) -> Vec<usize> {
    let mut output = vec![0];
    for (index, &byte) in buf.iter().enumerate() {
        if byte == b'\n' && index + 1 < buf.len() {
            output.push(index + 1);
        }
    }
    return output
}

/// Interleaved legacy literal and explicit regex retain original ids.
#[test]
fn mixed_rules_report_global_ids() {
    let rules = RuntimeRules::compile(
        "ALPHA_LITERAL_LONG\n/BETA[0-9]{2}/\nGAMMA_LITERAL_LONG\n",
    )
    .expect("compile runtime rules");
    let buf = b"GAMMA_LITERAL_LONG BETA42 ALPHA_LITERAL_LONG\n";
    assert_eq!(rules.line_matches(buf, &starts(buf)), vec![(0, 0), (0, 1), (0, 2)]);
}

/// Tail-format names remain parallel across literal and regex kinds.
#[test]
fn tail_names_remain_in_original_order() {
    let rules = RuntimeRules::compile(
        "==> qqq-literal <==\nALPHA_LITERAL_LONG\n==> qqq-regex <==\n/BETA[0-9]{2}/\n",
    )
    .expect("compile runtime rules");
    assert_eq!(
        rules.names(),
        &[Some("qqq-literal".to_string()), Some("qqq-regex".to_string())],
    );
}

/// Duplicate identical literals report each original rule exactly once.
#[test]
fn duplicate_literals_preserve_distinct_rule_ids() {
    let rules = RuntimeRules::compile("ALPHA_LITERAL_LONG\nALPHA_LITERAL_LONG\n")
        .expect("compile runtime rules");
    let buf = b"ALPHA_LITERAL_LONG ALPHA_LITERAL_LONG";
    assert_eq!(rules.line_matches(buf, &starts(buf)), vec![(0, 0), (0, 1)]);
    assert_eq!(rules.literal_groups().len(), 1);
}

/// Overlapping long literals both report despite containment.
#[test]
fn overlapping_literals_all_report() {
    let rules = RuntimeRules::compile("abcdefgh\nbcdefghi\n").expect("compile runtime rules");
    let buf = b"abcdefghi";
    assert_eq!(rules.line_matches(buf, &starts(buf)), vec![(0, 0), (0, 1)]);
}

/// Short literal keeps existing whole-word boundary gating.
#[test]
fn short_literal_keeps_word_boundaries() {
    let rules = RuntimeRules::compile("ACR\n").expect("compile runtime rules");
    let hit = b"see ACR here";
    let miss = b"see ACRYLIC here";
    assert_eq!(rules.line_matches(hit, &starts(hit)), vec![(0, 0)]);
    assert!(rules.line_matches(miss, &starts(miss)).is_empty());
}

/// Bare metacharacters stay exact while slash-delimited form stays regex.
#[test]
fn parser_kind_controls_matcher_selection() {
    let rules = RuntimeRules::compile("a.b{2}\n/a.b{2}/\n").expect("compile runtime rules");
    let literal_line = b"a.b{2}";
    let regex_line = b"axbb";
    assert_eq!(rules.line_matches(literal_line, &starts(literal_line)), vec![(0, 0)]);
    assert_eq!(rules.line_matches(regex_line, &starts(regex_line)), vec![(0, 1)]);
}

/// Hybrid results equal prior all-engine matcher over mixed adversarial corpus.
#[test]
fn hybrid_matches_all_engine_oracle() {
    let source = [
        "ALPHA_LITERAL_LONG",
        "/BETA[0-9]{2}/",
        "ACR",
        "a.b{2}",
        "abcdefgh",
        "bcdefghi",
    ]
    .join("\n");
    let hybrid = RuntimeRules::compile(&source).expect("compile hybrid");
    let oracle = crate::compile_rules(&source).expect("compile oracle");
    let buf = b"ALPHA_LITERAL_LONG BETA42\nACRYLIC\r\na.b{2}\nabcdefghi\n";
    let line_starts = starts(buf);
    assert_eq!(
        hybrid.line_matches(buf, &line_starts),
        oracle.set.line_matches(buf, &line_starts),
    );
}

/// CRLF and empty lines retain engine batch line indexing.
#[test]
fn line_terminators_match_engine_contract() {
    let rules = RuntimeRules::compile("ALPHA_LITERAL_LONG\n").expect("compile runtime rules");
    let buf = b"\r\nALPHA_LITERAL_LONG\r\n";
    assert_eq!(rules.line_matches(buf, &starts(buf)), vec![(1, 0)]);
}
