//! What: Patch-bump a plain `major.minor.patch` release version, and quote text the way
//!       `JSON.stringify` does.
//! Why: A dependent receives a patch bump. Only a plain release can be bumped
//!      automatically; anything else (a prerelease, build metadata, a leading zero, a
//!      missing component) is reported for a bump by hand, with the incumbent's message
//!      (`dependent-version-bump.ts:87-177`). The patch component is incremented as a
//!      decimal digit string, so it is exact at any size.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // patchBumpVersion({ name, version }); JSON.stringify(text);
//! ```

/// What: The UTF-16 code unit of `.`.
/// Why:  Versions are kept as code units; components are split on this unit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const DOT = '.'.charCodeAt(0);
/// ```
const DOT: u16 = 0x2e;

/// What: A dependent whose version cannot be bumped automatically. `Vec<u16>` keeps the
///       version as the code units JavaScript holds.
/// Why:  The policy turns this into a `dependent-version-unsupported` finding without a
///       patch, and the plan stops, exactly as the incumbent's throw stops it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class UnsupportedVersionError extends Error { name: string; version: string }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnsupportedVersion {
    /// The dependent's package name.
    pub name: String,
    /// The version found in its manifest.
    pub version: Vec<u16>,
}

/// What: The incumbent's message for an unsupported version, verbatim.
/// Why:  The finding text tells the person to bump that package by hand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${name} has version ${JSON.stringify(version)}, which is not a plain major.minor.patch release; bump it by hand in the same commit.`
/// ```
pub fn unsupported_message(unsupported: &UnsupportedVersion) -> String {
    return format!(
        "{} has version {}, which is not a plain major.minor.patch release; bump it by hand in the same commit.",
        unsupported.name,
        json_quote_units(&unsupported.version)
    );
}

/// What: One automatic patch bump: the version before and after.
/// Why:  Both appear in the finding message and the text edit.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ReleaseBump = { from: string; to: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReleaseBump {
    /// The current version.
    pub from: String,
    /// The version with the patch component increased by one.
    pub to: String,
}

/// What: Whether one component is a non-empty run of ASCII digits without a leading zero.
///       `&[u16]` borrows code units.
/// Why:  Semantic versioning's numeric identifiers; `0` alone is allowed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isReleaseComponent(text: string): boolean;
/// ```
fn is_release_component(component: &[u16]) -> bool {
    let digits: bool = !component.is_empty()
        && component
            .iter()
            .all(|unit: &u16| return (u16::from(b'0')..=u16::from(b'9')).contains(unit));
    return digits && !(component.len() > 1 && component[0] == u16::from(b'0'));
}

/// What: Add one to a decimal digit string. `.rev()` walks from the last digit.
/// Why:  An exact increment at any size; the incumbent's `Number(component) + 1` rounds
///       above 2^53 and prints exponent notation from 10^21.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function incrementDecimal(digits: string): string;
/// ```
fn increment_decimal(digits: &[u16]) -> String {
    let mut result: Vec<u8> = Vec::new();
    let mut carry: bool = true;
    for unit in digits.iter().rev() {
        // Every unit is an ASCII digit here, so the narrowing keeps its value.
        let digit: u8 = u8::try_from(*unit).unwrap_or(b'0');
        if carry && digit == b'9' {
            result.push(b'0');
        } else if carry {
            result.push(digit + 1);
            carry = false;
        } else {
            result.push(digit);
        }
    }
    if carry {
        result.push(b'1');
    }
    // `.rev()` restores most-significant-first order; `char::from` widens each byte.
    return result
        .iter()
        .rev()
        .map(|byte: &u8| return char::from(*byte))
        .collect();
}

/// What: Bump the patch component of a plain release version.
/// Why:  Only plain releases are bumped automatically.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function patchBumpVersion({ name, version }): string; // throws UnsupportedVersionError
/// ```
pub fn patch_bump_version(name: &str, version: &[u16]) -> Result<ReleaseBump, UnsupportedVersion> {
    let components: Vec<&[u16]> = version.split(|unit: &u16| return *unit == DOT).collect();
    let [major, minor, patch] = components.as_slice() else {
        return Err(UnsupportedVersion {
            name: String::from(name),
            version: version.to_vec(),
        });
    };
    if !(is_release_component(major) && is_release_component(minor) && is_release_component(patch))
    {
        return Err(UnsupportedVersion {
            name: String::from(name),
            version: version.to_vec(),
        });
    }
    // `String::from_utf16_lossy` is exact here: every unit is an ASCII digit or a dot.
    return Ok(ReleaseBump {
        from: String::from_utf16_lossy(version),
        to: format!(
            "{}.{}.{}",
            String::from_utf16_lossy(major),
            String::from_utf16_lossy(minor),
            increment_decimal(patch)
        ),
    });
}

/// What: Quote code units as `JSON.stringify` quotes a string. `char::decode_utf16` pairs
///       surrogates and yields an error for each unpaired one.
/// Why:  Messages and the version edit compare against the incumbent's quoting: `"`, `\`
///       and the short control escapes are escaped, other units below U+0020 and unpaired
///       surrogates become lower-case `\u` escapes, and everything else stays literal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const quoted = JSON.stringify(text);
/// ```
pub fn json_quote_units(units: &[u16]) -> String {
    let mut quoted: String = String::from("\"");
    for decoded in char::decode_utf16(units.iter().copied()) {
        match decoded {
            Ok('"') => quoted.push_str("\\\""),
            Ok('\\') => quoted.push_str("\\\\"),
            Ok('\u{8}') => quoted.push_str("\\b"),
            Ok('\u{c}') => quoted.push_str("\\f"),
            Ok('\n') => quoted.push_str("\\n"),
            Ok('\r') => quoted.push_str("\\r"),
            Ok('\t') => quoted.push_str("\\t"),
            Ok(control) if control < ' ' => {
                quoted.push_str(&format!("\\u{:04x}", u32::from(control)));
            }
            Ok(literal) => quoted.push(literal),
            Err(unpaired) => {
                quoted.push_str(&format!("\\u{:04x}", unpaired.unpaired_surrogate()));
            }
        }
    }
    quoted.push('"');
    return quoted;
}

/// Release checks, exact increments and quoting.
#[cfg(test)]
#[path = "dependent_version_release_tests.rs"]
mod tests;
