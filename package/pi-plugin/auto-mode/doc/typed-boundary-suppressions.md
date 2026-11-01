# Typed boundary suppressions

Lint suppressions in the typed policy-loader boundary stay few,
named,
and justified here.
Each entry cites the rule source,
the owning module,
and why the supported configuration cannot express the value.

## `no-restricted-syntax/no-nullish-union`

Rule source:
`package/oxlint-plugin/no-restricted-syntax/src/rule/no-nullish-union.ts`.
`package/config/oxlint/src/rule/restriction.ts`
configures `no-restricted-syntax/no-nullish-union` at `error`.
The rule message lists five supported branches:
 optional properties,
 if-guards,
 `nonNullishOrThrow`,
 unique-symbol sentinels,
 and a scoped disable only for mirrors of an external `T | undefined` API.

Three scoped disables remain,
all mirroring untyped owner modules whose `undefined` value must travel unchanged:

- `native/contract/integration/policy-source/registration.d.mts`
 (`copyAuthorityRegistration` return).
 Owner:
 `native/contract/integration/policy-source/registration.mjs`
 returns its validated copy or `undefined` for an absent declaration,
and untyped `registered-authority.mjs` compares that result by identity
 (`readAuthorityRegistration(policyInput)!==registration`).
- `native/contract/integration/policy-source/policy-input-collector.d.mts`
 (fixed-reader and host-prompt link fields).
 Owner:
 `native/contract/integration/policy-source/policy-input-collector.mjs`
 destructures these links and forwards the optional readers onward.
- `native/contract/integration/policy-source/policy-loader.ts`
 (`resourceInputs` return).
 Owner:
 `native/contract/integration/policy-source/policy-input-collector.mjs`
 consumes it as `nativeResourceToolInputs??[]`,
 so the unconfigured-reader absence must stay `undefined`.

Why the supported configuration cannot express these values:

- Branch (1) optional properties cannot hold an explicit `undefined` value
 under `exactOptionalPropertyTypes`,
 and these boundaries pass explicit values onward rather than omit keys.
- Branch (2) if-guards cannot cover a return position whose absence is the contract.
- Branch (3) `nonNullishOrThrow` would turn the original absence into a failure,
 changing behavior.
- Branch (4) unique-symbol sentinels would change values observed by untyped
 `??`,
 truthiness,
 and identity consumers.
- The rule exposes no configuration option,
 and the declaration-file override in
 `package/config/oxlint/src/overrides.ts` does not cover it.

## `unicorn/custom-error-definition`

`native/contract/integration/policy-source/registration.d.mts`
declares `InstructionRegistrationError` for the untyped
`native/contract/integration/policy-source/registration.mjs` owner,
which sets `this.name = this.constructor.name` at runtime.
An ambient declaration cannot carry an initializer
(TypeScript rejects initializers in declaration files),
so the declaration cannot reproduce that assignment.
A literal class-name type would be false for subclasses,
and `readonly` would conceal the owner's writable property.
The declaration uses writable `string` and the original message-only constructor.
The actual constructor ignores an extra options argument rather than copying a cause.

The executable `probe-registration-error-name` checks the base name,
the constructing subclass name,
and subsequent reassignment through a consumer.
The rule-source and rejected configuration-option evidence is recorded in
[`error-name-contract.md`](./error-name-contract.md).
Scoped lint reports nothing for this writable-`string` declaration without any disable,
so no `unicorn/custom-error-definition` suppression remains;
the earlier scoped disable was removed as unused after a bare-declaration check.
