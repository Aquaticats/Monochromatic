//! What:
//!  Controls of the decided coverage and the text it writes:
//!  the names,
//!  their order,
//!  the mount kinds,
//!  and the sections a person pastes.
//! Why:
//!  The generated text is the piece step 20 owns,
//!  so a control proves the whole decision rather than one section of it:
//!  the ancestor first with `recursive = yes`,
//!  every exclusion after it with `autosnap = no`,
//!  and the start probe's dataset among them.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const decision = decidedCoverage();
//! expect(decision.exclusions.map(e => e.dataset)).toEqual([...nine names...]);
//! expect(generatedText(decision)).toContain('[tank/home]\n  recursive = yes\n');
//! ```

use super::{ExclusionMount, GENERATED_HEADER, RECURSIVE_ROOT, TEMPLATE, decided_coverage, exclusion_section, generated_text, sections_of};

/// The host's decision keeps the design's ancestor and template.
#[test]
fn the_decision_keeps_the_designs_ancestor_and_template() {
    let decision = decided_coverage();
    assert_eq!(decision.recursive_root, RECURSIVE_ROOT);
    assert_eq!(decision.template, TEMPLATE);
    assert_eq!(decision.recursive_root, "zpcachyos/ROOT/cos/home");
    assert_eq!(decision.template, "home");
}

/// The nine exclusions stand in the design's order,
/// the probe dataset last,
/// with the two unmounted ones marked so.
#[test]
fn the_nine_exclusions_stand_in_the_designs_order() {
    let decision = decided_coverage();
    let names: Vec<&str> = decision.exclusions.iter().map(|exclusion| return exclusion.dataset.as_str()).collect();
    assert_eq!(
        names,
        vec![
            "zpcachyos/ROOT/cos/varcache",
            "zpcachyos/nosnap",
            "zpcachyos/nosnap/cargo-build",
            "zpcachyos/nosnap/cargo-registry",
            "zpcachyos/nosnap/npm",
            "zpcachyos/nosnap/user-cache",
            "zpcachyos/nosnap/vm-builder-output",
            "zpcachyos/nosnap/libvirt-images",
            "zpcachyos/nosnap/git-worktree-probe",
        ]
    );
    let unmounted: Vec<&str> = decision.exclusions.iter().filter(|exclusion| return exclusion.mount == ExclusionMount::Unmounted).map(|exclusion| return exclusion.dataset.as_str()).collect();
    assert_eq!(unmounted, vec!["zpcachyos/nosnap", "zpcachyos/nosnap/git-worktree-probe"]);
}

/// Every exclusion is one the decision names,
/// and an unlisted dataset is not.
#[test]
fn every_exclusion_is_one_the_decision_names() {
    let decision = decided_coverage();
    assert!(decision.excludes("zpcachyos/nosnap/npm"));
    assert!(!decision.excludes("zpcachyos/ROOT/cos/home"));
    assert!(!decision.excludes("zpcachyos/nosnap/npm/other"));
}

/// The ancestor's section stands first and writes the template and `yes`.
#[test]
fn the_ancestors_section_stands_first_and_writes_recursion() {
    let sections = sections_of(&decided_coverage());
    assert_eq!(sections.len(), 10, "the ancestor and nine exclusions");
    assert_eq!(sections[0].dataset, "zpcachyos/ROOT/cos/home");
    assert_eq!(sections[0].value_of("use_template"), Some("home"));
    assert_eq!(sections[0].value_of("recursive"), Some("yes"));
}

/// Every exclusion's section writes `autosnap = no` and nothing else.
#[test]
fn every_exclusions_section_writes_autosnap_no() {
    let sections = sections_of(&decided_coverage());
    for section in &sections[1..] {
        assert_eq!(section.settings.len(), 1, "{} writes one setting", section.dataset);
        assert_eq!(section.value_of("autosnap"), Some("no"), "{} turns its snapshots off", section.dataset);
    }
}

/// The generated text starts with the design's header,
/// names every dataset once,
/// and stands the ancestor before every exclusion.
#[test]
fn the_generated_text_is_the_header_and_the_sections_in_order() {
    let decision = decided_coverage();
    let text: String = generated_text(&decision);
    assert!(text.starts_with(GENERATED_HEADER), "{text}");
    assert!(text.contains("# Use yes, not zfs."));
    let ancestor_at: usize = text.find("[zpcachyos/ROOT/cos/home]\n  use_template = home\n  recursive = yes\n").expect("the ancestor section is written");
    for exclusion in &decision.exclusions {
        let header: String = format!("[{}]\n  autosnap = no\n", exclusion.dataset);
        assert_eq!(text.matches(header.as_str()).count(), 1, "{} is written once", exclusion.dataset);
        let at: usize = text.find(header.as_str()).expect("the exclusion is written");
        assert!(ancestor_at < at, "the ancestor stands before {}", exclusion.dataset);
    }
}

/// One exclusion's section is the correction text a person adds.
#[test]
fn one_exclusions_section_is_the_text_a_person_adds() {
    assert_eq!(exclusion_section("tank/nosnap/npm").dataset, "tank/nosnap/npm");
    assert_eq!(exclusion_section("tank/nosnap/npm").value_of("autosnap"), Some("no"));
}
