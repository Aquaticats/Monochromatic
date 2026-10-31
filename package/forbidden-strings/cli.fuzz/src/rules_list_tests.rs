// Controls for the rules-list oracle: it accepts the grammar's edge cases, rejects a wrong claim, and the
// generator reaches every outcome the fuzz target must exercise.

use super::*;

// Hand-written adversarial values: delimiters, doubled markers, empty entries, whitespace, lookalikes.
const EDGE_VALUES: &[&str] = &[
    "",
    ",",
    "?",
    "??",
    "???",
    ",?",
    "?,",
    "a",
    "a?",
    "a??",
    "a,b",
    "a,b?",
    "a?,b",
    "a,,b",
    ",a",
    "a,",
    "a,a",
    "a,a?",
    "a?,a",
    "a??,a?",
    " ",
    " ? ",
    "a, b",
    "a\n,b",
    "\u{ff0c}",
    "a%2Cb",
    "a\\,b",
    "C:/x,D:/y?",
];

#[test]
fn the_oracle_accepts_every_edge_value() {
    for value in EDGE_VALUES {
        check_rules_list_value(value);
    }
}

#[test]
fn the_reference_parse_pins_known_answers() {
    assert_eq!(reference_parse("a,b?"), Ok(vec![("a".to_string(), false), ("b".to_string(), true)]));
    assert_eq!(reference_parse("a??"), Ok(vec![("a?".to_string(), true)]));
    assert_eq!(reference_parse("a,,b"), Err(RulesListError::EmptyEntry { position: 2 }));
    assert_eq!(reference_parse("a,a?"), Err(RulesListError::DuplicatePath { first: 1, position: 2 }));
    assert_eq!(reference_parse(""), Err(RulesListError::Empty));
}

#[test]
fn a_wrong_rendering_is_caught() {
    // Negative control: render must invert the parse, so dropping the optional marker is a visible difference.
    let entries = parse_rules_list("a,b?").expect("valid");
    let mut changed = entries.clone();
    changed[1].optional = false;
    assert_eq!(render(&entries), "a,b?");
    assert_ne!(render(&changed), "a,b?");
}

#[test]
fn the_generator_reaches_acceptance_and_every_refusal() {
    let mut accepted = 0_usize;
    let mut empty = 0_usize;
    let mut empty_entry = 0_usize;
    let mut duplicate = 0_usize;
    let mut optional_seen = false;
    let mut doubled_seen = false;
    for seed in 0..4000_u32 {
        // A small deterministic byte stream per seed: a linear congruential walk, not a random source.
        let mut state = seed.wrapping_mul(2_654_435_761).wrapping_add(12_345);
        let mut data = Vec::new();
        for _ in 0..(seed % 9) {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            data.push((state >> 24) as u8);
        }
        let value = generated_rules_list(&data);
        check_rules_list_value(&value);
        match parse_rules_list(&value) {
            Ok(entries) => {
                accepted += 1;
                optional_seen |= entries.iter().any(|entry| return entry.optional);
                doubled_seen |= entries.iter().any(|entry| return entry.path.to_string_lossy().ends_with('?'));
            }
            Err(RulesListError::Empty) => empty += 1,
            Err(RulesListError::EmptyEntry { .. }) => empty_entry += 1,
            Err(RulesListError::DuplicatePath { .. }) => duplicate += 1,
        }
    }
    assert!(accepted > 0 && empty > 0 && empty_entry > 0 && duplicate > 0);
    assert!(optional_seen && doubled_seen);
}

#[test]
fn the_generator_is_bounded_and_deterministic() {
    let data = vec![0xff_u8; 4096];
    let first = generated_rules_list(&data);
    assert_eq!(first, generated_rules_list(&data));
    assert!(first.len() <= 48 * 16);
}
