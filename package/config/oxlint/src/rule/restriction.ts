/**
 Restriction rule configuration.
 
 Rules that ban specific language features, enforce coding conventions,
 and prevent patterns that conflict with the project's style decisions.
 
 @example
 ```typescript
 import { restrictionRules } from './rule/restriction.ts';
 ```
 */

import type { DummyRuleMap, } from 'oxlint';

/**
 Restriction rules.
 */
export const restrictionRules: DummyRuleMap = {
  'eslint/no-iterator': 'error',

  // Tagged template expressions (css``, html``, etc.) aren't unused expressions.
  'eslint/no-unused-expressions': [
    'error',
    { allowTaggedTemplates: true, },
  ],

  // `void` as expression is allowed as a statement for fire-and-forget calls.
  // The explicit "off" overrides the "error" that would come from the eslint category.
  'eslint/no-void': 'off',
  'import/no-commonjs': 'error',
  'import/unambiguous': 'error',
  'promise/spec-only': 'error',
  'typescript/explicit-function-return-type': 'error',
  'unicorn/no-document-cookie': 'error',

  // Switch statements are banned; default-case is moot.
  'eslint/default-case': 'off',

  // Use if/else chains or Record<K, V> lookups instead of switch.
  'no-restricted-syntax/no-switch': 'error',

  // Use Object.entries/Object.keys/Object.values instead of for...in.
  'no-restricted-syntax/no-for-in': 'error',

  // `let` at function-body root leaks scope to every subsequent statement.
  // IIFE callees and helper-function shape (ends with `return <root-binding>`)
  // are allowlisted by AST heuristic. Migration complete: every report is
  // refactored or carries a justified disable, so severity is now 'error'
  // (see doc/audit/let.md status table).
  'no-restricted-syntax/no-function-root-let': 'error',

  // `let` at module root is mutable across the entire module. No allowlist;
  // use Map/WeakMap/Set, memoize() from @monochromatic-dev/kv-store-memoize,
  // or an IIFE-into-const initialization. Migration complete; severity is now
  // 'error' (see doc/audit/let.md status table).
  'no-restricted-syntax/no-module-root-let': 'error',

  // `describe({ name: '<fn>' })` silently drifts on rename. Prefer
  // `describe({ name: <fn>.name })` whenever `<fn>` is an in-scope binding.
  // Scoped to `*.unit.test.ts` via overrides; harness self-tests opt out
  // via inline disable comments.
  'no-restricted-syntax/prefer-describe-function-ref-name': 'warn',

  // Classes are banned except when the direct superclass or the class's own
  // name ends with a configured suffix (default: Error, Element). Long-lived
  // stateful objects use a factory returning a frozen object instead.
  // Initial severity 'warn' to surface existing footprint without blocking
  // CI; see doc/migration/no-class.md. Flips to 'error' after migration.
  'no-restricted-syntax/no-class': 'warn',

  // Use using/await using for cleanup instead of try...finally.
  'no-restricted-syntax/no-try-finally': 'error',

  // Bind catch values instead of using `catch {}` so failures stay inspectable.
  'no-restricted-syntax/catch-binding': 'error',

  // Keep unknown caught-value coercion and its effect contract in one shared module.
  'no-restricted-syntax/prefer-caught-value-text': 'error',

  // Prefer Error.isError() over realm-fragile instanceof checks, string tag
  // checks, constructor comparisons, and deprecated Node util.types.isNativeError().
  // Initial severity is warn because the workspace has existing violations;
  // the rule autofixes ordinary cases for migration.
  'no-restricted-syntax/prefer-error-is-error': 'warn',

  // Use union types with `as const` instead of enum declarations.
  'no-restricted-syntax/no-enum': 'error',

  // Union types containing `null` or `undefined` (`T | null`, `T | undefined`)
  // are banned. exactOptionalPropertyTypes is on; widening a slot to a nullish
  // member skirts it, and pivoting `undefined` to `null` is the same escape.
  // Ranked fixes: optional object property or field as `foo?: T`; local
  // presence check with a guard and early return; boundary failure via
  // nonNullishOrThrow; traveling absence value by minting a domain-specific
  // `unique symbol` sentinel for this exact absence condition, or carrying a
  // distinct non-empty domain value; genuine external mirrors use a scoped
  // disable with justification.
  'no-restricted-syntax/no-nullish-union': 'error',

  // Every other statically-detectable type-level fake-optional encoding, banned
  // in one pass so the whack-a-mole stops. Covers union members `| void`,
  // `| never`, `| unknown`/`| any`, `| {}`, and falsy literals (`""`, empty
  // template, `0`, negative, `false`); empty/optional/rest-only tuples and
  // optional named tuple members; `Partial<T>`, `Record<K, never>`,
  // `Pick<T, never>`; and added-optionality mapped types. `| null`/`| undefined`
  // are left to no-nullish-union. Same fixes apply: `foo?: T`, an if-guard,
  // nonNullishOrThrow, or a real Symbol/non-empty sentinel. Disableable with a
  // justified scoped comment for genuine external-boundary mirrors.
  'no-restricted-syntax/no-optional-escape': 'error',

  // Use named function declarations or named function expressions, never arrow functions.
  // Callbacks still need a name: items.map(function getValue(item) { return item.value; }).
  'no-restricted-syntax/no-arrow-function': 'error',

  // Upstream unicorn/no-array-callback-reference reports wrapper calls such as
  // items.findIndex(unary(isBig,)), even though the wrapper fixes the arity
  // footgun. The project rule keeps direct-reference reports while accepting
  // explicit wrapper calls.
  'unicorn/no-array-callback-reference': 'off',
  'no-restricted-syntax/no-array-callback-reference': 'warn',

  // All .catch() calls are banned. Use try/catch with async/await.
  'no-restricted-syntax/no-promise-catch': 'error',

  // All .finally() calls are banned. Use using/await using for cleanup.
  'no-restricted-syntax/no-promise-finally': 'error',

  // Use Object.hasOwn(obj, key) instead of obj.hasOwnProperty(key).
  'no-restricted-syntax/no-hasownproperty': 'error',

  // Upstream unicorn/no-immediate-mutation reports efficient Set/Map clone-plus-mutate patterns.
  // The project rule keeps normal initializer folding while allowing clone cases that would need a spread temp.
  'unicorn/no-immediate-mutation': 'off',
  'no-restricted-syntax/no-immediate-mutation': 'warn',

  // Upstream unicorn/no-useless-spread strips `[...x.slice()]` by method name alone, so --fix
  // rewrites strings, typed arrays, and iterators into different values (oxc#26159, repo #563).
  // The project rule keeps every upstream check, fixes ambiguous method spreads only when
  // TypeScript proves an array, and reports them without a fix when no type is available.
  'unicorn/no-useless-spread': 'off',
  'no-restricted-syntax/no-useless-spread': 'error',

  // Upstream node/no-sync is suffix-only and reports non-Node APIs like Optique parseSync.
  // The project rule below enforces the narrower policy: Node sync APIs only.
  'node/no-sync': 'off',
  'no-restricted-syntax/no-sync': 'error',
  'no-restricted-syntax/no-system-tmpdir': 'error',

  // Rest parameters (...args) are banned. Accept an explicit array parameter instead.
  // Spread syntax in call expressions and array literals is not affected.
  'no-restricted-syntax/no-rest-params': 'error',

  // Use .trimStart()/.trimEnd() instead of deprecated .trimLeft()/.trimRight().
  'no-restricted-syntax/no-trim-left-right': 'error',

  // `const fn = function() {}` is banned. Use a function declaration instead.
  // Function expressions as callbacks (not assigned to variables) are not affected.
  'no-restricted-syntax/no-variable-function-expression': 'error',

  // Function declarations with 2+ params must use a single destructured object parameter.
  // function createUser({ name, age }: { name: string; age: number }): User {}
  // Function expressions (callbacks) are exempt since their signatures are often dictated by external APIs.
  'no-restricted-syntax/require-destructured-params': 'error',

  // querySelector(), querySelectorAll(), and closest() must specify a generic type parameter.
  // e.g. document.querySelector<HTMLInputElement>('.my-input') instead of document.querySelector('.my-input').
  'no-restricted-syntax/require-queryselector-generic': 'error',

  // Regex usage must be rare and justified at the use site. Prefer index scans,
  // parsers, or string APIs; necessary regex sites use a scoped disable with
  // why-regex, input-bound, and backtracking-safety rationale.
  'no-restricted-syntax/no-regex': 'error',

  // Report only mutable parameters with a proved deeply readonly replacement.
  'prefer-readonly-parameter-type/prefer-readonly-parameter-types': 'error',

  // Disabled until workspace remediation in https://github.com/Aquaticat/Monochromatic/issues/423.
  // Change directly to 'error' when the issue readiness criteria pass.
  'prefer-readonly-parameter-type/no-readonly-parameter-mutations': 'off',

  // Disabled until workspace remediation in https://github.com/Aquaticat/Monochromatic/issues/423.
  // Change directly to 'error' when the issue readiness criteria pass.
  'prefer-readonly-parameter-type/no-opaque-parameter-effects': 'off',

  // Disabled until workspace remediation in https://github.com/Aquaticat/Monochromatic/issues/423.
  // Change directly to 'error' when the issue readiness criteria pass.
  'prefer-readonly-parameter-type/no-invalid-parameter-effect-contracts': 'off',

  // Static Symbol descriptions must carry enough debugging information. Sentinel
  // Symbols stand in for nullish unions, so the description is the only identity
  // at a crash site. A structural classifier (word count, casing, namespace
  // shape, repetition, and a few grammar hooks) gates it; no Shannon entropy, no
  // global compression, no vocabulary lists. Initial severity 'warn' until the
  // remaining repo descriptions are remediated; see
  // package/oxlint-plugin/no-restricted-syntax/README.md.
  'no-restricted-syntax/no-low-information-symbol-description': 'warn',

  //region no-disable -- prevent inline oxlint-disable for rules with no legitimate exceptions
  'no-restricted-syntax/no-disable-require-tsdoc': 'error',
  'no-restricted-syntax/no-disable-no-switch': 'error',
  'no-restricted-syntax/no-disable-no-for-in': 'error',
  'no-restricted-syntax/no-disable-no-enum': 'error',
  'no-restricted-syntax/no-disable-no-hasownproperty': 'error',
  'no-restricted-syntax/no-disable-no-trim-left-right': 'error',
  'no-restricted-syntax/no-disable-no-promise-catch': 'error',
  'no-restricted-syntax/no-disable-no-promise-finally': 'error',
  'no-restricted-syntax/no-disable-no-variable-function-expression': 'error',
  'no-restricted-syntax/no-disable-require-destructured-params': 'error',
  'no-restricted-syntax/no-disable-no-rest-params': 'error',
  'no-restricted-syntax/no-disable-no-arrow-function': 'error',
  'no-restricted-syntax/no-disable-no-try-finally': 'error',
  'no-restricted-syntax/no-disable-no-non-null-assertion': 'error',
  'no-restricted-syntax/no-disable-no-useless-return': 'error',
  'no-restricted-syntax/no-disable-require-returns': 'error',
  'no-restricted-syntax/no-disable-prefer-regexp-exec': 'error',
  'no-restricted-syntax/no-disable-prefer-readonly-parameter-types': 'error',
  //endregion no-disable

  // Never process.exit(): throw errors instead.
  'unicorn/no-process-exit': 'error',

  // Ban non-null assertion (!): use nonNullishOrThrow instead.
  'typescript/no-non-null-assertion': 'error',

  // Enforce import type for type-only imports.
  'typescript/consistent-type-imports': 'error',

  // Prefer const over let when variable is never reassigned.
  'eslint/prefer-const': 'warn',

  // Ban explicit any -- use unknown instead. When any is genuinely needed, use oxlint-disable.
  'typescript/no-explicit-any': 'warn',

  // No eval() or Function() constructor: arbitrary code execution.
  'eslint/no-eval': 'error',

  // No setTimeout/setInterval with string arguments: implied eval.
  'typescript/no-implied-eval': 'error',

  // No new Boolean/String/Number wrapper objects: use primitives.
  'eslint/no-new-wrappers': 'error',

  // No alert/confirm/prompt: use promise-based UI alternatives.
  'eslint/no-alert': 'error',

  // No debugger statements in committed code.
  'eslint/no-debugger': 'error',

  // No comma operator: obscures evaluation order.
  'eslint/no-sequences': 'error',

  // No arguments.caller/arguments.callee: deprecated and non-optimizable.
  'eslint/no-caller': 'error',

  // No labels that shadow variable names: confusing.
  'eslint/no-label-var': 'error',

  // No instanceof Array: use Array.isArray() which works across realms.
  'unicorn/no-instanceof-array': 'error',
  'eslint/no-empty': 'error',
  'eslint/no-empty-function': 'error',
  'eslint/no-eq-null': 'error',
  'eslint/no-proto': 'error',
  'eslint/no-regex-spaces': 'error',
  'eslint/max-statements': 'off',

  'eslint/no-var': 'error',
  'eslint/unicode-bom': 'error',
  'import/no-amd': 'error',
  'import/no-cycle': 'error',
  'import/no-webpack-loader-syntax': 'error',
  'oxc/bad-bitwise-operator': 'error',
  'oxc/no-const-enum': 'error',
  'typescript/no-dynamic-delete': 'error',
  'typescript/no-empty-object-type': 'error',
  'typescript/no-import-type-side-effects': 'error',
  'typescript/no-namespace': 'error',
  'typescript/no-non-null-asserted-nullish-coalescing': 'error',
  'typescript/no-var-requires': 'error',
  'typescript/prefer-literal-enum-member': 'error',
  'unicorn/no-abusive-eslint-disable': 'error',
  'unicorn/no-length-as-slice-end': 'error',
  'unicorn/no-magic-array-flat-depth': 'error',
  'unicorn/prefer-modern-math-apis': 'error',
  'unicorn/prefer-node-protocol': 'error',

  // Prefer Number.* properties over legacy global number functions and values.
  // Reports are rare in this workspace; keep new usages gated at introduction
  // instead of accumulating migration debt.
  'unicorn/prefer-number-properties': 'error',

  // Linters reporting unused vars aren't wanted.
  // The editor will report them.
  // Those vars are often left as code for syntax highlighting or testing purposes,
  // or they're part of work-in-progress.
  // The bundler will remove them in the final bundle.
  'eslint/no-unused-vars': 'off',

  // TODO comments are tracked via issues, not lint.
  'eslint/no-warning-comments': 'off',

  // Tests must reach their own package's behavior through the artifact that
  // package ships, never through sibling source. Defects have previously
  // survived the suite by existing only in built output, so a module tested
  // through source proves nothing about what consumers load.
  // See doc/planning/oxlint-test-import-eventual-artifact.md.
  'test-import/require-eventual-artifact': 'error',
};
