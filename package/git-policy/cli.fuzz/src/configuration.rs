//! What:
//!  Generated configuration documents and the invariants of the JSONC schema.
//! Why:
//!  Raw bytes mostly exercise rejection.
//!  Generated documents reach every accepted
//!      setting,
//!  and each one comes with the typed result it must produce,
//!  so the
//!      schema is checked against an expectation it did not compute itself.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const { document, expected } = generatedConfig(data); expect(parseConfig(emit(document))).toEqual(expected);
//! ```

/// Import the schema under test,
///  its typed settings and the event renderer.
use git_policy_cli::config_error::ConfigError;
use git_policy_cli::config_parse::parse_config;
use git_policy_cli::config_schema::{
    CliGitConfig, MarkdownRule, PolicySetting, RulesFileEntry, markdown_rule_name,
};
use git_policy_cli::diagnostics::{EngineFailureCode, render_engine_failure};
use git_policy_cli::policy_registry::{POLICY_REGISTRY, PolicyId, Severity, severity_name};
/// Import the JSONC value model used to build and re-read documents.
use monochromatic_jsonc_edit::{
    JsoncEntry, JsoncKey, JsoncValue, emit_jsonc_value, parse_jsonc, units_to_string,
};

/// JavaScript's largest safe integer,
///  the schema's upper bound for whole numbers.
const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

/// What:
///  A generated document together with the settings it must parse to.
///       `#[derive(...)]` generates cloning,
///  debug printing and `==`.
/// Why:
///   The expectation is built beside the document from the same bytes,
///  not by
///       running the parser,
///  so agreement is evidence about the parser.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type GeneratedConfig = { document: JsoncValue; expected: CliGitConfig };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedConfig {
    /// A valid configuration document.
    pub document: JsoncValue,
    /// The typed settings that document states.
    pub expected: CliGitConfig,
}

/// What:
///  Build one object member.
///       `&str` borrows the key text;
///  the member owns its value.
/// Why:
///   Keys are escaped by the JSONC package at emission,
///  never by string pasting here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function entry(name: string, value: JsoncValue): JsoncEntry;
/// ```
fn entry(name: &str, value: JsoncValue) -> JsoncEntry {
    return JsoncEntry {
        key: JsoncKey::from_text(name),
        value,
    };
}

/// What:
///  Build a JSON string value from Rust text.
/// Why:
///   The text is stored as UTF-16 units;
///  quoting happens at emission.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function text(value: string): JsoncValue;
/// ```
fn text(value: &str) -> JsoncValue {
    // `.encode_utf16().collect()` gathers the text's UTF-16 units into an owned list.
    return JsoncValue::text_from_units(value.encode_utf16().collect());
}

/// What:
///  Build a JSON number value from a whole number.
///       `u64` is an unsigned 64-bit integer (siblings `u32`,
///  `usize`).
/// Why:
///   Decimal digits of a whole number are always a valid JSON number token.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function number(value: number): JsoncValue;
/// ```
fn number(value: u64) -> JsoncValue {
    // `.expect(..)` stops the program with this message if the token were ever rejected.
    return JsoncValue::number_from_token(value.to_string().as_str())
        .expect("decimal digits are a number token");
}

/// What:
///  Read byte `index` of the fuzz input,
///  or zero past its end.
///       `u8` is one byte,
///  0 to 255.
/// Why:
///   Short inputs still describe a complete,
///  valid document.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function byteAt(data: Uint8Array, index: number): number { return data[index] ?? 0; }
/// ```
fn byte_at(data: &[u8], index: usize) -> u8 {
    if index < data.len() {
        return data[index];
    }
    return 0;
}

