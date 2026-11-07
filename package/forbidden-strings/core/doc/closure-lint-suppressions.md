# Closure lint suppressions in the forbidden-regex engine

The Dylint lint `avoidable_closure` (`package/linter/monochromatic-lint-closure`) runs over this crate as the rule
`rust/avoidable-closure`.
The owner's decision `doc/decision/rust-closure-lint-dylint.md` reports a closure unless it captures state and is a
direct argument whose callee parameter requires `'static`.
The only suppression is rustc's own attribute on the statement that holds the closure:
`#[cfg_attr(dylint_lib = "monochromatic_lint_closure", expect(avoidable_closure, reason = "..."))]`.
`expect` makes rustc report `unfulfilled_lint_expectations` as soon as it suppresses nothing,
so a suppression cannot outlive its closure.
Configuration cannot do it instead:
the rule's settings give a severity and a library path per file pattern and no allow-list of callees
(`package/linter/monochromatic-lint/src/rust_rule_settings.rs`),
and lowering the severity would also let a replaceable closure in (`AGENTS.md` LN7).

This crate carries two suppressions,
both in `src/parallel.rs`,
one for each standard-library function that accepts only a callable.
The proof is the signature in the sysroot source,
as the Git wrapper's record shows in
`package/git-policy/cli/doc/closure-lint-suppressions.md`,
section "Standard-library functions that accept only a callable":

- `std::thread::scope`:
  `F: for<'scope> FnOnce(&'scope Scope<'scope, 'env>) -> T`.
  The callable is bounded by the scope,
  not by `'static`,
  because the workers borrow the patterns and the shared counters for the scope only.
- `Builder::spawn_scoped`:
  `F: FnOnce() -> T + Send + 'scope`.
  It passes no argument to the callable,
  so the worker's body must carry its state.

No named function can replace these closures,
because neither function passes an argument to its callable,
and no loop can run on another thread.
Every other closure the rule found in this crate was replaced:
by a named function,
a path such as `str::trim`,
a loop,
a `match`,
or a struct that implements the new `CandidateCheck` and `CandidateVisit` traits of `src/gate.rs`
(a plain closure still implements both,
so callers that have nothing to carry may pass one).
