//! Three-casing of one literal run under an inline `(?i)` scope.
//!
//! What:
//!  turns one keyword run into its distinct lowercase,
//!  per-run-Capitalized,
//!  and UPPERCASE
//! spellings,
//!  and recases a run into any one of the three while copying escapes intact.
//!  Why:
//!  matching only the three consistent shapes is the decided approximation of
//! case-insensitivity,
//!  so a mixed-case literal such as `AdOBe_` never matches;
//!  an escape such as `\_` or `\x60` must never have its bytes recased.
//!  The flag-scope walker that decides when a run is under the flag lives in the
//! parent `caseexpand.rs` module.

/// Selects which casing `recase` applies to a literal run.
enum CaseMode {
    /// Every letter lowercase.
    Lower,
    /// First letter of each alphabetic run uppercase,
    ///  the rest lowercase.
    Cap,
    /// Every letter uppercase.
    Upper,
}

/// Builds the non-capturing three-casing alternation of a literal run.
///
/// What:
///  the distinct lowercase,
///  per-run-Capitalized,
///  and UPPERCASE forms joined with `|` inside
/// `(?:...)`;
///  a run with no letter (all three forms equal) is returned bare.
///  Why:
///  matching only
/// the three consistent shapes is the decided approximation of case-insensitivity.
pub(super) fn three_case(run: &str) -> String {
    let mut forms: Vec<String> = Vec::new();
    for form in [
        recase(run, CaseMode::Lower),
        recase(run, CaseMode::Cap),
        recase(run, CaseMode::Upper),
    ] {
        if !forms.contains(&form) {
            forms.push(form);
        }
    }
    if forms.len() == 1 {
        return forms.into_iter().next().unwrap_or_default();
    }
    return format!("(?:{})", forms.join("|"));
}

/// Applies one casing to a literal run,
///  casing bare ASCII letters and copying escapes intact.
///
/// What:
///  lowercases,
///  uppercases,
///  or Capitalizes (first letter of each alphabetic run upper,
///  rest
/// lower);
///  a backslash escape and any non-letter break the alphabetic run and pass through.
///  Why:
/// an escape such as `\_` or `\x60` must never have its bytes recased.
fn recase(
    run: &str,
    mode: CaseMode,
) -> String {
    let b = run.as_bytes();
    let mut out = String::new();
    let mut fresh = true;
    let mut i = 0;
    while i < b.len() {
        if b[i] == b'\\' {
            out.push('\\');
            if i + 1 < b.len() {
                out.push(b[i + 1] as char);
            }
            fresh = true;
            i += 2;
            continue;
        }
        let c = b[i];
        if c.is_ascii_alphabetic() {
            let cased = match mode {
                CaseMode::Lower => c.to_ascii_lowercase(),
                CaseMode::Upper => c.to_ascii_uppercase(),
                CaseMode::Cap if fresh => c.to_ascii_uppercase(),
                CaseMode::Cap => c.to_ascii_lowercase(),
            };
            out.push(cased as char);
            fresh = false;
        } else {
            out.push(c as char);
            fresh = true;
        }
        i += 1;
    }
    return out;
}
