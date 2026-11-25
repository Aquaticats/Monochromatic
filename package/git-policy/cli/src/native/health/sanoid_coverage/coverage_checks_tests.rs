//! What:
//!  Controls of the coverage comparison:
//!  the healthy machine,
//!  every drift of the ancestor,
//!  of an exclusion's dataset,
//!  its section,
//!  its order,
//!  its value and its marker,
//!  of the store,
//!  and every fact that leaves the answer unknown.
//! Why:
//!  Each drift is a state that costs space or history silently,
//!  so each one has a control that changes exactly that field and asserts the
//!  field and the correction,
//!  while the healthy facts beside it pass.
//!  A correction that does not match the reading would put a person back in
//!  the same drift,
//!  so the applied correction is read back too where the test can.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = checkCoverage(factsOf(healthyTree(), textWithout(NPM)));
//! expect(answer.findings[0].field).toBe('exclusion-section');
//! ```

use crate::doctor_zfs_route::CheckOutcome;
use crate::health::sanoid_coverage::coverage_test_facts::{
    CACHE, CONFIG_PATH, CONTAINER, NPM, PROBE, ROOT, STORE, WORKTREES, dataset_in, facts_of, healthy_config_text, healthy_facts, healthy_tree, test_decision,
};
use crate::health::sanoid_coverage::{
    ConfigRefusal, CoverageAnswer, CoverageFinding, CoverageFacts, FIELD_EXCLUSION_AUTOSNAP, FIELD_EXCLUSION_DATASET, FIELD_EXCLUSION_MARKER, FIELD_EXCLUSION_MOUNT, FIELD_EXCLUSION_SECTION, FIELD_RECURSION,
    FIELD_ROOT_SECTION, FIELD_SECTION_ORDER, FIELD_STORE_COVERAGE, FIELD_TEMPLATE, read_sanoid_conf,
};

/// What one section writes in these controls.
const ROOT_BODY: &str = "  use_template = home\n  recursive = yes\n";

/// What an exclusion's section writes in these controls.
const OFF_BODY: &str = "  autosnap = no\n";

/// A configuration text from section names and the bodies written under them.
fn text_of(sections: &[(&str, &str)]) -> String {
    let written: Vec<String> = sections.iter().map(|(dataset, body)| return format!("[{dataset}]\n{body}\n")).collect();
    return written.join("");
}

/// The decision's own text with every section present.
fn decided_text() -> String {
    return text_of(&[(ROOT, ROOT_BODY), (CACHE, OFF_BODY), (CONTAINER, OFF_BODY), (NPM, OFF_BODY), (PROBE, OFF_BODY)]);
}

/// The facts of one configuration text on the healthy tree.
fn facts_with_text(text: &str) -> CoverageFacts {
    return facts_of(healthy_tree(), text);
}

/// The fields one answer found,
/// in the order the comparison made them.
fn fields(answer: &CoverageAnswer) -> Vec<&str> {
    return answer.findings.iter().map(|entry| return entry.field.as_str()).collect();
}

/// One finding by its field.
fn finding_of<'a>(answer: &'a CoverageAnswer, field: &str) -> &'a CoverageFinding {
    return answer.findings.iter().find(|entry| return entry.field == field).unwrap_or_else(|| panic!("no {field} finding in {:?}", fields(answer)));
}

/// One finding's correction text.
fn correction_of<'a>(answer: &'a CoverageAnswer, field: &str) -> &'a str {
    return finding_of(answer, field).correction.as_deref().unwrap_or_else(|| panic!("the {field} finding carries no correction"));
}

/// The detail of the answer's own report line.
fn detail_of(answer: &CoverageAnswer) -> &str {
    return match &answer.outcome {
        CheckOutcome::Pass { detail } | CheckOutcome::Finding { detail, .. } => detail.as_str(),
        CheckOutcome::Unknown { reason } => reason.as_str(),
    };
}

