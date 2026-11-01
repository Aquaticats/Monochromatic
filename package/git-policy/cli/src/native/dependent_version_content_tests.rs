//! What: The failure conversions and the complete diagnostics of the dependent-version
//!       planner.
//! Why: The wiring maps each variant to a failure code and prints the message as written,
//!      so every variant's text is pinned.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(planErrorMessage({ kind: 'policy-incomplete', ... })).toBe('...');
//! ```

/// The failure types, the conversions and the message builders.
use super::{ContentUnavailable, PlanError, PolicyIncomplete, display_path, plan_error_message};

/// Converted failures keep their cause.
#[test]
fn conversions_keep_the_cause() {
    let unavailable: ContentUnavailable = ContentUnavailable {
        path: None,
        reason: String::from("gone"),
    };
    assert_eq!(
        PlanError::from(unavailable.clone()),
        PlanError::ContentUnavailable(unavailable)
    );
    let incomplete: PolicyIncomplete = PolicyIncomplete::NotUtf8 {
        path: b"a".to_vec(),
    };
    assert_eq!(
        PlanError::from(incomplete.clone()),
        PlanError::PolicyIncomplete(incomplete)
    );
}

/// Paths that are not UTF-8 are shown with replacement characters.
#[test]
fn displays_paths_lossily() {
    assert_eq!(
        display_path(b"package/a/\xff/package.json"),
        "package/a/\u{fffd}/package.json"
    );
}

/// Unreadable content names the path, or the listing, and both remedies.
#[test]
fn content_unavailable_messages_name_the_subject() {
    assert_eq!(
        plan_error_message(&PlanError::ContentUnavailable(ContentUnavailable {
            path: Some(b"package/m/a/package.json".to_vec()),
            reason: String::from("blob missing"),
        })),
        "mono/dependent-version-bump could not read package/m/a/package.json: blob missing. \
         Nothing was decided; retry once the content is readable, or pass \
         --no-enforce-mono/dependent-version-bump to skip this policy for one command."
    );
    assert_eq!(
        plan_error_message(&PlanError::ContentUnavailable(ContentUnavailable {
            path: None,
            reason: String::from("index unreadable"),
        })),
        "mono/dependent-version-bump could not read the list of tracked files: index unreadable. \
         Nothing was decided; retry once the content is readable, or pass \
         --no-enforce-mono/dependent-version-bump to skip this policy for one command."
    );
}

/// Every machinery failure names the file and what is wrong with it.
#[test]
fn policy_incomplete_messages_name_the_cause() {
    let message = |incomplete: PolicyIncomplete| {
        return plan_error_message(&PlanError::PolicyIncomplete(incomplete));
    };
    let tail: &str = ". Correct the file and retry, or pass \
                      --no-enforce-mono/dependent-version-bump to skip this policy for one command.";
    assert_eq!(
        message(PolicyIncomplete::NotUtf8 {
            path: b"p/c.yaml".to_vec()
        }),
        format!("mono/dependent-version-bump could not finish: p/c.yaml is not valid UTF-8{tail}")
    );
    assert_eq!(
        message(PolicyIncomplete::ManifestSyntax {
            path: b"p/a/package.json".to_vec(),
            detail: String::from("expected colon"),
        }),
        format!(
            "mono/dependent-version-bump could not finish: p/a/package.json is not JSON (expected colon){tail}"
        )
    );
    assert_eq!(
        message(PolicyIncomplete::ManifestShape {
            path: b"p/a/package.json".to_vec(),
            problem: String::from("p/a/package.json has no string \"name\""),
        }),
        format!(
            "mono/dependent-version-bump could not finish: p/a/package.json has no string \"name\"{tail}"
        )
    );
    assert_eq!(
        message(PolicyIncomplete::DuplicateName {
            name: String::from("@s/a"),
            first: b"package/m/a/package.json".to_vec(),
            second: b"package/n/a/package.json".to_vec(),
        }),
        format!(
            "mono/dependent-version-bump could not finish: package/m/a/package.json and \
             package/n/a/package.json both declare the package name \"@s/a\", so which one a \
             dependency names is ambiguous{tail}"
        )
    );
}