/// What:
///  Derive one exclusion pattern from a byte:
///  path-like text with awkward characters.
/// Why:
///   Patterns are free text;
///  quotes,
///  backslashes,
///  control characters and non-ASCII
///       must survive emission and parsing unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pattern(byte: number): string;
/// ```
fn pattern(byte: u8) -> String {
    let pieces: [&str; 8] = [
        "package/ssg/",
        "",
        "a \"quoted\" name",
        "back\\slash",
        "line\nfeed\ttab",
        "\u{0}\u{1f}",
        "é\u{1F600}\u{2028}",
        "../../outside/**",
    ];
    // `usize::from` widens the byte to an index; `%` wraps it into the table.
    return format!("{}{}", pieces[usize::from(byte) % pieces.len()], byte);
}

/// Configured rules-file paths the generator draws from.
///  A comma and a question mark are ordinary characters in a configured path,
///  because the configuration is never parsed as a list.
const RULES_PATHS: [&str; 6] = [
    "rules.txt",
    ".cache/forbidden-strings.rules.txt",
    "shared/a.txt",
    "local.txt",
    "odd,name?.txt",
    "forbidden-strings.append.local.txt",
];

/// What:
///  Build a `rulesFile` value and the entries it must parse to.
///  `kind` 1 is a string,
///  and the other kinds are arrays of one to three distinct entries whose element forms
///       (string,
///  `{"path"}`,
///  `{"path","optional":true}`,
///  `{"path","optional":false}`) come from `seed`.
/// Why:
///   Every accepted spelling is reached, each with the typed result it must produce.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function generatedRulesFiles(kind: number, seed: number): { value: JsoncValue; files: RulesFileEntry[] };
/// ```
fn generated_rules_files(kind: u8, seed: u8) -> (JsoncValue, Vec<RulesFileEntry>) {
    let start: usize = usize::from(seed) % RULES_PATHS.len();
    if kind == 1 {
        return (
            text(RULES_PATHS[start]),
            vec![RulesFileEntry {
                path: String::from(RULES_PATHS[start]),
                optional: false,
            }],
        );
    }
    let count: usize = 1 + usize::from(kind) % 3;
    let mut values: Vec<JsoncValue> = Vec::<JsoncValue>::new();
    let mut files: Vec<RulesFileEntry> = Vec::<RulesFileEntry>::new();
    for offset in 0..count {
        let path: &str = RULES_PATHS[(start + offset) % RULES_PATHS.len()];
        let form: u8 = (seed / 8).wrapping_add(u8::try_from(offset).unwrap_or(0)) % 4;
        let optional: bool = form == 2;
        match form {
            0 => values.push(text(path)),
            1 => values.push(JsoncValue::record(vec![entry("path", text(path))])),
            _ => values.push(JsoncValue::record(vec![
                entry("path", text(path)),
                entry("optional", JsoncValue::boolean(optional)),
            ])),
        }
        files.push(RulesFileEntry {
            path: String::from(path),
            optional,
        });
    }
    return (JsoncValue::array(values), files);
}