/// The healthy machine passes and prints no line.
#[test]
fn the_healthy_machine_passes_and_prints_no_line() {
    let facts: CoverageFacts = healthy_facts();
    let answer: CoverageAnswer = super::check_coverage(&facts);
    assert!(matches!(answer.outcome, CheckOutcome::Pass { .. }), "{:?}", answer.outcome);
    assert_eq!(answer.findings, Vec::new());
    assert_eq!(answer.unproved, None);
    assert_eq!(answer.lines(), Vec::<String>::new());
    assert!(detail_of(&answer).contains("1 dataset(s) under tank/home"), "{}", detail_of(&answer));
    assert!(detail_of(&answer).contains(STORE));
}

/// A configuration with no ancestor section is a finding naming the section to
/// add,
/// and the correction reads back as the section the check wanted.
#[test]
fn a_missing_ancestor_section_is_a_finding_with_the_section_to_add() {
    let answer: CoverageAnswer = super::check_coverage(&facts_with_text(&text_of(&[(CACHE, OFF_BODY)])));
    assert_eq!(finding_of(&answer, FIELD_ROOT_SECTION).field, FIELD_ROOT_SECTION);
    let correction: &str = correction_of(&answer, FIELD_ROOT_SECTION);
    assert!(correction.contains(&format!("add these lines to {CONFIG_PATH}:\n[{ROOT}]\n")), "{correction}");
    let section_text: &str = correction.split_once('\n').expect("a sentence and the text").1;
    let read = read_sanoid_conf(&format!("{section_text}\n[{}]\n{OFF_BODY}", CACHE)).expect("the correction reads");
    assert_eq!(read.sections[0].dataset, ROOT);
    assert_eq!(read.sections[0].written_value("recursive"), Some("yes"));
    assert_eq!(read.sections[0].written_value("use_template"), Some("home"));
}

/// `recursive = zfs` is a finding whose correction writes `yes`.
#[test]
fn recursion_zfs_is_a_finding_with_the_correction_that_writes_yes() {
    let answer: CoverageAnswer = super::check_coverage(&facts_with_text(decided_text().replace("recursive = yes", "recursive = zfs").as_str()));
    assert_eq!(fields(&answer), vec![FIELD_RECURSION]);
    assert!(finding_of(&answer, FIELD_RECURSION).detail.contains("no exclusion can be left out"));
    assert!(correction_of(&answer, FIELD_RECURSION).contains("change `recursive = zfs` to `recursive = yes`"));
}

/// A section with no recursion value gets the correction that adds one.
#[test]
fn recursion_absent_is_a_finding_with_the_correction_that_adds_it() {
    let answer: CoverageAnswer = super::check_coverage(&facts_with_text(decided_text().replace("  recursive = yes\n", "").as_str()));
    assert_eq!(fields(&answer), vec![FIELD_RECURSION]);
    assert!(correction_of(&answer, FIELD_RECURSION).contains("add `recursive = yes`"));
}

/// A recursion value sanoid does not read as true is named as written.
#[test]
fn recursion_not_a_true_word_is_a_finding_that_names_the_value() {
    let answer: CoverageAnswer = super::check_coverage(&facts_with_text(decided_text().replace("recursive = yes", "recursive = yess").as_str()));
    assert_eq!(fields(&answer), vec![FIELD_RECURSION]);
    assert!(finding_of(&answer, FIELD_RECURSION).detail.contains("recursive = yess"));
}

/// Another template is a finding whose correction names the decided one.
#[test]
fn another_template_is_a_finding() {
    let answer: CoverageAnswer = super::check_coverage(&facts_with_text(decided_text().replace("use_template = home", "use_template = other").as_str()));
    assert_eq!(fields(&answer), vec![FIELD_TEMPLATE]);
    assert!(correction_of(&answer, FIELD_TEMPLATE).contains("to name `home`"));
}

/// A section that names no template is a finding whose correction adds one.
#[test]
fn a_section_with_no_template_is_a_finding() {
    let answer: CoverageAnswer = super::check_coverage(&facts_with_text(decided_text().replace("  use_template = home\n", "").as_str()));
    assert_eq!(fields(&answer), vec![FIELD_TEMPLATE]);
    assert!(correction_of(&answer, FIELD_TEMPLATE).contains("add `use_template = home`"));
}

