//! What: The plan over in-memory workspaces, with the cases of
//!       `bump-dependents-worktree.unit.test.ts` as the content they read.
//! Why: The plan is what both entry points apply; its bytes are the patch.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const plan = await planWorkspaceBumps(reader); expect(plan.bumps.map(b => `${b.name}@${b.to}`)).toEqual([...]);
//! ```

/// The plan, its outcome types and the in-memory workspace.
use super::{ManifestBump, PlanOutcome, WorkspaceBumpPlan, plan_workspace_bumps};
use crate::dependent_version_content::{ContentUnavailable, PlanError, PolicyIncomplete};
use crate::dependent_version_manifest::shape;
use crate::dependent_version_release::UnsupportedVersion;
use crate::dependent_version_test_support::{
    MemoryFile, MemoryWorkspace, Read, changed, config, manifest_text, unchanged, workspace,
};

/// The incumbent worktree test's baseline, with `@s/base` raised to `1.1.0`.
fn raised_baseline() -> Vec<MemoryFile> {
    return vec![
        unchanged(
            "package/config/pnpr/config.yaml",
            "packages:\n  - '@s/app'\n  - '@s/base'\n  - '@s/tool'\nnext: 1\n",
        ),
        changed(
            "package/module/base/package.json",
            &manifest_text("  \"name\": \"@s/base\",\n  \"version\": \"1.1.0\""),
            Some(&manifest_text(
                "  \"name\": \"@s/base\",\n  \"version\": \"1.0.0\"",
            )),
        ),
        unchanged(
            "package/module/app/package.json",
            &manifest_text(
                "  \"name\": \"@s/app\",\n  \"version\": \"2.0.0\",\n  \"dependencies\": {\n    \"@s/base\": \"workspace:*\"\n  }",
            ),
        ),
        unchanged(
            "package/module/tool/package.json",
            &manifest_text(
                "  \"name\": \"@s/tool\",\n  \"version\": \"0.1.0\",\n  \"devDependencies\": {\n    \"@s/base\": \"workspace:*\"\n  }",
            ),
        ),
        unchanged(
            "package/module/tool/src/index.ts",
            "export { base } from '@s/base/ts';\n",
        ),
    ];
}

/// The completed plan of a workspace that must plan.
fn planned(memory: &mut MemoryWorkspace) -> WorkspaceBumpPlan {
    let outcome: Result<PlanOutcome, PlanError> = plan_workspace_bumps(memory);
    let Ok(PlanOutcome::Planned(plan)) = outcome else {
        panic!("expected a plan, got {outcome:?}");
    };
    return plan;
}

/// Ported: runtime and bundled dependents of a raised package are bumped, bytes exact.
#[test]
fn bumps_runtime_and_bundled_dependents() {
    let mut memory: MemoryWorkspace = workspace(raised_baseline());
    let plan: WorkspaceBumpPlan = planned(&mut memory);
    assert_eq!(plan.bumped_names, ["@s/base"]);
    let summary: Vec<String> = plan
        .bumps
        .iter()
        .map(|bump: &ManifestBump| return format!("{}@{}", bump.planned.name, bump.planned.bump.to))
        .collect();
    assert_eq!(summary, ["@s/app@2.0.1", "@s/tool@0.1.1"]);
    let tool: &ManifestBump = &plan.bumps[1];
    assert_eq!(tool.path, b"package/module/tool/package.json");
    assert_eq!(tool.original, memory.files[3].current);
    assert_eq!(
        String::from_utf8_lossy(&tool.replacement),
        manifest_text(
            "  \"name\": \"@s/tool\",\n  \"version\": \"0.1.1\",\n  \"devDependencies\": {\n    \"@s/base\": \"workspace:*\"\n  }"
        )
    );
}

/// Ported: nothing is planned, and the configuration is not read, when nothing was raised.
#[test]
fn plans_nothing_without_a_raised_version() {
    let mut files: Vec<MemoryFile> = raised_baseline();
    files[1].current = files[1].base.clone().unwrap_or_default();
    let mut memory: MemoryWorkspace = workspace(files);
    assert_eq!(
        planned(&mut memory),
        WorkspaceBumpPlan {
            bumped_names: Vec::new(),
            bumps: Vec::new()
        }
    );
    assert!(!memory.reads.contains(&Read::Candidate(
        b"package/config/pnpr/config.yaml".to_vec()
    )));
}

