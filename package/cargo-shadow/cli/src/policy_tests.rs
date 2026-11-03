//! The unit tests of the release-age and trust decisions,
//! ported case for case from `policy.unit.test.ts`.

use std::collections::BTreeSet;

use monochromatic_dependency_catalog::{Settings, TrustPolicy};

use crate::policy::{VersionRecord, release_age_problems, trust_level, trust_problems};
use crate::time_text::parse_rfc3339_millis;

/// The repository's defaults.
fn settings() -> Settings {
    return Settings {
        catalog_prune: false,
        minimum_release_age: 1_440,
        minimum_release_age_exclude: Vec::new(),
        minimum_release_age_exclude_prune: false,
        minimum_release_age_ignore_missing_time: false,
        minimum_release_age_strict: true,
        trust_policy: TrustPolicy::NoDowngrade,
        trust_policy_exclude: Vec::new(),
        trust_policy_exclude_prune: false,
        trust_policy_ignore_after: None,
        trust_lockfile: false,
    };
}

/// The fixed clock.
fn now() -> i64 {
    return parse_rfc3339_millis("2026-10-29T12:00:00Z").expect("a fixed time");
}

fn record(num: &str, created_at: &str, publisher_id: &str, trustpub_repository: &str, repository: &str) -> VersionRecord {
    return VersionRecord {
        num: num.to_string(),
        created_at: created_at.to_string(),
        publisher_id: publisher_id.to_string(),
        trustpub_repository: trustpub_repository.to_string(),
        repository: repository.to_string(),
    };
}

const REPO: &str = "https://github.com/o/r";

fn owners(ids: &[&str]) -> BTreeSet<String> {
    return ids.iter().map(|id| { return id.to_string(); }).collect();
}

#[test]
fn a_mature_version_passes_and_a_young_one_fails_with_the_time_it_matures() {
    assert_eq!(release_age_problems("a", "1.0.0", "2026-10-27T12:00:00Z", &settings(), now()), Vec::<String>::new());

    let young: Vec<String> = release_age_problems("a", "1.0.1", "2026-10-29T10:00:00Z", &settings(), now());

    assert_eq!(young.len(), 1);
    assert!(young[0].contains("2026-10-30T10:00:00.000Z"));
}

#[test]
fn an_excluded_version_skips_the_age_rule() {
    let mut with_exclude: Settings = settings();
    with_exclude.minimum_release_age_exclude = vec!["a".to_string()];

    assert!(release_age_problems("a", "1.0.1", "2026-10-29T10:00:00Z", &with_exclude, now()).is_empty());
}

#[test]
fn a_missing_publish_time_follows_the_ignore_missing_time_setting() {
    assert_eq!(release_age_problems("a", "1.0.0", "", &settings(), now()).len(), 1);
    let mut ignoring: Settings = settings();
    ignoring.minimum_release_age_ignore_missing_time = true;

    assert!(release_age_problems("a", "1.0.0", "", &ignoring, now()).is_empty());
}

#[test]
fn trusted_publishing_rates_three_a_returning_publisher_two_and_a_first_time_publisher_one() {
    let first: VersionRecord = record("1.0.0", "2026-01-01T00:00:00Z", "7", "", REPO);
    let trusted: VersionRecord = record("1.0.1", "2026-02-01T00:00:00Z", "", "o/r", REPO);
    let returning: VersionRecord = record("1.0.1", "2026-02-01T00:00:00Z", "7", "", REPO);

    assert_eq!(trust_level(&trusted, std::slice::from_ref(&first), &owners(&[])), 3);
    assert_eq!(trust_level(&returning, std::slice::from_ref(&first), &owners(&[])), 2);
    assert_eq!(trust_level(&first, &[], &owners(&[])), 1);
}

