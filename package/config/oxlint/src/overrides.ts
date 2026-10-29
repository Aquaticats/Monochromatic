/**
 File-pattern overrides for oxlint.
 
 Each override relaxes or tightens rules for specific file patterns
 (declaration files, test files, config files, etc.) where the base
 ruleset would be too strict or inappropriate.
 
 @example
 ```typescript
 import { overrides } from './overrides.ts';
 ```
 */

import type { OxlintOverride, } from 'oxlint';

/**
 Figma plugin globals.
 */
const figmaOverride = {
  files: ['**/figma-plugin/**',],
  globals: {
    figma: 'readonly' as const,
  },
} satisfies OxlintOverride;

/**
 Numeric type files can use magic numbers.
 */
const typeFileOverride = {
  files: ['**/*.type.*.ts',],
  rules: {
    'eslint/no-magic-numbers': 'off',
  },
} satisfies OxlintOverride;

/**
 Fixture files are exempt from line limits and magic numbers.
 */
const fixtureOverride = {
  files: ['**/fixture.*',],
  rules: {
    'eslint/no-magic-numbers': 'off',
    'eslint/max-lines': 'off',
  },
} satisfies OxlintOverride;

/**
 Model-generated canary artifacts have no meaningful line budget.
 */
const canaryOverride = {
  files: ['**/canary-lint/**',],
  rules: {
    'eslint/max-lines': 'off',
  },
} satisfies OxlintOverride;

/**
 Data modules (`*.data.ts`) hold declarations, not logic, so the code line budget does not apply.
 Owner decision, 2026-10-29: "the 300 line limit doesn't apply to non-code".
 */
const dataOverride = {
  files: ['**/*.data.ts',],
  rules: {
    'eslint/max-lines': 'off',
  },
} satisfies OxlintOverride;

/**
 Astro components have implicit module context.
 */
const astroOverride = {
  files: ['**/*.astro',],
  rules: {
    // Astro components have implicit module context and don't need import/export statements.
    'import/unambiguous': 'off',
    // Astro frontmatter requires exports (like getStaticPaths) at the top before component logic.
    'import/exports-last': 'off',
    // Astro requires getStaticPaths to be async even when no await is needed.
    'eslint/require-await': 'off',
  },
} satisfies OxlintOverride;

/**
 Declaration files describe external shapes that violate source conventions.
 */
const declarationOverride = {
  files: ['**/*.d.{ts,mts,cts}',],
  rules: {
    //region import: Declaration files are ambient; module-system rules don't apply.
    'import/unambiguous': 'off',
    'import/no-commonjs': 'off',
    //endregion import

    //region typescript: Declaration files describe external shapes that violate source conventions.
    // Declaration merging requires `interface`, not `type`.
    'typescript/consistent-type-definitions': 'off',
    // `declare namespace` is standard in ambient declarations.
    'typescript/no-namespace': 'off',
    // Empty interfaces serve as declaration merging stubs.
    'typescript/no-empty-object-type': 'off',
    // CJS type definitions use `import x = require()` syntax.
    'typescript/no-var-requires': 'off',
    // All imports in declaration files are type-level; the distinction is meaningless.
    'typescript/consistent-type-imports': 'off',
    // External API types sometimes require `any`.
    'typescript/no-explicit-any': 'off',
    //endregion typescript

    //region eslint: No runtime code exists in declaration files.
    // `declare var` is standard for global augmentation.
    'eslint/no-var': 'off',
    // Numeric literal types are not magic numbers.
    'eslint/no-magic-numbers': 'off',
    // Declaration files often declare multiple classes from a single external module.
    'eslint/max-classes-per-file': 'off',
    //endregion eslint

    //region no-restricted-syntax: External API signatures don't follow source conventions.
    // External APIs describe enums with `declare enum`.
    'no-restricted-syntax/no-enum': 'off',
    // External function signatures may use rest parameters.
    'no-restricted-syntax/no-rest-params': 'off',
    // External function signatures use positional parameters.
    'no-restricted-syntax/require-destructured-params': 'off',
    // Ambient declarations preserve external contracts without semantic verification.
    'prefer-readonly-parameter-type/prefer-readonly-parameter-types': 'off',
    //endregion no-restricted-syntax

    //region tsdoc: Ambient declarations are often trivial stubs.
    'tsdoc/require-tsdoc': 'off',
    'tsdoc/require-param': 'off',
    'tsdoc/require-returns': 'off',
    'tsdoc/require-yields': 'off',
    //endregion tsdoc
  },
} satisfies OxlintOverride;

/**
 Config files are exempt from line limits.
 */
