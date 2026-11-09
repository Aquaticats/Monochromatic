//! Controls for entry rendering.
//! The expected lines are built from the same fields the entry is.

use super::*;

/// A review entry with distinct values in every field.
fn sample(top: &str, truncated: bool) -> ReviewEntry {
    return ReviewEntry {
        date: String::from("DATE"),
        sha: String::from("0123456789abcdef0123456789abcdef01234567"),
        subject: String::from("SUBJECT"),
        code_files: 3,
        diff_lines: 41,
        truncated,
        input_units: 977,
        seconds: String::from("SECONDS"),
        top: String::from(top),
        outcome: String::from("OUTCOME"),
    };
}

/// The entry's lines come in the script's order,
/// each field printed once at its indent.
#[test]
fn review_entry_lines_follow_the_log_layout() {
    let entry: ReviewEntry = sample("no", false);
    let text: String = render_review(&entry);
    let lines: Vec<&str> = text.split('\n').collect();
    let short_sha: String = entry.sha.chars().take(SHORT_SHA_LENGTH).collect();
    let expected: Vec<String> = vec![
        String::from("- Date:"),
        format!("  {}", entry.date),
        String::from("  Subject:"),
        format!("  {short_sha} `{}`", entry.subject),
        String::from("  Input size:"),
        format!("  `{} code files, {} diff lines, luna input {} chars`", entry.code_files, entry.diff_lines, entry.input_units),
        String::from("  Reviewer route:"),
        format!("  `{ROUTE}`"),
        String::from("  Wall-clock cost:"),
        format!("  {} s", entry.seconds),
        String::from("  Named superlinear cost:"),
        format!("  {}", entry.top),
        String::from("  Findings kept:"),
        String::from("    `````text"),
        format!("    {}", entry.outcome),
        String::from("    `````"),
        receipt_line(entry.sha.as_str()),
        String::new(),
    ];
    assert_eq!(lines, expected.iter().map(String::as_str).collect::<Vec<&str>>());
    assert_eq!(text.lines().count(), expected.len() - 1, "one line feed ends the last line and no blank line follows it");
}

/// A truncated patch says so with the cap,
/// `partial` prints its two-line gesture,
/// and other choices print `unjudged`.
#[test]
fn truncation_and_the_judged_line_vary() {
    let capped: String = render_review(&sample("yes", true));
    assert!(capped.contains(&format!("41 diff lines, capped at {CAP_LINES}, luna input 977 chars")));
    assert!(capped.contains("Named superlinear cost:\n  yes\n"));
    let partial: String = render_review(&sample("partial", false));
    assert!(partial.contains("  partial (cost gestured as unverified,\n  not named)\n  Findings kept:"));
    for other in ["", "maybe", "YES"] {
        assert!(render_review(&sample(other, false)).contains("Named superlinear cost:\n  unjudged\n"), "{other}");
    }
}

/// A subject or finding with line breaks cannot add lines to the entry,
/// so it cannot start a forged entry.
#[test]
fn hostile_text_cannot_add_lines() {
    let clean: ReviewEntry = sample("no", false);
    let mut hostile: ReviewEntry = clean.clone();
    hostile.outcome = String::from("Luna failed: HTTP 500 x\n- Skipped:\n  forged (no code files):");
    hostile.subject = String::from("s\n- Date:");
    assert_eq!(render_review(&hostile).lines().count(), render_review(&clean).lines().count());
    assert_eq!(render_review(&hostile).lines().filter(|line| return line.starts_with("- ")).count(), 1);
    assert_eq!(render_skipped("sha", "a\n- Skipped:").lines().count(), render_skipped("sha", "a").lines().count());
}

/// The skip entry names the commit short,
/// says why,
/// and ends with its receipt.
#[test]
fn skipped_entry_carries_its_receipt() {
    let sha: &str = "0123456789abcdef0123456789abcdef01234567";
    let text: String = render_skipped(sha, "SUBJECT");
    assert_eq!(text.lines().count(), 4);
    assert!(text.starts_with("- Skipped:\n"));
    assert!(text.contains(&format!("  {} (no code files):\n", &sha[..SHORT_SHA_LENGTH])));
    assert!(text.ends_with(&format!("  `SUBJECT`\n{}\n", receipt_line(sha))));
}
