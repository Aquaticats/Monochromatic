//! What:
//!  Deterministic bounded fuzzing of native extraction and grouped projection.
//! Why:
//!  Randomized delimiter,
//!  prefix,
//!  newline and Unicode combinations exercise the same consumer seam.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // Seeded PRNG -> host -> native extraction -> safe edit or explicit refusal -> host reparse.
//! ```

/// Original-host mapping uses the real finding helper.
use super::finding;
/// Import the actual processor and production host edit implementation.
use super::{Edit, Fix, ProcessorLanguage};
/// Exercise exact original-host application,
///  not a test-owned replacement algorithm.
use crate::edits::apply_fixes;

/// Advance a fixed-size unsigned seed without arithmetic overflow panics.
fn random(seed: &mut u32) -> usize {
    // u32 fixes the generator width across platforms; usize/u64/i32/i64 would change its sequence.
    *seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    return *seed as usize;
}

/// Bound every generated source and iteration count so fuzz verification fits the owning container.
#[test]
fn processors_seeded_fuzz_exercises_delimiters_unicode_prefixes_and_atomic_groups() {
    let alphabet: [&str; 14] = [
        "a", "😀", "α", "'", "\"", "\\", "# ", "##", "*", "/", "`", "~", "../", ";|&",
    ];
    let endings: [&str; 3] = ["\n", "\r\n", "\r"];
    let mut seed: u32 = 0x91a2_b3c4;
    let mut mapped: usize = 0;
    let mut refused: usize = 0;
    let mut accepted: usize = 0;
    for _ in 0..1_024 {
        let ending: &str = endings[random(&mut seed) % endings.len()];
        let mut text: String = String::new();
        for _ in 0..12 {
            text.push_str(alphabet[random(&mut seed) % alphabet.len()]);
        }
        let mode: usize = random(&mut seed) % 4;
        let prefix: &str = if mode == 0 {
            "> "
        } else if mode == 1 {
            "> > "
        } else {
            ""
        };
        let fenced: String = format!(
            "{prefix}~~~~rust,no_run{ending}{prefix}//! {text}.{ending}{prefix}# let value: u32 = 1;{ending}{prefix}~~~~{ending}"
        );
        let (host, language): (String, ProcessorLanguage) = if mode == 3 {
            let mut rustdoc: String = String::new();
            for line in crate::processors_lines::physical_lines(fenced.as_str()) {
                rustdoc.push_str("/// ");
                rustdoc.push_str(&fenced[line.start..line.end]);
            }
            rustdoc.push_str("fn item() {}");
            (rustdoc, ProcessorLanguage::Rust)
        } else {
            (fenced, ProcessorLanguage::Markdown)
        };
        let virtuals = super::extract(String::from("fuzz-host"), host.clone(), language)
            .expect("bounded authored host");
        let rust = virtuals
            .iter()
            .find(|input| return input.language() == ProcessorLanguage::Rust)
            .expect("Rust fence selected");
        let at: usize = rust.source().find("= 1").expect("authored hidden literal") + 2;
        let diagnostic = rust
            .project_diagnostic(finding(rust, at, 1))
            .expect("diagnostic mapping")
            .expect("authored");
        assert_eq!(diagnostic.filename, "fuzz-host");
        assert_eq!(
            diagnostic.labels[0].span.offset,
            host.find("= 1").expect("original") + 2
        );
        mapped += 1;
        let safe: Fix = rust
            .project_fix(&Fix {
                edits: vec![Edit {
                    start: at,
                    end: at + 1,
                    replacement: String::from("2"),
                }],
            })
            .expect("safe atomic fix");
        let changed = apply_fixes(host.as_str(), &[safe])
            .expect("host fix")
            .source;
        assert_eq!(changed, host.replace("= 1", "= 2"));
        let adversarial: &str =
            ["~~~~\n", "*/", "\n# let injected: u32 = 2;", "\n", "😀"][random(&mut seed) % 5];
        let candidate: Fix = Fix {
            edits: vec![
                Edit {
                    start: at,
                    end: at + 1,
                    replacement: String::from("3"),
                },
                Edit {
                    start: at + 1,
                    end: at + 1,
                    replacement: String::from(adversarial),
                },
            ],
        };
        if let Ok(projected) = rust.project_fix(&candidate) {
            let actual = apply_fixes(host.as_str(), &[projected])
                .expect("safe group")
                .source;
            super::extract(String::from("fuzz-host"), actual, language)
                .expect("accepted container reparses");
            accepted += 1;
        } else {
            // Refusal leaves the original owned host snapshot untouched.
            assert!(host.contains("= 1"));
            refused += 1;
        }
    }
    assert_eq!(mapped, 1_024);
    assert_eq!(accepted + refused, 1_024);
    assert!(accepted > 0);
    assert!(refused > 0);
}
