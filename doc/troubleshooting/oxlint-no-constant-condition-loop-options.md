# Oxlint 1.74.0 default no-constant-condition leaves while(true) and for(;;) unreported

## Symptom

The repository enables Oxlint's `correctness` category at error severity in
`package/config/oxlint/src/config-base.ts:24`.
The `eslint/no-constant-condition` rule belongs to that category,
but this source receives no diagnostic under its default configuration:

```ts
// input.ts
while (true) {
  work();
}
```

Setting `checkLoops` to `all` changes the result to:

```text
error eslint(no-constant-condition): Unexpected constant condition
help: Update the condition to not be constant, or remove the condition entirely
```

The same `all` setting reports `for (; true;)`,
but it still accepts `for (;;)`,
because that loop has no condition expression to classify.
This distinction matters when a policy is intended to expose a loop's continuation condition,
rather than only ban one spelling.

A workspace scan on 2026-07-23 found 36 textual `while (true)` matches in TypeScript.
Manual context classification removed three documentation examples,
leaving 33 executable statements.
Oxlint's ignore patterns in `package/config/oxlint/src/config-base.ts:58-86`
exclude paused,
 deprecated,
 test-fixture,
 and generated trees,
leaving 25 lint-scoped statements.
Two are deliberate nontermination or exhaustion fixtures in `package/runtime-error/bun/src/`.
The other 23 are parser scans,
 ancestor walks,
 storage retries,
 pagination,
 polling,
 and stream reads.

One lint-scoped `for (;;)` already exists at
`package/ssg/aquati.cat/src/lib/content.ts:110`.
A `while (true)` migration that merely changes spelling to `for (;;)` therefore does not establish
an explicit-continuation policy.

## Root cause

Source was inspected at Oxc tag `apps_v1.74.0`,
commit `2d4e8d20644e0e7446f0a381894b45ea339a0625`,
with origin `https://github.com/oxc-project/oxc.git`.
The exact release source remains available in the
[pinned Oxc release source tree][oxc-release-source].
The [generated Oxlint rule documentation][oxlint-rule-doc]
publishes the same option contract.

### The loop option defaults to the while-true exception

`crates/oxc_linter/src/rules/eslint/no_constant_condition.rs:29-34`
defines `AllExceptWhileTrue` as the default:

```rust
// crates/oxc_linter/src/rules/eslint/no_constant_condition.rs
#[derive(Debug, Default, Clone, PartialEq, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
enum CheckLoops {
    All,
    #[default]
    AllExceptWhileTrue,
    None,
}
```

The rule is not disabled.
Its declaration at
`crates/oxc_linter/src/rules/eslint/no_constant_condition.rs:91-139`
places it in the `correctness` category.
The repository's category configuration therefore activates the rule with the default option.

### While true returns before constant-expression analysis

`NoConstantCondition::run` dispatches `while`,
 `do...while`,
 and `for` statements to `check_loop` at
`crates/oxc_linter/src/rules/eslint/no_constant_condition.rs:142-181`.

`check_loop` then returns early for a literal `true` in a `while` statement when the default is active.
The deciding branch is at
`crates/oxc_linter/src/rules/eslint/no_constant_condition.rs:194-201`:

```rust
// crates/oxc_linter/src/rules/eslint/no_constant_condition.rs
match self.check_loops {
    CheckLoops::None => return,
    CheckLoops::AllExceptWhileTrue if is_while => match test {
        Expression::BooleanLiteral(bool) if bool.value => return,
        _ => {}
    },
    _ => {}
}
```

With `CheckLoops::All`,
that early return does not apply.
The method reaches the constant-expression test and emits the diagnostic at
`crates/oxc_linter/src/rules/eslint/no_constant_condition.rs:203-211`:

```rust
// crates/oxc_linter/src/rules/eslint/no_constant_condition.rs
if !test.is_constant(true, ctx) {
    return;
}

if self.check_loops == CheckLoops::AllExceptWhileTrue && has_yield_before_loop_exit() {
    return;
}

ctx.diagnostic(no_constant_condition_diagnostic(test.span()));
```

### For-ever loops have no test node

The `ForStatement` arm at
`crates/oxc_linter/src/rules/eslint/no_constant_condition.rs:165-170`
returns when the AST carries no `test` expression:

```rust
// crates/oxc_linter/src/rules/eslint/no_constant_condition.rs
AstKind::ForStatement(for_stmt) => {
    let Some(test) = &for_stmt.test else {
        return;
    };
```

That is why `for (; true;)` is checked but `for (;;)` is not.
The upstream test catalogs make the intent explicit:

- `crates/oxc_linter/src/rules/eslint/no_constant_condition.rs:395` accepts `for(;;)`.
- `crates/oxc_linter/src/rules/eslint/no_constant_condition.rs:409` accepts default `while(true)`.
- `crates/oxc_linter/src/rules/eslint/no_constant_condition.rs:558` rejects `while(true)` with `all`.
- `crates/oxc_linter/src/rules/eslint/no_constant_condition.rs:566` rejects `for (;true;)` with `all`.