const configOverride = {
  files: ['**/*.config.*',],
  rules: {
    'eslint/max-lines': 'off',
  },
} satisfies OxlintOverride;

/**
 The test module exposes Jest-style matchers (`toHaveBeenCalledWith`,
 `toHaveBeenLastCalledWith`, `toHaveBeenNthCalledWith`,
 `toHaveBeenCalledExactlyOnceWith`) whose public signatures take
 variadic positional arguments to match the Vitest/Jest convention
 documented in this package's README. The internal wrappers that
 forward through `MatcherSet[K]` inherit the same shape. External
 function signatures may use rest parameters.
 The module lives in `package/test-harness/core-expect` since the
 generic category elimination moved it from `package/module/test-expect`
 (2026-10-29), and the globs name that current location.
 */
const jestMatcherApiOverride = {
  files: [
    '**/test-harness/core-expect/src/expect.ts',
    '**/test-harness/core-expect/src/expect-matchers.ts',
  ],
  rules: {
    'no-restricted-syntax/no-rest-params': 'off',
  },
} satisfies OxlintOverride;

/**
 Test and benchmark files have relaxed rules for flexibility.
 */
const testOverride = {
  files: ['**/*.{test,bench}.ts',],
  rules: {
    'eslint/max-lines': 'off',
    'eslint/func-names': [
      'warn',
      'as-needed',
    ],
    'eslint/no-magic-numbers': 'off',
    'typescript/explicit-function-return-type': 'off',
    'unicorn/consistent-function-scoping': 'off',
    'promise/avoid-new': 'off',
    'eslint/require-await': 'off',
    'eslint/no-array-constructor': 'off',
    'promise/prefer-await-to-then': 'off',
    'node/no-sync': 'off',
    'no-restricted-syntax/no-sync': 'off',

    // Some test files just have too many TypeScript errors.
    'typescript/ban-ts-comment': [
      'error',
      {
        'ts-ignore': 'allow-with-description',
        'ts-nocheck': 'allow-with-description',
        'ts-expect-error': 'allow-with-description',
      },
    ],

    // Test assertions commonly use `as Type` for test data shaping.
    'typescript/no-unsafe-type-assertion': 'off',
    // Tests use `any` for mocking and edge-case coverage.
    'typescript/no-explicit-any': 'off',
    // Test callbacks often receive framework-owned mutable objects.
    'prefer-readonly-parameter-type/prefer-readonly-parameter-types': 'off',

    // Test TSDoc doesn't need full tag formatting but declarations still require TSDoc.
    'tsdoc/tag-lines': 'off',
    'tsdoc/require-param': 'off',
    'tsdoc/require-returns': 'off',
    'tsdoc/check-param-names': 'off',

    // Tests may use arrow functions in callbacks freely.
    'no-restricted-syntax/no-arrow-function': 'off',

    // Test helpers, spies, fixture APIs, and framework adapters often mirror positional signatures.
    'no-restricted-syntax/require-destructured-params': 'off',
    'no-restricted-syntax/no-disable-require-destructured-params': 'off',

    // Test code uses inline arrays, objects, and multi-arg calls freely.
    'stylistic/param-per-line': 'off',
    'stylistic/argument-per-line': 'off',
    'stylistic/array-element-per-line': 'off',
    'stylistic/object-property-per-line': 'off',
    'stylistic/import-per-line': 'off',
    'stylistic/export-per-line': 'off',
    'stylistic/type-property-per-line': 'off',
    'stylistic/tuple-per-line': 'off',
    'stylistic/destructure-per-line': 'off',
    'stylistic/chain-per-line': 'off',

    // Test setup/teardown may use init without assignment.
    'eslint/init-declarations': 'off',

    // Test fns are async for consistency even when they contain only sync assertions.
    'typescript/require-await': 'off',

    // Test harness expect().rejects/resolves return Promises that oxlint's type analysis doesn't recognize.
    'typescript/await-thenable': 'off',

    // Test harness expect() chains produce void-in-expression patterns that are intentional.
    'typescript/no-confusing-void-expression': 'off',

    // Test callbacks often have empty bodies for skip/noop cases.
    'eslint/no-empty-function': 'off',

    'no-empty-pattern': 'allow',
  },
} satisfies OxlintOverride;

/**
 The `prefer-describe-function-ref-name` rule fires when a `describe()` call
 uses a string literal name that matches an in-scope binding. Unit tests
 exercise a specific export and benefit from the function-reference form
 (`describe({ name: myFn.name })`) so suite names follow renames. Other
 test shapes (e2e, browser, bench) describe scenarios rather than single
 exports; the literal form is correct there.
 */