/// What:
///  Build a valid configuration document and its expected settings from fuzz bytes.
/// Why:
///   Each policy is omitted or given a severity,
///  option-bearing policies sometimes
///       use the `[severity, options]` form,
///  and each concurrency section is omitted,
///       empty or set,
///  so every accepted shape is reached.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function generatedConfig(data: Uint8Array): GeneratedConfig;
/// ```
pub fn generated_config(data: &[u8]) -> GeneratedConfig {
    let mut expected: CliGitConfig = CliGitConfig::defaults();
    // `Vec::<JsoncEntry>::new()` is an empty owned list; `mut` allows pushing.
    let mut policies: Vec<JsoncEntry> = Vec::<JsoncEntry>::new();
    // `.iter().enumerate()` yields `(index, row)` pairs over the registry.
    for (index, descriptor) in POLICY_REGISTRY.iter().enumerate() {
        let choice: u8 = byte_at(data, index);
        // `.is_multiple_of(4)` is `choice % 4 === 0`: one input in four omits the policy.
        if choice.is_multiple_of(4) {
            continue;
        }
        let severity: Severity =
            [Severity::Off, Severity::Warn, Severity::Error][usize::from(choice % 4) - 1];
        expected.policies.settings[index] = PolicySetting {
            id: descriptor.id,
            severity,
            explicit: true,
        };
        let word: JsoncValue = text(severity_name(severity));
        if !descriptor.accepts_options || (choice / 4).is_multiple_of(2) {
            policies.push(entry(descriptor.name, word));
            continue;
        }
        let mut options: Vec<JsoncEntry> = Vec::<JsoncEntry>::new();
        if descriptor.id == PolicyId::ForbiddenStrings {
            if (choice / 8) % 2 == 1 {
                let enabled: bool = (choice / 16) % 2 == 1;
                expected.policies.forbidden_strings.builtin_rules = enabled;
                options.push(entry("builtinRules", JsoncValue::boolean(enabled)));
            }
            // The top three bits pick the `rulesFile` shape: absent, a string, or an array.
            let shape: u8 = choice / 32;
            if shape > 0 {
                let (value, files): (JsoncValue, Vec<RulesFileEntry>) =
                    generated_rules_files(shape, byte_at(data, 11));
                expected.policies.forbidden_strings.rules_files = Some(files);
                options.push(entry("rulesFile", value));
            }
        } else {
            if (choice / 8) % 2 == 1 {
                options.push(entry(
                    "rules",
                    JsoncValue::array(vec![text(markdown_rule_name(MarkdownRule::LfsImageUrl))]),
                ));
            }
            if (choice / 16) % 2 == 1 {
                let mut patterns: Vec<JsoncValue> = Vec::<JsoncValue>::new();
                let mut texts: Vec<String> = Vec::<String>::new();
                // `0..n` counts from 0 up to, not including, `n`.
                for offset in 0..usize::from(choice / 32) {
                    let generated: String = pattern(byte_at(data, 16 + offset));
                    patterns.push(text(generated.as_str()));
                    texts.push(generated);
                }
                expected.policies.markdown_autofix.exclude = texts;
                options.push(entry("exclude", JsoncValue::array(patterns)));
            }
        }
        policies.push(entry(
            descriptor.name,
            JsoncValue::array(vec![word, JsoncValue::record(options)]),
        ));
    }
    let mut members: Vec<JsoncEntry> = Vec::<JsoncEntry>::new();
    let shape: u8 = byte_at(data, 9);
    if shape % 2 == 1 {
        members.push(entry("policies", JsoncValue::record(policies)));
    } else {
        // Without a `policies` member every policy keeps its default.
        expected.policies = CliGitConfig::defaults().policies;
    }
    // Each section: 0 omitted, 1 present and empty, 2 or 3 present with its key.
    let hooks: u8 = (shape / 2) % 4;
    if hooks == 1 {
        members.push(entry("hooks", JsoncValue::record(Vec::<JsoncEntry>::new())));
    } else if hooks >= 2 {
        let overlap: bool = hooks == 3;
        expected.concurrency.hooks.concurrent_commits = overlap;
        members.push(entry(
            "hooks",
            JsoncValue::record(vec![entry(
                "concurrentCommits",
                JsoncValue::boolean(overlap),
            )]),
        ));
    }
    let lock: u8 = (shape / 8) % 4;
    if lock == 1 {
        members.push(entry(
            "indexLock",
            JsoncValue::record(Vec::<JsoncEntry>::new()),
        ));
    } else if lock >= 2 {
        // Zero, small values and the upper bound are all valid timeouts.
        let timeout: u64 = if lock == 3 {
            MAX_SAFE_INTEGER - u64::from(byte_at(data, 10))
        } else {
            u64::from(byte_at(data, 10)) * 17
        };
        expected.concurrency.index_lock.unproven_owner_timeout_ms = timeout;
        members.push(entry(
            "indexLock",
            JsoncValue::record(vec![entry("unprovenOwnerTimeoutMs", number(timeout))]),
        ));
    }
    let landing: u8 = (shape / 32) % 4;
    if landing == 1 {
        members.push(entry(
            "landing",
            JsoncValue::record(Vec::<JsoncEntry>::new()),
        ));
    } else if landing >= 2 {
        let reserve: u64 = 1 + u64::from(byte_at(data, 11));
        expected.concurrency.landing.reserve_after_lost_races = reserve;
        members.push(entry(
            "landing",
            JsoncValue::record(vec![entry("reserveAfterLostRaces", number(reserve))]),
        ));
    }
    return GeneratedConfig {
        document: JsoncValue::record(members),
        expected,
    };
}

