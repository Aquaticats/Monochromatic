# Oxlint 1.88.0 rejects constructor-derived custom error names

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

1.  Upstream fault: no.
    The emitted diagnostic matches the literal-name policy and its stated minification rationale.
2.  Fixability: implementation could accept another expression,
    but that would change the documented policy rather than repair this observation.
3.  Supported use case: dynamic constructor names are explicitly listed as rejected code.
4.  Contribution policy: `CONTRIBUTING.md` welcomes contributions,
    requires AI disclosure,
    and rejects unreviewed generated content.
    Its linked [contribution guide](https://oxc.rs/docs/contribute/introduction.html) repeats those requirements.
5.  Likelihood of acceptance: the explicit literal-name policy is evidence against requesting this exception
    as a correction to the existing rule.
6.  Prototype: no upstream prototype is justified because the fault and supported-use gates fail.
    The owned literal-name implementation and its consumer observations provide the local resolution.

The `.out-of-scope/` entries were checked;
`low-impact-typescript-formatting.md` concerns formatter syntax rather than this error-category policy.
`gh search issues` and `gh search prs` searched `oxc-project/oxc`
for `"custom-error-definition" "constructor"` on 2026-10-15 and returned no matches.
That empty search does not establish the absence of related reports;
the deciding evidence is the rule's explicit policy.
There is nothing additive to file and no external message was sent.
