# @monochromatic-dev/oxlint-plugin-no-restricted-syntax

Oxlint JS plugin implementing `no-restricted-syntax` rules
that oxlint does not support natively.

Oxlint lacks ESLint's `no-restricted-syntax` rule because it requires
a full AST selector engine.
This plugin provides individual rules for each banned syntax pattern instead.

## Rules

- **no-arrow-function**:
   bans arrow functions in favor of named function declarations/expressions
- **no-array-callback-reference**:
   bans direct callback references in array iterator methods while allowing explicit arity wrapper calls
- **no-class**:
   bans class declarations and expressions in favor of composition-oriented objects and functions
- **no-enum**:
   bans `enum` declarations in favor of union types with `as const`
- **no-for-in**:
   bans `for...in` loops in favor of `Object.entries`/`Object.keys`/`Object.values`
- **no-function-root-let**:
   bans `let` at function-body root unless the helper-shape exception applies
- **no-hasownproperty**:
   bans `.hasOwnProperty()` in favor of `Object.hasOwn()`
- **no-immediate-mutation**:
   bans immediate clone-then-mutate patterns when immutable construction is clearer
- **no-low-information-symbol-description**:
   requires static `Symbol('...')` descriptions to carry enough debugging information;
   rejects generic identifiers,
   absence labels,
   and repeated low-information phrases
- **catch-binding**:
   bans `catch {}`;
   bind the caught value with `catch (error) {}` or another named binding
- **no-module-root-let**:
   bans module-root `let` in favor of containers or memoization helpers
- **no-promise-catch**:
   bans `.catch()` chaining in favor of `try`/`catch` with `async`/`await`
- **no-promise-finally**:
   bans `.finally()` chaining in favor of `using`/`await using`
- **no-regex**:
   requires regex usage to be justified through a scoped `oxlint-disable` comment
- **no-rest-params**:
   bans rest parameters (`...args`) in favor of explicit array parameters
- **no-switch**:
   bans `switch` statements in favor of if/else chains or `Record` lookups
- **no-sync**:
   bans Node sync APIs while allowing non-Node `Sync`-named APIs
- **no-trim-left-right**:
   bans `.trimLeft()`/`.trimRight()` in favor of `.trimStart()`/`.trimEnd()`
- **no-try-finally**:
   bans `try...finally` blocks in favor of `using`/`await using`
