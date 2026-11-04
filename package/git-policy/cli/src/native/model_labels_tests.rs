//! What:
//!  A guard on the evidence labels of the model modules: every `model_*.rs` module is either one of the revision 13 modules (the only
//!  approval evidence for revision 13) or opens with the label `Historical control, not approval evidence for revision 13`.
//! Why:
//!  The third models review asked that retired or narrowly scoped greens be labelled out of the approval evidence. A new model file that
//!  is neither listed nor labelled fails here, so an old green cannot be read as current evidence by omission.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // for (const file of readdirSync(dir).filter(isModel)) expect(approval.has(file) || firstLine(file).startsWith(LABEL)).toBe(true);
//! ```

use std::fs;
use std::path::PathBuf;

/// The label every historical module opens with.
const LABEL: &str = "//! Historical control, not approval evidence for revision 13:";

/// The modules that are evidence for revision 13, and the tools and test files that are not models.
const CURRENT: [&str; 24] = [
    "model_explore.rs",
    "model_fixture.rs",
    "model_w1_r13.rs",
    "model_register_r13.rs",
    "model_barrier_r13.rs",
    "model_capture_r13.rs",
    "model_watch_r13.rs",
    "model_pipe_r13.rs",
    "model_m1_split_r13.rs",
    "model_discharge_r14.rs",
    "model_bound_r14.rs",
    "model_merge_r14.rs",
    "model_merge_r14_tests.rs",
    "model_bound_r14_tests.rs",
    "model_discharge_r14_tests.rs",
    "model_m1_split_r13_tests.rs",
    "model_tests.rs",
    "model_labels_tests.rs",
    "model_w1_r13_tests.rs",
    "model_register_r13_tests.rs",
    "model_barrier_r13_tests.rs",
    "model_capture_r13_tests.rs",
    "model_watch_r13_tests.rs",
    "model_pipe_r13_tests.rs",
];

/// Every model module is current or labelled historical.
#[test]
fn every_model_module_is_current_or_labelled_historical() {
    let directory: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("native");
    let mut checked: usize = 0;
    for entry in fs::read_dir(&directory).expect("the source directory").flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        let is_model: bool = name.starts_with("model_") && name.ends_with(".rs") && !name.ends_with("_tests.rs");
        if !is_model || CURRENT.contains(&name.as_str()) {
            continue;
        }
        let text: String = fs::read_to_string(entry.path()).expect("the module source");
        assert!(text.starts_with(LABEL), "{} is neither a revision 13 module nor labelled historical", name);
        checked += 1;
    }
    assert!(checked > 0, "no historical module was found, so the scan did not run");
}

/// A revision 13 module never carries the historical label (the label would hide current evidence).
#[test]
fn the_current_modules_do_not_carry_the_historical_label() {
    let directory: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src").join("native");
    for name in CURRENT {
        let text: String = fs::read_to_string(directory.join(name)).expect("the module source");
        assert!(!text.starts_with(LABEL), "{} is current evidence", name);
    }
}
