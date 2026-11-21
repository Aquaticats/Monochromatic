//! What:
//!  Controls of the shared slot-name rule,
//!  which the store,
//!  the walk and the per-slot claim each apply.
//! Why:
//!  A name the walk accepts and the claim refuses would be two rules,
//!  so the one rule's accepts and refusals are held here.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! expect(slotNameRefusal('one')).toBeNull(); expect(slotNameRefusal('.one')).not.toBeNull();
//! ```

use super::slot_name_refusal;

/// Plain branch-name components are slot names.
#[test]
fn plain_names_are_slot_names() {
    for name in ["one", "fix-thing", "a.b", "v1.2", "x_y", "0"] {
        assert_eq!(slot_name_refusal(name), None, "{name}");
    }
}

/// Empty names,
/// path separators,
/// parent names,
/// a leading dot and any whitespace or control character are refused.
#[test]
fn every_other_name_is_refused_with_its_reason() {
    for name in ["", ".", "..", "a/b", ".hidden", "two words", "tab\there", "line\nbreak"] {
        let reason: Option<String> = slot_name_refusal(name);
        assert!(reason.is_some(), "{name}");
        assert!(!reason.unwrap_or_default().is_empty());
    }
}
