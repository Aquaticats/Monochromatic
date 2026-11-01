// aux-build:external_macro.rs
// edition:2024
// Policy: a closure is reported only where its span is in the user's own code. A closure the user writes as a macro
// argument is checked (the macro only moves it into place); a closure that the external macro itself generates is
// skipped because the user cannot rewrite it.

#[macro_use]
extern crate external_macro;

fn allowed_user_closure_given_to_external_spawning_macro() {
    let state = String::new();
    // verdict: allowed
    let _ = spawn_arg!(move || state.len());
}

fn reported_user_closure_given_to_external_borrowing_macro() {
    let offset = 2;
    // The macro applies the closure with `map`, which does not keep it.
    // verdict: reported
    let _ = apply_arg!(|x| x + offset);
}

fn skipped_closure_generated_by_external_macro() {
    // The expansion contains a capture-free `move || 7`, which would be reported if it were the user's own code.
    let _ = make_capture_free_closure!();
}

fn skipped_borrowing_closure_generated_by_external_macro() {
    let v = vec![1, 2];
    // The expansion contains `|x| x + v.len()` over the user's expression; the closure itself comes from the macro.
    let _ = make_borrowing_closure!(v);
}

fn main() {}
