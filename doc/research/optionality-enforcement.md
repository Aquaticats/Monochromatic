# Optionality enforcement

How to further reduce "fake optionality" mistakes in this monorepo:
 encodings that smuggle an absent
value into a typed slot instead of using one of the accepted forms.
 This is a research note;
 it changes
no source.
 It surveys what the existing lint rules can and cannot catch,
 what the type system can add,
whether a single helper would remove the temptation,
 and how comparable strict-TypeScript projects
enforce the same discipline.
 Recommendations are ranked at the end.

`doc/research/` did not exist before this file;
 it is a new doc family of one,
 created because the task
named the path.
 Doc-placement conventions in `AGENTS.md` allow per-family directories under `doc/`.

## Problem statement

The accepted ways to model an optional or absent value in this repo are exactly four:

1.    An optional property,
       `foo?: T`.
       Under `exactOptionalPropertyTypes` this already means "absent or `T`".
2.    An `if`-guard that narrows before the value reaches the typed slot,
       so the slot only ever holds `T`.
3.    Throwing at the boundary with `nonNullishOrThrow` from `@monochromatic-dev/or-throw-core`.
4.    A real,
       non-nullish,
       non-empty sentinel:
       a unique `Symbol`,
       or a distinct meaningful domain value.
      Never `null`,
       `undefined`,
       `''`,
       `0`,
       or an empty array.

Agents repeatedly dodge the rules along a predictable ladder,
 each rung evading the previous ban:
`| undefined` becomes `| null`,
 which becomes a `[value]` / `[]` tuple-as-maybe,
 which becomes a
disguised-empty sentinel such as `''` or `0`.
 The first two rungs are now banned by `no-nullish-union`.
The question is how far down the ladder static enforcement can reach,
 and what closes the rest.

## Verified repo state

Facts below come from reading the tree on 2026-05-25,
 not from the task brief.

- `exactOptionalPropertyTypes: true`,
   `strict: true`,
   `noImplicitAny: true`,
   and
  `noUncheckedIndexedAccess: true` are all set in
  `package/config/typescript/tsconfig.options.json:45` to `:52`.
- `no-nullish-union` exists and is registered:
  `package/oxlint-plugin/no-restricted-syntax/src/rule/no-nullish-union.ts`,
   wired in `index.ts:38`
  and `:89`.
   It bans `null` or `undefined` as a member of any `TSUnionType`,
   and its own TSDoc states
  `void` is out of scope (`no-nullish-union.ts:42` to `:45`).
   Its unit test asserts ten distinct
  nullish-union forms are caught (`oxlint-no-restricted-syntax.unit.test.ts:262` to `:273`).
- `no-optional-escape` does not exist.
   A whole-tree `rg` for `optional-escape`,
   `optionalEscape`,
  `noOptionalEscape`,
   and `no-optional-escape` returned nothing;
   `git status` of the rules directory is
  clean;
   the most recent commit is `5954a771 feat(oxlint-no-restricted-syntax): ban T | null too, rename
  to no-nullish-union`.
   The task brief describes it as "just-added",
   which is not true relative to the
  tree.
   This note treats it as proposed and not yet present,
   which is why shipping it is a recommendation
  rather than a reaffirmation.
- No `no-disable-no-nullish-union` companion exists.
   Every other syntax rule has a paired
  `no-disable-*` built from `banDisableRule` in `_ban-disable-factory.ts`;
   `no-nullish-union` is the lone
  syntax rule without one (compare the two `//region` blocks in `index.ts:81` to `:123`).