/// What:
///  Rebuild a canonical document that states every setting of a parsed configuration.
/// Why:
///   Parsing that document again must give the same settings;
///  this checks that
///       every accepted value has a spelling the schema accepts back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function canonicalDocument(config: CliGitConfig): JsoncValue;
/// ```
pub fn canonical_document(config: &CliGitConfig) -> JsoncValue {
    let mut policies: Vec<JsoncEntry> = Vec::<JsoncEntry>::new();
    for descriptor in POLICY_REGISTRY {
        let word: JsoncValue = text(severity_name(
            config.policies.setting(descriptor.id).severity,
        ));
        if descriptor.id == PolicyId::ForbiddenStrings {
            let mut members: Vec<JsoncEntry> = vec![entry(
                "builtinRules",
                JsoncValue::boolean(config.policies.forbidden_strings.builtin_rules),
            )];
            // Every configured file is stated as an object, whichever form it was written in.
            if let Some(files) = &config.policies.forbidden_strings.rules_files {
                let mut states: Vec<JsoncValue> = Vec::<JsoncValue>::new();
                for file in files {
                    states.push(JsoncValue::record(vec![
                        entry("path", text(file.path.as_str())),
                        entry("optional", JsoncValue::boolean(file.optional)),
                    ]));
                }
                members.push(entry("rulesFile", JsoncValue::array(states)));
            }
            let options: JsoncValue = JsoncValue::record(members);
            policies.push(entry(
                descriptor.name,
                JsoncValue::array(vec![word, options]),
            ));
        } else if descriptor.id == PolicyId::MarkdownAutofix {
            let mut rules: Vec<JsoncValue> = Vec::<JsoncValue>::new();
            // `&config...rules` lends the list; `*rule` copies one small rule out of it.
            for rule in &config.policies.markdown_autofix.rules {
                rules.push(text(markdown_rule_name(*rule)));
            }
            let mut patterns: Vec<JsoncValue> = Vec::<JsoncValue>::new();
            for excluded in &config.policies.markdown_autofix.exclude {
                patterns.push(text(excluded.as_str()));
            }
            let options: JsoncValue = JsoncValue::record(vec![
                entry("rules", JsoncValue::array(rules)),
                entry("exclude", JsoncValue::array(patterns)),
            ]);
            policies.push(entry(
                descriptor.name,
                JsoncValue::array(vec![word, options]),
            ));
        } else {
            policies.push(entry(descriptor.name, word));
        }
    }
    return JsoncValue::record(vec![
        entry("policies", JsoncValue::record(policies)),
        entry(
            "hooks",
            JsoncValue::record(vec![entry(
                "concurrentCommits",
                JsoncValue::boolean(config.concurrency.hooks.concurrent_commits),
            )]),
        ),
        entry(
            "indexLock",
            JsoncValue::record(vec![entry(
                "unprovenOwnerTimeoutMs",
                number(config.concurrency.index_lock.unproven_owner_timeout_ms),
            )]),
        ),
        entry(
            "landing",
            JsoncValue::record(vec![entry(
                "reserveAfterLostRaces",
                number(config.concurrency.landing.reserve_after_lost_races),
            )]),
        ),
    ]);
}