#[test]
fn a_token_publish_after_trusted_publishing_fails_and_a_returning_publisher_passes() {
    let history: Vec<VersionRecord> = vec![
        record("1.0.0", "2026-01-01T00:00:00Z", "", "o/r", REPO),
        record("1.0.1", "2026-02-01T00:00:00Z", "", "o/r", REPO),
        record("1.0.2", "2026-03-01T00:00:00Z", "9", "", REPO),
    ];

    assert!(trust_problems("a", "1.0.1", &history, &settings(), now(), &owners(&[])).is_empty());
    let downgraded: Vec<String> = trust_problems("a", "1.0.2", &history, &settings(), now(), &owners(&[]));
    assert_eq!(downgraded.len(), 1);
    assert!(downgraded[0].contains("trust level 1, below an earlier release's 3"));
}

#[test]
fn each_earlier_stable_release_is_rated_against_the_full_history_prereleases_included() {
    let history: Vec<VersionRecord> = vec![
        record("1.0.0-rc.1", "2026-01-01T00:00:00Z", "7", "", REPO),
        record("1.0.0", "2026-02-01T00:00:00Z", "7", "", REPO),
        record("1.0.1", "2026-03-01T00:00:00Z", "9", "", REPO),
    ];

    let problems: Vec<String> = trust_problems("a", "1.0.1", &history, &settings(), now(), &owners(&[]));

    assert_eq!(problems.len(), 1);
    assert!(problems[0].contains("trust level 1, below an earlier release's 2"));
}

#[test]
fn a_trusted_publishing_repository_change_and_a_declared_repository_change_both_fail() {
    let history: Vec<VersionRecord> = vec![
        record("1.0.0", "2026-01-01T00:00:00Z", "", "o/r", REPO),
        record("1.0.1", "2026-02-01T00:00:00Z", "", "evil/r", "https://github.com/evil/r"),
    ];

    let problems: Vec<String> = trust_problems("a", "1.0.1", &history, &settings(), now(), &owners(&[]));

    assert_eq!(problems.len(), 2);
    assert!(problems[0].contains("trusted-published from evil/r, not o/r"));
    assert!(problems[1].contains("declares repository \"https://github.com/evil/r\""));
}

#[test]
fn the_check_is_skipped_when_off_excluded_or_older_than_the_ignore_after_window() {
    let history: Vec<VersionRecord> = vec![
        record("1.0.0", "2026-01-01T00:00:00Z", "", "o/r", REPO),
        record("1.0.1", "2026-02-01T00:00:00Z", "9", "", REPO),
    ];
    let mut off: Settings = settings();
    off.trust_policy = TrustPolicy::Off;
    let mut excluded: Settings = settings();
    excluded.trust_policy_exclude = vec!["a@1.0.1".to_string()];
    let mut old: Settings = settings();
    old.trust_policy_ignore_after = Some(60);

    assert!(trust_problems("a", "1.0.1", &history, &off, now(), &owners(&[])).is_empty());
    assert!(trust_problems("a", "1.0.1", &history, &excluded, now(), &owners(&[])).is_empty());
    assert!(trust_problems("a", "1.0.1", &history, &old, now(), &owners(&[])).is_empty());
}

#[test]
fn a_listed_owners_first_release_is_a_known_publisher_and_a_non_owner_is_still_refused() {
    let history: Vec<VersionRecord> = vec![
        record("1.0.0", "2026-01-01T00:00:00Z", "7", "", REPO),
        record("1.0.1", "2026-02-01T00:00:00Z", "7", "", REPO),
        record("1.0.2", "2026-03-01T00:00:00Z", "9", "", REPO),
    ];

    assert!(trust_problems("a", "1.0.2", &history, &settings(), now(), &owners(&["7", "9"])).is_empty());
    assert_eq!(trust_problems("a", "1.0.2", &history, &settings(), now(), &owners(&["7"])).len(), 1);
}

