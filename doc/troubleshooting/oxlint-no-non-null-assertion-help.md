# Oxlint 1.78.0 `typescript/no-non-null-assertion` recommends optional chaining instead of the fail-loud repository fix

## Symptom

Oxlint reports a non-null assertion used before member access with this help text:

```text
x typescript(no-non-null-assertion): Forbidden non-null assertion.
 ,-[src/issue-442-repro.ts:14:1]
14 | issue442Rows[0]!.outcome = 'verified';
   : ^^^^^^^^^^^^^^^^
 `----
help: Consider using the optional chain operator `?.` instead. `x!.y` is equivalent to `x.y` at runtime and will throw if `x` is `null` or `undefined`, but `x?.y` will return `undefined`.
note: The non-null assertion operator (`!`) removes `null` and `undefined` from the type. For example, it changes `number | undefined` to `number`.
```

Optional chaining is not the repository's replacement for this use.
`rows[0]!.outcome = value` fails when the row is missing.
`rows[0]?.outcome = value` is not valid assignment syntax,
and optional chaining in expression positions returns `undefined` instead of failing.
Following the generic suggestion can therefore replace a required failure with accepted absence.

The repository policy at `package/config/oxlint/src/rule/restriction.ts:214-215` is explicit:

```typescript
// Ban non-null assertion (!): use nonNullishOrThrow instead.
'typescript/no-non-null-assertion': 'error',
```

The equivalent fail-loud replacement is:

```typescript
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

nonNullishOrThrow(rows[0],).outcome = value;
```

A consuming package needs `@monochromatic-dev/or-throw-core: "workspace:*"`
when it does not already declare that dependency.

## Root cause

### Oxlint deliberately emits generic optional-chaining guidance

Oxlint release tag `oxlint_v1.78.0` resolves to commit
`c42d6397eab5b2d5bb2bd6746c57bc2a9cad21bd`.
Its built-in rule constructs a base diagnostic,
then adds the optional-chaining help only when the assertion's parent is a member expression.
`crates/oxc_linter/src/rules/typescript/no_non_null_assertion.rs:47-57` says:

```rust
fn no_non_null_assertion_diagnostic(span: Span, is_member_expression: bool) -> OxcDiagnostic {
    let diagnostic = OxcDiagnostic::warn("Forbidden non-null assertion.")
        .with_note(
            "The non-null assertion operator (`!`) removes `null` and `undefined` from the type. For example, it changes `number | undefined` to `number`.",
        )
        .with_label(span);

    if is_member_expression {
        diagnostic.with_help("Consider using the optional chain operator `?.` instead. `x!.y` is equivalent to `x.y` at runtime and will throw if `x` is `null` or `undefined`, but `x?.y` will return `undefined`.")
    } else {
        diagnostic
    }
}
```

The visitor decides that branch from the immediate parent node at the same file's lines 61 to 65:

```rust
let AstKind::TSNonNullExpression(expr) = node.kind() else { return };
let is_member_expression = ctx.nodes().parent_kind(node.id()).is_member_expression_kind();
ctx.diagnostic(no_non_null_assertion_diagnostic(expr.span, is_member_expression));
```

This is not a rule swap in Monochromatic.
`package/config/oxlint/src/rule/restriction.ts:215` still enables the built-in
`typescript/no-non-null-assertion` rule.

### The April wording change made existing optional-chaining advice more accurate

The previous reading in issue 442 treated the appearance of optional-chaining guidance as a recent regression.
The source history disproves that reading.
Before upstream PR [oxc-project/oxc#21616][oxc-pr-21616],
commit `c2ada2c366cf439c267b1e8f2c1214ae4c90ba90` already had this at
`crates/oxc_linter/src/rules/typescript/no_non_null_assertion.rs:47-51`:

```rust
fn no_non_null_assertion_diagnostic(span: Span) -> OxcDiagnostic {
    OxcDiagnostic::warn("Forbidden non-null assertion.")
        .with_help("Consider using the optional chain operator `?.` instead. This operator includes runtime checks, so it is safer than the compile-only non-null assertion operator.")
        .with_label(span)
}
```

Merged commit `f3a02cc4930234c159f52098f303389f183f4957` retained the suggestion,
restricted it to member-expression cases,
and added the explicit warning that optional chaining returns `undefined`.
The current message is generic policy from upstream,
not evidence that this repository changed its preferred remediation.

### The repository's diagnostic wrapper now owns the local guidance

`package/monorepo-task/core-exec/src/oxlint-guidance.ts:44` defines `RULE_GUIDANCE`,
which owns project-specific additions to Oxlint diagnostics.
The implemented fix adds the `no-non-null-assertion` entry at the same file's line 53.

`package/monorepo-task/core-exec/src/oxlint-guidance.ts:134-136` keeps returning a sentinel for an absent entry:

```typescript
const ruleGuidance = RULE_GUIDANCE[ruleName];
if (ruleGuidance === undefined)
  return NO_DIAGNOSTIC_GUIDANCE;
