# Oxlint rejects custom error names that are not string literals

Tested on `1.88.0`, the version this note was first written against, and on
`1.92.0`, where the same rule refuses a shared base class.

## Symptom

During the Pi native governing-source TypeScript port,
`unicorn(custom-error-definition)` reported:

```text
The `name` property should be set to `GoverningRuleBindingError`.
```

The triggering declaration used `override readonly name: string = this.constructor.name`.
The existing JavaScript constructor assigned the same derived value.
Replacing that value with a literal changed one observed behavior:
construction with `TypeError` as `newTarget` previously named the instance `TypeError`,
while the literal keeps `GoverningRuleBindingError`.
Message and cause identity remain independent of that category choice.

This is an intentional style rule,
not evidence that the constructor-derived JavaScript is invalid.
The unpublished Pi owner now adopts the stable operation category.
The temporary attempt to preserve constructor-derived naming in `55ebbbcda2` is superseded.

## Root cause

Installed `node_modules/oxlint/package.json` identifies version `1.88.0`.
The read-only upstream clone is `${HOME}/temp/agent/oxc-oxlint188-20261015`,
at tag `oxlint_v1.88.0`,
commit `6941b4ed45ead21c45bc2164cb80a180eed85b43`.

In that clone,
`crates/oxc_linter/src/rules/unicorn/custom_error_definition.rs:209`
checks the field initializer when no constructor body exists:

```rust
// crates/oxc_linter/src/rules/unicorn/custom_error_definition.rs
if is_valid_name_property(name_property, name) {
    return;
}
```

The deciding helper at `custom_error_definition.rs:377`
accepts only a string literal equal to the class identifier:

```rust
// crates/oxc_linter/src/rules/unicorn/custom_error_definition.rs
if let Some(prop) = name_property
    && let Some(Expression::StringLiteral(lit)) = &prop.value
{
    return lit.value.as_str() == class_name;
}
```

Constructor assignment uses the same literal requirement through
`is_expected_string_literal` at `custom_error_definition.rs:387`.
The upstream explanation at `custom_error_definition.rs:83`
ties the literal to minification stability:

```javascript
// crates/oxc_linter/src/rules/unicorn/custom_error_definition.rs
// Use a string literal to set the `name` property as it will not change after minifying.
this.name = this.constructor.name;
```

The installed schema at `node_modules/oxlint/configuration_schema.json:8912`
exposes no per-rule behavior options:

```json
"unicorn/custom-error-definition": {
  "$ref": "#/definitions/RuleNoConfig"
}
```

There is no discovered configuration alternative that preserves the rule while allowing this dynamic initializer.
No rule severity or suppression was changed.

## Verification

The project command exercises the actual maintained error declaration:

```sh
# Repository root
mise run //package/pi-plugin/auto-mode:lint:oxlint -- \
  native/contract/lifecycle/governing-rule-candidates/binding-error.ts
```

The failing catalog is the constructor-derived initializer:

```typescript
// package/pi-plugin/auto-mode/native/contract/lifecycle/governing-rule-candidates/binding-error.ts
export class GoverningRuleBindingError extends Error {
  override readonly name: string = this.constructor.name;
}
```

Its actual diagnostic is retained in
`${HOME}/temp/agent/pi-native-governing-bindings-format-fourth-20261015.log`.
The passing catalog uses the literal class category:

```typescript
// package/pi-plugin/auto-mode/native/contract/lifecycle/governing-rule-candidates/binding-error.ts
export class GoverningRuleBindingError extends Error {
  override readonly name: string = 'GoverningRuleBindingError';
}
```

The focused formatter/linter reports zero diagnostics across 14 source and consumer files in
`${HOME}/temp/agent/pi-native-governing-bindings-final-lint-20261015.log`.

`src/native-governing-boundary.unit.test.ts`
checks ordinary construction and `Reflect.construct` with `TypeError` as `newTarget`.
The original derived-name expectation failed against the literal version with
`AssertionError [ERR_ASSERTION]`,
actual `GoverningRuleBindingError` versus expected `TypeError`,
in `${HOME}/temp/agent/pi-native-governing-error-category-before-20261015.log`.
The saved original implementation passed that expectation in
`${HOME}/temp/agent/pi-native-governing-baseline-consumers-20261015.log`.
The current test explicitly expects the adopted stable operation category,
while retaining message and original cause assertions.

## Verified workaround and design tradeoff

