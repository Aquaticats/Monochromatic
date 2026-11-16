//! What:
//!  Guard for the one way a sweep removes a tree it has decided against,
//!  and the one way a maker is told a tree is being removed.
//! Why:
//!  A sweep that looks at who holds a tree and then removes it without the claim
//!  removes the tree under a holder that arrives in between,
//!  which is the seventh review's third finding in the pool's sweep
//!  (`doc/`, that review;
//!  `foundation/tree_held_by_a_live_process.rs`,
//!  `remove_as_a_sweep`).
//!  The claim is taken by one function, it is held from before the removal until the removal is over,
//!  and the take itself is private to the module that owns the claim,
//!  so a fourth sweep cannot be written that looks and lets go:
//!  it has no way to take a claim and no way to remove a swept tree except through the one function.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const file of SWEEP_FILES) { expect(file).toContain('remove_as_a_sweep('); expect(file).not.toContain('whether_held('); expect(file).not.toContain('take_to_remove(') }
//! ```

/// The modules that remove, or decide about removing, a tree a sweep has decided against,
/// with their release text.
/// Why:
///  A guard has to name the files it reads,
///  because a new sweep is a new file.
const SWEEP_FILES: [(&str, &str); 3] = [
    ("shadow/shadow_pool.rs", include_str!("../shadow/shadow_pool.rs")),
    ("shadow/shadow_drawn.rs", include_str!("../shadow/shadow_drawn.rs")),
    ("foundation/scratch_location.rs", include_str!("../foundation/scratch_location.rs")),
];

/// The modules that decide whether a tree may go without removing one,
/// with their release text.
/// Why:
///  A decision and a removal are two files' business,
///  and the rule is that the decision never asks who holds the tree:
///  only the removal's own claim answers that.
const AGE_DECISION_FILES: [(&str, &str); 1] = [
    ("landing/service_scratch_abandonment.rs", include_str!("../landing/service_scratch_abandonment.rs")),
];

/// What:
///  Every sweep removes through the one function that holds the claim,
///  and no sweep looks at who holds a tree, and no sweep takes a claim itself.
/// Why:
///  This is the sentence the code is meant to keep true,
///  and a reader needs to know which file broke it.
#[test]
fn every_sweep_removes_under_the_claim() {
    for (name, source) in SWEEP_FILES {
        assert!(
            source.contains("remove_as_a_sweep("),
            "{name} removes a swept tree through remove_as_a_sweep, which holds the claim until the removal is over"
        );
        assert!(
            !source.contains("whether_held("),
            "{name} asks no question of who holds a tree: a look that lets go is the defect the claim exists to prevent"
        );
        assert!(
            !source.contains("take_to_remove("),
            "{name} takes no removal claim of its own: the take belongs to remove_as_a_sweep"
        );
    }
}

/// What:
///  The age decision asks nothing of who holds a tree.
/// Why:
///  A look here whose answer a later removal does not hold is the whole of the defect:
///  the removal's claim is the one answer, and it is taken by `remove_as_a_sweep`.
#[test]
fn the_age_decision_asks_nobody_who_holds_a_tree() {
    for (name, source) in AGE_DECISION_FILES {
        assert!(
            !source.contains("whether_held("),
            "{name} decides by age alone, and the removal's claim decides who holds the tree"
        );
    }
}

/// What:
///  The take and the claim are the claim module's own,
///  so no other file of the crate can take one and then remove without holding it.
/// Why:
///  A private take is what makes the one removal function the only door,
///  and this is the line that says it.
#[test]
fn the_removal_claim_is_taken_by_one_function_of_one_module() {
    let holders: &str = include_str!("../foundation/tree_held_by_a_live_process.rs");
    assert!(
        holders.contains("\nfn take_to_remove("),
        "the take is a function of the claim's own module, and no other module's to call"
    );
    assert!(
        !holders.contains("pub fn take_to_remove("),
        "and no other module can name it"
    );
    assert!(
        holders.contains("pub fn remove_as_a_sweep<F>("),
        "the one removal function is public to the crate"
    );
}
