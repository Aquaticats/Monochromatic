// edition:2024
// Suppression is rustc-native: `#[cfg_attr(dylint_lib = "<library>", expect(<lint>, reason = ".."))]` on the closure
// argument or an enclosing item. An `expect` with nothing to suppress is rustc's own unfulfilled-expectation warning.

use std::thread;

fn reported_without_attribute() {
    let v = vec![1, 2];
    // verdict: reported
    let _ = v.iter().map(|x| x + v.len()).count();
}

fn suppressed_by_expect_on_the_call_argument() {
    let v = vec![1, 2];
    let _ = v.iter().map(
        #[cfg_attr(dylint_lib = "monochromatic_lint_closure", expect(avoidable_closure, reason = "callee keeps the state"))]
        // verdict: suppressed
        |x| x + v.len(),
    ).count();
}

#[cfg_attr(dylint_lib = "monochromatic_lint_closure", expect(avoidable_closure, reason = "enclosing item"))]
fn suppressed_by_expect_on_the_enclosing_item() {
    let v = vec![1, 2];
    // verdict: suppressed
    let _ = v.iter().map(|x| x + v.len()).count();
}

#[cfg_attr(dylint_lib = "monochromatic_lint_closure", allow(avoidable_closure, reason = "plain allow"))]
fn suppressed_by_allow_on_the_enclosing_item() {
    let v = vec![1, 2];
    // verdict: suppressed
    let _ = v.iter().map(|x| x + v.len()).count();
}

fn suppressed_by_expect_on_the_let_statement() {
    let n = 1;
    #[cfg_attr(dylint_lib = "monochromatic_lint_closure", expect(avoidable_closure, reason = "statement scope"))]
    // verdict: suppressed
    let f = move || n;
    let _ = f;
}

// verdict: unfulfilled
#[cfg_attr(dylint_lib = "monochromatic_lint_closure", expect(avoidable_closure, reason = "nothing to suppress here"))]
fn unfulfilled_expect_on_an_item_without_closures() -> i32 {
    return 3;
}

fn unfulfilled_expect_on_an_allowed_closure() {
    let state = String::new();
    // The closure is kept by `spawn`, so the lint has nothing to report and the expectation is unfulfilled.
    // verdict: unfulfilled
    let _ = thread::spawn(#[cfg_attr(dylint_lib = "monochromatic_lint_closure", expect(avoidable_closure, reason = "stale"))] move || state.len());
}

fn reported_despite_an_expect_with_a_wrong_lint_name() {
    let v = vec![1, 2];
    // verdict: unknown-lint
    #[cfg_attr(dylint_lib = "monochromatic_lint_closure", expect(avoidable_closur, reason = "misspelled"))]
    // verdict: reported
    let g = |x: usize| x + v.len();
    // verdict: reported
    let _ = (1..3).map(|x| x + v.len()).count();
    let _ = g;
}

fn reported_despite_an_attribute_for_another_library() {
    let v = vec![1, 2];
    let _ = v.iter().map(
        #[cfg_attr(dylint_lib = "some_other_library", expect(avoidable_closure, reason = "wrong library"))]
        // verdict: reported
        |x| x + v.len(),
    ).count();
}

fn main() {}
