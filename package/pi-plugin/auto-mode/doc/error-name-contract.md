# Policy error names and typed ports

## Consumer behavior

`PolicySlotBindingError` assigns `this.constructor.name` to `this.name`.
This preserves the constructor name observed by a consumer's inherited setter.
The constructor retains its original arity,
message,
optional cause identity,
and `Error` prototype chain.
The JavaScript facade re-exports the same constructor as the typed implementation.

The built consumer catalog is `src/native-policy-slot-error.unit.test.ts`.
Run it with `mise run //package/pi-plugin/auto-mode:test:native:policy-slot-error` after building.
Its inherited-setter case checks actual setter execution and the absence of an own `name` property.
It also checks the consumer's resulting name.
A final string comparison alone would miss part of this contract.

## Why the declaration has a lint exception

Oxlint `1.88.0` reports `unicorn/custom-error-definition` on the original dynamic assignment.
The rule requires the literal `'PolicySlotBindingError'`.
Its stated reason for rejecting `this.constructor.name` is minification safety.
The exception applies to this declaration only,
with the rule enabled immediately after the class.

The installed release source was inspected at Oxc commit
`6941b4ed45ead21c45bc2164cb80a180eed85b43`, tag `oxlint_v1.88.0`:

- `crates/oxc_linter/src/rules/unicorn/custom_error_definition.rs:84`
  explicitly lists the dynamic assignment as invalid.
- The same file's constructor checks at lines 248 to 260 inspect assignments and fields.
  `is_expected_string_literal` at line 387 accepts a matching string literal.
- `npm/oxlint/configuration_schema.json:8912` declares this rule as `RuleNoConfig`.

A real configuration probe with an attempted dynamic-name allow-list fails with:

```text
# error-name-rule-unsupported-options.log
Invalid configuration for rule `unicorn/custom-error-definition`:
  This rule does not accept configuration options.
```

The ordinary configuration reports one name warning for the dynamic assignment.
Changing only that assignment to the suggested literal removes the warning.
The same literal changes the inherited setter's observed argument from the consumer constructor's name
to `PolicySlotBindingError`.
It therefore fails the existing consumer contract.
No global rule configuration was weakened.

The native build copies this typed module without minification.
At source checkpoint `98add8596e`, its source and private built copy are byte-identical:
808 bytes,
SHA-256 `7963f2c721d6641934b0d1455b4a6c70e2556771b2d18afab2660458a57a5253`.
This measurement precedes the formatting and exception comment.
It establishes the measured build behavior at that checkpoint,
not a guarantee about arbitrary future bundlers.

## Why a typed field is not a replacement

Adding `override readonly name: string` creates an own runtime property before the constructor assignment.
That property shadows an inherited accessor and prevents its setter from running.
The rejected proposal and corrected class were both exercised with the same consumer.
The rejected proposal records no setter calls;
the corrected class records the consumer constructor name once.

Node's type stripping preserves this field declaration's runtime effect.
The source trace and executable pass/fail reproduction are recorded in
[`node-typescript-error-class-fields.md`](../../../../doc/troubleshooting/node-typescript-error-class-fields.md).

The package's typed-port checks include inherited-property effects as well as plain message and cause values.
Root `TCV` remains unchanged,
following the user's decision to keep boundary lessons in package documentation.
