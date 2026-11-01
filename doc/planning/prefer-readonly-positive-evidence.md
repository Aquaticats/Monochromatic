# Separate readonly replacement evidence from unresolved effect auditing

Status:
 proposal from the investigation of [issue #422](https://github.com/Aquaticat/Monochromatic/issues/422).

Investigated:
 2026-08-13.

Scope:
 `package/oxlint-plugin/prefer-readonly-parameter-type` and its shared Oxlint configuration.

## Question

Issue #422 records `prefer-readonly-parameter-types` errors on inputs whose types are already deeply readonly.
The diagnostic says “or accept it”,
but shared configuration rejects inline suppression and provides no site-specific acceptance mechanism.

The design question is whether one rule should continue to report all of these states:

- a mutable parameter whose type can be proved safely readonly;
- a readonly parameter with a proved reachable mutation;
- a parameter that reaches a callee whose effects are unresolved.

The recommended answer is no.
A readonly preference should report only a proved replacement.
Proved unsupported declarations and unresolved effects are different policies with different severity and remediation.

## Local source trace

`package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types.ts`
defines `ALREADY_READONLY_EXPLANATION`.
It says “or accept it” and offers source inclusion,
identity isolation,
or call replacement.

`package/config/oxlint/src/rule/restriction.ts` enables both
`prefer-readonly-parameter-type/prefer-readonly-parameter-types` and
`no-restricted-syntax/no-disable-prefer-readonly-parameter-types` at error severity.

The no-disable rule is implemented in
`package/oxlint-plugin/no-restricted-syntax/src/rule/no-disable-prefer-readonly-parameter-types.ts`.
Its factory,
`package/oxlint-plugin/no-restricted-syntax/src/rule/_ban-disable-factory.ts`,
scans comments for `oxlint-disable*` directives containing the semantic rule ID.
The fixture at
`package/test-fixture/oxlint-no-restricted-syntax/src/invalid/no-disable-prefer-readonly-parameter-types.ts`
covers line,
block,
and mixed-list directives.

`doc/troubleshooting/oxlint-prefer-readonly-retained-closure.md` uses “Accept the withholding” for a different state.
That state means leaving a parameter mutable after the rule declines to offer readonly,
and it emits no error.
It does not express acceptance of an error on an already-readonly declaration.

The implementation already has a positive-evidence seam.
In
`package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types/verifier.ts`,
`shouldBeReadonly` is reached only after opaque effects return,
and only when the parameter is not mutated,
not retained,
not foreign-borrowed,
and classified mutable.

`package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types.ts`
builds the complete effect-summary index before it verifies and reports declarations.
Reporting can therefore be separated from effect propagation.
Silencing an already-readonly declaration does not require discarding its charge at mutable callers.

## External prior art

### TypeScript ESLint

The closest rule by name,
`@typescript-eslint/prefer-readonly-parameter-types`,
reports only when `isTypeReadonly(...)` is false.
Its source has one `if (!isReadOnly)` reporting branch and no call-effect analysis
([source](https://github.com/typescript-eslint/typescript-eslint/blob/main/packages/eslint-plugin/src/rules/prefer-readonly-parameter-types.ts)).
A deeply readonly parameter cannot trigger that rule.

Its documented `allow`,
`ignoreInferredTypes`,
and `treatMethodsAsReadonly` options make applicability tradeoffs explicit
([documentation](https://typescript-eslint.io/rules/prefer-readonly-parameter-types/)).
This design has a weaker behavioral guarantee than the project rule,
but its rule name and diagnostic describe one operation the author can perform.

### Clang

Clang's `misc-const-correctness` check calls mutation analysis and returns without reporting when the value is mutated
([source](https://github.com/llvm/llvm-project/blob/main/clang-tools-extra/clang-tidy/misc/ConstCorrectnessCheck.cpp)).
Its documentation says a value used to create a non-const handle that might escape is not diagnosed.
It also skips template functions because all instantiations cannot be known
([documentation](https://clang.llvm.org/extra/clang-tidy/checks/misc/const-correctness.html)).

Clang's `readability-non-const-parameter` check states that it warns only
 when constness makes the function interface safer.
It withholds a warning when nested pointer writes could make the const declaration misleading
([documentation](https://clang.llvm.org/extra/clang-tidy/checks/readability/non-const-parameter.html)).
Both checks prefer reduced recall over an unproved const replacement.

### Clippy

Clippy's `needless_pass_by_ref_mut` checks whether a `&mut` parameter is actually used mutably.
Its implementation follows aliases and closures,
but skips unsafe functions,
exact-signature contexts,
trait methods,
and function values that require the mutable signature
([source](https://github.com/rust-lang/rust-clippy/blob/master/clippy_lints/src/needless_pass_by_ref_mut.rs)).

The unsafe exclusions were added after a reported suggestion produced invalid and potentially unsound code
([rust-lang/rust-clippy#11180](https://github.com/rust-lang/rust-clippy/issues/11180)).
The repair was to withhold the suggestion where proof was insufficient,
not to emit an uncertainty error.

### Evidence-based mutation checks

ESLint's `no-param-reassign` reports observed parameter assignment and,
when configured,
observed property modification
([documentation](https://eslint.org/docs/latest/rules/no-param-reassign)).
JetBrains' `AssignmentToMethodParameter` similarly reports assignment or modification
 and supplies a local-variable rewrite
([documentation](https://www.jetbrains.com/help/inspectopedia/AssignmentToMethodParameter.html)).
These inverse rules report positive mutation evidence rather than unresolved possibilities.

### Acceptance registries

ESLint bulk suppressions use a committed external file,
enforce error-severity rules on new code,
and report stale suppression entries for pruning
([documentation](https://eslint.org/docs/latest/use/suppressions)).
If this project retains an acceptance mechanism,
a site-specific manifest with stable fingerprints
 and stale-entry enforcement is stronger than unstructured inline prose.
It is still weaker than not producing a nonactionable preference finding.

## Prototype measurement

The prototype used a detached worktree at `0d54ea643`,
the source commit that reduced the package from 119 to the issue's 118 findings.
It inserted one reporting guard:

```ts
// package/oxlint-plugin/prefer-readonly-parameter-type/src/prefer-readonly-parameter-types/verifier.ts
if (opaque
  && (!acceptedHostOpacity)
  && (classification.kind === 'sound-readonly')) {
  return;
}
```

On one installed dependency tree,
`mise run //package/module/translation-repair:lint:oxlint` (moved on 2026-10-18 to
`//package/translation-repair/<package>:lint:oxlint`) produced:

- baseline:
  118 `prefer-readonly-parameter-types` errors;
- prototype:
  66 errors;
- removed:
  52 errors;
- semantic-rule failures:
  zero in both runs.

The removed diagnostics comprised 46 general opaque-call messages,
one method message,
and five collection messages.
`opaque-effect-diagnostic.ts` selects the already-readonly wording only after its method and collection variants.
Changing only `opaqueEffectAlreadyReadonly` would therefore leave six reports on sound-readonly inputs.

The direct control at
`package/test-fixture/oxlint-no-restricted-syntax/src/readonly-result-provenance-invalid.ts`
behaved as required.
The baseline reported both
`handsReadonlyNamesOnward(handedNames: readonly string[])` and its mutable caller.
The prototype silenced the readonly declaration and retained the general unresolved-effect error on
`handsMutableNamesOnward(handedNames: string[])`.
The mutable function's only body statement calls the readonly function,
so the surviving error proves the charge still propagated.

The patched plugin built successfully.
`mise run //package/oxlint-plugin/prefer-readonly-parameter-type:lint:types` passed after the build.
The full unit suite was not run because its current assertions require the already-readonly diagnostic.
An implementation must update that expectation while retaining the mutable-caller propagation control.

The historical frozen installation was not independently reproducible with the current pnpm.
`pnpm install --offline --frozen-lockfile --ignore-scripts` emitted
`ERR_PNPM_LOCKFILE_MISSING_DEPENDENCY` for missing `@types/node@26.1.1` in `pnpm-lock.yaml`.
The 118 to 66 result is a matched source experiment,
not a cross-environment release reproduction.

## Distinct cause families

### Error cause constructors

The four `super(..., { cause })` sites pass a primitive message and a fresh ordinary options object.
ECMA-262 specifies that `InstallErrorCause` reads `cause` from the options object and installs the value
 as a data property on the new Error.
It does not inspect the cause value
([specification](https://tc39.es/ecma262/2026/multipage/fundamental-objects.html#sec-installerrorcause)).

A probe supplied a cause proxy trapping `get`,
`has`,
and `ownKeys`.
`new Error('message', { cause })` triggered none of those traps and preserved cause identity.
These calls can gain a verified intrinsic authority rather than site-specific acceptance.

The local precedent is
`doc/decision/prefer-readonly-default-library-readers.md`,
which admits readers under structural gates and hostile-object tests,
and
`doc/decision/prefer-readonly-member-channel-authority.md`,
which requires engine probes and positive controls for each channel claim.

### Serialization and coercion hooks

ECMA-262 requires `JSON.stringify` to read serialized properties,
read `toJSON`,
and call it when callable
([specification](https://tc39.es/ecma262/2026/multipage/structured-data.html#sec-serializejsonproperty)).
It requires object-to-primitive conversion to call `Symbol.toPrimitive`,
`valueOf`,
or `toString` when applicable
([specification](https://tc39.es/ecma262/2026/multipage/abstract-operations.html#sec-toprimitive)).

A probe observed `JSON.stringify` invoke both `toJSON` and a getter,
and observed `String` invoke `Symbol.toPrimitive`.
These are genuine user-code channels.
They justify withholding a readonly suggestion when effects remain unresolved.
They do not make an already-readonly preference error actionable.

The `grade-agreement.ts` finding is a valid example of this uncertainty.
`String(verdict)` runs inside the branch where `isGradeVerdict(verdict)` returned false,
so `verdict` remains `unknown` and can be an object carrying conversion hooks.
An investigation comment initially misread this as the positively narrowed branch;
[the correction][correction]
withdraws that claim.

### Host state

`Response.text()` and `body.getReader()` consume or lock response state.
Abort listener registration and request-lifetime signal retention affect host state.
These effects fit a host-effect contract rule.
They do not establish that a readonly preference must report uncertainty on ordinary data declarations.

### Injected `TextEncoder`

The `TextEncoder` finding is on the injected `sizer` receiver,
not its primitive string argument.
The Encoding Standard's built-in `encode(input)` creates and returns a new `Uint8Array`
([specification](https://encoding.spec.whatwg.org/#interface-textencoder)).

A caller-supplied value typed `TextEncoder` does not prove that the exact built-in method runs.
A positive-evidence preference rule should withhold the suggestion at that boundary.
It should not convert the withholding into an error.

## Accepted policy requiring reconciliation

`doc/audit/tech-prefer-readonly-native-effect-analysis-vet-2026-07-22.md` requires every unresolved effect to be
derived,
contained by a verified isolation boundary,
or reported as opaque.
Its hard constraints also require fail-closed behavior.

Splitting the diagnostics can preserve that policy when the separate unresolved-effect rule remains enabled at error
severity in the enforced scope.
Making that rule optional,
a warning,
or disabled would weaken the accepted policy and requires an explicit amendment.
The rule split itself does not authorize that change.

## Recommended diagnostic split

### `prefer-readonly-parameter-types`

Report only a mutable parameter with a proved readonly replacement.
Suggested wording:

> Parameter `X` can be deeply readonly.
> No reachable mutation,
> retention,
> or unresolved effect was found.

Include the exact suggested type.

### Proved unsupported readonly

Move `projectedReadonlyCapability` to a correctness rule if this policy remains an error.
Suggested wording:

> Parameter `X` is declared readonly,
> but this path mutates caller-reachable state:
> …

Include the mutation path.

### Unresolved effects

Move `opaqueEffect*` to a separately configured audit rule.
Name the exact cause rather than presenting uncertainty as a readonly edit.
Examples:

> Serialization can invoke getters and `toJSON` methods reachable from this input.
> Their effects are unresolved.

> String conversion can invoke `Symbol.toPrimitive`,
> `valueOf`,
> or `toString` on this value.
> Their effects are unresolved.

Do not say “or accept it” unless the message links to the exact acceptance mechanism.

## Ranked options

### Split the policies

- Pros:
  sound rule names,
  separate severity,
  preserved uncertainty audit,
  and only actionable preference errors.
- Cons:
  new rule IDs,
  configuration,
  migration,
  and documentation.

### Narrow the existing rule

- Pros:
  less configuration surface and no nonactionable opacity errors.
- Cons:
  removes the unresolved-effect audit rather than preserving it independently.

### Add an acceptance manifest

- Pros:
  explicit review records and stale-entry enforcement.
- Cons:
  exception debt,
  fingerprint design,
  and continued conflation of replacement with uncertainty.

### Downgrade already-readonly opacity

- Pros:
  limited configuration change and retained visibility.
- Cons:
  nonactionable messages remain,
  one rule still carries incompatible policies,
  and warning volume can normalize ignoring output.

Ranking:
 split > narrow > manifest > warning.
Splitting beats narrowing because it preserves the uncertainty audit under an sound name.
Narrowing beats a manifest because positive evidence avoids exception debt.
A manifest beats a blanket warning because each accepted site is explicit and can become stale.

## Posted investigation record

The investigation was posted incrementally:

- [source trace and wording](https://github.com/Aquaticat/Monochromatic/issues/422#issuecomment-5285601690);
- [external prior art and ranked design](https://github.com/Aquaticat/Monochromatic/issues/422#issuecomment-5285939316);
- [matched prototype and propagation control](https://github.com/Aquaticat/Monochromatic/issues/422#issuecomment-5286022557);
- [intrinsic and host cause families](https://github.com/Aquaticat/Monochromatic/issues/422#issuecomment-5286036647);
- [correction of the `grade-agreement.ts` branch and accepted-policy constraint][correction];
- [final synthesis and recommendation](https://github.com/Aquaticat/Monochromatic/issues/422#issuecomment-5286101731).

[correction]: https://github.com/Aquaticat/Monochromatic/issues/422#issuecomment-5286087654