This is documented behavior inherited from ESLint,
not an Oxlint parsing defect.
The implementation PR,
[oxc-project/oxc#10949][oxc-pr-10949],
states that it re-imported ESLint's tests and implemented the three loop-option variants.

## Verification

### Version and source identity

- Installed npm package:
   `oxlint@1.74.0`.
- Release tag:
   `apps_v1.74.0`.
- Release commit:
   `2d4e8d20644e0e7446f0a381894b45ea339a0625`.
- Source origin:
   `https://github.com/oxc-project/oxc.git`.
- Host used for the probe:
   Linux x86-64.

### Workspace inventory harness

The lint-scoped inventory used this root-explicit command:

```bash
rg --line-number --multiline \
  --glob '*.ts' \
  --glob '*.tsx' \
  --glob '!package-paused/**' \
  --glob '!package-deprecated/**' \
  --glob '!**/test-fixture/**' \
  --glob '!**/perf-test-data/**' \
  --glob '!**/*.generated.ts' \
  'while\s*\(\s*true\s*\)' \
  .
```

The command returned this lint-scoped catalog:

- `package/config/tofu/src/fetch_ips.ts`:
   one occurrence.
- `package/module/css-edit/src/parse-contents.ts`:
   three occurrences.
- `package/module/css-edit/src/parse-classify.ts`:
   two occurrences.
- `package/mvm/cli/src/exec.ts`:
   one occurrence.
- `package/mvm/cli/src/virsh-wait.ts`:
   two occurrences.
- `package/mvm/cli/src/template-windows.ts`:
   one occurrence.
- `package/mvm/cli/src/backend/hetzner/api.ts`:
   one occurrence.
- `package/runtime-error/bun/src/oom.ts`:
   one occurrence.
- `package/runtime-error/bun/src/infinite-loop.ts`:
   one occurrence.
Within
`package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types/`:

- `lockfile-package-eligibility.ts`:
   one occurrence.
- `effect-summary-cache-identity.ts`:
   two occurrences.
- `effect-project-fingerprint.ts`:
   one occurrence.
- `effect-intrinsic-result-origin.ts`:
   one occurrence.
- `installed-package-identity.ts`:
   one occurrence.
- `intrinsic-effect-query.ts`:
   one occurrence.
- `direct-effect-summary.ts`:
   one occurrence.
- `package/dev-script/file-enforcer/src/io/staleness-manifest-lock.ts`:
   one occurrence.
- `package/oxlint-plugin/tsdoc/src/ast-access.ts`:
   one occurrence.
- `package/module/logger/src/sink/local-storage-store.ts`:
   one occurrence.
- `package/module/logger/src/sink/session-storage-store.ts`:
   one occurrence.

Every catalog entry is executable syntax,
so the command totals 25 lint-scoped statements without comment filtering.

### Runnable harness

Create these files in a disposable directory.

```toml
# mise.toml
[tools]
"npm:oxlint" = "1.74.0"

[tasks."probe:default"]
description = "Probe Oxlint default constant-loop handling"
run = "oxlint --config oxlint-default.json sample.ts"

[tasks."probe:all"]
description = "Probe Oxlint all constant-loop handling"
run = "oxlint --config oxlint.json sample.ts"
```

```jsonc
// oxlint-default.json
{
  "categories": {
    "correctness": "off",
    "suspicious": "off",
    "pedantic": "off",
    "style": "off"
  },
  "rules": {
    "eslint/no-constant-condition": "error"
  }
}
```

```jsonc
// oxlint.json
{
  "categories": {
    "correctness": "off",
    "suspicious": "off",
    "pedantic": "off",
    "style": "off"
  },
  "rules": {
    "eslint/no-constant-condition": [
      "error",
      {
        "checkLoops": "all"
      }
    ]
  }
}
```

```ts
// sample.ts
declare function keepGoing(): boolean;

while (true) {}
for (; true;) {}
do {} while (true);
while (1) {}
for (;;) {}
while (keepGoing()) {}
```

Run both tasks:

```bash
mise run probe:default
mise run probe:all
```

The harness was executed through those Mise tasks.
Both tasks exit nonzero because each intentionally includes rejected input.

### Default-option catalog

`mise run probe:default` reports `sample.ts:4:8`,
`sample.ts:5:14`,
and `sample.ts:6:8`.

Patterns accepted cleanly:

- `while (true)`
- `for (;;)`
- `while (keepGoing())`

Patterns rejected with `Unexpected constant condition`:

- `for (; true;)`
- `do {} while (true)`
- `while (1)`

### All-option catalog

`mise run probe:all` reports `sample.ts:3:8` through `sample.ts:6:8`.

Patterns accepted cleanly:

- `for (;;)`
- `while (keepGoing())`

Patterns rejected with `Unexpected constant condition`:

- `while (true)`
- `for (; true;)`
- `do {} while (true)`
- `while (1)`

## Verified workarounds

### Configure checkLoops as all

Use the built-in rule rather than adding a literal-only custom rule:

```ts
// package/config/oxlint/src/rule/correctness.ts
'eslint/no-constant-condition': [
  'error',
  { checkLoops: 'all', },
],
```

The harness verifies that this reports `while (true)` and `for (; true;)`.
It also reports other constant loop expressions.
The option enforces constant condition expressions only.
It does not guarantee that every ordinary loop exposes continuation state,
because a conditionless `for (;;)` never reaches the rule's expression analysis.
An airtight unconditional-loop policy needs an additional rule for that AST shape.

Tradeoffs:

- It does not report `for (;;)`.
- It is broader than a literal-only `while (true)` ban.
- Moving an internal exit into a loop header can change first-attempt,
   timeout,
   EOF,
   or retry-count semantics.
  Migrate and test each loop by behavior class rather than applying a text replacement.

### Reserve for-ever syntax for deliberate nontermination

The harness verifies that `for (;;)` remains accepted under `checkLoops: all`.
It can represent the two runtime-error fixtures whose purpose is intentional nontermination or exhaustion.

Tradeoff:
this is a policy convention rather than enforcement by `no-constant-condition`.
Using it for parser,
 retry,
 polling,
 or ancestor-walk migrations would preserve the hidden-continuation problem.
An airtight ban on every unconditional loop header needs an additional project rule for conditionless `for` statements.

## What does not work

### Enabling no-constant-condition without options

The repository already enables the rule through the `correctness` category.
Its default is `allExceptWhileTrue`,
so adding only `'eslint/no-constant-condition': 'error'` does not ban `while (true)`.
The default probe reproduces this.

### Treating checkLoops all as an infinite-loop ban

The option checks condition expressions.
A conditionless `for (;;)` has no expression and returns before analysis.
The all-option probe and upstream test at
`crates/oxc_linter/src/rules/eslint/no_constant_condition.rs:395`
confirm the gap.

### Replacing every while true with for-ever

That removes the diagnostic but does not expose progress or termination.
It is appropriate only where nontermination itself is the intended behavior.
The active production occurrence at `package/ssg/aquati.cat/src/lib/content.ts:110`
shows that this spelling already exists and cannot be treated as a hypothetical edge case.

### Adding a custom literal-only while-true rule

A project rule can match that exact AST shape,
but it duplicates behavior available from `checkLoops: all`,
adds plugin and test maintenance,
and permits trivial alternative spellings such as `while (1)` or `for (; true;)` unless continually expanded.
A custom rule is justified only if the policy also covers the built-in rule's `for (;;)` gap.

## Upstream filing decision

No `.out-of-scope/` entry covers Oxlint loop-condition behavior.
Searches across open and closed Oxc issues and merged pull requests used
`no-constant-condition`,
 `while true`,
 and `checkLoops` terms.
They found the implementation PR
[oxc-project/oxc#10949][oxc-pr-10949],
but no unresolved bug matching this behavior.

### Constraint 1: Is it really upstream's fault?

No.
The default exception and the `all` option are documented,
and the source and tests implement that contract.
The `for (;;)` result follows from the loop having no condition expression.

### Constraint 2: Can upstream fix it?

Technically yes,
by changing the default or broadening the rule to conditionless `for` statements.
That would be a policy and ESLint-compatibility change,
not a correction to observed behavior.

### Constraint 3: Are they supporting this use case?

Yes.
The rule documentation exposes `all`,
 `allExceptWhileTrue`,
 and `none`,
and PR #10949 specifically implemented loop configuration.

### Constraint 4: Would the repository welcome our contribution?

Oxc's `CONTRIBUTING.md` welcomes contributions and permits AI assistance when disclosed,
reviewed,
 tested,
 and understood.
Oxc's [PR Rules and Policies][oxc-contribution-rules]
request an issue or discussion before architectural changes.
No policy forbids a well-tested report,
but there is no defect to report here.

### Constraint 5: Will they likely fix it?

No evidence supports changing the current contract.
The default and the conditionless-for behavior are locked by tests imported from ESLint.
Recent rule history through the 1.74.0 release contains configuration-schema,
generator-yield,
 diagnostic-text,
 and visitor changes,
but no move away from those loop semantics.

### Constraint 6: Have we prototyped a minimal upstream fix?

No.
Constraints 1 and 5 fail,
so the auto-prototype gate does not apply.
The consumer-side `checkLoops: all` configuration already implements the desired `while (true)` policy.

## Upstream filing artifact

Nothing to add upstream.
There is no new-issue or additive-comment draft because the behavior is documented,
configurable,
 source-verified,
 and covered by upstream tests.
Filing would request a policy change rather than report a defect.

[oxc-contribution-rules]: https://oxc.rs/docs/contribute/rules.html#pr-rules
[oxc-pr-10949]: https://github.com/oxc-project/oxc/pull/10949
[oxc-release-source]: https://github.com/oxc-project/oxc/tree/2d4e8d20644e0e7446f0a381894b45ea339a0625
[oxlint-rule-doc]: https://oxc.rs/docs/guide/usage/linter/rules/eslint/no-constant-condition.html
