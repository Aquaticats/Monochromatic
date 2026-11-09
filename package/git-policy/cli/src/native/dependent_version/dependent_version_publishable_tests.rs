//! What:
//! Publishable-name reading,
//! over the document the generator writes,
//! a narrower list
//!       above the publish set,
//! and every text the reader refuses.
//! Why:
//! A name missing from the list is never bumped,
//! and a publish set the reader cannot find must
//!      never read as "nothing to bump".
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // expect(readPublishableNames(configText)).toEqual(['@s/a']);
//! ```

/// The reader under test.
use super::read_publishable_names;

/// The names a text lists,
/// in order.
fn names(text: &str) -> Vec<String> {
    return read_publishable_names(text)
        .unwrap_or_else(|reason: String| panic!("expected names, got {reason}"));
}

/// The reason a text is refused.
fn reason(text: &str) -> String {
    return read_publishable_names(text)
        .err()
        .unwrap_or_else(|| panic!("expected a problem for {text:?}"));
}

/// The provider the generator writes,
/// with one workload whose package list is `items`.
fn provider(items: &str) -> String {
    return [
        "auth:\n",
        "  oidc:\n",
        "    - name: github\n",
        "      issuer: https://example\n",
        "      workloads:\n",
        "        - identity:\n",
        "            subject: 'repo:example/main'\n",
        "          registry: monochromatic-dev\n",
        "          packages:\n",
        items,
        "\n",
    ]
    .concat();
}

/// The list items of one workload.
fn items(names: &[&str]) -> String {
    let listed: Vec<String> = names
        .iter()
        .map(|name: &&str| return format!("            - '{name}'"))
        .collect();
    return listed.join("\n");
}

/// The publish set is the workload list,
/// not the permission pattern below it,
/// and
///       the rest of the generated file is read on the way.
#[test]
fn reads_the_workload_list_of_the_generated_document() {
    let mut text: String = provider(&items(&["@s/a", "@s/b"]));
    text.push_str(concat!(
        "registries:\n",
        "  monochromatic-dev:\n",
        "    type: hosted\n",
        "    access: $all\n",
        "    packages:\n",
        "      '@monochromatic-dev/*':\n",
        "        access: $all\n",
        "        publish: pnpr-publish-ci\n",
    ));
    assert_eq!(names(&text), ["@s/a", "@s/b"]);
}

/// A narrower workload list above the publish set does not hide it:
/// the set is
///       every list a trusted workload may publish.
#[test]
fn reads_every_workload_list_below_the_first() {
    let text: &str = "\
auth:
  oidc:
    - name: github
      workloads:
        - identity:
            subject: 'repo:example/release'
          registry: monochromatic-dev
          packages:
            - '@s/narrow'
        - identity:
            subject: 'repo:example/main'
          registry: monochromatic-dev
          packages:
            - '@s/a'
            - '@s/b'
";
    assert_eq!(names(text), ["@s/narrow", "@s/a", "@s/b"]);
}

/// A second provider above the first counts too,
/// and a name listed twice is read
///       once.
#[test]
fn reads_every_provider_and_drops_a_repeat() {
    let text: &str = "\
auth:
  oidc:
    - name: release-branch
      workloads:
        - registry: monochromatic-dev
          packages:
            - '@s/a'
    - name: github
      workloads:
        - registry: monochromatic-dev
          packages:
            - '@s/a'
            - '@s/b'
";
    assert_eq!(names(text), ["@s/a", "@s/b"]);
}

/// An empty publish set,
/// a document without `auth.oidc`,
/// and a workload whose
///       `packages` is a mapping are each refused by name.
#[test]
fn refuses_a_publish_set_it_cannot_read() {
    assert_eq!(
        reason(&provider("")),
        "no `auth.oidc` workload lists a package name"
    );
    assert_eq!(
        reason("registries:\n  monochromatic-dev:\n    packages:\n      '@s/*':\n"),
        "it has no `auth.oidc` list of providers"
    );
    assert_eq!(
        reason("auth:\n  oidc: monochromatic-dev\n"),
        "it has no `auth.oidc` list of providers"
    );
    assert_eq!(
        reason("auth:\n  oidc:\n    - workloads:\n        - registry: r\n"),
        "no `auth.oidc` workload lists a package name"
    );
    assert_eq!(
        reason("auth:\n  oidc:\n    - workloads: many\n"),
        "`auth.oidc[0].workloads` is not a list of workloads"
    );
    assert_eq!(
        reason("auth:\n  oidc:\n    - workloads:\n        - packages:\n            a: b\n"),
        "`auth.oidc[0].workloads[0].packages` is not a list of package names"
    );
    assert_eq!(
        reason("auth:\n  oidc:\n    - workloads:\n        - packages:\n            - ''\n"),
        "`auth.oidc[0].workloads[0].packages` holds an empty package name"
    );
    assert_eq!(
        reason("auth:\n  oidc:\n    - workloads:\n        - packages:\n            - k: v\n"),
        "`auth.oidc[0].workloads[0].packages` holds a value that is not a package name"
    );
}

/// A text this reader cannot read is refused with the line it stopped at,
/// so
///       the diagnostic names a place to look.
#[test]
fn refuses_text_it_cannot_read_by_line() {
    assert_eq!(
        reason("auth:\n\toidc:\n"),
        "is indented with a tab, which starts no block on line 2"
    );
    assert_eq!(
        reason("auth: [\n"),
        "begins with '[', which opens a value this reader does not read on line 1"
    );
}