```

When guidance exists,
`package/monorepo-task/core-exec/src/oxlint-augment.ts:656-660` appends it to Oxlint's existing help line:

```typescript
if ((activeGuidance !== NO_RULE) && (!injected)
  && isHelpLine(line,)) {
  result.push(`${line} ${activeGuidance}`,);
  injected = true;
  continue;
}
```

When Oxlint emits no help line,
`package/monorepo-task/core-exec/src/oxlint-augment.ts:664-670` injects one at the diagnostic boundary.
The missing registry entry was therefore the local defect.
Commit `07362d55d` added it without changing the parser,
rule configuration,
or output-augmentation architecture.

## Verification

### Version and source identity

The reproduced installation reports `Version: 1.78.0`.
`pnpm-lock.yaml:6515-6516` resolves `oxlint@1.78.0`,
and the corresponding upstream tag points at
`c42d6397eab5b2d5bb2bd6746c57bc2a9cad21bd`.
The upstream source clone used for the trace had:

```text
origin: https://github.com/oxc-project/oxc.git
HEAD:   c42d6397eab5b2d5bb2bd6746c57bc2a9cad21bd
```

### Runnable repository-boundary harness

Use a disposable worktree with this fixture at
`package/monorepo-task/core-exec/src/issue-442-repro.ts`:

```typescript
/**
 * Row whose outcome must be assigned during verification.
 */
type Row = {
  outcome?: string;
};

/**
 * Rows supplied by caller.
 */
declare const rows: Row[];

rows[0]!.outcome = 'verified';
```

With workspace dependencies available in that disposable worktree,
run the same consumer boundary used by the repository:

```bash
mise run //package/monorepo-task/core-exec:lint:oxlint
```

Before implementation,
the output contained Oxlint's optional-chaining help and no mention of `nonNullishOrThrow`.
The command exited nonzero because the fixture intentionally violated the rule.

The implemented registry entry was built and exercised through
`package/monorepo-task/core-exec/dist/final/node/oxlint-wrapper.mjs`.
The same diagnostic became:

```text
help: Consider using the optional chain operator `?.` instead. `x!.y` is equivalent to `x.y` at runtime and will throw if `x` is `null` or `undefined`, but `x?.y` will return `undefined`. Repository policy: preserve fail-loud semantics. Replace `value!` with `nonNullishOrThrow(value,)` from `@monochromatic-dev/or-throw-core/ts`. Do not use optional chaining unless a missing value is intentionally accepted.
```

The implementation uses this registry value:

```typescript
'no-non-null-assertion': {
  guidance: [
    'Repository policy: preserve fail-loud semantics.',
    'Replace `value!` with `nonNullishOrThrow(value,)` from `@monochromatic-dev/or-throw-core/ts`.',
    'Do not use optional chaining unless a missing value is intentionally accepted.',
  ]
    .join(' ',),
},
```

The focused `oxlint-augment.unit.test.ts` probe first failed in the detached prototype because the registry
returned an empty string,
then passed after the entry was added.
Commit `9ded59670` added the production tests.
The test asserts both `nonNullishOrThrow` and the explicit instruction not to use optional chaining.
It covers both current upstream variants:

- Member assertions such as `x!.y` already have upstream help,
  so the wrapper appends the repository guidance and leaves one `help:` prefix.
- Standalone assertions such as `x!` have no upstream help,
  so the wrapper injects the repository guidance as a new `help:` line.

Both variants were also exercised through the built wrapper.
For the standalone `issue442Rows[0]!` fixture,
the built wrapper retained Oxlint's note and injected this separate line:

```text
help: Repository policy: preserve fail-loud semantics. Replace `value!` with `nonNullishOrThrow(value,)` from `@monochromatic-dev/or-throw-core/ts`. Do not use optional chaining unless a missing value is intentionally accepted.
```

`mise run //package/monorepo-task/core-exec:lint:types` passed after implementation.
The package's `lint:oxlint` task still reports seven pre-existing
`test-import(require-eventual-artifact)` errors.
A disposable worktree at parent commit `e815bdff5` reproduced the same seven errors,
so this change added no Oxlint finding.

