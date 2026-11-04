//! What: The closure and plan, with every `planDependentBumps` case of
//!       `dependent-version-bump.unit.test.ts`.
//! Why: The walk decides which packages are released again.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(planDependentBumps({ manifests, bumpedNames: ['base'], publishableNames })).toEqual([...]);
//! ```

/// The walk, the plan and their types.
use super::{
    PlannedBump, WorkspaceNode, plan_dependent_bumps, transitive_dependent_names, utf16_order,
};
use crate::dependent_version_release::{ReleaseBump, UnsupportedVersion};
use std::cmp::Ordering;
use std::collections::HashSet;

/// A node whose directory is derived from its name, as in the incumbent's fixture.
fn node(name: &str, version: Option<&str>, edges: &[&str]) -> WorkspaceNode {
    return WorkspaceNode {
        name: String::from(name),
        directory: format!("package/module/{name}").into_bytes(),
        version: version.map(|text: &str| return text.encode_utf16().collect()),
        edge_names: owned(edges),
    };
}

/// Names as owned strings.
fn owned(values: &[&str]) -> Vec<String> {
    return values
        .iter()
        .map(|value: &&str| return String::from(*value))
        .collect();
}

/// The bump a planned dependent should receive.
fn planned(name: &str, from: &str, to: &str) -> PlannedBump {
    return PlannedBump {
        name: String::from(name),
        directory: format!("package/module/{name}").into_bytes(),
        bump: ReleaseBump {
            from: String::from(from),
            to: String::from(to),
        },
    };
}

/// Plan with the given raised and publishable names.
fn plan(
    nodes: &[WorkspaceNode],
    bumped: &[&str],
    publishable: &[&str],
) -> Result<Vec<PlannedBump>, UnsupportedVersion> {
    return plan_dependent_bumps(nodes, &owned(bumped), &owned(publishable));
}

/// Ported: direct and transitive publishable dependents are bumped in name order.
#[test]
fn bumps_direct_and_transitive_dependents_in_name_order() {
    let nodes: Vec<WorkspaceNode> = vec![
        node("z-app", Some("2.0.0"), &["mid"]),
        node("mid", Some("1.0.0"), &["base"]),
        node("base", Some("1.1.0"), &[]),
        node("a-tool", Some("0.1.0"), &["base"]),
    ];
    assert_eq!(
        plan(&nodes, &["base"], &["z-app", "mid", "base", "a-tool"]),
        Ok(vec![
            planned("a-tool", "0.1.0", "0.1.1"),
            planned("mid", "1.0.0", "1.0.1"),
            planned("z-app", "2.0.0", "2.0.1"),
        ])
    );
}

/// Ported: the walk passes through unpublishable packages but bumps only publishable ones.
#[test]
fn walks_through_unpublishable_packages() {
    let nodes: Vec<WorkspaceNode> = vec![
        node("app", Some("1.0.0"), &["internal"]),
        node("internal", Some("1.0.0"), &["base"]),
        node("base", Some("1.0.0"), &[]),
    ];
    assert_eq!(
        plan(&nodes, &["base"], &["app", "base"]),
        Ok(vec![planned("app", "1.0.0", "1.0.1")])
    );
}

/// Ported: already raised and versionless dependents are skipped.
#[test]
fn skips_raised_and_versionless_dependents() {
    let nodes: Vec<WorkspaceNode> = vec![
        node("bumped-dependent", Some("3.0.0"), &["base"]),
        node("versionless", None, &["base"]),
        node("base", Some("1.0.0"), &[]),
    ];
    assert_eq!(
        plan(
            &nodes,
            &["base", "bumped-dependent"],
            &["bumped-dependent", "versionless", "base"]
        ),
        Ok(Vec::new())
    );
}

/// Ported: external and self edges are ignored and cycles end.
#[test]
fn ignores_external_and_self_edges_and_ends_on_cycles() {
    let nodes: Vec<WorkspaceNode> = vec![
        node("left", Some("1.0.0"), &["right", "left", "external"]),
        node("right", Some("1.0.0"), &["left"]),
    ];
    assert_eq!(
        plan(&nodes, &["left"], &["left", "right"]),
        Ok(vec![planned("right", "1.0.0", "1.0.1")])
    );
    assert_eq!(
        transitive_dependent_names(&nodes, &owned(&["left"])),
        HashSet::from([String::from("left"), String::from("right")])
    );
}

/// A raised name outside the workspace reaches nothing, even through an edge naming it.
#[test]
fn ignores_edges_to_names_outside_the_workspace() {
    let nodes: Vec<WorkspaceNode> = vec![node("a", Some("1.0.0"), &["external"])];
    assert_eq!(
        plan(&nodes, &["external"], &["a", "external"]),
        Ok(Vec::new())
    );
    let looped: Vec<WorkspaceNode> = vec![node("a", Some("1.0.0"), &["a"])];
    assert_eq!(plan(&looped, &["a"], &["a"]), Ok(Vec::new()));
    assert!(transitive_dependent_names(&looped, &owned(&["a"])).is_empty());
}

/// Ported: nothing is planned when nothing was raised.
#[test]
fn plans_nothing_without_a_raised_package() {
    assert_eq!(
        plan(&[node("a", Some("1.0.0"), &["b"])], &[], &["a"]),
        Ok(Vec::new())
    );
}

/// Ported: a dependent on a prerelease stops the plan; the first such node is reported.
#[test]
fn reports_the_first_unsupported_dependent_in_node_order() {
    let nodes: Vec<WorkspaceNode> = vec![
        node("z", Some("1.0.0-rc.2"), &["b"]),
        node("a", Some("1.0.0-rc.1"), &["b"]),
        node("b", Some("1.0.0"), &[]),
    ];
    assert_eq!(
        plan(&nodes, &["b"], &["a", "b", "z"]),
        Err(UnsupportedVersion {
            name: String::from("z"),
            version: "1.0.0-rc.2".encode_utf16().collect(),
        })
    );
}

/// Names sort by UTF-16 code units, as JavaScript's `<` does.
#[test]
fn sorts_by_utf16_code_units() {
    assert_eq!(utf16_order("\u{10000}", "\u{ffff}"), Ordering::Less);
    assert_eq!(utf16_order("b", "a"), Ordering::Greater);
    assert_eq!(utf16_order("a", "a"), Ordering::Equal);
    let nodes: Vec<WorkspaceNode> = vec![
        node("\u{ffff}", Some("1.0.0"), &["base"]),
        node("\u{10000}", Some("1.0.0"), &["base"]),
        node("base", Some("1.0.0"), &[]),
    ];
    assert_eq!(
        plan(&nodes, &["base"], &["\u{ffff}", "\u{10000}"]),
        Ok(vec![
            planned("\u{10000}", "1.0.0", "1.0.1"),
            planned("\u{ffff}", "1.0.0", "1.0.1"),
        ])
    );
}
