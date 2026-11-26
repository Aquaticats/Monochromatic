//! What:
//!  Rule identifiers and option shapes accepted by unified-linter configuration.
//! Why:
//!  The shipped-only design must reject misspelled rules and unsupported options.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Validate options for the fixed shipped rule registry.
//! ```

/// Import shared schema readers and typed errors.
use crate::config_data::{
    key_text,
    strings,
    text,
};
/// Import the configuration error type returned when a rule value fails validation.
use crate::config_error::ConfigError;
/// Import the largest bound the tracked-image reader reads one image under,
///  which the option's refusal names.
use crate::markdown_lfs_context::MAX_IMAGE_BYTES_BOUND;
/// Import the JSONC model so numbers stay exact until their rule-specific conversion.
use monochromatic_jsonc_edit::{
    JsoncKind,
    JsoncValue,
};

/// What:
///  The exact identifiers of the currently specified built-in rules.
/// Why:
///  No package name,
///  executable or callback can add a runtime policy.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ruleIds = new Set([...]);
/// ```
pub const RULE_IDS: &[&str] = &[
    "rust/max-lines",
    "rust/require-rustdoc",
    "rust/require-explicit-types",
    "rust/tests-in-sidecars",
    "rust/avoidable-closure",
    "rust/folder-uses-links",
    "rust/file-order-in-folder",
    "markdown/heading-increment",
    "markdown/commands-show-output",
    "markdown/no-duplicate-heading",
    "markdown/single-h1",
    "markdown/no-trailing-punctuation",
    "markdown/no-bare-urls",
    "markdown/no-emphasis-as-heading",
    "markdown/fenced-code-language",
    "markdown/link-image-reference-definitions",
    "markdown/link-image-style",
    "markdown/no-pipe-tables",
    "markdown/semantic-line-breaks",
    "markdown/lfs-image-url",
    "shell/max-lines",
    "shell/no-pipe",
    "shell/no-and-chain",
    "shell/no-or-chain",
    "shell/no-pipe-stderr",
    "shell/no-semicolon",
    "shell/no-dollar-paren",
    "shell/no-backtick",
    "shell/no-ampersand",
];

/// What:
///  Convert an exact JSON number to an unsigned platform-sized line count.
/// Why:
///  usize matches source indexing;
///  fractional,
///  negative and overflowing counts are errors,
/// rather than rounded floating-point values.
///  u32 or u64 would impose a different indexing range.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lineLimit(value: JsoncValue): number;
/// ```
pub(crate) fn line_limit(value: &JsoncValue) -> Result<usize, ConfigError> {
    return count_limit("rust/max-lines", value);
}

/// What:
///  Convert an exact JSON number to an unsigned platform-sized count for the named rule's `max`.
/// Why:
///  `rust/max-lines` and the `shell/` rules share the number grammar,
///  and an error must name the rule whose `max` was wrong.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function countLimit(id: string, value: JsoncValue): number;
/// ```
pub(crate) fn count_limit(
    id: &str,
    value: &JsoncValue,
) -> Result<usize, ConfigError> {
    return count_option(id, "max", value);
}

/// What:
///  Convert an exact JSON number to an unsigned platform-sized count for one named option of one rule.
/// Why:
///  `max` and `max-image-bytes` share the number grammar,
///  and an error must name the option whose value was wrong.
///  A string such as `"300"` is refused rather than accepted as a number,
///  and a negative or fractional spelling is refused too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function countOption(id: string, name: string, value: JsoncValue): number;
/// ```
fn count_option(
    id: &str,
    name: &str,
    value: &JsoncValue,
) -> Result<usize, ConfigError> {
    return count_option_in(id, name, "line index", value);
}

/// What:
///  Convert an exact JSON number to the `max-image-bytes` bound,
///  refusing a value the one-byte probe past it cannot hold.
/// Why:
///  A name for the option and a unit for the number,
///  so a wrong byte count never says "line index";
///  and the check happens at configuration load,
///  so a release build cannot meet the wrapping arithmetic at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function imageByteBound(value: JsoncValue): bigint;
/// ```
pub(crate) fn image_byte_bound(value: &JsoncValue) -> Result<u64, ConfigError> {
    // What: the count the shared number grammar reads, before the probe past it is checked.
    // Why: strings, fractions, negatives and values over the platform's word are refused there.
    let bytes: usize = count_option_in(
        "markdown/lfs-image-url",
        "max-image-bytes",
        "byte count",
        value,
    )?;
    // What: the same count as the reader's own type, which is what it adds one to.
    // Why: the reader takes a `u64` bound and reads `bound + 1` bytes.
    let wide: u64 = bytes as u64;
    // What: the bound past which the reader's `+ 1` has no answer.
    // Why: wrapping there reads nothing in a release build, which hashes empty input.
    let largest: u64 = MAX_IMAGE_BYTES_BOUND;
    if wide > largest {
        return Err(ConfigError::new(
            format!(
                "markdown/lfs-image-url max-image-bytes must be at most {largest}: the rule reads one byte past the bound to notice an image that grew."
            )
            .as_str(),
        ));
    }
    return Ok(wide);
}