The existing helper verification passed through:

```bash
mise run //package/or-throw/core:test:unit -- package/or-throw/core/src/non-nullish-or-throw.unit.test.ts
```

It covered non-nullish pass-through,
throws for `null` and `undefined`,
falsy non-nullish values,
and static narrowing.

### Inputs that do not trigger the rule

The upstream rule's test catalog accepts:

- `x;`
- `x.y;`
- `x?.y;`
- `x?.y?.z;`
- `!x;`,
  which is boolean negation rather than a TypeScript non-null assertion

The repository-safe replacement also avoids the rule:

```typescript
nonNullishOrThrow(rows[0],).outcome = 'verified';
```

### Inputs that trigger the optional-chaining help variant

The immediate parent of the assertion is a member expression in these forms:

- `x!.y;`
- `x![key];`
- `rows[0]!.outcome = value;`
- multiline member access after `x!`

Oxlint emits its optional-chaining help for these forms.
The local wrapper should append repository guidance.

### Inputs that trigger the no-upstream-help variant

These forms still violate the rule,
but their immediate parent is not a member expression:

- `x!;`
- `x.y!;`
- `x.y.z!();`
- repeated assertions such as `x!!;`

Oxlint emits the warning and note without its optional-chaining help.
The local wrapper now injects repository guidance.

## Verified workarounds

### Implemented project guidance at the existing wrapper boundary

`RULE_GUIDANCE` in `package/monorepo-task/core-exec/src/oxlint-guidance.ts` now contains
`no-non-null-assertion` guidance.
`oxlint-augment.ts` remains generic.
Focused tests in `package/monorepo-task/core-exec/src/oxlint-augment.unit.test.ts` cover the existing-help
and no-help variants.

This is the implemented repository fix.
It uses the mechanism documented at
`package/monorepo-task/core-exec/README.md:159-165`,
which says repository lint runs through `task-oxlint` and that the wrapper augments diagnostics without removing them.
The location also keeps private repository policy out of the published config package:
`package/monorepo-task/core-exec/package.json:6` is private,
while `package/config/oxlint/package.json:3` is published.

Use the bare `no-non-null-assertion` registry key without a header guard.
The repository policy applies to this syntax regardless of which Oxlint plugin label produced the diagnostic.

Tradeoff:
the upstream sentence remains visible because the wrapper is intentionally append-only.
The final sentence must therefore be direct enough to override the generic suggestion for this repository.
The augmentation applies to the graphical `task-oxlint` path;
direct Oxlint calls,
explicitly selected non-graphical formats,
and language-server diagnostics do not receive it.
The language server therefore continues to show the raw upstream message.

### Replace each assertion with `nonNullishOrThrow`

At a violation site,
import `nonNullishOrThrow` and wrap the nullable expression:

```typescript
const row = nonNullishOrThrow(rows[0],);
row.outcome = value;
```