#[test]
fn a_repository_undeclared_before_or_after_is_ignored_and_a_change_between_declared_ones_is_flagged() {
    let placeholder: Vec<VersionRecord> =
        vec![record("0.0.0", "2026-01-01T00:00:00Z", "7", "", ""), record("0.12.0", "2026-02-01T00:00:00Z", "7", "", REPO)];
    let dropped: Vec<VersionRecord> =
        vec![record("1.0.0", "2026-01-01T00:00:00Z", "7", "", REPO), record("1.0.1", "2026-02-01T00:00:00Z", "7", "", "")];
    let changed: Vec<VersionRecord> = vec![
        record("1.0.0", "2026-01-01T00:00:00Z", "7", "", REPO),
        record("1.0.1", "2026-02-01T00:00:00Z", "7", "", "https://github.com/other/r"),
    ];

    assert!(trust_problems("a", "0.12.0", &placeholder, &settings(), now(), &owners(&[])).is_empty());
    assert!(trust_problems("a", "1.0.1", &dropped, &settings(), now(), &owners(&[])).is_empty());
    assert_eq!(trust_problems("a", "1.0.1", &changed, &settings(), now(), &owners(&[])).len(), 1);
}

#[test]
fn a_version_missing_from_the_history_cannot_be_checked_and_is_reported() {
    let problems: Vec<String> = trust_problems("a", "9.9.9", &[], &settings(), now(), &owners(&[]));

    assert_eq!(problems.len(), 1);
    assert!(problems[0].contains("is not in the crates.io version list"));
}

/// Review finding (invalid publish timestamps):
/// a record whose `created_at` cannot be read must not look old enough to
/// skip the trust check when `trustPolicyIgnoreAfter` is set.
#[test]
fn a_release_with_an_unreadable_publish_time_never_gets_the_ignore_after_exemption() {
    // What:     `let mut x` declares a variable that can be reassigned or changed; plain `let x` is a constant binding (TS `const`).
    // Why:      The value is built up or replaced step by step below.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // let x = ...;   // plain Rust `let` is TS `const`
    // ```
    // What:     `let name: Type = value;` declares a variable with an explicit type, written after a colon, as in TS.
    // Why:      The type is stated so the reader sees what the value is without tracing the right-hand side.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const name: Type = value;
    // ```
    let mut with_ignore_after: Settings = settings();
    // What:     `Some(value)` is the "present" variant of `Option<T>` and `None` is the "absent" one (siblings: `Result<T, E>`, which carries an error; a TS `T | undefined`).
    // Why:      Absence is a value the caller must check; there is no `null`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // Some(value) is  value
    // // None        is  undefined
    // ```
    with_ignore_after.trust_policy_ignore_after = Some(60);
    // Typed binding: `history: Vec<VersionRecord>`.
    // What:     A name ending in `!` is a macro: code that expands before compiling. `format!("..{}..", x)` builds a `String` from a template (a TS template literal); `eprintln!`/`println!` print a line to standard error/output; `vec![..]` builds a `Vec`; `matches!(v, Pattern)` tests a pattern.
    // Why:      Text building and printing are written as one call instead of string concatenation.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // `..${x}..`   // template literal
    // console.error(text);   // eprintln!
    // [a, b]            // vec![a, b]
    // ```
    // What:     `Vec<T>` is an owned growable list; the siblings are `&[T]`, a borrowed view of a list, and `[T; N]`, a fixed-size array.
    // Why:      The list is built or returned here, so `Vec<T>` and not a borrowed slice.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: T[] / Set / Map / number
    // ```
    let history: Vec<VersionRecord> = vec![
        record("1.0.0", "2026-01-01T00:00:00Z", "", "o/r", REPO),
        record("1.0.1", "not a timestamp", "9", "", REPO),
    ];

    // What:     `&x` lends `x` read-only to the callee, `&mut x` lends it for changes; nothing is copied and the caller keeps ownership. `*x` reads through such a reference.
    // Why:      Large values (lists, texts, paths) are passed without copying and the callee cannot keep them.
    // Gotcha:   A borrow cannot outlive the value it points to; the compiler rejects code that would keep one.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS passes objects by reference already; `&x` is just `x`, `&mut x` is `x` that the callee may mutate
    // ```
    // Typed binding: `problems: Vec<String>`.
    // What:     `Vec::new()`, `String::new()` and `BTreeSet::new()` build an empty owned collection; `Box::new(x)` puts a value on the heap behind an owning pointer (siblings: `Rc`/`Arc`, shared ownership; here one owner is enough).
    // Why:      An empty collection is filled below, or a value of a trait type is stored behind one pointer.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // const list: string[] = [];   // or new Set() / new Map()
    // ```
    // What:     `String` is an owned, growable UTF-8 text; the sibling `&str` is a borrowed read-only view that owns nothing.
    // Why:      A returned value or a stored field must own its text, so `String` and not `&str`.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // // TS has one type for this: string
    // ```
    let problems: Vec<String> = trust_problems("a", "1.0.1", &history, &with_ignore_after, now(), &BTreeSet::new());

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(!problems.is_empty(), "an unknown publish time granted the age exemption");
}

