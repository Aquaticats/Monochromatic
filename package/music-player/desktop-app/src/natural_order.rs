//! Natural order for file and folder names: `Track 2` comes before `Track 10`.
//!
//! What: `natural_cmp` compares two names so that a run of digits counts by its number, not by its characters, and
//! letters compare without regard to case.
//! Why: the folder walk sorted names by their bytes, so `Loose 1`, `Loose 10`, `Loose 11`, `Loose 2` played in that
//! order. A person reads the number, so the walk now sorts by it (the polish review finding LS5).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! names.sort(new Intl.Collator(undefined, { numeric: true, sensitivity: "base" }).compare);
//! ```

use std::cmp::Ordering;

/// What:
///      `fn digit_run(text: &str) -> (&str, &str)`.
///  Splits `text` into its leading run of ASCII digits and the rest.
/// Why:
///       Both sides of a comparison are cut at the same kind of boundary, so a run can be compared as a whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const digitRun = (text: string): [string, string] => { const m = /^\d*/.exec(text)!; return [m[0], text.slice(m[0].length)]; };
/// ```
fn digit_run(text: &str) -> (&str, &str) {
    let end: usize = text.find(|character: char| return !character.is_ascii_digit()).unwrap_or(text.len());
    return text.split_at(end);
}

/// What:
///      `fn compare_digit_runs(left: &str, right: &str) -> Ordering`.
///  Orders two runs of ASCII digits by their numeric value, however long they are.
/// Why:
///       Leading zeros are dropped and then a longer run is a bigger number, so a run of fifty digits needs no integer
///  type and cannot overflow.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const compareDigitRuns = (a: string, b: string) => { const x = a.replace(/^0+/, ""), y = b.replace(/^0+/, ""); return x.length - y.length || (x < y ? -1 : x > y ? 1 : 0); };
/// ```
fn compare_digit_runs(left: &str, right: &str) -> Ordering {
    let left_trimmed: &str = left.trim_start_matches('0');
    let right_trimmed: &str = right.trim_start_matches('0');
    return left_trimmed
        .len()
        .cmp(&right_trimmed.len())
        .then_with(|| return left_trimmed.cmp(right_trimmed));
}

/// What:
///      `pub fn natural_cmp(left: &str, right: &str) -> Ordering`.
///  Compares two names in natural order: digit runs by value, other characters by their lowercase form, and, when
///  everything so far is equal, by the plain bytes of the names.
/// Why:
///       The final byte comparison makes the order total, so two different names never tie and a sort gives one answer
///  on every run. `a2` and `a02` are equal by value, so the bytes decide (`a02` first).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function naturalCmp(a: string, b: string): number {
///   // walk both strings; compare digit runs numerically and other characters lowercased
///   // ...
///   return a < b ? -1 : a > b ? 1 : 0; // last resort
/// }
/// ```
pub fn natural_cmp(left: &str, right: &str) -> Ordering {
    let mut left_rest: &str = left;
    let mut right_rest: &str = right;
    loop {
        match (left_rest.chars().next(), right_rest.chars().next()) {
            (None, None) => return left.cmp(right),
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(left_character), Some(right_character)) => {
                if left_character.is_ascii_digit() && right_character.is_ascii_digit() {
                    let (left_digits, left_after): (&str, &str) = digit_run(left_rest);
                    let (right_digits, right_after): (&str, &str) = digit_run(right_rest);
                    let by_value: Ordering = compare_digit_runs(left_digits, right_digits);
                    if by_value != Ordering::Equal {
                        return by_value;
                    }
                    left_rest = left_after;
                    right_rest = right_after;
                    continue;
                }
                let by_character: Ordering = left_character
                    .to_lowercase()
                    .cmp(right_character.to_lowercase());
                if by_character != Ordering::Equal {
                    return by_character;
                }
                left_rest = &left_rest[left_character.len_utf8()..];
                right_rest = &right_rest[right_character.len_utf8()..];
            }
        }
    }
}

#[cfg(test)]
#[path = "natural_order_tests.rs"]
mod tests;
