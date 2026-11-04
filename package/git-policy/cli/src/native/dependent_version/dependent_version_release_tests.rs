//! What: Release checks, exact increments and `JSON.stringify` quoting, with every
//!       `patchBumpVersion` case of `dependent-version-bump.unit.test.ts`.
//! Why: A wrong increment or a wrong quote changes a patch or a finding message.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(patchBumpVersion({ name: 'a', version: '1.2.9' })).toBe('1.2.10');
//! ```

/// The functions under test.
use super::{
    ReleaseBump, UnsupportedVersion, json_quote_units, patch_bump_version, unsupported_message,
};

/// Code units of text.
fn units(text: &str) -> Vec<u16> {
    return text.encode_utf16().collect();
}

/// The bumped version, or the unsupported one.
fn bump(version: &str) -> Result<String, Vec<u16>> {
    return patch_bump_version("@scope/a", &units(version))
        .map(|done: ReleaseBump| {
            assert_eq!(done.from, version);
            return done.to;
        })
        .map_err(|unsupported: UnsupportedVersion| {
            assert_eq!(unsupported.name, "@scope/a");
            return unsupported.version;
        });
}

/// Ported: the patch component is incremented numerically.
#[test]
fn increments_the_patch_component() {
    assert_eq!(bump("1.2.3"), Ok(String::from("1.2.4")));
    assert_eq!(bump("0.0.9"), Ok(String::from("0.0.10")));
    assert_eq!(bump("0.0.0"), Ok(String::from("0.0.1")));
    assert_eq!(bump("10.20.199"), Ok(String::from("10.20.200")));
    assert_eq!(bump("1.0.909"), Ok(String::from("1.0.910")));
}

/// The increment is exact beyond 2^53 and beyond 10^21.
#[test]
fn increments_exactly_at_any_size() {
    assert_eq!(
        bump("1.0.9007199254740993"),
        Ok(String::from("1.0.9007199254740994"))
    );
    assert_eq!(
        bump("1.0.999999999999999999999"),
        Ok(String::from("1.0.1000000000000000000000"))
    );
}

/// Ported: prereleases, build metadata, leading zeros, short and non-numeric versions.
#[test]
fn rejects_versions_that_are_not_plain_releases() {
    for version in ["1.0.0-alpha.1", "1.0.0+build", "01.0.0", "1.0", "1.0.x", ""] {
        assert_eq!(bump(version), Err(units(version)), "{version}");
    }
}

/// Every component is checked: empty, leading zero, extra, and digits outside ASCII.
#[test]
fn checks_every_component() {
    for version in [
        "1..0",
        ".1.0",
        "1.0.",
        "1.01.0",
        "1.0.01",
        "1.0.0.0",
        "1.0.\u{663}",
        "1./.0",
        "1.:.0",
    ] {
        assert_eq!(bump(version), Err(units(version)), "{version}");
    }
    assert_eq!(bump("1.0.0"), Ok(String::from("1.0.1")));
    assert_eq!(bump("10.0.0"), Ok(String::from("10.0.1")));
}

/// The incumbent's message, with the version quoted as `JSON.stringify` quotes it.
#[test]
fn states_the_unsupported_message() {
    assert_eq!(
        unsupported_message(&UnsupportedVersion {
            name: String::from("@s/runtime"),
            version: units("2.0.0-rc.1"),
        }),
        "@s/runtime has version \"2.0.0-rc.1\", which is not a plain major.minor.patch release; \
         bump it by hand in the same commit."
    );
}

/// `JSON.stringify` escapes, lower-case `\u` escapes, and literal text.
#[test]
fn quotes_as_json_stringify() {
    assert_eq!(json_quote_units(&units("a\"b\\c")), "\"a\\\"b\\\\c\"");
    assert_eq!(
        json_quote_units(&units("\u{8}\u{c}\n\r\t")),
        "\"\\b\\f\\n\\r\\t\""
    );
    assert_eq!(
        json_quote_units(&units("\u{1}\u{1f} ")),
        "\"\\u0001\\u001f \""
    );
    assert_eq!(
        json_quote_units(&units("\u{7f}\u{2028}\u{1f600}")),
        "\"\u{7f}\u{2028}\u{1f600}\""
    );
    assert_eq!(
        json_quote_units(&[0xd800, 0x41, 0xdc0f]),
        "\"\\ud800A\\udc0f\""
    );
    assert_eq!(json_quote_units(&[]), "\"\"");
}
