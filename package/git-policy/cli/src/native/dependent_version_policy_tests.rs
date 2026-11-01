//! What: The trigger rule, the candidate rule and the findings, with every
//!       `findDependentBumps` case of `dependent-version-bump-policy.unit.test.ts`.
//! Why: The policy decides when a commit is blocked and what its fix adds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect((await findDependentBumps(contextOf(files))).map(f => f.path)).toEqual([...]);
//! ```

/// The decision, its request and finding types, and the in-memory workspace.
use super::{
    Candidate, CandidateChange, DEPENDENT_VERSION_STALE_CODE, DEPENDENT_VERSION_UNSUPPORTED_CODE,
    DependentFinding, DependentRequest, find_dependent_bumps, should_plan,
};
use crate::dependent_version_content::{PlanError, TrackedMode};
use crate::dependent_version_test_support::{
    MemoryFile, MemoryWorkspace, changed, config, manifest_text, unchanged, workspace,
};
use crate::policy_trigger::Trigger;

/// The incumbent's two-space manifest with the given fields.
fn manifest(fields: &str) -> String {
    return manifest_text(fields);
}

/// Candidates as the incumbent's fixture derives them: files whose text differs from the base.
fn candidates(files: &[MemoryFile]) -> Vec<Candidate> {
    return files
        .iter()
        .filter(|file: &&MemoryFile| return file.base.as_ref() != Some(&file.current))
        .map(|file: &MemoryFile| {
            return Candidate {
                path: file.path.clone(),
                change: if file.base.is_some() {
                    CandidateChange::Modified
                } else {
                    CandidateChange::Added
                },
            };
        })
        .collect();
}

/// Findings for a trigger and forwarded command over files.
fn find(
    files: Vec<MemoryFile>,
    trigger: Trigger,
    forwards_commit: bool,
) -> Result<Vec<DependentFinding>, PlanError> {
    let listed: Vec<Candidate> = candidates(&files);
    let request: DependentRequest = DependentRequest {
        trigger,
        forwards_commit,
        candidates: &listed,
    };
    return find_dependent_bumps(&request, &mut workspace(files));
}

/// Findings of a pre-forward commit that must succeed.
fn commit_findings(files: Vec<MemoryFile>) -> Vec<DependentFinding> {
    return find(files, Trigger::PreForward, true)
        .unwrap_or_else(|error: PlanError| panic!("{error:?}"));
}

/// `@s/base` raised from 1.0.0 to 1.1.0.
fn raised_base() -> MemoryFile {
    return changed(
        "package/module/base/package.json",
        &manifest("  \"name\": \"@s/base\",\n  \"version\": \"1.1.0\""),
        Some(&manifest(
            "  \"name\": \"@s/base\",\n  \"version\": \"1.0.0\"",
        )),
    );
}

/// `@s/runtime` at a version, depending on `@s/base` at run time.
fn runtime(version: &str) -> MemoryFile {
    return unchanged(
        "package/module/runtime/package.json",
        &manifest(&format!(
            "  \"name\": \"@s/runtime\",\n  \"version\": \"{version}\",\n  \"dependencies\": {{\n    \"@s/base\": \"workspace:*\"\n  }}"
        )),
    );
}

/// A package with a development dependency on `@s/base`.
fn dev_dependent(name: &str, version: &str) -> MemoryFile {
    return unchanged(
        &format!("package/module/{name}/package.json"),
        &manifest(&format!(
            "  \"name\": \"@s/{name}\",\n  \"version\": \"{version}\",\n  \"devDependencies\": {{\n    \"@s/base\": \"workspace:*\"\n  }}"
        )),
    );
}

/// The paths of findings.
fn paths(findings: &[DependentFinding]) -> Vec<String> {
    return findings
        .iter()
        .map(|finding: &DependentFinding| {
            return String::from_utf8_lossy(finding.path.as_deref().unwrap_or_default())
                .into_owned();
        })
        .collect();
}

/// Ported: runtime and bundled dependents are patched; unbundled and test-only ones are not.
#[test]
fn patches_runtime_and_bundled_dependents_only() {
    let findings: Vec<DependentFinding> = commit_findings(vec![
        config(&[
            "@s/base",
            "@s/runtime",
            "@s/bundled",
            "@s/unbundled",
            "@s/test-only",
        ]),
        raised_base(),
        runtime("2.0.0"),
        dev_dependent("bundled", "0.3.9"),
        unchanged(
            "package/module/bundled/src/index.ts",
            "import { base } from '@s/base/ts';\n",
        ),
        dev_dependent("unbundled", "1.0.0"),
        dev_dependent("test-only", "1.0.0"),
        unchanged(
            "package/module/test-only/src/index.unit.test.ts",
            "import '@s/base';\n",
        ),
    ]);
    assert_eq!(
        paths(&findings),
        [
            "package/module/bundled/package.json",
            "package/module/runtime/package.json"
        ]
    );
    assert!(
        findings
            .iter()
            .all(|finding: &DependentFinding| return finding.code == DEPENDENT_VERSION_STALE_CODE)
    );
    assert_eq!(
        findings[1].message,
        "@s/runtime reaches a package bumped in this commit (@s/base); bump it from 2.0.0 to 2.0.1 in the same commit."
    );
    let replacement: String = findings[1]
        .patch
        .as_ref()
        .map(|patch| return String::from_utf8_lossy(&patch.replacement).into_owned())
        .unwrap_or_default();
    assert!(replacement.contains("\n  \"version\": \"2.0.1\",\n"));
}

