//! What: Evaluate one differential case with the native planner and render its canonical
//!       result.
//! Why: The incumbent's driver renders the same result shape from the TypeScript planner,
//!      so equal texts mean equal plans, findings, patched bytes and failure kinds.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const output = evaluate(testCase); // the driver's TypeScript twin of this function
//! ```

/// The failure a case may end in, and the modes a fixture names.
use crate::dependent_version_content::{PlanError, TrackedMode};
/// The fixture reader.
use crate::dependent_version_fixture_json::{
    array, elements, field, hex, is_null, is_true, object, quote, text, texts, units,
};
/// The canonical renderings of failures, plans and findings.
use crate::dependent_version_fixture_output::{failed, plan_output, policy_output};
/// The dependency walk under test.
use crate::dependent_version_graph::{PlannedBump, WorkspaceNode, plan_dependent_bumps};
/// The specifier scan under test.
use crate::dependent_version_imports::imports_package;
/// The manifest reader under test.
use crate::dependent_version_manifest::read_manifest_facts;
/// The source path rule under test.
use crate::dependent_version_paths::is_non_test_source_path;
/// The workspace plan under test.
use crate::dependent_version_plan::plan_workspace_bumps;
/// The policy under test and its request.
use crate::dependent_version_policy::{
    Candidate, CandidateChange, DependentRequest, find_dependent_bumps,
};
/// The configuration reader under test.
use crate::dependent_version_publishable::read_publishable_names;
/// The release bump under test and the incumbent's quoting.
use crate::dependent_version_release::{json_quote_units, patch_bump_version, unsupported_message};
/// The in-memory workspace the cases read.
use crate::dependent_version_test_support::{MemoryFile, MemoryWorkspace};
/// The version rewrite under test.
use crate::dependent_version_text::replace_manifest_version;
/// The lifecycle point a workspace case names.
use crate::policy_trigger::Trigger;
/// The parsed fixture value.
use monochromatic_jsonc_edit::JsoncValue;

/// The mode a fixture names.
fn mode(value: &JsoncValue) -> TrackedMode {
    match text(value).as_str() {
        "regular" => return TrackedMode::Regular,
        "executable" => return TrackedMode::Executable,
        "symlink" => return TrackedMode::Symlink,
        "submodule" => return TrackedMode::Submodule,
        other => panic!("unknown mode {other}"),
    }
}

/// Load fixture files into a workspace, replacing files with the same path. A file whose
/// `current` is `null` was not provided, and any read of it fails.
pub fn load_files(files: &[JsoncValue], memory: &mut MemoryWorkspace) {
    for file in files {
        let path: Vec<u8> = hex(field(file, "path"));
        let current: &JsoncValue = field(file, "current");
        let base: &JsoncValue = field(file, "base");
        let bytes: Vec<u8> = if is_null(current) {
            Vec::new()
        } else {
            hex(current)
        };
        let entry: MemoryFile = MemoryFile {
            path: path.clone(),
            mode: mode(field(file, "mode")),
            base: if is_null(base) {
                None
            } else if is_true(base) {
                Some(bytes.clone())
            } else {
                Some(hex(base))
            },
            current: bytes,
        };
        memory.unreadable.retain(|refused| return *refused != path);
        if is_null(current) {
            memory.unreadable.push(path.clone());
        }
        match memory
            .files
            .iter_mut()
            .find(|existing| return existing.path == path)
        {
            Some(existing) => *existing = entry,
            None => memory.files.push(entry),
        }
    }
}

/// The workspace a case reads: a shared workspace, then the case's own files.
fn memory_workspace(
    input: &JsoncValue,
    shared: &mut dyn FnMut(&str) -> MemoryWorkspace,
) -> MemoryWorkspace {
    let base_name: &JsoncValue = field(input, "workspace");
    let mut memory: MemoryWorkspace = if is_null(base_name) {
        MemoryWorkspace::default()
    } else {
        shared(&text(base_name))
    };
    load_files(elements(field(input, "files")), &mut memory);
    return memory;
}

/// A workspace case: the plan and the policy findings over the same content.
fn workspace_case(input: &JsoncValue, shared: &mut dyn FnMut(&str) -> MemoryWorkspace) -> String {
    let memory: MemoryWorkspace = memory_workspace(input, shared);
    let candidates: Vec<Candidate> = elements(field(input, "candidates"))
        .iter()
        .map(|candidate| {
            let change: CandidateChange = match text(field(candidate, "change")).as_str() {
                "added" => CandidateChange::Added,
                "modified" => CandidateChange::Modified,
                _ => CandidateChange::Deleted,
            };
            return Candidate {
                path: hex(field(candidate, "path")),
                change,
            };
        })
        .collect();
    let trigger: Trigger = match text(field(input, "trigger")).as_str() {
        "pre-forward" => Trigger::PreForward,
        "direct-check" => Trigger::DirectCheck,
        "direct-fix" => Trigger::DirectFix,
        "post-commit" => Trigger::PostCommit,
        other => panic!("unknown trigger {other}"),
    };
    let request: DependentRequest = DependentRequest {
        trigger,
        forwards_commit: is_true(field(input, "forwardsCommit")),
        candidates: &candidates,
    };
    let plan: String = plan_output(&plan_workspace_bumps(&mut memory.clone()));
    let policy: String = policy_output(&find_dependent_bumps(&request, &mut memory.clone()));
    return object(&[("plan", plan), ("policy", policy)]);
}

