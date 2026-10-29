# Rust closure rule as a Dylint lint

## Accepted decision

On 2026-10-18 the user chose, from three options, a rustc-driver lint through Dylint plus comment directives.
The alternatives were a callee allow-list in `monochromatic-lint` and a type criterion inside `monochromatic-lint`
with a build-generated load.
The evidence is in [the proposal][proposal], sections "Spike results", "Research results" and "Round two results".

The Rust closure rule becomes a Dylint library lint.
It reports a closure unless two things hold:
the closure captures state, and it is a direct argument whose callee parameter requires `'static`
(so the callee keeps the state past the call and no named function, loop, or `match` can replace the closure).
rustc answers both questions: captures from `tcx.upvars_mentioned`, the bound from the callee's `TypeOutlives` clauses.

## Suppression

The Dylint lint is suppressed with rustc's own attributes:
`#[cfg_attr(dylint_lib = "<library>", expect(<lint>, reason = "..."))]` on the closure argument or an enclosing item.
rustc then reports an expectation that suppresses nothing (`unfulfilled_lint_expectations`).
The comment directives of `doc/planning/rust-linter-oxlint-parity.md` (decision D2) remain decided for
`monochromatic-lint`'s own rules; they are a separate piece of work and the Dylint lint does not use them.
If the owner meant the comment form for the Dylint lint too, that reading was not asked and is open to veto.

## What the user approved with the choice

- Using the Dylint crates (`dylint_linting`, `dylint_testing`, `cargo-dylint`, `dylint-link`).
- The `rustc-dev` toolchain component for the package that builds the lint.

## Root toolchain change approved on 2026-10-18

Stage 1 kept the lint's toolchain package-local (a dated nightly in the package) because the root toolchain was not
approved to change.
On 2026-10-18 the owner answered three questions through the question tool, and stage 2a carries them out:

- Add `rustc-dev` to the shared floating nightly.
  `mise.no-env.toml` lists the component and the tools `cargo-dylint` and `dylint-link`,
  `mise.toml` is regenerated from it with file-enforcer,
  and `mise.lock` records the tools and the new component list.
  Changing the component list changes the locked options of the `rust` entry,
  so refreshing the lock also moved the floating nightly from `nightly-2026-10-17` to `nightly-2026-10-18`.
- Exempt `**/tests/ui/**` from `rust/no-anonymous-functions` in `monochromatic-lint.config.jsonc`,
  because the lint's fixtures contain closures on purpose
  (the exemption was removed together with the rule,
  see the section "Removal of `rust/no-anonymous-functions`").
- The two stray mise symlinks under `~/.local/share/mise/installs` were repaired by the coordinator.

The package therefore has no `rust-toolchain.toml` and no `RUSTUP_TOOLCHAIN` override;
it uses the root toolchain and its aggregate tasks are named `build` and `lint` again,
so the root fan-out reaches them.
Still not approved:
any other change to the root toolchain declaration,
a pin of the nightly,
an edit to the IDE package,
a push,
or a publish.
A floating-nightly upgrade can break the lint source,
because rustc's internal API moves;
the package README says so.

## Staging

1. A new package for the lint with tests that include positive controls and adversarial cases,
   a README, and tasks in its `mise.toml`.
   No change to the IDE package.
2. Run it over the IDE, record the verdicts for the 67 closures, and propose the attribute placement and the
   `check-cfg` entry the IDE needs (package `Cargo.toml` files are managed by file-enforcer).
3. After review: the IDE integration, the five known reports, and the fate of the syntax-only
   `rust/no-anonymous-functions` rule in `monochromatic-lint`.

## Removal of `rust/no-anonymous-functions`

On 2026-10-18,
after the IDE gate landed (stage 2b),
the owner decided through the question tool to remove the syntax-only `rust/no-anonymous-functions` rule
of `package/linter/monochromatic-lint` altogether,
repo-wide.
This closes the third staging step.
No other rule is loosened:
`rust/max-lines`,
`rust/require-rustdoc`,
`rust/tests-in-sidecars`,
`rust/require-explicit-types`,
and the Markdown rules are unchanged.

What was removed:

- The rule module and its tests,
  its settings key,
  its dispatch arms (host files and virtual Rust in fences and doc tests),
  its entry in `--rules` and in the starter configuration,
  and its rule identifier.