const nonUnitTestRuleOverride = {
  files: [
    '**/*.e2e.test.ts',
    '**/*.browser.test.ts',
    '**/*.bench.ts',
    '**/*.spec.ts',
  ],
  rules: {
    'no-restricted-syntax/prefer-describe-function-ref-name': 'off' as const,
  },
} satisfies OxlintOverride;

/**
 The effect rule cannot soundly use its own strict opacity policy to prove
 TypeScript semantic handles or Oxlint's host context. Self-application would
 require precisely the handwritten host authorities the rule forbids. Other
 rules remain active for its implementation and tests.
 
 ECMAScript collections were a third ground and no longer are. Read-only view
 receivers derive through `doc/decision/prefer-readonly-effect-model-split.md`,
 mutable ones through `doc/decision/prefer-readonly-mutable-collection-members.md`,
 and a verified member channel discharges the receiver claim through
 `doc/decision/prefer-readonly-member-channel-authority.md`.
 
 The exemption still covers the whole package because the remaining two grounds
 reach 52 of its 96 source files, and because what blocks the rest is now a
 short named list rather than collections generally.
 
 Measured on the three modules that import no semantic API, by linting them
 with this override removed: four findings, at
 `effect-element-application.ts:39`, `effect-callback-relation.ts:35`, and
 `effect-fixed-point-propagation.ts:37` and `:69`.
 
 An earlier revision of this comment described each finding by one cause and
 concluded "every one is a `Map.get`". That was measured too narrowly. Each
 finding names a list, and the `:37` finding alone names six causes:
 `(provenanceBySlot.get(slot,) ?? []).forEach` at
 `effect-slot-projection.ts:182`, `summaries.get` at `:90`, `summaries.values`
 at `:49`, `summary.opaqueProvenanceBySlot.set` at
 `effect-call-resolution.ts:419`, and `target.get` and `target.set` at
 `effect-uncertainty-provenance.ts:47` and `:55`.
 
 What blocks narrowing is result provenance: a summary fact recording that a
 call result is reachable from the receiver's parameter. That covers every
 `.get` cause, and `target.set` once the value it stores carries a known
 origin, because the code mutates what those lookups return and nothing tracks
 the alias.
 
 Iterator members were listed here as a second, independent blocker, on the
 ground that `summaries.values` stayed unproven whatever result provenance did.
 That is no longer true and the reason it was stated is no longer the reason it
 holds. The iterator members are in the channel authority now, claiming
 creation and drainage together, which discharges an iterator yielding
 primitives and nothing else. `summaries.values` yields `MutableEffectSummary`,
 so it still reports, and it now reports for exactly the same reason every
 `.get` cause does: nothing describes a container whose elements are receiver
 state. Measured after the entries landed, this probe's output was
 byte-identical.
 
 So one thing blocks narrowing, not two, and `doc/planning/prefer-readonly-iterator-members.md`
 records why closing it is larger than a table entry: the discharge would also
 need `effect-result-escape.ts` to recognise a spread and a for-of position,
 which is a global widening rather than a relation.
 
 Neither is a wider authority, which remains the thing this exemption is not
 waiting for.
 */
const readonlyEffectSelfHostingOverride = {
  files: [
    '**/oxlint-plugin/prefer-readonly-parameter-type/**',
  ],
  rules: {
    'prefer-readonly-parameter-type/prefer-readonly-parameter-types': 'off' as const,
  },
} satisfies OxlintOverride;

/**
 Two tests read the system temporary directory only to prove that a scratch
 directory is not inside it, which is the property the `no-system-tmpdir` rule
 protects, so they must name `tmpdir()` to compare against it. Vendored
 `upstream-fork` code follows its upstream and is not rewritten to this
 repository's scratch conventions.
 */
const systemTmpdirComparisonOverride = {
  files: [
    '**/test-harness/core/src/scratch-root.unit.test.ts',
    '**/git-clone-size/cli/src/temp-dir.unit.test.ts',
    '**/upstream-fork/**',
  ],
  rules: {
    'no-restricted-syntax/no-system-tmpdir': 'off' as const,
  },
} satisfies OxlintOverride;

/**
 All overrides, ordered from most specific to least specific.
 */
export const overrides: OxlintOverride[] = [
  systemTmpdirComparisonOverride,
  readonlyEffectSelfHostingOverride,
  figmaOverride,
  typeFileOverride,
  fixtureOverride,
  canaryOverride,
  dataOverride,
  astroOverride,
  declarationOverride,
  configOverride,
  jestMatcherApiOverride,
  nonUnitTestRuleOverride,
  testOverride,
];