- `AGENTS.md` has no dedicated rule for modeling optional or absent values.
   The only adjacent lines are
  `AGENTS.md:412` ("Custom error classes;
   throw over error codes/null/result types") and `AGENTS.md:413`
  (`nonNullishOrThrow` instead of `!`).
   The sentinel discipline currently lives only inside the lint
  rule's TSDoc and error message,
   not in the agent-facing instructions.
- `omitUndefined` already exists at
  `package/dev-script/inference-canary-viewer/src/data/omit-undefined.ts`.
   It drops `undefined`-valued
  keys and re-narrows each remaining key to `Exclude<T[K], undefined>`,
   so a record can be built with
  exact-optional fields without widening any slot to `T | undefined`.

## Why the oxlint plugin is syntax-only

The detectability ceiling is set by one fact.
 The `@oxlint/plugins` type definition states,
 verbatim,
"Oxlint does not offer any parser services" and types `parserServices` as
`Readonly<Record<string, unknown>>`
(`node_modules/.pnpm/@oxlint+plugins@1.58.0/node_modules/@oxlint/plugins/index.d.ts:3383` to `:3387`).
A rule receives the ESTree AST,
 the scope manager,
 comments,
 and source text,
 but no TypeScript
type-checker and no type-flow information.

The consequence:
 a rule can read what is written (the type annotation,
 the literal token) but cannot read
what a value's declared type is,
 nor whether a given literal is being used to mean "absent".
 The AST
exposes the relevant nodes (`TSUnionType`,
 `TSVoidKeyword`,
 `TSTupleType`,
 `TSOptionalType`,
`TSArrayType`,
 `StringLiteral`,
 `NumericLiteral`,
 `ArrayExpression`,
 all present in the exported node
list at `index.d.ts:1136`),
 so type-position syntax is reachable;
 semantic intent is not.

## Escape-hatch taxonomy: detectable versus not

The decisive split is type position versus value position.
 A type annotation is a fixed AST shape a
syntactic rule can match.
 A runtime value is an ordinary expression whose "absent" meaning lives in a
human convention the AST does not record.

### Statically detectable (type-position syntax)

- `T | undefined` and `T | null`.
   A `TSUnionType` whose members include `TSUndefinedKeyword` or
  `TSNullKeyword`.
   Already banned by `no-nullish-union`.
- `T | void`.
   A `TSUnionType` member that is `TSVoidKeyword`.
   Detectable,
   and not currently banned;
  `no-nullish-union` deliberately excludes `void`.
   See the scope note below.
- Empty tuple type `[]`.
   A `TSTupleType` with zero elements used as a value type.
   Detectable.
- Optional tuple element `[T?]`.
   A `TSOptionalType` inside a `TSTupleType`.
   Detectable.

These are exactly the shapes a proposed `no-optional-escape` rule would target:
 it operates entirely in
type position,
 where the AST is faithful.

### Not statically detectable by the oxlint plugin

- `''`-as-sentinel.
   The literal `''` is a `StringLiteral`;
   the slot's declared type is `string`.
   An
  empty string used to mean "absent" is type-correct and syntactically identical to a legitimate empty
  string.
   Distinguishing them needs both type-flow information the plugin lacks and an intent that is
  never written down.
- `0`-as-sentinel.
   Same shape:
   a `NumericLiteral` in a `number` slot.
   No node separates a sentinel zero
  from an arithmetic zero.
- Plain `T[]`-as-maybe at the value level.
   `const x: T[] = []` then "empty means none" is a convention
  over a normal array.
   The `T[]` annotation (`TSArrayType`) is detectable,
   but banning all arrays is
  absurd;
   the escape-hatch use is indistinguishable from every legitimate array,
   so it cannot be flagged
  without massive false positives.
- Value-position tuple-maybe,
   `[value]` / `[]` as runtime values.
   These are ordinary `ArrayExpression`
  nodes.
   Only their type-position cousins (`[]` and `[T?]` annotations) are reachable;
   the runtime dodge
  stays unenforceable.

The general rule:
 the lower three rungs of the ladder (`''`,
 `0`,
 array-as-maybe) are semantic,
 not
syntactic.
 Even a fully type-aware linter cannot decide "is this empty string a sentinel",
 because
emptiness-as-absence is a convention,
 not a type.
 Static enforcement stops at the type-annotation
boundary;
 everything past it is closed by type-system design,
 helpers,
 and instructions,
 not by a rule.

### Scope note on banning `void` in a union

`no-nullish-union.ts:42` to `:45` excludes `void` on purpose,
 so the proposed `no-optional-escape` would
expand scope.
 The expansion is defensible but should be justified rather than inherited:
 a function whose
return annotation is `: void` is fine and common,
 but `T | void` in a value position is the same fake
optionality as `T | undefined` (`void` is assignable from `undefined`).
 The rule should therefore match
`void` only as a `TSUnionType` member,
 never the bare `: void` return annotation,
 mirroring how
`no-nullish-union` matches nullish keywords only inside a union.

## TypeScript type-system levers

These make some fake-optional encodings ill-typed at a chosen boundary.
 None is a global enforcer;
 each is
opt-in at the point a value enters typed code,
 and none is checkable by the syntax-only lint plugin.
Verified against the TypeScript template-literal-types and conditional-types documentation
(<https://www.typescriptlang.org/docs/handbook/2/template-literal-types.html> and
<https://www.typescriptlang.org/docs/handbook/2/conditional-types.html>).

- Non-empty string brand via a conditional template-literal type:
  `type NonEmpty<S extends string> = S extends '' ? never : S`.
   A constructor typed
  `function requireNonEmpty<const S extends string>(s: S): NonEmpty<S>` rejects a literal `''` at compile
  time (the argument resolves to `never`),
   so `''`-as-sentinel fails at the assignment even though the
  AST shows a normal string.
   This only bites on literal types,
   not on a widened `string`,
   so it is a
  boundary constructor,
   not a blanket guard.
- Branded or opaque domain primitives,
   already endorsed at `AGENTS.md:386` ("branded types for domain
  primitives").
   A `NonEmptyString` or `PositiveInt` brand built behind a `requireNonEmpty` /
  `requirePositive` boundary that throws on `''` or `0` means the typed slot downstream can never carry
  the disguised-empty sentinel;
   the only way in is through the constructor that rejects it.
- `noUncheckedIndexedAccess` (already on) is the closest thing to a global lever already paying off:
   it
  makes `arr[i]` and `record[key]` yield `T | undefined`,
   which forces a guard or `nonNullishOrThrow`
  rather than silently trusting presence.
   It does not catch the `''` / `0` encodings,
   but it removes one
  common source of accidental `undefined` that agents would otherwise paper over with a sentinel.

There is no tsconfig flag that surfaces "this literal is a disguised-absent sentinel";
 that judgment is
semantic and lives outside the type system.

## Codebase remedy survey

### `omitUndefined` (exists)

`omit-undefined.ts` is the right shape and should be promoted,
 not replaced.
 It removes the single most
common reason an agent reaches for `T | undefined`:
 building a record where some fields may be absent
under `exactOptionalPropertyTypes`.
 Spreading `...omitUndefined({ a, b })` keeps `undefined` out of the
typed slot while still omitting the missing fields,
 and the mapped return type re-narrows each key to
`Exclude<T[K], undefined>`.
 Cost:
 one `no-unsafe-type-assertion` disable (justified inline at
`omit-undefined.ts:34`),
 unavoidable because `Object.fromEntries` widens.
 Limitation:
 it lives in a
dev-script package,
 so it is not a shared utility yet.
 If it is to be the canonical answer,
 it belongs in
a shared module (for example a `@monochromatic-dev/module-*` package) so every package can import it.

### A tagged Option or Maybe type (reject)

A proper `Option<T>` (effect Option,
 fp-ts `Option`,
 neverthrow's `Result`,
 ts-belt) is the textbook
answer in functional TypeScript,
 and the libraries are real and maintained to varying degrees:
 effect is
the official successor to fp-ts with fp-ts's author now on the team
(<https://effect.website/docs/additional-resources/effect-vs-fp-ts/>),
 while neverthrow is simpler but no
longer actively maintained (<https://npm-compare.com/fp-ts,neverthrow,ts-results,ts-toolbelt>).

It should still be rejected here,
 for two repo-specific reasons:

- It contradicts an existing documented stance.
   `AGENTS.md:412` reads "throw over error codes/null/result
  types".
   An `Option`/`Maybe` import is the same shape as the result types that line rejects.
- It would become the next escape hatch.
   `Option.none` is a blessed stand-in for "absent",
   which is
  precisely the move the four accepted forms exist to prevent.
   A team that reflexively wraps values in
  `Option` has re-introduced nullable everywhere,
   only with a heavier import and a `.map` chain on top.

The idiomatic answer this repo already has is the pairing of `omitUndefined` (for object construction) and
`nonNullishOrThrow` (for "should be present,
 fail loud if not"),
 plus the `if`-guard and `Symbol`
sentinel forms.
 Making that explicit in `AGENTS.md` is more valuable than importing a library.

### Guard helpers (extend)

`nonNullishOrThrow` already covers the "throw at the boundary" form.
 The gap is non-empty and non-zero
boundaries.
 Small throwing constructors (`requireNonEmpty`,
 `requirePositive`) in the same module family
would give the `''` / `0` rungs a one-line,
 in-philosophy remedy,
 so an agent tempted to use a
disguised-empty sentinel has an obvious,
 shorter alternative.

## Process levers

### AGENTS.md wording (proposed, exact)

Add a subsection under "Type system" (after `AGENTS.md:392`),
 because that is where related type-shape
rules already live and where a future session looks first.
 Proposed text:

```md
#### Modeling optional and absent values

`exactOptionalPropertyTypes` is on. Model an optional or absent value in exactly one of four ways;
anything else is a fake-optionality escape hatch and is banned.

1.    Optional property: `foo?: T`. Never `foo?: T | undefined`, never `foo: T | undefined`.
2.    `if`-guard that narrows before the value reaches the typed slot, so the slot only holds `T`.
3.    Throw at the boundary: `nonNullishOrThrow` from `@monochromatic-dev/or-throw-core`;
      `requireNonEmpty` / `requirePositive` for non-empty-string and non-zero boundaries.
4.    Real sentinel: a unique `Symbol`, or a distinct meaningful domain value.
      Never `null`, `undefined`, `''`, `0`, or an empty array as a stand-in for "absent".

Banned encodings, in the order agents reach for them: `T | undefined` (caught by `no-nullish-union`),
`T | null` (same rule), `T | void` and empty or optional tuple types (caught by `no-optional-escape`),
then `''` / `0` / empty-array as a disguised-absent sentinel. The last group is not statically
detectable, because emptiness-as-absence is a convention, not a type; do not use it. For object
construction with optional fields, spread `omitUndefined(...)` rather than widening a slot.
```

### Child-prompt wording

When spawning a child session that will write TypeScript,
 include one line in the prompt so the rung
ladder is named before code is written,
 for example:
 "Optional or absent values:
 use `foo?: T`,
 an
`if`-guard,
 `nonNullishOrThrow`,
 or a real `Symbol`/domain sentinel.
 Never `| undefined`,
 `| null`,
`| void`,
 an optional or empty tuple,
 or `''` / `0` / `[]` as a stand-in for absent.
" Naming the
undetectable rungs explicitly matters more in a child prompt than in the lint config,
 precisely because
the lint cannot catch them.

### no-disable-* companions (warranted)

The codebase pattern is unambiguous:
 every syntax rule has a paired `no-disable-*` built from
`banDisableRule`.
 `no-nullish-union` is currently the only syntax rule without one.
 Add
`no-disable-no-nullish-union` now,
 and `no-disable-no-optional-escape` alongside the new rule,
 both from
the existing factory,
 both registered in the ban-disable `//region` of `index.ts`.
 Without the companion,
the entire ladder is reopened by a single `// oxlint-disable-next-line no-restricted-syntax/no-nullish-union`.

## How comparable strict-TypeScript projects enforce this

- typescript-eslint `strict-boolean-expressions` and `no-unnecessary-condition`
  (<https://typescript-eslint.io/rules/strict-boolean-expressions/> and
  <https://typescript-eslint.io/rules/no-unnecessary-condition/>).
   Both are type-aware;
   they need
  parserServices and the TS program.
   `strict-boolean-expressions` even has a dedicated option about
  treating an empty string as falsy,
   which confirms that the strongest mainstream tool addresses the
  coercion site ("is this used in a boolean position"),
   not the encoding decision ("is this `''` meant as
  absent").
   Not applicable to the oxlint plugin,
   which has no type info;
   the lesson is that even with full
  type access,
   the `''`-as-sentinel intent is not what these rules check.
- eslint-plugin-total-functions (<https://github.com/danielnixon/eslint-plugin-total-functions> and
  <https://www.npmjs.com/package/eslint-plugin-total-functions>).
   `require-strict-mode` enforces `strict`
  plus `noUncheckedIndexedAccess` (both already on here);
   `no-unsafe-type-assertion` blocks the
  `{} as Foo` move.
   The philosophy ("total functions,
   no partial functions") matches this repo's
  throw-at-the-boundary stance,
   but every rule is type-aware,
   so none ports to oxlint directly.
   It does
  validate that `noUncheckedIndexedAccess` belongs in the strict baseline,
   which it already is here.
- effect Option,
   fp-ts `Option`,
   neverthrow,
   ts-belt
  (<https://effect.website/docs/additional-resources/effect-vs-neverthrow/> and
  <https://npm-compare.com/fp-ts,neverthrow,ts-results,ts-toolbelt>).
   These encode absence as a first-class
  value rather than enforcing its absence.
   As argued above,
   that is the opposite of this repo's stance at
  `AGENTS.md:412`,
   and adopting one would convert "no escape hatch" into "one blessed escape hatch".

Net:
 the type-aware ecosystem catches the coercion and assertion sites,
 never the disguised-sentinel
intent;
 the functional ecosystem reifies absence rather than banning it.
 Neither closes the lower rungs
for free,
 which is why the remedy here is type-position lint plus boundary constructors plus instructions.

## Ranked recommendations

Ordered by value-to-effort,
 each with the next concrete step.

1.    Ship `no-optional-escape` and the two `no-disable-*` companions.
      Highest value:
       it is the only item that adds new static coverage (the `T | void` and tuple-type
      rungs),
       and the companions close the disable-comment hole that currently reopens the whole ladder.
      The pattern is already established,
       so effort is low.
       Next steps:
       add
      `src/rule/no-optional-escape.ts` matching `TSVoidKeyword` as a `TSUnionType` member,
       empty
      `TSTupleType`,
       and `TSOptionalType` in a tuple,
       with the `void`-in-union scope note above;
       add
      `no-disable-no-optional-escape.ts` and `no-disable-no-nullish-union.ts` from `banDisableRule`;
      register all three in `index.ts`;
       extend the unit test with valid and invalid fixtures the way
      `no-nullish-union` is tested.
2.    Add the "Modeling optional and absent values" subsection to `AGENTS.md` under "Type system".
      Closes the lower rungs that no rule can reach,
       by naming them as banned in the place agents read
      first,
       and by routing object construction to `omitUndefined`.
       Low effort,
       no code.
       Next step:
       insert
      the proposed block after `AGENTS.md:392`;
       regenerate `CLAUDE.md` via file-enforcer.
3.    Promote `omitUndefined` to a shared module and add `requireNonEmpty` / `requirePositive` boundary
      constructors next to `nonNullishOrThrow`.
      Gives the undetectable `''` / `0` rungs a shorter,
       in-philosophy alternative,
       so the instruction in
      recommendation 2 has somewhere to point.
       Medium effort.
       Next step:
       move `omitUndefined` into a
      shared `@monochromatic-dev/module-*` package with its own README and tests;
       add the two throwing
      constructors to `@monochromatic-dev/or-throw-core`,
       each with the `NonEmpty<S>` template-literal
      brand from the type-system section.

Explicitly not recommended:
 introducing an `Option`/`Maybe`/`Result` type.
 It contradicts `AGENTS.md:412`
and would itself become the next escape hatch.
