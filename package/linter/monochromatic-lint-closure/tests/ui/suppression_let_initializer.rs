// edition:2024
// An attribute on a closure that is a `let` initializer is rejected by stable-compatible rustc with E0658
// (attributes on expressions are experimental), so the lint's help for let-bound closures says to restructure or to
// attach the attribute to the `let` statement (see suppression.rs, `suppressed_by_expect_on_the_let_statement`).

fn rejected_attribute_on_the_initializer() {
    let n = 1;
    // verdict: error E0658
    let f = #[cfg_attr(dylint_lib = "monochromatic_lint_closure", expect(avoidable_closure, reason = "initializer"))] move || n;
    let _ = f;
}

fn main() {}