/// The same for a missing value.
#[test]
fn a_release_with_an_empty_publish_time_never_gets_the_ignore_after_exemption() {
    // `let mut`: a binding that may change.
    // Typed binding: `with_ignore_after: Settings`.
    let mut with_ignore_after: Settings = settings();
    // `Some`: the present or the absent value of an `Option`.
    with_ignore_after.trust_policy_ignore_after = Some(60);
    // Typed binding: `history: Vec<VersionRecord>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let history: Vec<VersionRecord> = vec![
        record("1.0.0", "2026-01-01T00:00:00Z", "", "o/r", REPO),
        record("1.0.1", "", "9", "", REPO),
    ];

    // Borrow `&history`: lends the value without copying or giving it away.
    // Typed binding: `problems: Vec<String>`.
    // Constructor `BTreeSet::new()`: an empty owned collection or a boxed value.
    let problems: Vec<String> = trust_problems("a", "1.0.1", &history, &with_ignore_after, now(), &BTreeSet::new());

    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(!problems.is_empty(), "a missing publish time granted the age exemption");
}

/// Delegated pick 9 (2026-11-03):
/// the trust-downgrade finding names the new publisher account and the earlier release with its
/// publisher (the account,
/// or the trusted-publishing repository),
/// as well as the trust levels.
#[test]
fn a_trust_downgrade_names_the_earlier_and_the_new_publisher() {
    // Typed binding: `history: Vec<VersionRecord>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let history: Vec<VersionRecord> = vec![
        record("1.0.0", "2026-01-01T00:00:00Z", "", "o/trusted-ci", REPO),
        record("1.0.1", "2026-02-01T00:00:00Z", "4242", "", REPO),
    ];

    // Borrow `&history`: lends the value without copying or giving it away.
    // Typed binding: `problems: Vec<String>`.
    // Constructor `BTreeSet::new()`: an empty owned collection or a boxed value.
    let problems: Vec<String> = trust_problems("a", "1.0.1", &history, &settings(), now(), &BTreeSet::new());

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(problems.len(), 1, "{problems:?}");
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(problems[0].contains("4242"), "the new publisher account is not named: {problems:?}");
    // Macro `assert!`: builds or prints text (or a list) from a template.
    assert!(problems[0].contains("o/trusted-ci") && problems[0].contains("1.0.0"), "the earlier publisher is not named: {problems:?}");
}

/// Review finding (2c 7):
/// a historical record with an unreadable publish time is sorted first and made to look like earlier history
/// of its publisher.
/// It must not establish publisher reputation.
#[test]
fn an_unreadable_historical_timestamp_cannot_give_a_new_publisher_an_earlier_history() {
    // Typed binding: `history: Vec<VersionRecord>`.
    // Macro `vec!`: builds or prints text (or a list) from a template.
    let history: Vec<VersionRecord> = vec![
        record("1.0.0", "2026-01-01T00:00:00Z", "A", "", REPO),
        record("1.0.1", "2026-02-01T00:00:00Z", "B", "", REPO),
        record("1.0.2", "malformed", "B", "", REPO),
    ];

    // Borrow `&history`: lends the value without copying or giving it away.
    // Typed binding: `problems: Vec<String>`.
    let problems: Vec<String> = trust_problems("a", "1.0.1", &history, &settings(), now(), &owners(&["A"]));

    // Macro `assert_eq!`: builds or prints text (or a list) from a template.
    assert_eq!(problems.len(), 1, "publisher B has no earlier history and A is an owner: {problems:?}");
}