Tradeoff:
this adds a function call and may require a workspace dependency.
It preserves the required fail-loud boundary and gives TypeScript a narrowed value.

### Use an explicit guard when absence is expected

When missing data is a normal branch rather than an invariant violation,
use an explicit check and return or branch before member access.

Tradeoff:
this intentionally changes control flow.
It is correct only when absence is part of the domain behavior.

## What does not work

- **Following the optional-chaining suggestion unconditionally**:
  it changes missing-value behavior from throwing to returning `undefined`,
  and it cannot appear on the left side of an assignment such as the issue's example.
- **Changing only the comment in `restriction.ts` or a handover document**:
  those files do not alter the diagnostic shown at the lint boundary.
- **Replacing the built-in rule with a project JS rule**:
  it duplicates Oxlint's working `TSNonNullExpression` detection,
  expands the test surface,
  and loses the built-in rule's maintained syntax coverage.
  `package/config/oxlint/src/index.ts:33` also records that the language server does not support JS plugins.
- **Teaching `oxlint-augment.ts` about this rule directly**:
  repository design keeps rule-specific policy in `oxlint-guidance.ts`.
  The generic augmenter already handles both help-line shapes.
- **Removing or replacing Oxlint's help text in the wrapper**:
  the wrapper's documented contract is augmentation,
  and the established guidance model appends one string rather than introducing replacement modes.
- **Filing the repository-specific helper requirement upstream**:
  Oxlint cannot prescribe a private workspace package used by one consumer.

## Upstream filing artifact

### Upstream filing decision

No `.out-of-scope/` entry matches Oxlint or this diagnostic class.
Searches across open and closed Oxc issues and pull requests used
`"no-non-null-assertion" "optional chain"`,
`"Forbidden non-null assertion"`,
and
`"optional chaining" "non-null assertion"`.
They found no matching report.
[oxc-project/oxc#21616][oxc-pr-21616] is the merged source change that deliberately produced the current wording,
not a duplicate bug report.

The filing constraints resolve as follows:

1.  Is it really upstream's fault?
    No.
    Upstream gives generic guidance and explicitly states the semantic difference.
    The actionable defect is that Monochromatic's existing guidance registry lacks its repository-specific replacement.
2.  Can upstream fix it?
    Upstream can revise generic wording,
    but it cannot direct users to `@monochromatic-dev/or-throw-core/ts`.
    It cannot complete this repository's fix.
3.  Are they supporting this use case?
    Oxlint supports detecting non-null assertions.
    Its rule documentation treats optional chaining as one valid general alternative.
    It does not claim to encode consumer-specific invariant policies or helper libraries.
4.  Would the repo welcome our contribution?
    Yes,
    subject to review and disclosure.
    Upstream `CONTRIBUTING.md:10-21` welcomes contributions and requires disclosure,
    understanding,
    and validation of AI-assisted work.
5.  Will they likely fix it?
    There is no matching tracker decision.
    Current main still contains the same help text,
    and PR 21616 deliberately added the parent-sensitive branch and snapshot coverage.
    This does not indicate an upstream defect that maintainers are expected to reverse.
6.  Have we prototyped a minimal fix compatible with their architecture?
    No upstream prototype is warranted because the first and third constraints fail.
    A consumer-side prototype at Monochromatic's existing wrapper boundary passed focused tests,
    type lint,
    and the built-wrapper diagnostic probe.

The decision is not to file upstream.
Fix the consumer guidance registry.

### Draft (do not file as-is)

~~~md
Title: `typescript/no-non-null-assertion` should recommend a repository-specific helper

Do not file this issue upstream.
The requested behavior depends on Monochromatic's private policy and
`@monochromatic-dev/or-throw-core/ts` package.
Oxlint 1.78.0 already explains that optional chaining changes runtime behavior.
The missing actionable guidance belongs in Monochromatic's `task-oxlint` wrapper,
not in Oxc.
~~~

[oxc-pr-21616]: https://github.com/oxc-project/oxc/pull/21616