/// A planned bump of the graph-level case.
fn graph_bump(bump: &PlannedBump) -> String {
    return object(&[
        ("name", quote(&bump.name)),
        (
            "directory",
            quote(&String::from_utf8_lossy(&bump.directory)),
        ),
        ("from", quote(&bump.bump.from)),
        ("to", quote(&bump.bump.to)),
    ]);
}

/// A graph-level case: `planDependentBumps` over listed nodes.
fn graph_case(input: &JsoncValue) -> String {
    let nodes: Vec<WorkspaceNode> = elements(field(input, "manifests"))
        .iter()
        .map(|manifest| {
            let version: &JsoncValue = field(manifest, "version");
            return WorkspaceNode {
                name: text(field(manifest, "name")),
                directory: text(field(manifest, "directory")).into_bytes(),
                version: if is_null(version) {
                    None
                } else {
                    Some(units(version))
                },
                edge_names: texts(field(manifest, "edgeNames")),
            };
        })
        .collect();
    match plan_dependent_bumps(
        &nodes,
        &texts(field(input, "bumpedNames")),
        &texts(field(input, "publishableNames")),
    ) {
        Ok(bumps) => {
            return object(&[
                ("kind", quote("bumps")),
                (
                    "value",
                    array(&bumps.iter().map(graph_bump).collect::<Vec<String>>()),
                ),
            ]);
        }
        Err(unsupported) => {
            return object(&[
                ("kind", quote("unsupported")),
                ("message", quote(&unsupported_message(&unsupported))),
            ]);
        }
    }
}

/// A canonical boolean result.
fn boolean(value: bool) -> String {
    return object(&[("kind", quote("bool")), ("value", value.to_string())]);
}

/// Evaluate one case of any kind.
pub fn evaluate(case: &JsoncValue, shared: &mut dyn FnMut(&str) -> MemoryWorkspace) -> String {
    let input: &JsoncValue = field(case, "input");
    let string = |key: &str| return text(field(input, key));
    match string_kind(case).as_str() {
        "workspace" => return workspace_case(input, shared),
        "planDependentBumps" => return graph_case(input),
        "patchBumpVersion" => {
            match patch_bump_version(&string("name"), &units(field(input, "version"))) {
                Ok(bump) => return object(&[("kind", quote("ok")), ("value", quote(&bump.to))]),
                Err(unsupported) => {
                    return object(&[
                        ("kind", quote("unsupported")),
                        ("message", quote(&unsupported_message(&unsupported))),
                    ]);
                }
            }
        }
        "readManifestDependencyFacts" => {
            match read_manifest_facts(string("path").as_bytes(), &string("text")) {
                Err(error) => return failed(&PlanError::PolicyIncomplete(error)),
                Ok(facts) => {
                    let names = |list: &[String]| {
                        return array(
                            &list
                                .iter()
                                .map(|name| return quote(name))
                                .collect::<Vec<String>>(),
                        );
                    };
                    return object(&[
                        ("kind", quote("facts")),
                        ("name", quote(&facts.name)),
                        (
                            "version",
                            facts
                                .version
                                .as_deref()
                                .map_or_else(|| return String::from("null"), json_quote_units),
                        ),
                        ("runtime", names(&facts.runtime_dependency_names)),
                        ("dev", names(&facts.dev_dependency_names)),
                    ]);
                }
            }
        }
        "replaceManifestVersion" => {
            match replace_manifest_version(
                string("path").as_bytes(),
                &string("text"),
                &string("from"),
                &string("to"),
            ) {
                Ok(replaced) => {
                    return object(&[("kind", quote("ok")), ("value", quote(&replaced))]);
                }
                Err(error) => return failed(&PlanError::PolicyIncomplete(error)),
            }
        }
        "importsPackage" => {
            return boolean(imports_package(
                &string("sourceText"),
                &string("packageName"),
            ));
        }
        "isNonTestSourcePath" => {
            return boolean(is_non_test_source_path(
                string("directory").as_bytes(),
                string("path").as_bytes(),
            ));
        }
        "readPublishableNames" => {
            let names: Vec<String> = read_publishable_names(&string("configText"))
                .iter()
                .map(|name| return quote(name))
                .collect();
            return object(&[("kind", quote("names")), ("value", array(&names))]);
        }
        other => panic!("unknown case kind {other}"),
    }
}

/// The kind of a case.
fn string_kind(case: &JsoncValue) -> String {
    return text(field(case, "kind"));
}
