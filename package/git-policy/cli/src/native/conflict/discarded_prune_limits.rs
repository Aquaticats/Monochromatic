//! What:
//!  The two limits of the folder of saved copies,
//! their defaults,
//! and the reading of a size such as `256m`.
//! Why:
//!  `discarded_prune.rs` plans and carries out the pruning and stays under the file-size limit;
//! the limits need nothing from it,
//! and the settings reader and the doctor import them from here through it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const limits = { maxAgeDays: 30, maxBytes: parseSize("1g") };
//! ```

/// Days after which a copy is removed unless the setting says otherwise.
///
/// No measurement sets this yet.
/// Measured on this host (read-only,
/// 2026-11-03T14:55Z):
/// the folder `monochromatic-git-policy-cli/discarded` does
/// not exist in any repository with a Git directory within three levels of the home directory,
/// and the audit file of the main
/// repository (950 lines,
/// 2026-10-21T00:35:32Z to 2026-11-03T14:15:51Z) holds no `saved-conflict` or `pruned` line,
/// while the
/// feature is a day old (commit 1d2f4bec8e,
/// 2026-11-02T14:08:46-05:00).
/// The value is the coordinator's earlier default,
/// kept
/// until growth can be measured;
/// `doc/planning/cli-git-shipped-limits-audit.md`,
/// "Defaults for saved conflict copies",
/// has
/// the measurement,
/// the options and the open question about copies that may be the only copy of a person's work.
pub const DEFAULT_MAX_AGE_DAYS: u64 = 30;

/// Bytes the folder is kept under unless the setting says otherwise:
/// one gibibyte.
/// No measurement sets this yet;
/// see
/// `DEFAULT_MAX_AGE_DAYS`.
pub const DEFAULT_MAX_BYTES: u64 = 1024 * 1024 * 1024;

/// The two limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Limits {
    /// Days after which a copy goes.
    pub max_age_days: u64,
    /// Bytes the folder is kept under.
    pub max_bytes: u64,
}

impl Default for Limits {
    fn default() -> Self {
        return Self { max_age_days: DEFAULT_MAX_AGE_DAYS, max_bytes: DEFAULT_MAX_BYTES };
    }
}

/// What:
///  A size as Git's integer settings write it:
/// whole bytes,
/// or a number followed by `k`,
/// `m` or `g` (any case,
/// powers of 1024).
/// Why:
///   `git config --type=int` reads these,
/// so a person who writes `"512m"` means what Git would read;
/// zero,
/// a sign,
/// a fraction
///  and anything else is no size.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseSize('512m') // 536870912
/// ```
pub fn parse_size(text: &str) -> Option<u64> {
    let trimmed: &str = text.trim();
    let (digits, unit): (&str, u64) = match trimmed.chars().last()?.to_ascii_lowercase() {
        'k' => (&trimmed[..trimmed.len() - 1], 1024),
        'm' => (&trimmed[..trimmed.len() - 1], 1024 * 1024),
        'g' => (&trimmed[..trimmed.len() - 1], 1024 * 1024 * 1024),
        _ => (trimmed, 1),
    };
    if digits.is_empty() || !digits.as_bytes().iter().all(u8::is_ascii_digit) {
        return None;
    }
    let bytes: u64 = digits.parse::<u64>().ok()?.checked_mul(unit)?;
    if bytes == 0 {
        return None;
    }
    return Some(bytes);
}