/// What:
///  Assert the invariants every accepted configuration satisfies.
/// Why:
///   Later stages index settings by registry position and rely on the bounds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkAccepted(config: CliGitConfig): void;
/// ```
fn check_accepted(config: &CliGitConfig) {
    assert_eq!(config.policies.settings.len(), POLICY_REGISTRY.len());
    for (index, descriptor) in POLICY_REGISTRY.iter().enumerate() {
        assert_eq!(
            config.policies.settings[index].id, descriptor.id,
            "registry order"
        );
        assert_eq!(
            config.policies.setting(descriptor.id),
            config.policies.settings[index]
        );
    }
    assert!(config.concurrency.index_lock.unproven_owner_timeout_ms <= MAX_SAFE_INTEGER);
    assert!(config.concurrency.landing.reserve_after_lost_races >= 1);
    assert!(config.concurrency.landing.reserve_after_lost_races <= MAX_SAFE_INTEGER);
    assert!(
        !config.policies.markdown_autofix.rules.is_empty(),
        "rules are never empty"
    );
    // Stating every setting explicitly and parsing again changes nothing but `explicit`.
    let restated: CliGitConfig =
        parse_config(emit_jsonc_value(&canonical_document(config)).as_str())
            .expect("a canonical document is accepted");
    assert_eq!(restated.concurrency, config.concurrency);
    assert_eq!(
        restated.policies.forbidden_strings,
        config.policies.forbidden_strings
    );
    assert_eq!(
        restated.policies.markdown_autofix,
        config.policies.markdown_autofix
    );
    for descriptor in POLICY_REGISTRY {
        assert_eq!(
            restated.policies.setting(descriptor.id).severity,
            config.policies.setting(descriptor.id).severity
        );
        assert!(restated.policies.setting(descriptor.id).explicit);
    }
}

/// What:
///  Assert that a rejection message survives event rendering as one JSON line.
/// Why:
///   Messages quote user-written keys and values.
///  Whatever they contain,
///  the
///       `config-invalid` event must stay one line and decode back to the same message.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkRejected(message: string): void;
/// ```
fn check_rejected(message: &str) {
    assert!(!message.is_empty(), "a rejection explains itself");
    let line: String = render_engine_failure(0, EngineFailureCode::ConfigInvalid, message);
    assert!(line.ends_with('\n'));
    assert_eq!(
        line.matches('\n').count(),
        1,
        "an event is exactly one line"
    );
    let event: JsoncValue =
        parse_jsonc(line.trim_end_matches('\n')).expect("an event is valid JSON");
    let entries: &[JsoncEntry] = event.entries().expect("an event is an object");
    // Five fixed members, then the code's generic hint when the code has one.
    assert!(
        entries.len() == 5 || entries.len() == 6,
        "an event has five members and an optional hint"
    );
    if entries.len() == 6 {
        assert_eq!(
            entries[5].key.raw, "\"hint\"",
            "the sixth member is the hint"
        );
    }
    // The last member is the message; its decoded text must equal the original.
    let decoded: String = units_to_string(
        entries[4]
            .value
            .text_units()
            .expect("the message is a string"),
    )
    .expect("the message is valid Unicode");
    assert_eq!(decoded, message);
}

/// What:
///  Assert the schema's invariants for arbitrary source text.
/// Why:
///   Parsing must be repeatable and must end in a checked acceptance or a
///       renderable rejection,
///  never a panic.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkConfigSource(source: string): void;
/// ```
pub fn check_config_source(source: &str) {
    let first: Result<CliGitConfig, ConfigError> = parse_config(source);
    assert_eq!(parse_config(source), first, "parsing is not repeatable");
    // `match` picks by variant: accepted settings or the rejection.
    match first {
        Ok(config) => check_accepted(&config),
        Err(error) => check_rejected(error.message.as_str()),
    }
}

/// What:
///  Assert that a generated document parses to exactly its expected settings.
/// Why:
///   This is the differential check:
///  the generator's expectation against the schema.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function checkGeneratedConfig(generated: GeneratedConfig): void;
/// ```
pub fn check_generated_config(generated: &GeneratedConfig) {
    let source: String = emit_jsonc_value(&generated.document);
    assert_eq!(
        parse_config(source.as_str()),
        Ok(generated.expected.clone()),
        "generated document: {source}"
    );
    check_config_source(source.as_str());
}

/// Generator and invariant controls stay out of the fuzz binaries.
#[cfg(test)]
#[path = "configuration_tests.rs"]
mod tests;
