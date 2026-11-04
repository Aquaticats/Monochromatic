//! What: The byte-preserving version rewrite, with every `replaceManifestVersion` case of
//!       `manifest-text.unit.test.ts` and the scanner's depth, escape and resume rules.
//! Why: The patch is the whole file, so one wrong byte is a wrong commit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(replaceManifestVersion({ path, text, from: '1.0.0', to: '1.0.1' })).toBe('...');
//! ```

/// The rewrite and the failure type.
use super::replace_manifest_version;
use crate::dependent_version_content::PolicyIncomplete;

/// The path every case names.
const PATH: &[u8] = b"package.json";

/// Rewrite `1.0.0` to `1.0.1`.
fn rewrite(text: &str) -> Result<String, PolicyIncomplete> {
    return replace_manifest_version(PATH, text, "1.0.0", "1.0.1");
}

/// The shape failure with the incumbent's message.
fn problem(message: &str) -> Result<String, PolicyIncomplete> {
    return Err(PolicyIncomplete::ManifestShape {
        path: PATH.to_vec(),
        problem: String::from(message),
    });
}

/// Ported: only the top-level version value changes and formatting is kept.
#[test]
fn changes_only_the_top_level_version_and_keeps_formatting() {
    let text: &str = "{\n  \"name\": \"a\",\n  \"description\": \"version \\\"x\\\"\",\n  \"publishConfig\": { \"version\": \"9.9.9\" },\n  \"version\" :  \"1.0.0\",\n  \"tail\": [\"version\"]\n}\n";
    assert_eq!(
        rewrite(text),
        Ok(String::from(
            "{\n  \"name\": \"a\",\n  \"description\": \"version \\\"x\\\"\",\n  \"publishConfig\": { \"version\": \"9.9.9\" },\n  \"version\" :  \"1.0.1\",\n  \"tail\": [\"version\"]\n}\n"
        ))
    );
}

/// Ported: a different version, no top-level version, a non-string one, an open literal.
#[test]
fn rejects_the_incumbent_cases() {
    assert_eq!(
        rewrite("{\"version\":\"2.0.0\"}"),
        problem("package.json declares version \"2.0.0\", expected \"1.0.0\"")
    );
    assert_eq!(
        rewrite("{\"name\":\"a\",\"nested\":{\"version\":\"1.0.0\"}}"),
        problem("package.json has no top-level \"version\"")
    );
    assert_eq!(
        rewrite("{\"version\":1}"),
        problem("package.json has a non-string top-level \"version\"")
    );
    assert_eq!(
        rewrite("{\"name\":\"a"),
        problem("manifest ends inside a string literal")
    );
}

/// Arrays and objects before the key change the depth and restore it.
#[test]
fn tracks_depth_through_arrays_and_objects() {
    assert_eq!(
        rewrite("{\"a\":[\"x\"],\"version\":\"1.0.0\"}"),
        Ok(String::from("{\"a\":[\"x\"],\"version\":\"1.0.1\"}"))
    );
    assert_eq!(
        rewrite("{\"a\":[{\"version\":\"1.0.0\"}],\"version\":\"1.0.0\"}"),
        Ok(String::from(
            "{\"a\":[{\"version\":\"1.0.0\"}],\"version\":\"1.0.1\"}"
        ))
    );
    assert_eq!(
        rewrite("{\"a\":{\"b\":\"c\"},\"version\":\"1.0.0\"}"),
        Ok(String::from("{\"a\":{\"b\":\"c\"},\"version\":\"1.0.1\"}"))
    );
    assert_eq!(
        rewrite("{\"a\":{},\"version\":\"1.0.0\"}"),
        Ok(String::from("{\"a\":{},\"version\":\"1.0.1\"}"))
    );
    assert_eq!(
        rewrite("[{\"version\":\"1.0.0\"}]"),
        problem("package.json has no top-level \"version\"")
    );
}

/// Whitespace around the colon, a string value spelled `version`, and a key at the end.
#[test]
fn reads_keys_by_the_colon_after_them() {
    assert_eq!(
        rewrite("{\"version\"\t:\r\n \"1.0.0\" }"),
        Ok(String::from("{\"version\"\t:\r\n \"1.0.1\" }"))
    );
    assert_eq!(
        rewrite("{\"a\":\"version\",\"version\":\"1.0.0\"}"),
        Ok(String::from("{\"a\":\"version\",\"version\":\"1.0.1\"}"))
    );
    assert_eq!(
        rewrite("{\"version\""),
        problem("package.json has no top-level \"version\"")
    );
    assert_eq!(
        rewrite("{\"version\":"),
        problem("package.json has a non-string top-level \"version\"")
    );
}

/// The key and the value are compared by their raw spelling.
#[test]
fn compares_raw_spellings() {
    assert_eq!(
        rewrite("{\"vers\\u0069on\":\"1.0.0\"}"),
        problem("package.json has no top-level \"version\"")
    );
    assert_eq!(
        rewrite("{\"version\":\"1.0.\\u0030\"}"),
        problem("package.json declares version \"1.0.\\u0030\", expected \"1.0.0\"")
    );
    assert_eq!(
        rewrite("{\"version\":\"0.9.0\",\"version\":\"1.0.0\"}"),
        problem("package.json declares version \"0.9.0\", expected \"1.0.0\"")
    );
    assert_eq!(
        replace_manifest_version(PATH, "{\"version\":\"a\\\"b\"}", "a\"b", "c\nd"),
        Ok(String::from("{\"version\":\"c\\nd\"}"))
    );
}

/// Backslashes hide the next byte; text outside the value is kept byte for byte.
#[test]
fn honours_escapes_and_keeps_other_text() {
    assert_eq!(
        rewrite("{\"a\":\"x\\\\\",\"version\":\"1.0.0\"}"),
        Ok(String::from("{\"a\":\"x\\\\\",\"version\":\"1.0.1\"}"))
    );
    assert_eq!(
        rewrite("{\"a\":\"\\"),
        problem("manifest ends inside a string literal")
    );
    assert_eq!(
        rewrite("{\"version\":\"1.0"),
        problem("manifest ends inside a string literal")
    );
    assert_eq!(
        rewrite("{\"d\":\"\u{e9}\u{1f600}\",\"version\":\"1.0.0\"}\r\n"),
        Ok(String::from(
            "{\"d\":\"\u{e9}\u{1f600}\",\"version\":\"1.0.1\"}\r\n"
        ))
    );
}