/// What:
///  Convert an exact JSON number to an unsigned platform-sized count,
///  naming the unit the count is measured in.
/// Why:
///  The same grammar carries line counts and byte counts,
///  and an error that calls a byte count a "line index" sends the reader to the wrong option.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function countOptionIn(id: string, name: string, unit: string, value: JsoncValue): number;
/// ```
fn count_option_in(
    id: &str,
    name: &str,
    unit: &str,
    value: &JsoncValue,
) -> Result<usize, ConfigError> {
    // What: Extract the exact-number payload from a borrowed value.
    // Why: A string such as "300" must not be silently accepted as a numeric option.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (value.kind !== 'number') throw new ConfigError('...');
    // ```
    let JsoncKind::Number {
        identity,
        ..
    } = &value.kind
    else {
        return Err(ConfigError::new(
            format!("{id} {name} must be a nonnegative integer.").as_str(),
        ));
    };
    if identity.is_negative() {
        return Err(ConfigError::new(
            format!("{id} {name} must be a nonnegative integer.").as_str(),
        ));
    }
    // Exact zero accepts every zero spelling, including -0 and 0e100000.
    if identity.is_zero() {
        return Ok(0);
    }
    // Parse the normalized coefficient; no floating-point conversion or rounding occurs.
    let coefficient_result = identity.digits().parse::<usize>();
    let exponent_result = identity.exponent().parse::<u32>();
    // What: Narrow both numeric parses together into their successful variants.
    // Why: A negative exponent represents a fraction after canonical trailing-zero removal.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // if (coefficientIsInteger && exponentIsUnsigned) { ... }
    // ```
    let (Ok(coefficient), Ok(exponent)) = (coefficient_result, exponent_result) else {
        return Err(ConfigError::new(
            format!("{id} {name} must be a nonnegative integer that fits this platform's {unit}.")
                .as_str(),
        ));
    };
    // Checked arithmetic returns absence on overflow instead of wrapping the limit.
    let Some(scale) = 10_usize.checked_pow(exponent) else {
        return Err(ConfigError::new(
            format!("{id} {name} exceeds this platform's {unit}.").as_str(),
        ));
    };
    let Some(result) = coefficient.checked_mul(scale) else {
        return Err(ConfigError::new(
            format!("{id} {name} exceeds this platform's {unit}.").as_str(),
        ));
    };
    return Ok(result);
}

/// What:
///  Validate one rule-settings object without filling in missing fields.
/// Why:
///  Later matching blocks can supply severity or override options before final resolution.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function validateSetting(id: string, setting: JsoncValue): void;
/// ```
pub(crate) fn validate_setting(
    id: &str,
    setting: &JsoncValue,
) -> Result<(), ConfigError> {
    // Borrow the rule name for registry lookup; membership does not allocate.
    if !RULE_IDS.contains(&id) {
        // The constructor keeps the identifier so startup composition can tell a stale binary
        // from a misspelled rule only when a repository pin measures the difference.
        return Err(ConfigError::unknown_built_in_rule(id));
    }
    let Some(entries) = setting.entries() else {
        return Err(ConfigError::new(
            format!("Rule {id} settings must be an object.").as_str(),
        ));
    };
    for entry in entries {
        // Decode the key and propagate invalid UTF-16 as a typed setup failure.
        let name = key_text(&entry.key.units)?;
        if name == "severity" {
            let severity = text(&entry.value, "severity")?;
            if severity != "off" && severity != "warn" && severity != "error" {
                return Err(ConfigError::new(
                    format!("Rule {id} severity must be off, warn, or error.").as_str(),
                ));
            }
            continue;
        }
        if (id == "rust/max-lines" || id.starts_with("shell/")) && name == "max" {
            // Validate the count now; final resolution reuses this conversion after merging.
            count_limit(id, &entry.value)?;
            continue;
        }
        if id == "markdown/lfs-image-url" && name == "max-image-bytes" {
            // Validate the image bound now, including the one-byte probe past it;
            // the typed rule settings read it once resolution has defaulted it.
            image_byte_bound(&entry.value)?;
            continue;
        }
        if id == "markdown/lfs-image-url" && name == "exclude" {
            strings(&entry.value, "exclude")?;
            continue;
        }
        if (id == "rust/folder-uses-links" || id == "rust/file-order-in-folder") && name == "allow"
        {
            // The path is read when the rule runs; an empty one names no file.
            if text(&entry.value, "allow")?.is_empty() {
                return Err(ConfigError::new(
                    format!(
                        "Rule {id} option allow must name the file that holds the findings it tolerates."
                    )
                    .as_str(),
                ));
            }
            continue;
        }
        if id == "rust/avoidable-closure" && name == "library" {
            // The path is read when the rule runs; an empty one can name no directory.
            if text(&entry.value, "library")?.is_empty() {
                return Err(ConfigError::new(
                    "Rule rust/avoidable-closure option library must not be empty.",
                ));
            }
            continue;
        }
        return Err(ConfigError::new(
            format!("Rule {id} has unknown option {name}.").as_str(),
        ));
    }
    return Ok(());
}

/// What:
///  Validate the names and option objects within one rules record.
/// Why:
///  Disabled rules are still checked,
///  so invalid settings do not become latent surprises.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function validateRules(rules: JsoncValue): void;
/// ```
pub(crate) fn validate_rules(rules: &JsoncValue) -> Result<(), ConfigError> {
    let Some(entries) = rules.entries() else {
        return Err(ConfigError::new("Configuration rules must be an object."));
    };
    for entry in entries {
        let name = key_text(&entry.key.units)?;
        validate_setting(name.as_str(), &entry.value)?;
    }
    return Ok(());
}
