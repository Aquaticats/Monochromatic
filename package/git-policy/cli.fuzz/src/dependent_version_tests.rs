//! What: Controls proving the dependent-version invariants are reached: generated manifests
//!       of every layout, raw texts reaching every rewrite outcome and the completeness
//!       branch, and specifier scans of every short token sequence.
//! Why: An invariant that is never reached cannot fail; these count what each view reaches.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (const data of samples) checkGeneratedManifest(generatedManifest(data));
//! ```

/// Import the generators and invariants under control.
use super::{
    GeneratedManifest, check_generated_manifest, check_imports, check_version_edit,
    generated_manifest, import_view, version_edit_view,
};

/// Every single-byte fill and a spread of mixed inputs rewrite to their expectation.
#[test]
fn generated_manifests_rewrite_to_their_expectation() {
    let mut decoys_before: usize = 0;
    let mut decoys_after: usize = 0;
    let mut spaced: usize = 0;
    for byte in 0..=u8::MAX {
        for data in [
            vec![byte; 20],
            (0..20)
                .map(|offset: u8| return byte.wrapping_mul(offset).wrapping_add(offset))
                .collect::<Vec<u8>>(),
            vec![
                byte,
                byte.wrapping_add(1),
                3,
                4,
                5,
                6,
                7,
                3,
                byte,
                byte.wrapping_mul(3),
                2,
                1,
                3,
                byte,
                5,
                8,
                2,
                byte,
            ],
        ] {
            let generated: GeneratedManifest = generated_manifest(data.as_slice());
            check_generated_manifest(&generated);
            // A member before the version puts another quote first; one after puts a comma after it.
            decoys_before +=
                usize::from(generated.before.find("\"version\"") != generated.before.find('"'));
            let literal: String = format!("\"{}\"", generated.from);
            decoys_after += usize::from(
                generated
                    .before
                    .split(literal.as_str())
                    .nth(1)
                    .is_some_and(|rest: &str| return rest.trim_start().starts_with(',')),
            );
            spaced += usize::from(generated.before.contains('\n'));
        }
    }
    assert!(
        decoys_before > 100 && decoys_after > 100 && spaced > 100,
        "{decoys_before} {decoys_after} {spaced}"
    );
}

/// Raw texts reach success, every failure, and the completeness branch both ways.
#[test]
fn raw_texts_reach_every_rewrite_outcome() {
    let cases: &[(&str, Option<bool>)] = &[
        ("{\"name\":\"a\",\"version\":\"1.0.0\"}", Some(true)),
        ("{\"name\":\"a\",\"version\" :\n \"1.0.0\"}", Some(true)),
        ("{\"name\":\"a\",\"version\":\"2.0.0\"}", Some(false)),
        (
            "{\"name\":\"a\",\"nested\":{\"version\":\"1.0.0\"}}",
            Some(false),
        ),
        ("{\"name\":\"a\",\"version\":1}", None),
        ("{\"name\":\"a", None),
        ("{\"name\":\"a\",\"vers\\u0069on\":\"1.0.0\"}", Some(false)),
        ("{\"name\":\"a\",\"version\":\"1.0.\\u0030\"}", Some(false)),
        (
            "{\"name\":\"a\",\"version\":\"0.9.0\",\"version\":\"1.0.0\"}",
            Some(false),
        ),
        ("[{\"version\":\"1.0.0\"}]", None),
        ("{\"version\":\"1.0.0\"", None),
        ("", None),
    ];
    for (text, plain) in cases {
        assert_eq!(check_version_edit(text, "1.0.0", "1.0.1"), *plain, "{text}");
    }
    assert_eq!(
        version_edit_view(b"a\0b\0{}"),
        (String::from("a"), String::from("b"), String::from("{}"))
    );
    assert_eq!(
        version_edit_view(b"{}"),
        (
            String::from("1.0.0"),
            String::from("1.0.1"),
            String::from("{}")
        )
    );
}

/// Every token sequence up to four long rewrites without breaking an invariant.
#[test]
fn short_token_sequences_hold_the_rewrite_invariants() {
    let tokens: &[&str] = &[
        "{",
        "}",
        "[",
        "]",
        "\"",
        "\\",
        ":",
        ",",
        "\"version\"",
        "\"1.0.0\"",
        " ",
    ];
    let mut texts: Vec<String> = vec![String::new()];
    for _ in 0..4 {
        texts = texts
            .iter()
            .flat_map(|prefix: &String| {
                return tokens
                    .iter()
                    .map(move |token: &&str| return format!("{prefix}{token}"));
            })
            .collect();
        for text in &texts {
            let _ = check_version_edit(text, "1.0.0", "1.0.1");
        }
    }
    assert_eq!(texts.len(), 14_641);
}

/// Every sequence of up to four scan tokens agrees with the reference, with both results
/// reached, for a scoped name, an empty name and a name with a quote.
#[test]
fn short_token_sequences_scan_like_the_reference() {
    let tokens: &[&str] = &[
        "from", "import", "require", "(", " ", "\n", "'", "\"", "`", "/", "@s/b", "x", ";",
    ];
    let mut found: [usize; 2] = [0, 0];
    let mut texts: Vec<String> = vec![String::new()];
    for _ in 0..4 {
        texts = texts
            .iter()
            .flat_map(|prefix: &String| {
                return tokens
                    .iter()
                    .map(move |token: &&str| return format!("{prefix}{token}"));
            })
            .collect();
        for text in &texts {
            for name in ["@s/b", "", "a'b"] {
                found[usize::from(check_imports(text, name))] += 1;
            }
        }
    }
    assert!(found[0] > 10_000 && found[1] > 200, "{found:?}");
    assert_eq!(
        import_view(b"@s/b\0import '@s/b'"),
        (String::from("@s/b"), String::from("import '@s/b'"))
    );
    assert_eq!(import_view(b"@s/b"), (String::from("@s/b"), String::new()));
}