/// Without the registry configuration nothing is publishable.
#[test]
fn plans_nothing_without_the_configuration() {
    let mut memory: MemoryWorkspace = workspace(raised_baseline()[1..].to_vec());
    assert_eq!(
        planned(&mut memory),
        WorkspaceBumpPlan {
            bumped_names: vec![String::from("@s/base")],
            bumps: Vec::new()
        }
    );
}

/// A configuration that is not UTF-8 stops the plan.
#[test]
fn refuses_a_configuration_that_is_not_utf8() {
    let mut files: Vec<MemoryFile> = raised_baseline();
    files[0].current = b"packages:\n  - '\xff'\n".to_vec();
    assert_eq!(
        plan_workspace_bumps(&mut workspace(files)),
        Err(PlanError::PolicyIncomplete(PolicyIncomplete::NotUtf8 {
            path: b"package/config/pnpr/config.yaml".to_vec()
        }))
    );
}

/// Two manifests with one name stop a plan that needs the graph, and only such a plan.
#[test]
fn refuses_duplicate_names_once_the_graph_is_needed() {
    let mut files: Vec<MemoryFile> = raised_baseline();
    files.push(unchanged(
        "package/other/app/package.json",
        "{\"name\":\"@s/app\"}",
    ));
    assert_eq!(
        plan_workspace_bumps(&mut workspace(files.clone())),
        Err(PlanError::PolicyIncomplete(
            PolicyIncomplete::DuplicateName {
                name: String::from("@s/app"),
                first: b"package/module/app/package.json".to_vec(),
                second: b"package/other/app/package.json".to_vec(),
            }
        ))
    );
    files[1].current = files[1].base.clone().unwrap_or_default();
    assert!(matches!(
        plan_workspace_bumps(&mut workspace(files)),
        Ok(PlanOutcome::Planned(_))
    ));
}

/// A dependent on a prerelease stops the plan as unsupported.
#[test]
fn reports_an_unsupported_dependent() {
    let mut files: Vec<MemoryFile> = raised_baseline();
    files[2] = unchanged(
        "package/module/app/package.json",
        "{\"name\":\"@s/app\",\"version\":\"2.0.0-rc.1\",\"dependencies\":{\"@s/base\":\"1\"}}",
    );
    assert_eq!(
        plan_workspace_bumps(&mut workspace(files)),
        Ok(PlanOutcome::Unsupported(UnsupportedVersion {
            name: String::from("@s/app"),
            version: "2.0.0-rc.1".encode_utf16().collect(),
        }))
    );
}

/// A byte-order mark is kept in the patched bytes.
#[test]
fn keeps_a_byte_order_mark() {
    let mut files: Vec<MemoryFile> = raised_baseline();
    files[2].current = [b"\xef\xbb\xbf".as_slice(), &files[2].current].concat();
    let plan: WorkspaceBumpPlan = planned(&mut workspace(files.clone()));
    assert_eq!(plan.bumps[0].original, files[2].current);
    assert!(
        plan.bumps[0]
            .replacement
            .starts_with(b"\xef\xbb\xbf{\n  \"name\": \"@s/app\",\n  \"version\": \"2.0.1\"")
    );
}

/// A version the edit cannot find as parsed stops the plan.
#[test]
fn reports_a_version_the_edit_cannot_rewrite() {
    let mut files: Vec<MemoryFile> = raised_baseline();
    files[2] = unchanged(
        "package/module/app/package.json",
        "{\"name\":\"@s/app\",\"version\":\"9.0.0\",\"version\":\"2.0.0\",\"dependencies\":{\"@s/base\":\"1\"}}",
    );
    assert_eq!(
        plan_workspace_bumps(&mut workspace(files)),
        Err(PlanError::PolicyIncomplete(shape(
            b"package/module/app/package.json",
            "declares version \"9.0.0\", expected \"2.0.0\""
        )))
    );
}

/// A listing that cannot be read is content that is unavailable.
#[test]
fn reports_an_unreadable_listing() {
    let mut memory: MemoryWorkspace = workspace(raised_baseline());
    memory.listing_unavailable = true;
    assert_eq!(
        plan_workspace_bumps(&mut memory),
        Err(PlanError::ContentUnavailable(ContentUnavailable {
            path: None,
            reason: String::from("listing refused"),
        }))
    );
}

/// The incumbent's policy fixture shape for the configuration is read as well.
#[test]
fn reads_the_generator_configuration_shape() {
    let mut files: Vec<MemoryFile> = raised_baseline();
    files[0] = config(&["@s/app"]);
    let plan: WorkspaceBumpPlan = planned(&mut workspace(files));
    assert_eq!(plan.bumps.len(), 1);
    assert_eq!(plan.bumps[0].planned.name, "@s/app");
}