/// An excluded dataset the tree does not hold is a finding that names why no
/// correction can end it.
#[test]
fn an_excluded_dataset_the_tree_lacks_is_a_finding() {
    let mut tree = healthy_tree();
    tree.datasets.retain(|dataset| return dataset.name != NPM);
    let answer: CoverageAnswer = super::check_coverage(&facts_of(tree, decided_text().as_str()));
    assert_eq!(fields(&answer), vec![FIELD_EXCLUSION_DATASET]);
    assert!(finding_of(&answer, FIELD_EXCLUSION_DATASET).detail.contains("its own dataset"));
    assert_eq!(finding_of(&answer, FIELD_EXCLUSION_DATASET).correction, None);
}

/// A mounted exclusion that reads no path is a finding with no correction.
#[test]
fn a_mounted_exclusion_with_no_path_mount_point_is_a_finding() {
    let mut tree = healthy_tree();
    dataset_in(&mut tree, CACHE).mountpoint = Some(String::from("none"));
    let answer: CoverageAnswer = super::check_coverage(&facts_of(tree, decided_text().as_str()));
    assert_eq!(fields(&answer), vec![FIELD_EXCLUSION_MOUNT]);
    assert!(finding_of(&answer, FIELD_EXCLUSION_MOUNT).detail.contains("covers no folder"));
    assert_eq!(finding_of(&answer, FIELD_EXCLUSION_MOUNT).correction, None);
}

/// An unmounted exclusion that reads a path is a finding.
#[test]
fn an_unmounted_exclusion_that_reads_a_path_is_a_finding() {
    let mut tree = healthy_tree();
    dataset_in(&mut tree, CONTAINER).mountpoint = Some(String::from("/fixture/nosnap"));
    let answer: CoverageAnswer = super::check_coverage(&facts_of(tree, decided_text().as_str()));
    assert_eq!(fields(&answer), vec![FIELD_EXCLUSION_MOUNT]);
    assert!(finding_of(&answer, FIELD_EXCLUSION_MOUNT).detail.contains("leaves tank/nosnap unmounted"));
}

/// A missing exclusion section is a finding whose correction reads back as that
/// section.
#[test]
fn a_missing_exclusion_section_is_a_finding_with_its_section() {
    let text: String = text_of(&[(ROOT, ROOT_BODY), (CACHE, OFF_BODY), (CONTAINER, OFF_BODY), (PROBE, OFF_BODY)]);
    let answer: CoverageAnswer = super::check_coverage(&facts_with_text(text.as_str()));
    assert_eq!(fields(&answer), vec![FIELD_EXCLUSION_SECTION]);
    let correction: &str = correction_of(&answer, FIELD_EXCLUSION_SECTION);
    assert!(correction.contains(&format!("[{NPM}]\n  autosnap = no\n")), "{correction}");
}

/// An exclusion whose warning comment stayed in its value is still snapshotted,
/// and the correction is the value the decision writes.
#[test]
fn an_exclusion_whose_value_keeps_a_comment_is_a_finding() {
    let text: String = decided_text().replace("[tank/nosnap/npm]\n  autosnap = no", "[tank/nosnap/npm]\n  autosnap = no # keep");
    let answer: CoverageAnswer = super::check_coverage(&facts_with_text(text.as_str()));
    assert_eq!(fields(&answer), vec![FIELD_EXCLUSION_AUTOSNAP]);
    assert!(correction_of(&answer, FIELD_EXCLUSION_AUTOSNAP).contains("change `autosnap = no # keep` to `autosnap = no`"));
}

/// An exclusion whose section writes no value is a finding whose correction is
/// its section.
#[test]
fn an_exclusion_with_no_autosnap_value_is_a_finding() {
    let text: String = text_of(&[(ROOT, ROOT_BODY), (CACHE, OFF_BODY), (CONTAINER, OFF_BODY), (NPM, ""), (PROBE, OFF_BODY)]);
    let answer: CoverageAnswer = super::check_coverage(&facts_with_text(text.as_str()));
    assert_eq!(fields(&answer), vec![FIELD_EXCLUSION_AUTOSNAP]);
    assert!(correction_of(&answer, FIELD_EXCLUSION_AUTOSNAP).contains("autosnap = no"));
}

