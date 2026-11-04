//! What: The native planner against the incumbent's recorded results, on the shared
//!       fixtures of every incumbent unit-test scenario.
//! Why: The release workflow calls the native wrapper instead of the deleted TypeScript task;
//!      the fixtures' `expected` values were written by the incumbent planner (last at commit
//!      `f4fc1fa213ece65672e4314983a75266837892f9`) and stay as its recorded truth.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (const c of cases) expect(native(c)).toBe(JSON.stringify(c.expected));
//! ```

/// The case evaluator, the fixture reader and the parser.
use crate::dependent_version_fixture_cases::evaluate;
use crate::dependent_version_fixture_json::{canonical, elements, field, text};
use crate::dependent_version_test_support::MemoryWorkspace;
use monochromatic_jsonc_edit::{JsoncValue, parse_jsonc};

/// The shared fixtures: every scenario of the incumbent's unit tests, with its result.
const UNIT_CASES: &str = include_str!("dependent_version_fixtures/unit_cases.json");

/// A shared workspace that no committed case names.
fn no_shared_workspace(name: &str) -> MemoryWorkspace {
    panic!("committed cases name no shared workspace, got {name}");
}

/// Every committed case yields the incumbent's recorded result.
#[test]
fn unit_fixtures_match_the_incumbent() {
    let cases: JsoncValue = parse_jsonc(UNIT_CASES).unwrap_or_else(|error| panic!("{error:?}"));
    let mut mismatches: Vec<String> = Vec::new();
    for case in elements(&cases) {
        let actual: String = evaluate(case, &mut no_shared_workspace);
        let expected: String = canonical(field(case, "expected"));
        if actual != expected {
            mismatches.push(format!(
                "{}\n  native:    {actual}\n  incumbent: {expected}",
                text(field(case, "name"))
            ));
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
    // A fixture file that lost its cases would pass vacuously.
    assert!(
        elements(&cases).len() >= 60,
        "only {} cases",
        elements(&cases).len()
    );
}
