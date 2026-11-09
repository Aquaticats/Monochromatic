# Closure lint suppressions in git-policy-cli

The Dylint lint `avoidable_closure` (`package/linter/monochromatic-lint-closure`) runs over this crate as the rule `rust/avoidable-closure`.
The owner's decision `doc/decision/rust-closure-lint-dylint.md` reports a closure unless it captures state and is a direct argument whose callee parameter requires `'static`.
The only suppression is rustc's own attribute on the item or statement that holds the closure:
`#[cfg_attr(dylint_lib = "monochromatic_lint_closure", expect(avoidable_closure, reason = "..."))]`.
`expect` makes rustc report `unfulfilled_lint_expectations` as soon as it suppresses nothing,
so a suppression cannot outlive its closure.
Configuration cannot do it instead:
the rule's settings give a severity and a library path per file pattern and no allow-list of callees (`package/linter/monochromatic-lint/src/rust_rule_settings.rs`),
and lowering the severity would also let a replaceable closure in (`AGENTS.md` LN7).

## The two phases of one batch in `service_server.rs`

`land_then_release` takes the batch's landings and then its released scratch directories as one argument each,
and calls them in that order.
The order is the behaviour the control `a_batch_lands_before_it_removes_a_released_tree` guards:
a client killed while its request is still queued closes its watch first,
so one batch can hold both that request and the release of the tree its landing still reads
(`SPEC.md`,
"What a scratch announcement can name").

Neither argument can be a named function or a value that implements a trait:
the landings are the landing loop's own body,
whose locals it borrows mutably,
and the releases are the batch's own list.
A struct holding those borrows would pass the same borrowed state under another name,
and the two calls would then sit inside it,
which is the same code with the order further from the reader.
The definition carries one suppression for both argument positions,
so a later refactor that removes the closure has to remove the attribute too,
and an `expect` that then suppresses nothing fails the build.
