//! What:
//!  The finding text one run emits:
//!  `PATH:LINE rule=N`,
//!  columnless,
//!  with the rule's own identity token and the matched text left out.
//! Why:
//!  CI parses that shape,
//!  and the redaction guarantee is why a rule body itself may be a secret;
//!  a column range or a quoted match would be a leak surface.
//!  The three cases cover a regex rule,
//!  a literal rule on a later line,
//!  and the word-boundary gate of a short literal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // finding-shape.test.ts: `path:line rule=token`, nothing else
//! ```

use std::fs;

use crate::{
    BIN,
    Command,
    unique_tmp,
};

// What:     A user regex rule in the engine dialect
//           (`/AKIA[A-Z2-7]{4}/`, a bounded class + bounded
//           repetition) compiles at startup via `compile_from_text`
//           and matches a line containing the shape, emitting a
//           `PATH:LINE rule=N` finding.
// Why:      Replaces the old BUG-8 test, whose rule `/(?i)adafruit[\s]+=/`
//           the new dialect rejects (inline `(?i)` and unbounded `+`
//           are both unsupported). This pins the migration's actual
//           contract: a supported regex rule compiles and fires
//           through the engine line scan.
#[test]
fn user_regex_rule_matches_bounded_pattern() {
    let dir = unique_tmp("user-regex");
    let rules = dir.join("rules.txt");
    fs::write(&rules, "/AKIA[A-Z2-7]{4}/\n").expect("write rules");
    let target = dir.join("key.txt");
    fs::write(&target, b"prefix AKIA2345 suffix\n").expect("write target");

    let output = Command::configured(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&target)
        .output()
        .expect("spawn binary");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "user regex rule must match the AKIA shape; got success.\n\
         stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        stderr,
    );
    assert!(
        stderr.contains("key.txt:1 rule=0"),
        "stderr must carry the columnless finding for line 1 rule 0; got: {}",
        stderr
    );

    let _ = fs::remove_dir_all(&dir);
}

// What:     A finding on the third line of a file emits exactly
//           `PATH:LINE rule=N` with no column segment, and reports the
//           correct 1-based line number.
// Why:      The migration drops columns (`PATH:LINE:COL_START..COL_END`
//           becomes `PATH:LINE rule=N`); this pins the new columnless
//           shape and the line-number mapping at the binary boundary.
#[test]
fn output_is_columnless_with_correct_line_number() {
    let dir = unique_tmp("columnless");
    let rules = dir.join("rules.txt");
    fs::write(&rules, "NEEDLE_LITERAL_LONG_ENOUGH\n").expect("write rules");
    let target = dir.join("multi.txt");
    fs::write(
        &target,
        "clean one\nclean two\nNEEDLE_LITERAL_LONG_ENOUGH\n",
    )
    .expect("write target");

    let output = Command::configured(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&target)
        .output()
        .expect("spawn binary");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "must exit non-zero on a hit; stderr: {}",
        stderr
    );
    let hit_line = stderr
        .lines()
        .find(|line| return line.contains("rule="))
        .expect("a rule= finding line");
    // Exact columnless shape: the path, a colon, the line number, then ` rule=`.
    assert!(
        hit_line.ends_with("multi.txt:3 rule=0"),
        "finding must be columnless `PATH:3 rule=0`; got: {}",
        hit_line,
    );
    // No column-range segment survives the migration.
    assert!(
        !hit_line.contains(".."),
        "finding must not carry a column range; got: {}",
        hit_line,
    );

    let _ = fs::remove_dir_all(&dir);
}

// What:     A short bare literal rule matches only at word boundaries.
//           Rule "ACR" must hit a standalone "see ACR here" but must NOT
//           hit a glued "ACRYLIC" occurrence.
// Why:      Bare literals under eight bytes are gated behind word
//           boundaries (so "ACR" compiles to \bACR\b), confining a short
//           run to whole-token matches instead of firing inside a longer
//           token or a base64 blob. The over-matching preference still
//           holds for longer literals and regex rules; this narrowing is
//           specific to short bare literals.
#[test]
fn short_literal_matches_whole_word_not_substring() {
    let dir = unique_tmp("short-word-boundary");
    let rules = dir.join("rules.txt");
    fs::write(&rules, "ACR\n").expect("write rules");

    // (1) Standalone occurrence: must match (word boundaries on both sides).
    let hit_file = dir.join("hit.txt");
    fs::write(&hit_file, "see ACR here\n").expect("write hit file");
    let hit_output = Command::configured(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&hit_file)
        .output()
        .expect("spawn binary");
    assert!(
        !hit_output.status.success(),
        "literal `ACR` must match standalone occurrence; exit was success.\n\
         stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&hit_output.stdout),
        String::from_utf8_lossy(&hit_output.stderr),
    );

    // (2) Glued occurrence: must NOT match (no trailing word boundary).
    let glued_file = dir.join("glued.txt");
    fs::write(&glued_file, "see ACRYLIC here\n").expect("write glued file");
    let glued_output = Command::configured(BIN)
        .args(["--rules"])
        .arg(&rules)
        .arg(&glued_file)
        .output()
        .expect("spawn binary");
    assert!(
        glued_output.status.success(),
        "literal `ACR` must NOT match inside `ACRYLIC` under word-boundary gating.\n\
         stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&glued_output.stdout),
        String::from_utf8_lossy(&glued_output.stderr),
    );

    let _ = fs::remove_dir_all(&dir);
}