/// A section written before the ancestor whose recursion reaches it is a finding
/// whose correction moves it.
#[test]
fn a_section_before_its_recursive_ancestor_is_a_finding() {
    let text: String = text_of(&[(NPM, OFF_BODY), (ROOT, ROOT_BODY), (CACHE, OFF_BODY), (CONTAINER, OFF_BODY), (PROBE, OFF_BODY)]);
    let answer: CoverageAnswer = super::check_coverage(&facts_with_text(text.as_str()));
    assert_eq!(fields(&answer), vec![FIELD_SECTION_ORDER]);
    assert!(correction_of(&answer, FIELD_SECTION_ORDER).contains(&format!("move the [{NPM}] section below the [{ROOT}] section")));
    assert!(finding_of(&answer, FIELD_SECTION_ORDER).detail.contains("without checking for a child section"));
}

/// An ancestor section without recursion does not reorder anything,
/// so a section after the recursive ancestor and before it stands in no drift.
#[test]
fn a_section_before_an_ancestor_without_recursion_is_no_finding() {
    let text: String = text_of(&[(ROOT, ROOT_BODY), (NPM, OFF_BODY), (CONTAINER, OFF_BODY), (CACHE, OFF_BODY), (PROBE, OFF_BODY)]);
    let answer: CoverageAnswer = super::check_coverage(&facts_with_text(text.as_str()));
    assert_eq!(fields(&answer), Vec::<&str>::new());
}

/// A section before a recursing dataset ancestor that stands later is a finding
/// whose correction names the section it has to pass,
/// which is the latest of the sections that reach it.
#[test]
fn a_section_before_a_later_recursing_ancestor_is_a_finding() {
    let text: String = text_of(&[(ROOT, ROOT_BODY), (NPM, OFF_BODY), (CONTAINER, "  autosnap = no\n  recursive = yes\n"), (CACHE, OFF_BODY), (PROBE, OFF_BODY)]);
    let answer: CoverageAnswer = super::check_coverage(&facts_with_text(text.as_str()));
    assert_eq!(fields(&answer), vec![FIELD_SECTION_ORDER]);
    assert!(correction_of(&answer, FIELD_SECTION_ORDER).contains(&format!("below the [{CONTAINER}] section")));
}

/// An unset marker is a finding whose correction is the `zfs set`.
#[test]
fn an_unset_marker_is_a_finding_with_the_zfs_set() {
    let mut tree = healthy_tree();
    dataset_in(&mut tree, NPM).auto_snapshot = None;
    let answer: CoverageAnswer = super::check_coverage(&facts_of(tree, decided_text().as_str()));
    assert_eq!(fields(&answer), vec![FIELD_EXCLUSION_MARKER]);
    assert_eq!(correction_of(&answer, FIELD_EXCLUSION_MARKER), format!("zfs set com.sun:auto-snapshot=false {NPM}"));
    assert!(finding_of(&answer, FIELD_EXCLUSION_MARKER).detail.contains("is not set"));
}

/// A marker set to another value is a finding that reads it.
#[test]
fn a_marker_set_to_another_value_is_a_finding_that_reads_it() {
    let mut tree = healthy_tree();
    dataset_in(&mut tree, PROBE).auto_snapshot = Some(String::from("true"));
    let answer: CoverageAnswer = super::check_coverage(&facts_of(tree, decided_text().as_str()));
    assert_eq!(fields(&answer), vec![FIELD_EXCLUSION_MARKER]);
    assert!(finding_of(&answer, FIELD_EXCLUSION_MARKER).detail.contains("reads true"));
}

/// A store held by an excluded dataset is a finding with no correction.
#[test]
fn a_store_held_by_an_excluded_dataset_is_a_finding() {
    let mut facts: CoverageFacts = healthy_facts();
    facts.store_path = String::from("/fixture/cache/pnpm/store");
    let answer: CoverageAnswer = super::check_coverage(&facts);
    assert_eq!(fields(&answer), vec![FIELD_STORE_COVERAGE]);
    assert!(finding_of(&answer, FIELD_STORE_COVERAGE).detail.contains(CACHE));
    assert_eq!(finding_of(&answer, FIELD_STORE_COVERAGE).correction, None);
}

