/**
 Style and pedantic rule configuration.
 
 Rules governing code style preferences, formatting conventions,
 and pedantic checks that improve consistency but have legitimate
 exceptions in certain contexts.
 
 @example
 ```typescript
 import { styleRules } from './rule/style.ts';
 ```
 */

import type { DummyRuleMap, } from 'oxlint';

/**
 Style and pedantic rules.
 */
export const styleRules: DummyRuleMap = {
  //region pedantic

  // Conflicts with some required tsdoc placements.
  'eslint/no-inline-comments': 'off',

  // unicorn/no-negated-condition isn't enabled because early fail is the practice.
  'unicorn/no-negated-condition': 'off',
  // Nested calls keep transformations readable in this codebase.
  'unicorn/max-nested-calls': 'off',
  // Same preference as the unicorn variant above.
  'eslint/no-negated-condition': 'off',

  // Three-way conflict: bare `return;` triggers TS7030 + no-useless-return;
  // omitting return triggers TS7030; `return undefined;` triggers this rule.
  // Disabling no-useless-undefined is the least wrong option because the
  // return IS meaningful (it satisfies the type checker's control-flow analysis).
  'unicorn/no-useless-undefined': 'off',

  // unicorn/no-object-as-default-parameter isn't enabled
  // because sometimes obj destructuring just can't.
  'unicorn/no-object-as-default-parameter': 'off',

  // Sometimes the type-checking if-statement isn't for checking type.
  'unicorn/prefer-type-error': 'off',
  'import/max-dependencies': 'off',

  // Since we're skipping blank lines and comments, 300 is generous enough that splitting up code files is always feasible.
  'eslint/max-lines': [
    'error',
    {
      max: 300,
      skipBlankLines: true,
      skipComments: true,
    },
  ],
  // NEVER, meaning one declaration per statement, which is the direction this
  // repo has always written. The rule arrived enabled by the `style` category
  // with its opposite default and reported 1447 warnings across code nobody had
  // touched, telling every consecutive `const` to merge into its predecessor.
  // That is also unfollowable wherever an exported and a module-private
  // constant sit next to each other, since one statement cannot be half
  // exported.
  'eslint/one-var': [
    'warn',
    'never',
  ],
  'eslint/max-params': 'off',
  'eslint/max-depth': 'off',
  'eslint/max-lines-per-function': 'off',
  'eslint/max-nested-callbacks': 'off',
  'eslint/max-classes-per-file': 'off',
  'id-length': 'allow',

  // Nested ternaries work great!
  'eslint/no-nested-ternary': 'off',
  'unicorn/no-nested-ternary': 'off',

  // Conflicts with TypeScript function overloading.
  'eslint/no-redeclare': 'off',

  // The unnecessary microtask tick from `return await` is negligible;
  // disabling avoids a three-way conflict with require-await and await-thenable
  // when an async function's only await sites are tail-position returns.
  'typescript/return-await': 'off',

  // The rule's documented promise-handler rationale does not fit
  // Promise.all(input.map(callback)) callbacks, and its raw-value rewrite can
  // conflict with require-await and await-thenable in mixed sync/async mappers.
  'promise/no-return-wrap': 'off',

  // Doesn't support eslint parameter allow when intersecting.
  'typescript/ban-types': 'off',

  // Sometimes a generic Function type is necessary.
  // Use `(...args: any) => any` instead.
  'typescript/no-unsafe-function-type': 'error',

  //endregion pedantic

  //region style

  'capitalized-comments': 'off',

  // Sometimes `import { a, type B } from 'module'` is needed.
  'import/consistent-type-specifier-style': 'off',

  // Good rule generally. Bad for library with a barrel file.
  'import/prefer-default-export': 'off',

  'import/exports-last': 'off',

  // Good traces are desired.
  'eslint/func-names': [
    'warn',
    'as-needed',
  ],

  // Named callbacks in fixed property slots, such as test harness `fn`, keep useful stack traces.
  'eslint/func-name-matching': 'off',

  // Uppercase factory and JSX-style component function names are common in this codebase.
  'eslint/new-cap': 'off',

  // Switch statements are banned; switch-case-braces is moot.
  'unicorn/switch-case-braces': 'off',

  // TypeScript function overloads work better with function declarations.
  'eslint/func-style': 'off',

  // Named function callbacks keep traces readable and match repository style.
  'prefer-arrow-callback': 'off',

  // for...in is banned entirely by no-restricted-syntax/no-for-in; guard-for-in is moot.
  'eslint/guard-for-in': 'off',
  'eslint/no-continue': 'off',

  // Leading underscores mark intentionally unused bindings, including disposable guards.
  'eslint/no-underscore-dangle': 'off',

  // oxlint supports ignoring specific values.
  'eslint/no-magic-numbers': [
    'warn',
    {
      ignore: [
        1,
        -1,
        0,
        2,
        -2,
        10,
        16,
        100,
        255,
        0.1,
        10,
        '0n',
        '1n',
        '2n',
        '-1n',
        '-2n',
      ],
      enforceConst: true,
      detectObjects: false,
      ignoreEnums: true,
      ignoreNumericLiteralTypes: true,
      ignoreTypeIndexes: true,
      ignoreReadonlyClassProperties: true,
      ignoreClassFieldInitialValues: true,
      ignoreDefaultValues: true,
    },
  ],
  'eslint/no-ternary': 'off',

  // Sometimes method chaining is more concise.
  'eslint/prefer-spread': 'off',

  // Upstream unicorn/prefer-spread rewrites `x.slice()` to `[...x]` by guessing from the name
  // whether `x` is an array, turning typed-array copies into plain arrays (repo #565).
  // The project rule fixes only receivers TypeScript proves are arrays, leaves typed arrays and
  // strings alone, and reports untyped `slice`/`concat` receivers without a fix.
  'unicorn/prefer-spread': 'off',
  'no-restricted-syntax/prefer-spread': 'warn',

  // Doesn't matter.
  'eslint/sort-imports': 'off',
  'eslint/sort-keys': 'off',

  // Too many false positives.
  'eslint/yoda': 'off',

  'import/no-named-export': 'off',
  'import/no-namespace': 'off',

  //region stylistic: line indentation
  // Leading whitespace follows the plugin's anchor-based indent model.
  'stylistic/indentation': 'warn',
  //endregion stylistic: line indentation

  //region stylistic: comment layout
  // TSDoc body lines omit canonical `* ` prefixes while retaining one-space block indentation.
  'stylistic/require-asterisk-prefix': [
    'warn',
    'never',
  ],
  //endregion stylistic: comment layout

  //region stylistic: one-item-per-line enforcement
  'stylistic/param-per-line': 'warn',
  'stylistic/argument-per-line': 'warn',
  'stylistic/array-element-per-line': 'warn',
  'stylistic/object-property-per-line': 'warn',
  'stylistic/import-per-line': 'warn',
  'stylistic/export-per-line': 'warn',
  'stylistic/type-property-per-line': 'warn',
  'stylistic/tuple-per-line': 'warn',
  'stylistic/destructure-per-line': 'warn',
  //endregion stylistic

  //region stylistic: statement boundaries
  'stylistic/one-var-declaration-per-line': 'warn',
  'stylistic/max-statements-per-line': 'warn',
  'stylistic/semi': 'warn',
  //endregion stylistic: statement boundaries

  //region stylistic: expression structure
  'stylistic/no-mixed-operators': 'warn',
  'stylistic/chain-per-line': 'warn',
  'stylistic/invocation-depth-per-line': 'warn',
  //endregion stylistic: expression structure

  // Formatting concern only.
  'eslint/curly': 'off',

  // Readable enough.
  'unicorn/no-await-expression-member': 'off',
  'typescript/consistent-type-definitions': [
    'error',
    'type',
  ],
  'typescript/consistent-indexed-object-style': [
    'error',
    'record',
  ],
  'typescript/no-inferrable-types': 'off',
  'unicorn/catch-error-name': 'off',
  'unicorn/filename-case': 'off',

  // Arrow functions are banned; capturing `this` in named function closures is the standard pattern.
  'unicorn/no-this-assignment': 'off',

  // Some libraries require null.
  'unicorn/no-null': 'off',

  // no-restricted-syntax/catch-binding owns the opposite convention:
  // every catch must bind the thrown value.
  'unicorn/prefer-optional-catch-binding': 'off',

  // Always assume property/method not on called.
  'unicorn/prefer-reflect-apply': 'error',

  // Because JS's default Array join separator isn't an empty string.
  'unicorn/require-array-join-separator': 'error',

  // Default minimumDigits of 5 rejects separators on 4-digit numbers (e.g. 3_000) while accepting 12_000.
  // Setting to 0 makes grouping consistent for all numeric literals that already contain separators.
  'unicorn/numeric-separators-style': [
    'warn',
    { number: { minimumDigits: 0, }, },
  ],
  //endregion style
};