- **no-useless-spread**:
   replaces `unicorn/no-useless-spread`;
   reports spreads that only copy a fresh value,
   and asks TypeScript before touching ambiguous spreads such as `[...text.slice(0, 3)]`
   (see [no-useless-spread](#no-useless-spread))
- **no-nullish-union**:
   bans union types containing `null` or `undefined`
  (`T | null`,
   `T | undefined`).
   Ranked fixes are `foo?: T` for optional object properties or fields,
  local guard and early return,
   `nonNullishOrThrow`,
   a domain-specific `unique symbol` sentinel for the exact
  absence condition or distinct non-empty domain value,
   and a scoped disable for genuine external API mirrors.
- **no-optional-escape**:
   bans every other statically-detectable type-level fake-optional encoding (`| void`,
   `| never`,
   `| unknown`/`| any`,
   falsy literal members,
   `| {}`,
   empty/optional/rest-only tuples,
   `Partial<T>`,
   `Record<K, never>`,
   added-optionality mapped types)
- **no-variable-function-expression**:
   bans `const x = function() {}`,
   use a function declaration instead
- **prefer-describe-function-ref-name**:
   requires `describe({ name: fn.name })` when a test suite name mirrors an in-scope function binding
- **prefer-caught-value-text**:
   requires the shared `caughtValueText` or `caughtValueStack` helper instead of local Error-and-fallback formatters
- **prefer-error-is-error**:
   bans legacy Error detection (`instanceof Error`,
   `Object.prototype.toString` tags,
   constructor comparisons,
   and Node `util.types.isNativeError()`),
   autofixing them to `Error.isError(value,)`
- **no-disable-prefer-readonly-parameter-types**:
   prohibits inline suppression of semantic readonly-effect checks
- **require-destructured-params**:
   function declarations with 2+ params must use a single destructured object
- **require-queryselector-generic**:
   requires explicit generic typing for querySelector-style calls

`no-regex` is enabled by the shared `@monochromatic-dev/config-oxlint` package.
Necessary regex sites must use scoped disable justifications.

## Semantic readonly companion

The semantic `prefer-readonly-parameter-types` rule now lives in
`package/oxlint-plugin/prefer-readonly-parameter-type`.
This package retains `no-disable-prefer-readonly-parameter-types` so inline comments cannot bypass the dedicated
plugin's rule.

## no-useless-spread

Oxlint's `unicorn/no-useless-spread` removes the spread in `[...x.slice()]` because `slice` returns a new array
on arrays.
It decides by method name alone,
so `--fix` also rewrites strings,
typed arrays,
and iterators,
whose spread converts rather than copies
(oxc issue 26159;
repo issue 563).

This rule keeps every upstream check and applies fixes only where the rewrite provably keeps behavior:

- array literals spread into arrays or argument lists,
  and object literals spread into objects,
  are inlined;
  holes,
  accessors,
  `__proto__` keys,
  and edge comments withhold the fix;
- `new Set([...x])`,
  `Promise.all([...x])`,
  `Array.from([...x])`,
  and `Object.fromEntries([...x])` drop the spread;
  `for…of` and `yield*` are reported without a fix,
  because a loop that changes its source may need the snapshot;
- copies of values syntax proves fresh,
  such as `[...Array.from(x)]`,
  `[...Object.keys(x)]`,
  `[...new Array(n)]`
  (fixed to `new Array(n).fill()`),
  and `{ ...Object.fromEntries(x) }`,
  drop the spread;
  `{ ...Object.create(p) }` is not a copy,
  since the spread drops the prototype.

Spreads of methods shared by arrays and other receivers
(`concat`,
`copyWithin`,
`filter`,
`flat`,
`flatMap`,
`map`,
`slice`,
`splice`,
`toReversed`,
`toSorted`,
`toSpliced`,
`with`,
`split`,
and `reduce` with an array accumulator)
use progressive enhancement:

- where the TypeScript 7 semantic bridge from `@monochromatic-dev/oxlint-plugin-prefer-readonly-parameter-type`
  proves the value is an array,
  the spread is a useless copy and is fixed;
- where it proves a typed array,
  string,
  iterator,
  or mixed union,
  the spread converts and is not reported;
- where no type is available
  (`any`,
  `unknown`,
  error types,
  or a file outside every tsconfig project),
  the spread is reported without a fix.
  Code without types states its conversions:
  remove the spread for an array,
  use `.values().toArray()` for a typed array,
  `.toArray()` for an iterator,
  and a string API or `Intl.Segmenter` for a string.

The bundled `@monochromatic-dev/config-oxlint` output shares one bridge instance between this plugin and the
readonly plugin.
Design record:
`doc/planning/issue-563-array-conversion-lint.md`.

## no-nullish-union

`tsconfig` sets `exactOptionalPropertyTypes: true`.
Widening a type to `T | undefined` skirts that setting instead of fixing the underlying problem;
it lets `undefined` flow into a typed position the optional-property machinery was meant to keep absent.
Pivoting the same slot to `T | null` is not a fix;
 it is the identical nullish escape with a different keyword.
The rule flags any `TSUnionType` with a `TSUndefinedKeyword` or `TSNullKeyword` member:
`T | undefined`,
 `undefined | T`,
 `T | null`,
 `null | T`,
 and either nullish keyword anywhere in a union,
including nested forms such as `Promise<T | null>` and `Array<T | undefined>`.

This rule owns only `null` and `undefined` union members.
The sibling `no-optional-escape` rule owns `| void`,
 tuple encodings,
 `Partial<T>`,
and related type-level fake-optionality escapes.
`TSNullKeyword` is the `null` type keyword,
 distinct from the `null` literal node `TSNullLiteral`.
A standalone `type X = undefined` or `type X = null` is not a union and is not flagged.

Take the first branch that fits:

1.   **Optional object property or field**:
     write `foo?: T`,
      never `foo?: T | undefined` and never `foo: T | undefined`.
     Under `exactOptionalPropertyTypes`,
      `?:` already means "absent or `T`";
     the `| undefined` adds nothing and reopens the hole the setting closes.
2.   **Presence establishable here**:
     guard with `if` and return early so the typed slot receives only `T`.
3.   **Absence should fail loud at this boundary**:
     throw via `nonNullishOrThrow` from `@monochromatic-dev/or-throw-core`.
4.   **Absence must travel onward as a real value**:
     mint a domain-specific `unique symbol` sentinel for this exact absence condition,
     or carry a distinct non-empty domain value when the domain has one.
     Sentinels are local to one semantic absence condition and should not be reused across unrelated conditions.
     Export the sentinel only when consumers must compare against values returned by that API.
     Consumers narrow symbols with `typeof value === 'symbol'` first,
     then identity (`value === KEY_NOT_FOUND`).
     This is the heaviest ordinary fix;
      reach for it last.
5.   **Genuine external API mirror**:
     use a scoped
     `oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- <reason>`
     comment whose reason names the external API and why the mirror is unavoidable.

Never use `null`,
 `undefined`,
 empty string,
 zero,
 negative one,
 `false`,
 `void`,
 empty tuple,
empty object,
 or `Partial` as a stand-in for absent.
Empty arrays and `Option`/`Maybe`/`Result` wrappers are also not accepted absence modeling in this repo;
keep this diagnostic's canonical never-list aligned with `no-optional-escape`.

```typescript
// Bad
let cached: Provider | undefined;
let head: Node | null;
type Options = { existing?: ExistingNode | undefined; };
function find(): string | null {}

// Good
type Options = { existing?: ExistingNode; };

const value = lookup(key,);
if (value === undefined) {
  return;
}
// value is now `T`, never `T | undefined`

// Good; domain-specific Symbol sentinel instead of the union
const KEY_NOT_FOUND: unique symbol = Symbol('requested key not found in store',);
type Result = string | typeof KEY_NOT_FOUND;
```

## no-optional-escape

`exactOptionalPropertyTypes: true` keeps `undefined` out of typed slots.
Agents repeatedly invent new type-level encodings to dodge it:
 once `| undefined` and `| null` were banned,
 the next dodge was `| void`,
 then tuple-as-Maybe,
 then literal sentinels,
 then `Partial<T>`.
This rule enumerates and bans the whole statically-detectable space in one pass.
`| undefined` and `| null` stay with `no-nullish-union`;
 everything else lives here.

The fixes are the same four:
 `foo?: T` for an optional property;
 an `if`-guard so the value is always present where typed;
 throw via `nonNullishOrThrow` (`@monochromatic-dev/or-throw-core`);
 or a real sentinel (a unique `Symbol`,
 or a distinct non-empty domain value).
A genuine external-boundary mirror uses a scoped
`oxlint-disable-next-line no-restricted-syntax/no-optional-escape`
with a justification.

### Banned (each its own diagnostic)

Union members:

- `T | void`:
   `void` is assignable from `undefined`,
   so it widens the slot.
- `T | never`:
   collapses to `T`;
   a hand-written `| never` is a stubbed-out absence branch.
- `T | unknown` and `T | any`:
   collapse to the wide type,
   accepting everything including nullish.
- `T | {}`:
   an empty object type widens to any non-nullish value.
- Falsy literal members:
   the empty string `""`,
   an empty template literal type,
   zero `0`,
   a negative number such as `-1`,
   and `false`.
   Flagged only when the union also has a non-literal member,
   so a finite literal domain like `0 | 1 | 2` is left alone.

Tuples:

- Empty tuple `[]`.
- Optional element `[T?]`.
- Optional named member `[foo?: T]`.
- Rest-only tuple `[...T[]]` (functionally `T[]` dressed as 0-or-many).

Type references and mapped types:

- `Partial<T>` (makes every property optional).
- `Record<K, never>` and `Pick<T, never>` (produce an empty object).
- A mapped type that adds optionality,
   `{ [K in keyof T]?: ... }` (a hand-rolled `Partial`).

### Allowed (not flagged)

- A bare `(): void` return;
   only `void` inside a union is banned.
- `T | null` and `T | undefined`,
   owned by `no-nullish-union`.
- A fixed non-empty tuple `[number, string]`,
   and a leading-element variadic tuple `[T, ...U[]]` (one-or-more).
- A real `Symbol` sentinel via `typeof MY_SYMBOL`.
- A non-empty literal member (`T | 42`,
   `T | "pending"`) and pure literal domains (`0 | 1 | 2`,
   `"a" | "b"`).
- A real `Record<K, V>` or `Pick<T, K>`,
   and the `Required` mapped form `{ [K in keyof T]-?: ... }`.

### Statically undetectable (review-only blind spots)

A pattern is undetectable when the type annotation itself is honest and carries no syntactic marker of absence;
 there is nothing for an AST rule to see.

- A field typed `string` but defaulted to `""` at runtime:
   the type is `string`,
   an honest annotation.
- A `T[]` whose emptiness encodes absence:
   the type is `T[]`,
   honest.
- `0` or `-1` used as absent on a plain `number`:
   the type is `number`,
   honest.
   (Contrast `T | 0`,
   which is a literal-type union and is banned.
  )
- `T | typeof CONST` where `CONST` resolves to a falsy literal:
   the `typeof` node is identical whether `CONST` is a real `Symbol` or an empty string (verified by AST probe),
   so distinguishing it needs binding resolution.
   Reliable only for a same-file literal-initialized `const`;
   cross-file or imported `CONST` needs the type-checker the JS plugin lacks.
   Not implemented.
- `Omit<T, keyof T>` (equals `{}`):
   detectable in principle but needs a structural match between the `keyof T` argument and `T`;
   fragile,
   not implemented.
- `class Sentinel {}` plus `T | typeof Sentinel`:
   same blind spot as `typeof CONST`,
   and a class instance type is itself a distinct non-empty value,
   so it is an allowed sentinel anyway.
- `0n` (bigint zero) as a union literal:
   deliberately skipped;
   rare,
   and bigint literals add magic-literal friction for little gain.
- `NaN` as a union member is not expressible (`NaN` is a value,
   not a type),
   so there is nothing to detect.

## no-low-information-symbol-description

Sentinel `Symbol`s replace nullish unions across this codebase,
 so a Symbol's description is often the only debugging identity at a crash site.
This rule requires static descriptions to carry enough context.
It checks `Symbol('...')`,
 `Symbol.for('...')`,
 and zero-expression template literals.
Absent,
 dynamic,
 and non-string descriptions are skipped because an oxlint JS plugin has no type information;
 no-argument `Symbol()` is never reported.

The classifier is structural,
 not semantic.
It uses word count,
 distinct-word count,
 casing,
 namespace shape (`prefix/tail`,
 `prefix:tail`),
 meaningful-word repetition,
 and a small set of named grammar hooks (`no`,
 `not`,
 `because`,
 `ed`,
 `ing`).
It deliberately uses no Shannon entropy,
 no global compression,
 and no broad vocabulary lists.
A description passes when it carries a structural specificity marker (an uppercase letter,
 a digit,
 a dot,
 an underscore,
 or a consonant-dense token) or otherwise reads like a phrase rather than a bare identifier,
 even when short.

```ts
// Pass: enough context, even when short
Symbol('github token expired');
Symbol('file log.jsonl exists');
Symbol('penpot/figma-input-has-no-counterpart');
Symbol('average divisor is zero');

// Fail: too few words, all-caps constant, bare identifier, repetition, generic namespace tail
Symbol('meow');
Symbol('STATE IS UNKNOWN');
Symbol('runWithContext');
Symbol('file file exists');
Symbol('tsdoc/no-tag');
```

Dynamic and absent descriptions are skipped:

```ts
Symbol(buildId());            // skipped: not a static string
Symbol(`prefix-${dynamic}`);  // skipped: template literal with an expression
Symbol();                     // skipped: no description argument
```

Some descriptions are intentionally borderline and excluded from the calibration data,
 so they are labeled neither pass nor fail.
`no-static-method-name` and its uppercase variant are the current borderline rows.
If the rule reports a borderline-style description in real source,
 rewrite that Symbol description to carry more context rather than weakening the classifier.

The calibration data lives in `package/test-fixture/oxlint-no-restricted-syntax/data/`.
A browser benchmark there (`no-low-information-symbol-description.benchmark.html`)
compares this production classifier against threshold-only baselines
(minimum length,
 distinct words,
 type-token ratio) over the labeled pass and fail rows.
The classifier reaches zero misclassifications where the threshold baselines do not.

## Ban-disable rules

The plugin also includes `no-disable-*` rules that block broad or stale disable comments.
Use a tightly-scoped `oxlint-disable-next-line` with a justification when a rule cannot be satisfied structurally.

## Usage

```typescript
// oxlint.config.ts
import { defineConfig, } from 'oxlint';
export default defineConfig({
  jsPlugins: ['@monochromatic-dev/oxlint-plugin-no-restricted-syntax',],
},);
```

The package default export resolves to the prebuilt,
 self-contained
`dist/final/node/index.mjs` (run `mise run //package/oxlint-plugin/no-restricted-syntax:build`
first).
 TypeScript source is available at the `/ts` subpath (`/ts/*` for individual files)
for development.

## Tests

Fixture-based tests cover syntax rules,
ban-disable rules,
autofixes,
and symbol-description calibration data:

```bash
mise run buildAndTest -- package/oxlint-plugin/no-restricted-syntax/src/oxlint-no-restricted-syntax.unit.test.ts
```

Test fixtures live in `package/test-fixture/oxlint-no-restricted-syntax/src/`.
Symbol-description pass/fail/borderline rows live in
`package/test-fixture/oxlint-no-restricted-syntax/data/`.
