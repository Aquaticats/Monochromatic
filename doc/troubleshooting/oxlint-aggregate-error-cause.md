# Oxlint 1.88.0 adds a cause to aggregate failures during autofix

## Symptom

During the Pi judgment lifecycle TypeScript extraction,
the package's `format:oxlint` task changed the original aggregate failure from
`new AggregateError([error, retentionError], message)`
to a constructor call with `{ cause: retentionError }`.
Both original errors were already retained in the ordered `errors` array.
The autofix added an own `cause` property that the original native failure did not have.

The emitting rule is `eslint(preserve-caught-error)`.
Its no-fix diagnostic is:

```text
There is no cause error attached to this new thrown error.
```

This is the rule's documented cause-property policy,
not evidence that `AggregateError.errors` loses either original failure.
The [rule documentation](https://oxc.rs/docs/guide/usage/linter/rules/eslint/preserve-caught-error.html)
describes that policy and exposes only the `requireCatchParameter` option.

## Root cause

Installed `node_modules/oxlint/package.json` identifies version `1.88.0`.
The read-only source clone is `${HOME}/temp/agent/oxc-oxlint188-20261015`,
at tag `oxlint_v1.88.0`,
commit `6941b4ed45ead21c45bc2164cb80a180eed85b43`.

In `crates/oxc_linter/src/rules/eslint/preserve_caught_error.rs:58`,
the catch visitor examines directly thrown construction and call expressions.
At line 68,
it asks for the recognized constructor's options argument:

```rust
// crates/oxc_linter/src/rules/eslint/preserve_caught_error.rs
let Some(options_argument_index) = error_options_argument_index(callee, self.ctx) else {
    return;
};
```

The dispatcher at `preserve_caught_error.rs:212`
recognizes global `Error`,
`TypeError`,
and `AggregateError`.
The aggregate uses argument index two:

```rust
// crates/oxc_linter/src/rules/eslint/preserve_caught_error.rs
} else if is_aggregate_error(ident, ctx) {
    Some(2)
} else {
    None
}
```

The exemption at `preserve_caught_error.rs:72`
requires the recognized options object to contain the caught error as its cause.
The visitor does not treat membership in the aggregate's first argument as that exemption.
The two-argument aggregate fixer at line 136 inserts the new options object:

```rust
// crates/oxc_linter/src/rules/eslint/preserve_caught_error.rs
let span = args[1].span();
let mut fix = fixer.new_fix_with_capacity(3);
fix.push(fixer.insert_text_after_range(span, ", "));
fix.push(
    fixer.insert_text_after_range(span, format!("{{ {cause_prop_text} }}")),
);
```

The configurable fields at `preserve_caught_error.rs:43`
contain only `require_catch_parameter`.
There is no aggregate-array alternative in those options.

## Verification

The disposable probe is
`${HOME}/temp/agent/pi-tool-call-scope-20261014/probe-preserve-error.ts`.
It writes independent fixtures and a rule-only configuration under `preserve-error-probe/`,
runs installed Oxlint before and after `--fix`,
then imports each version under a distinct module URL.
It checks both original error identities,
own property names,
and cause identity.

The actual command was:

```sh
# Existing private probe workspace
cd -- "${HOME}/temp/agent/pi-tool-call-scope-20261014" && mise run probe-preserve-error
```

The failing catalog is direct construction inside a catch:

```javascript
// preserve-error-probe/direct.mjs
export function combine(first, second) {
  try {
    throw second;
  } catch (caught) {
    throw new AggregateError([first, caught], 'both');
  }
}
```

The passing catalog contains explicit cause construction and an ordered aggregate factory:

```javascript
// preserve-error-probe/helper.mjs
function combined({ first, second }) {
  return new AggregateError([first, second], 'both');
}

export function combine(first, second) {
  try {
    throw second;
  } catch (caught) {
    throw combined({ first, second: caught });
  }
}
```

The separate explicit-cause fixture constructs
`new AggregateError([first, caught], 'both', { cause: caught })`.
It passes the rule but already has the changed error shape.

`${HOME}/temp/agent/pi-preserve-caught-error-probe-20261015.log`
and `preserve-error-probe/receipt.json` record:

- Direct construction: one target diagnostic;
  own keys change from `stack,message,errors` to `stack,message,cause,errors`.
  The new cause is the second original failure.
- Factory construction: zero target diagnostics;
  source and own keys stay unchanged after autofix.
- Explicit cause: zero target diagnostics;
  source and own keys stay unchanged after autofix.
- Every variant retains both original failure objects,
  in their original order,
  in `errors`.

## Verified workaround and tradeoff

Create the domain's ordered aggregate in a dedicated function,
then throw that retained result from the catch.
The Pi implementation is
`package/pi-plugin/auto-mode/native/contract/collector/program-rule-batch-sdk-copy-v2/input/judgment-assessment.ts`.
Its `createAssessmentRetentionError` function receives both original failures explicitly
and preserves the original aggregate message and ordered `errors` array.

The behavioral patch is:

```diff
// package/pi-plugin/auto-mode/native/contract/collector/program-rule-batch-sdk-copy-v2/input/judgment-assessment.ts
-throw new AggregateError([error, retentionError], message, { cause: retentionError });
+throw createAssessmentRetentionError({ error, retentionError });
```

The factory creates `new AggregateError([error, retentionError], message)` outside the catch visitor.
It represents the domain's paired failures without choosing a single cause.
The tradeoff is an additional factory frame in the error's stack;
exact stack text is not preserved across this extraction.
No lint rule,
severity,
or suppression changed.

## What does not work

- Keeping direct aggregate construction in the catch and rerunning `--fix`
  adds the cause property again.
- Accepting that autofix preserves both errors but changes the original public error shape.
  It is not a behavior-preserving formatting change.
- Assuming `errors` satisfies this rule's cause requirement is incorrect.
  The source checks the options argument,
  and the direct-construction probe reproduces the diagnostic.

## Upstream filing decision

No upstream report is warranted for the observed enforcement of this rule's documented policy.

1.  Upstream fault: no demonstrated mismatch.
    The rule explicitly requires a cause property and recognizes aggregate constructors.
2.  Fixability: accepting aggregate membership would be possible by changing the rule's policy.
    That is a different policy from the current options-argument test.
3.  Supported use case: direct aggregate construction is supported with an explicit cause;
    source tests at `preserve_caught_error.rs:437` include that form.
4.  Contribution policy: the checked `CONTRIBUTING.md` welcomes contributions,
    requires AI disclosure,
    and rejects unreviewed generated content.
    No contribution is proposed here.
5.  Acceptance evidence: the documented cause requirement is evidence against treating the observed behavior
    as an implementation defect.
6.  Prototype: no upstream prototype is justified because the fault gate fails.
    The owned factory and disposable before/after probe establish the local resolution.

The `.out-of-scope/` inventory was checked;
`low-impact-typescript-formatting.md` concerns formatting syntax,
not aggregate error topology.
On 2026-10-15,
`gh search issues` and `gh search prs` searched `oxc-project/oxc`
for `"preserve-caught-error" "AggregateError"` across states and returned no matches.
That empty search does not establish absence of related discussions.
The deciding evidence is the rule's source and documented policy.
There is nothing additive to file and no external message was sent.
