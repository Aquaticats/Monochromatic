//! What: Which dependents are scanned, which edges their source confirms, and which files
//!       are read.
//! Why: Scanning reads source files; it must read only what can change the plan and
//!      confirm exactly the edges the incumbent confirms.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(await bundledDevelopmentEdges({ reader, states, bumpedNames: ['@s/base'] })).toEqual(...);
//! ```

/// The scan, the state reader and the in-memory workspace.
use super::bundled_development_edges;
use crate::dependent_version_content::{ContentUnavailable, TrackedPath, WorkspaceContent};
use crate::dependent_version_states::{ManifestState, read_manifest_states};
use crate::dependent_version_test_support::{MemoryWorkspace, Read, unchanged, workspace};

/// Scan a workspace for the given raised names, returning the edges and the source reads.
fn scan(
    memory: &mut MemoryWorkspace,
    bumped: &[&str],
) -> Result<Vec<Vec<String>>, ContentUnavailable> {
    let tracked: Vec<TrackedPath> = memory.tracked_paths().unwrap_or_default();
    let states: Vec<ManifestState> =
        read_manifest_states(memory, &tracked).unwrap_or_else(|error| panic!("{error:?}"));
    memory.reads.clear();
    let names: Vec<String> = bumped
        .iter()
        .map(|name: &&str| return String::from(*name))
        .collect();
    return bundled_development_edges(memory, &tracked, &states, &names);
}

/// A manifest file with dev and runtime dependencies on the given names.
fn manifest(
    name: &str,
    runtime: &[&str],
    dev: &[&str],
) -> crate::dependent_version_test_support::MemoryFile {
    let list = |names: &[&str]| {
        return names
            .iter()
            .map(|dependency: &&str| return format!("\"{dependency}\":\"workspace:*\""))
            .collect::<Vec<String>>()
            .join(",");
    };
    return unchanged(
        &format!("package/m/{name}/package.json"),
        &format!(
            "{{\"name\":\"{name}\",\"dependencies\":{{{}}},\"devDependencies\":{{{}}}}}",
            list(runtime),
            list(dev)
        ),
    );
}

/// A development edge to a raised package counts when non-test source imports it.
#[test]
fn confirms_imported_development_edges() {
    let mut memory: MemoryWorkspace = workspace(vec![
        manifest("base", &[], &[]),
        manifest("bundled", &[], &["base"]),
        unchanged(
            "package/m/bundled/src/index.ts",
            "import { b } from 'base/ts';\n",
        ),
        unchanged("package/m/bundled/src/notes.md", "import 'base';\n"),
        manifest("tested", &[], &["base"]),
        unchanged(
            "package/m/tested/src/index.unit.test.ts",
            "import 'base';\n",
        ),
        manifest("bystander", &[], &[]),
        unchanged("package/m/bystander/src/index.ts", "import 'base';\n"),
    ]);
    assert_eq!(
        scan(&mut memory, &["base"]),
        Ok(vec![
            Vec::new(),
            vec![String::from("base")],
            Vec::new(),
            Vec::new()
        ])
    );
    assert_eq!(
        memory.reads,
        vec![Read::Candidate(b"package/m/bundled/src/index.ts".to_vec())]
    );
}

/// Edges through a package that reaches a raised one are scanned; others are not read.
#[test]
fn scans_only_edges_that_lead_to_a_raised_package() {
    let mut memory: MemoryWorkspace = workspace(vec![
        manifest("base", &[], &[]),
        manifest("mid", &["base"], &[]),
        manifest("top", &[], &["mid", "other", "top"]),
        unchanged(
            "package/m/top/src/index.ts",
            "import 'mid'; import 'other'; import 'top';\n",
        ),
        manifest("other", &[], &[]),
        manifest("loner", &[], &["other"]),
        unchanged("package/m/loner/src/index.ts", "import 'other';\n"),
    ]);
    assert_eq!(
        scan(&mut memory, &["base"]),
        Ok(vec![
            Vec::new(),
            Vec::new(),
            vec![String::from("mid")],
            Vec::new(),
            Vec::new()
        ])
    );
    assert_eq!(
        memory.reads,
        vec![Read::Candidate(b"package/m/top/src/index.ts".to_vec())]
    );
}

/// Undecodable source bytes are replaced, and the import around them is still found.
#[test]
fn decodes_source_leniently() {
    let mut memory: MemoryWorkspace = workspace(vec![
        manifest("base", &[], &[]),
        manifest("user", &[], &["base"]),
        unchanged("package/m/user/src/index.ts", "x"),
    ]);
    memory.files[2].current = b"\xff\xfe import 'base';\n".to_vec();
    assert_eq!(
        scan(&mut memory, &["base"]),
        Ok(vec![Vec::new(), vec![String::from("base")]])
    );
}

/// A source file that cannot be read stops the scan.
#[test]
fn reports_an_unreadable_source_file() {
    let mut memory: MemoryWorkspace = workspace(vec![
        manifest("base", &[], &[]),
        manifest("user", &[], &["base"]),
        unchanged("package/m/user/src/index.ts", "import 'base';\n"),
    ]);
    memory
        .unreadable
        .push(b"package/m/user/src/index.ts".to_vec());
    assert_eq!(
        scan(&mut memory, &["base"]),
        Err(ContentUnavailable {
            path: Some(b"package/m/user/src/index.ts".to_vec()),
            reason: String::from("read refused"),
        })
    );
}