/// Ported: a forwarded command other than commit stays silent; commit does not.
#[test]
fn stays_silent_for_forwarded_commands_other_than_commit() {
    let files: Vec<MemoryFile> = vec![
        config(&["@s/base", "@s/runtime"]),
        raised_base(),
        runtime("2.0.0"),
    ];
    assert_eq!(
        find(files.clone(), Trigger::PreForward, false),
        Ok(Vec::new())
    );
    assert_eq!(commit_findings(files).len(), 1);
}

/// Ported: direct fix proposes the bumps; direct check and other points plan regardless of
/// the command word.
#[test]
fn plans_for_direct_commands_whatever_the_command_word() {
    let files: Vec<MemoryFile> = vec![
        config(&["@s/base", "@s/runtime"]),
        raised_base(),
        runtime("2.0.0"),
    ];
    for trigger in [
        Trigger::DirectFix,
        Trigger::DirectCheck,
        Trigger::PostCommit,
    ] {
        let findings: Vec<DependentFinding> =
            find(files.clone(), trigger, false).unwrap_or_default();
        assert_eq!(
            paths(&findings),
            ["package/module/runtime/package.json"],
            "{trigger:?}"
        );
    }
}

/// Ported: nothing is reported when no manifest version changed.
#[test]
fn reports_nothing_without_a_version_change() {
    let described: MemoryFile = changed(
        "package/module/base/package.json",
        &manifest("  \"name\": \"@s/base\",\n  \"version\": \"1.0.0\",\n  \"description\": \"x\""),
        Some(&manifest(
            "  \"name\": \"@s/base\",\n  \"version\": \"1.0.0\"",
        )),
    );
    assert_eq!(
        commit_findings(vec![
            config(&["@s/base", "@s/runtime"]),
            described,
            runtime("2.0.0")
        ]),
        Vec::new()
    );
    assert_eq!(commit_findings(vec![config(&[])]), Vec::new());
}

/// Ported: the policy settles once the dependents are bumped too.
#[test]
fn settles_once_dependents_are_bumped() {
    let mut bumped_runtime: MemoryFile = runtime("2.0.1");
    bumped_runtime.base = runtime("2.0.0").base;
    assert_eq!(
        commit_findings(vec![
            config(&["@s/base", "@s/runtime"]),
            raised_base(),
            bumped_runtime
        ]),
        Vec::new()
    );
}

/// Ported: a dependent on a prerelease is reported without a patch.
#[test]
fn reports_an_unsupported_dependent_without_a_patch() {
    assert_eq!(
        commit_findings(vec![
            config(&["@s/base", "@s/runtime"]),
            raised_base(),
            runtime("2.0.0-rc.1")
        ]),
        vec![DependentFinding {
            code: DEPENDENT_VERSION_UNSUPPORTED_CODE,
            message: String::from(
                "@s/runtime has version \"2.0.0-rc.1\", which is not a plain major.minor.patch release; bump it by hand in the same commit."
            ),
            path: None,
            patch: None,
        }]
    );
}

/// Ported: without the registry configuration nothing is reported.
#[test]
fn reports_nothing_without_the_configuration() {
    assert_eq!(
        commit_findings(vec![raised_base(), runtime("2.0.0")]),
        Vec::new()
    );
}

/// Only a modified workspace manifest starts planning.
#[test]
fn plans_only_for_a_modified_workspace_manifest() {
    let request = |path: &[u8], change: CandidateChange| {
        let listed: Vec<Candidate> = vec![Candidate {
            path: path.to_vec(),
            change,
        }];
        return should_plan(&DependentRequest {
            trigger: Trigger::DirectCheck,
            forwards_commit: false,
            candidates: &listed,
        });
    };
    assert!(request(
        b"package/module/a/package.json",
        CandidateChange::Modified
    ));
    assert!(!request(
        b"package/module/a/package.json",
        CandidateChange::Added
    ));
    assert!(!request(
        b"package/module/a/package.json",
        CandidateChange::Deleted
    ));
    assert!(!request(
        b"package/module/a/src/package.json",
        CandidateChange::Modified
    ));
}

/// A bump of a manifest that is not an ordinary file is dropped; an executable one is kept.
#[test]
fn drops_bumps_of_manifests_that_are_not_ordinary_files() {
    for (mode, kept) in [
        (TrackedMode::Regular, true),
        (TrackedMode::Executable, true),
        (TrackedMode::Symlink, false),
        (TrackedMode::Submodule, false),
    ] {
        let mut dependent: MemoryFile = runtime("2.0.0");
        dependent.mode = mode;
        let findings: Vec<DependentFinding> = commit_findings(vec![
            config(&["@s/base", "@s/runtime"]),
            raised_base(),
            dependent,
        ]);
        assert_eq!(findings.len(), usize::from(kept), "{mode:?}");
    }
}

/// A failure to plan is returned to the caller.
#[test]
fn returns_planning_failures() {
    let mut memory: MemoryWorkspace = workspace(vec![raised_base()]);
    memory.listing_unavailable = true;
    let listed: Vec<Candidate> = candidates(&memory.files);
    let request: DependentRequest = DependentRequest {
        trigger: Trigger::PreForward,
        forwards_commit: true,
        candidates: &listed,
    };
    assert!(matches!(
        find_dependent_bumps(&request, &mut memory),
        Err(PlanError::ContentUnavailable(_))
    ));
}