Use the literal class category at the owned error boundary.
The exact source change is:

```diff
// package/pi-plugin/auto-mode/native/contract/lifecycle/governing-rule-candidates/binding-error.ts
-  override readonly name: string = this.constructor.name;
+  override readonly name: string = 'GoverningRuleBindingError';
```

This is a design change in the unpublished package.
Derived construction no longer changes this error's operation category automatically.
A separate error category must declare its own name.
Original message and cause behavior remain tested.
No governing-source eligibility,
membership,
or policy-authority condition changes with this choice.

## What does not work

- Restoring `this.constructor.name` preserves the original derived-name observation
  but triggers the quoted lint diagnostic.
- Assuming a literal initializer preserves every constructor observation is incorrect.
  The committed failing control distinguishes the derived-name behavior.
- A blanket suppression or rule downgrade would hide the design choice.
  Neither was applied.

## Upstream filing decision

No upstream report is warranted.
The rule documents and tests the observed rejection.

1.  Upstream fault:
    no.
    The emitted diagnostic matches the literal-name policy and its stated minification rationale.
2.  Fixability:
    implementation could accept another expression,
    but that would change the documented policy rather than repair this observation.
3.  Supported use case:
    dynamic constructor names are explicitly listed as rejected code.
4.  Contribution policy:
    `CONTRIBUTING.md` welcomes contributions,
    requires AI disclosure,
    and rejects unreviewed generated content.
    Its linked [contribution guide](https://oxc.rs/docs/contribute/introduction.html) repeats those requirements.
5.  Likelihood of acceptance:
    the explicit literal-name policy is evidence against requesting this exception
    as a correction to the existing rule.
6.  Prototype:
    no upstream prototype is justified because the fault and supported-use gates fail.
    The owned literal-name implementation and its consumer observations provide the local resolution.

The `.out-of-scope/` entries were checked;
`low-impact-typescript-formatting.md` concerns formatter syntax rather than this error-category policy.
`gh search issues` and `gh search prs` searched `oxc-project/oxc`
for `"custom-error-definition" "constructor"` on 2026-10-15 and returned no matches.
That empty search does not establish the absence of related reports;
the deciding evidence is the rule's explicit policy.
There is nothing additive to file and no external message was sent.

## A shared base class that derives the name trips the same rule (oxlint 1.92.0)

### Symptom

`package/named-error/core` holds one base class that assigns
`this.name = new.target.name`, so a subclass whose message is all it carries
declares no constructor at all.
Linting that package with oxlint `1.92.0` reported one finding per error class:

```text
! unicorn(custom-error-definition): The `name` property should be set to `TabbyMissingError`.
 ,-[src/named-error.unit.test.ts:22:1]
```

The base class itself is reported too, with `NamedError` in place of a
subclass name, because the rule inspects every class whose superclass
identifier ends in
`Error`, whatever the constructor body holds.

### Root cause

`node_modules/oxlint/package.json` identifies version `1.92.0`,
and the rule source is
`crates/oxc_linter/src/rules/unicorn/custom_error_definition.rs`
at tag `oxlint_v1.92.0`, commit `29daf901a1ea6b0a7a56c2b9d488bd2850d0abd4`.

Which classes the rule inspects, from
[`is_valid_super_class_name`](https://github.com/oxc-project/oxc/blob/29daf901a1ea6b0a7a56c2b9d488bd2850d0abd4/crates/oxc_linter/src/rules/unicorn/custom_error_definition.rs#L308-L326)
at line 312:

```rust
// crates/oxc_linter/src/rules/unicorn/custom_error_definition.rs
fn is_valid_super_class_name(name: &str) -> bool {
    let Some(prefix) = name.strip_suffix("Error") else {
        return false;
    };
```

`NamedError` ends in `Error`, so `class TabbyMissingError extends NamedError {}`
is inspected exactly like `class TabbyMissingError extends Error {}`.

What the rule accepts, from
[`is_expected_string_literal`](https://github.com/oxc-project/oxc/blob/29daf901a1ea6b0a7a56c2b9d488bd2850d0abd4/crates/oxc_linter/src/rules/unicorn/custom_error_definition.rs#L385-L389)
at line 387:

```rust
// crates/oxc_linter/src/rules/unicorn/custom_error_definition.rs
fn is_expected_string_literal(expr: &Expression, expected: &str) -> bool {
    matches!(expr, Expression::StringLiteral(lit) if lit.value == expected)
}
```

A constructor assignment must be a string literal equal to the class
identifier, and the class-field form must be one as well:
`new.target.name`, `NamedError.name` and `this.constructor.name` are all
rejected.
The rule's stated reason, from its documentation comment at line 81:

```javascript
// crates/oxc_linter/src/rules/unicorn/custom_error_definition.rs
// Use a string literal to set the `name` property as it will not change after minifying.
```

The installed schema declares no options for the rule,
so no allow-list can exempt a derived name
(`node_modules/oxlint/configuration_schema.json:8981`):

```json
"unicorn/custom-error-definition": {
  "$ref": "#/definitions/RuleNoConfig"
}
```

The rule arrives through the `style` category configured in
`package/config/oxlint/src/config-base.ts:23-31`,
and `denyWarnings: true` at line 31 turns its warning into a failure.

### Verification

Run on 2026-11-10 in the `feat/named-error-package` worktree with oxlint `1.92.0`.

The failing catalog is a subclass with no disable around it:

```ts
// package/named-error/core/src/named-error.unit.test.ts
class TabbyMissingError extends NamedError {}
```

```text
$ mise run //package/named-error/core:lint:oxlint
  ! unicorn(custom-error-definition): The `name` property should be set to `TabbyMissingError`.
  ! unicorn(custom-error-definition): The `name` property should be set to `NamedError`.
```

A positive control showed the rule was still on once the disables landed:
a file holding only `export class ProbeMissingError extends Error {}` reported
the same diagnostic with `ProbeMissingError` as the expected name, and exited 1,
and deleting that file returned `Found 0 warnings and 0 errors`.

The passing catalog wraps only the error-class block, with the rule enabled on
the line after the last class:

```ts
/* oxlint-disable unicorn/custom-error-definition -- <reason citing this doc> */
class TabbyMissingError extends NamedError {}
/* oxlint-enable unicorn/custom-error-definition */
```

```text
$ mise run //package/named-error/core:lint:oxlint
Found 0 warnings and 0 errors.
```

### Verified workaround and its tradeoff

Disable the rule around the error classes of a file that derives its names,
with the reason naming this document, and enable it again on the line after the
last such class.
Every other file keeps the rule, so a hand-written error class that forgets its
name is still reported.

The tradeoff is that the minification guarantee moves from the literal to the
bundler configuration.
Every rolldown config sets `mangle: false`:
`package/config/rolldown/src/index.ts:64` documents it as
"Mangle breaks func.name and makes output difficult for users to audit",
with the same setting at `package/config/rolldown/src/index.client.ts:125` and
`package/config/rolldown/src/index.node.ts:138`.
What keeps that honest per package is a unit test that asserts an error's `name`
against the built bundle, which
`package/cloudflare-worker/rand/src/errors.unit.test.ts`,
`package/search-fetch/core/src/tool-exec.unit.test.ts` and
`package/upstream-fork/conf/src/errors.unit.test.ts` do.
A migrated package without such an assertion has no automatic check:
`package/css/cli`, `package/mvm/cli`, `package/android-exempt-unused/cli` and
`package/open-code-review-issue/cli` carry none today.

One family cannot use the base class at all.
The `TypeError` subclasses of `package/upstream-fork/conf` must keep extending
`TypeError`, because upstream `conf` throws `TypeError` there and a caller's
`instanceof TypeError` has to keep its answer.
They take the name from their own identifier instead
(`this.name = InvalidKeyError.name`), which cannot drift from the class, and
`package/upstream-fork/conf/src/errors.unit.test.ts` asserts both the base type
and the name for every one of them.

### What does not work

- Rule options or an allow-list: the installed schema declares the rule
  `RuleNoConfig`, quoted above.
- A file-pattern override in `package/config/oxlint/src/overrides.ts` for
  subclasses of `NamedError`: no glob expresses "extends NamedError", so the
  override would have to name every migrated file, and the list would grow with
  each package the migration reaches.
- Keeping the literal in new code: that reintroduces the drift between the
  identifier and the name string that the base class exists to remove.
- Turning the rule off repo-wide: that is a lint loosening nobody approved, and
  it would let a hand-written error class report `Error` silently.

### Upstream filing decision

Nothing to file, and the version bump changes nothing: the rule documents and
tests this rejection, its stated reason is minification safety, and the
substrate is our own bundler configuration.
The six-constraint walk in the first section of this file answers the same way
for `1.92.0`.