/// A store no measured mount point holds answers unknown.
#[test]
fn a_store_no_mount_point_holds_answers_unknown() {
    let mut facts: CoverageFacts = healthy_facts();
    facts.store_path = String::from("/elsewhere/pnpm/store");
    let answer: CoverageAnswer = super::check_coverage(&facts);
    assert!(matches!(answer.outcome, CheckOutcome::Unknown { .. }), "{:?}", answer.outcome);
    assert!(answer.unproved.as_deref().is_some_and(|reason| return reason.contains("cannot prove the store is snapshotted")));
    assert!(answer.lines()[0].starts_with("unknown: "));
}

/// A configuration the reading refuses answers unknown and names the file.
#[test]
fn a_configuration_the_reading_refuses_answers_unknown() {
    let mut facts: CoverageFacts = healthy_facts();
    facts.config = Err(ConfigRefusal::Line { line: 3, problem: String::from("a setting must name a key") });
    let answer: CoverageAnswer = super::check_coverage(&facts);
    assert!(matches!(answer.outcome, CheckOutcome::Unknown { .. }), "{:?}", answer.outcome);
    assert!(detail_of(&answer).contains(CONFIG_PATH) && detail_of(&answer).contains("line 3"));
    assert_eq!(answer.findings, Vec::new());
}

/// A section that replaces its dataset with `path` answers unknown,
/// because this build does not model what that changes.
#[test]
fn a_section_that_sets_path_answers_unknown() {
    let text: String = decided_text().replace(ROOT_BODY, "  use_template = home\n  recursive = yes\n  path = tank/other\n");
    let answer: CoverageAnswer = super::check_coverage(&facts_with_text(text.as_str()));
    assert!(matches!(answer.outcome, CheckOutcome::Unknown { .. }), "{:?}", answer.outcome);
    assert!(detail_of(&answer).contains("path = tank/other"));
}

/// A key spelled with other capitals answers unknown,
/// because sanoid refuses a setting nobody proved it reads.
#[test]
fn a_key_with_other_capitals_answers_unknown() {
    let text: String = decided_text().replace("[tank/nosnap/npm]\n  autosnap = no", "[tank/nosnap/npm]\n  Autosnap = no");
    let answer: CoverageAnswer = super::check_coverage(&facts_with_text(text.as_str()));
    assert!(matches!(answer.outcome, CheckOutcome::Unknown { .. }), "{:?}", answer.outcome);
    assert!(detail_of(&answer).contains("Autosnap"));
}

/// Several drifts are all reported,
/// each with its own correction line,
/// and the answer's detail names every field.
#[test]
fn several_drifts_are_all_reported_with_their_corrections() {
    let mut tree = healthy_tree();
    dataset_in(&mut tree, PROBE).auto_snapshot = Some(String::from("true"));
    let text: String = text_of(&[(NPM, OFF_BODY), (ROOT, ROOT_BODY.replace("recursive = yes", "recursive = zfs").as_str()), (CACHE, OFF_BODY), (CONTAINER, OFF_BODY)]);
    let answer: CoverageAnswer = super::check_coverage(&facts_of(tree, text.as_str()));
    assert_eq!(fields(&answer), vec![FIELD_RECURSION, FIELD_SECTION_ORDER, FIELD_EXCLUSION_SECTION, FIELD_EXCLUSION_MARKER]);
    let detail: &str = detail_of(&answer);
    for field in fields(&answer) {
        assert!(detail.contains(field), "{detail} names {field}");
    }
    let lines: Vec<String> = answer.lines();
    assert!(lines.iter().any(|line| return line.starts_with("finding: exclusion-marker: ")));
    assert!(lines.iter().any(|line| return line == &format!("correction: zfs set com.sun:auto-snapshot=false {PROBE}")));
    assert!(lines.iter().any(|line| return line.starts_with("correction: move the [tank/nosnap/npm] section")));
}

/// The decision's own text on the healthy tree is what the healthy facts hold,
/// so the pass above is the generated text read back.
#[test]
fn the_healthy_text_is_the_decisions_own() {
    assert_eq!(healthy_config_text(), decided_text());
    assert_eq!(test_decision().exclusions.len(), 4);
    assert_eq!(healthy_tree().datasets_under(ROOT).len(), 1, "only {WORKTREES} stands under the ancestor");
}