- The rule's setting and the `ui-fixture-exemptions` block in `monochromatic-lint.config.jsonc`
  (the block existed only to exempt the lint's UI fixtures from this rule).
- The fuzz and mutation plumbing that named the rule.

What a configuration that still names the rule gets:
an unknown built-in rule,
the same setup error as any other unregistered rule id
(`Unknown built-in rule rust/no-anonymous-functions.`,
exit status 2,
nothing linted).
The linter has no aliases or deprecation list for removed ids,
so no special case was added.

What replaced it:
the Dylint lint `avoidable_closure` of `package/linter/monochromatic-lint-closure`,
run by a package's own `lint:closures` task.
Only `package/ide/app` has that task today.
Note of 2026-10-19:
`monochromatic-lint` now runs the lint for every Rust package as the rule `rust/avoidable-closure`,
and the IDE task was removed;
see the section "Implementation of 2026-10-19" at the end of this record.

Consequences:

- Rust packages other than the IDE no longer have any closure check
  until each gets its own `lint:closures` Dylint gate.
  Before the removal,
  the syntax-only rule reported at warning severity in every package.
- The lint's known limits (section "Known limits to carry into the lint's README") apply to the only gate that remains.
  Wrappers such as `Arc::new(closure)` are not recognised,
  and only code that compiles is linted,
  so features,
  `cfg`,
  and the selected targets decide coverage.
- Closures inside macro arguments that the syntax-only rule never saw are reported by the Dylint lint
  where the closure span is in the author's own code.

## Owner requirement of 2026-10-19

Asked whether to add a `lint:closures` gate to the other Rust packages,
the owner answered through the question tool that `avoidable_closure` should be enabled by default everywhere in
the linter, and that there should be no separate `lint:closures` task.
The owner had taken the closure check for a built-in rule of `monochromatic-lint` enabled everywhere;
it was until the removal of 2026-10-18, and the consequence above (only the IDE keeps a check) was not known to them.
The mechanism is not decided:
the lint needs rustc type information,
so `monochromatic-lint` would have to run the Dylint library over each Rust package itself,
or carry the criterion in its own type analysis (the rejected option of the original choice).
The feasibility and the options are in `doc/planning/closure-lint-everywhere.md`.
### Answers of 2026-10-19 to the proposal

The owner answered four questions about `doc/planning/closure-lint-everywhere.md` through the question tool:

- Mechanism A: `monochromatic-lint` runs `cargo dylint` for each Rust package itself,
  with the library path named in the rule options and a per-package manifest of build prerequisites.
- Severity: warnings first.
- Exemptions: tests, fuzz crates and bench crates are exempt.
- A package whose prerequisites are missing on the machine is skipped, and the linter says so.

Adopted as defaults without asking, open to veto:
the floating nightly stays (no pin),
only rustc's `expect` suppresses the Dylint lint (no comment directives),
generated code is skipped by the lint itself (spans outside the package's own source),
and the IDE keeps error severity because it has no findings.
## Known limits to carry into the lint's README

- Wrappers are not recognised (`Arc::new(closure)`, `Some(closure)`, struct fields, `Option<F>`, a callee that is
  a value of fn-trait type); the spike counted none in the final IDE, and 8 `Arc::new(closure)` plus 37
  `Box::new(closure)` elsewhere in the repository.
- Only code that compiles is linted: features, `cfg`, and `--all-targets` decide coverage.
- Dylint builds the target with the toolchain that built the library.
- The generated Slint window type is unverified under rustc; the IDE build needs its runtime prepared first
  (`mise run //package/ide/app:runtime`).

## Implementation of 2026-10-19

Mechanism A of `doc/planning/closure-lint-everywhere.md` is built on the branch `feat/linter-closure-everywhere`.
`monochromatic-lint` runs `cargo dylint --path <library>` for each Rust package that selects the rule
`rust/avoidable-closure`,
at warning severity by default,
with the library named in the rule's `library` option.
It compiles the library and binaries only,
skips directories ending in `.fuzz` or `.bench`,
and reads the per-package prerequisites from `monochromatic-lint.prerequisites.jsonc`.
A package with a missing prerequisite or a cold cargo cache is skipped with a message;
the linter does not fetch.
The lint itself skips closures in generated code,
so the Slint window code needs no per-package attribute,
and only rustc's `expect` suppresses.
The IDE keeps error severity through a per-path block of `monochromatic-lint.config.jsonc`.
The behaviour is documented in `package/linter/monochromatic-lint/README.md`,
section "Closure check".
Activation in a checkout needs a released `monochromatic-lint` that knows the rule:
the 0.2.0 release rejects the configuration with status 2.

[proposal]: ../planning/linter-selective-anonymous-functions.md
