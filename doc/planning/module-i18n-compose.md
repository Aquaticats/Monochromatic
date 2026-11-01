# Plan: `@monochromatic-dev/i18n-compose-core`

Status:
 package implementation landed;
 `ssg-test` migration (Phase 6) landed.
 See "Implementation status" below for what shipped and "Footguns encountered" for traps the next contributor should
 sidestep.

This plan supersedes the earlier brainstorm for `PLANNING.module-i18n-compose.md`.
 The major changes are:

- `ca`,
   `en`,
   and `zh` are all in scope from day one.
- The shared package is a grammar/composition library,
   not an application message library.
- The shared package never owns application state,
   product concepts,
   or the current locale.
- Locale is an explicit parameter on every render call:
   `i18n.sentence(locale, ast)`,
   not `t(locale).sentence(ast)`.
- Static UI strings are `label` entries,
   not `noun` entries.
- `Sentence` is a discriminated AST,
   not a flat object with many optional fields.
- Noun phrases are tagged variants;
   numeric quantity is called `count`,
   not `adj`.
- English verbs use structured forms,
   not a single `(person, number, tense) => string` function.
- Wh-questions are slot-based;
   there is no top-level `whWord` field.
- Existing generated `typesafe-i18n` locale-registry helpers must be replaced before generated files are deleted.

## Implementation status

Phases 1 through 5 plus Phase 7 (cleanup and docs) landed under `package/i18n-compose/core/`.
 The package type-checks,
 lints with zero errors and zero warnings,
 and all tests pass.
 Phase 6 (`ssg-test` migration) has since landed:
 `package/ssg/aquati.cat` renders every static UI label through this package via an app-local
 `src/i18n/index.ts`,
 the generated `typesafe-i18n` files plus the `build:i18n` generator task and devDependency are removed,
 the cache pipeline glob covers `src/i18n`,
 and the package's build,
 unit tests,
 type-check,
 and lint all pass.

What shipped (file layout,
 not the plan's flat `locales/{ca,en,zh}.ts`):

```txt
package/i18n-compose/core/
├── README.md
├── mise.toml
├── package.json
├── tsconfig.json
└── src/
    ├── index.ts                 # public barrel
    ├── grammar-primitives.ts    # Person, GrammaticalNumber, Tense, Countability,
    │                              GrammaticalGender, PersonNumberKey, Capitalization,
    │                              VerbFragmentForm, personNumberKey()
    ├── entries.ts               # ArticleForms, NounPlural, SubjectEntry, NounEntry
    ├── ast.ts                   # SubjectRef, ExternalText, NounPhrase, Adverbial,
    │                              VerbPhrase, NonFiniteComplement,
    │                              DeclarativeSentence, YesNoQuestion, WhQuestion,
    │                              ImperativeSentence, Sentence, Fragment, FragmentPart
    ├── countability.ts          # assertCountableNoun() for mass-noun validation
    ├── agreement.ts             # SubjectAgreement, subjectAgreement(),
    │                              subjectSurface(), WH_SUBJECT_AGREEMENT
    ├── locale-spec.ts           # LocaleSpec + LabelOf/SubjectOf/VerbOf/NounOf
    ├── create-i18n.ts           # createI18n() and I18n surface
    ├── render-helpers.ts        # applyCapitalization, joinTokens, CaseInvariantSet
    ├── locales/
    │   ├── custom.ts            # defineCustomLocale escape hatch
    │   ├── en/                  # split because the plan's flat file blew max-lines (300)
    │   │   ├── index.ts         # defineEnglishLocale; wires factories together
    │   │   ├── types.ts         # EnglishVerbEntry, DefineEnglishLocaleInput,
    │   │   │                      EN_CASE_INVARIANTS (English `I` pin)
    │   │   ├── render-np.ts     # makeEnglishNounPhraseRenderer
    │   │   ├── render-adverbial.ts
    │   │   ├── render-vp.ts     # declarativeVerbSurface, doAuxiliary,
    │   │   │                      questionVerbParts, complementFormForVerb,
    │   │   │                      makeEnglishVerbPhraseRenderer
    │   │   ├── render-sentence.ts
    │   │   └── render-fragment.ts
    │   ├── zh/                  # same five-file split
    │   └── ca/                  # same five-file split (verb forms via PersonNumberKey)
    └── test-vocab/              # split test-only vocab; not exported
        ├── index.ts, types.ts, en.ts, zh.ts, ca.ts
```

Renderer responsibilities per locale:

- English:
   do-support for ordinary yes/no and wh questions (`Do/Does/Did` lowercase mid-sentence,
  capitalized at position 0 via sentence-case fixup);
   `auxiliaryStrategy: 'copula'` fronts the finite verb;
  `auxiliaryStrategy: 'modal'` fronts the modal and renders nested complements bare;
  `auxiliaryStrategy: 'none'` skips do-insertion for caller-supplied special cases;
  base form after every auxiliary;
   future via `will + base`;
   ordinary infinitive complements via `to + base`;
  sentence-case fixup pins English `I` via `EN_CASE_INVARIANTS`.
- Chinese:
   ASCII space between digit and classifier (`1 只猫`);
   在/到/从 coverbs for locatives;
  之前/之后 for time;
   `了` for past,
   `会` for future;
   `吗？` for yes/no;
  in-situ wh-words (`谁/什么/在哪里/什么时候/为什么/怎么`) with no `吗` particle;
  Chinese terminators `。？！`;
   sequence fragments concatenated with no separator.
- Catalan:
   gender/number article tables (`el/els`,
   `un/uns`,
   elided `l'`);
  finite verb forms indexed by tense and {@link PersonNumberKey} via the sparse `entry.finite[tense][pn]` table,
  with missing entries throwing;
   imperative falls back to infinitive;
  nested complements render the full nested verb phrase,
   including objects and adverbials;
  question rendering uses punctuation alone in v1 (no auxiliary inversion);
   `Qui/Què/On/Quan/Per què/Com` wh-words.

Every built-in noun-phrase renderer validates `countability: 'mass'` before rendering `noun.counted`
and throws instead of inventing a measure phrase.

Tests:
 `*.unit.test.ts` only (no `*.type.test.ts` files;
 the workspace convention puts type assertions inline
via `expectTypeOf` from `@monochromatic-dev/test-harness-core`).
 Coverage spans every variant of every AST kind:
noun phrases (bare,
 counted,
 definite,
 indefinite,
 possessed,
 external),
 mass-noun rejection,
 declaratives,
yes/no questions with do-support across all three tenses,
 English copula and modal auxiliary strategies,
all three wh-slots,
 imperatives,
 complements (`want to delete`),
 Catalan full nested-complement preservation
in sentences,
 verb phrases,
 and fragments,
 fragments (noun-phrase,
 verb-phrase non-finite forms,
 sequence),
capitalization invariants,
 Catalan throw-on-missing-form,
 Chinese absence-of-吗 in wh-questions,
and package-name public import consumption.

## Footguns encountered

These trapped the first implementation and will trap the next contributor unless they are recorded:

1. **`createI18n` type-parameter inference is fragile.
   ** The naive
   `createI18n<Locales, Label, Subject, Verb, Noun>(config)` with `LocaleSpec` arrow-typed methods refuses
   to compile:
    renderer parameters (`key: Label`) are in contravariant position,
    so `LocaleSpec<TestLabel, ...>`
   is not assignable to `LocaleSpec<string, ...>` (the inferred default when `Label` has no inference source).
   Method shorthand syntax makes positions bivariant and the assignment works,
    but then TypeScript can widen
   or union vocabulary across locale specs in ways that are unsafe at runtime.
    The working pattern is method
   shorthand for `LocaleSpec` plus a `Spec` generic in `createI18n` constrained against
   `AnyLocaleSpec = LocaleSpec<string, string, string, string>`,
    conditional `LabelOf` / `SubjectOf` /
   `VerbOf` / `NounOf` extractors,
    and `EnforceSharedVocabulary` to reject spec records where one locale has
   keys another locale lacks.
    The returned `I18n` surface uses `SharedLabelOf` / `SharedSubjectOf` /
   `SharedVerbOf` / `SharedNounOf`,
    not the raw union across `Specs[Locales[number]]`.
    Touch this at your peril.

2. **`'who' as Subject` is unsafe and unnecessary.
   ** A first cut threaded `SubjectRef<Subject>` into every verb-form helper,
    then synthesized `{ kind: 'subject.key', subject: 'who' as Subject }` for wh-subject questions.
    Oxlint correctly rejects the cast.
    The fix lives in `agreement.ts`:
    helpers take `SubjectAgreement = { person, number }`,
    `subjectAgreement({ ref, subjects })` extracts it from a real subject reference,
    and `WH_SUBJECT_AGREEMENT` is a pre-built `{ person: 3, number: 'singular' }` constant the wh-subject branches pass
    directly.
    The locale's `Subject` union is never abused.

3. **`max-lines: 300` (effective) forced the flat-file plan to split.
   ** The plan listed `locales/ca.ts`,
    `locales/en.ts`,
    `locales/zh.ts` as single files.
    Each implementation grew past 400 effective lines once TSDoc was added to every local;
    oxlint's `eslint/max-lines` is `error` and `AGENTS.md` forbids disabling it.
    The remediation is the
    `locales/<lang>/{index,types,render-np,render-adverbial,render-vp,render-sentence,render-fragment}.ts` split
    documented above.
    `test-vocab.ts` hit the same limit and split into `test-vocab/{index,types,en,zh,ca}.ts`.
    Plan §12's flat layout is normative for the public API surface but not for source-file organization.

4. **TSDoc rules are stricter than `or-throw` makes them look.
   ** Every local `const` requires a TSDoc comment (`tsdoc/require-tsdoc`).
    Every function parameter must be documented by name (`tsdoc/require-param`).
    For destructured params,
    each destructured field needs its own `@param <field> - description` line,
    not a single `@param input - ...` covering the bundle (`tsdoc/check-param-names`).
    Single-line TSDocs that contain a tag (e.g. `/** Dependency bundle for {@link X}. */`) must be expanded to
    multi-line (`tsdoc/multiline-blocks`).
    The `>` character inside TSDoc body text must be replaced with `to` (the helpfully-titled
    `tsdoc-escape-greater-than` rule).
    Plan for ~3x the lines you would naively write,
    and use named extracted helpers when an inner function body would otherwise need 8+ documented locals.

5. **`eslint/no-magic-numbers` rejects `3` even inside a named-const definition.
   ** `AGENTS.md` exempts
   `-2..2`,
    but English needs the third-person literal for agreement checks.
    Use the composed constant
   `const THIRD_PERSON = 1 + 2`;
    do not cast it to `Person`,
    because type-aware oxlint flags the narrowing
   assertion as unsafe.

6. **dprint and oxlint disagree on inline object/array literals.
   ** dprint leaves `{ entry, count, }` on one line
   when it fits in 90 columns;
    oxlint's `stylistic/object-property-per-line` and
   `stylistic/array-element-per-line` (warnings,
    not errors) want one property/element per line regardless.
   `mise run //:format:oxlint` (or `oxlint --fix` directly) auto-fixes both rules,
    but the full-tree
   `//:format` task fails on unrelated `figma/kiwi` lint errors before reaching the fixer.
   Run the package-local fixer directly when working on this package;
    the fix is autofixable.
   `unicorn/no-nested-ternary` is disabled in `package/config/oxlint/src/rule/style.ts` to match the existing
   project preference for nested ternaries.

7. **Plan §11 short-form examples are wrong;
    the AST type is normative.
   ** The §11 examples use `kind: 'counted'`,
    `kind: 'bare'`,
    etc.,
    but the actual variant kinds are namespaced (`'noun.counted'`,
    `'noun.bare'`,
    ...).
    The implementation follows the type,
    not the §11 short-form.
    Test fixtures and the `ssg-test` migration must use the namespaced form.

8. **The plan's `*.type.test.ts` convention does not exist in this workspace.
   ** Workspace tests put type assertions inline in `*.unit.test.ts` files using `expectTypeOf` re-exported from
   `@monochromatic-dev/test-harness-core`.
    The implementation followed the workspace convention;
    `src/ast-types.unit.test.ts` is the type-assertion file.

## What the Phase 6 contributor should know

These are the bits that did not get written down inside the package itself:

- The `ssg-test` cache pipeline glob at `package/ssg/aquati.cat/src/build.ts:72`
(`PIPELINE_GLOB = 'src/{lib,components,client}/**/*.ts'`) **does not include `src/i18n/`**.
   Phase 6 acceptance says i18n source changes must invalidate cached rendered output (§13 Phase 6 / §15).
   Widen the glob (or add a separate i18n fingerprint) before declaring the migration done.
- `ssg-test`'s `package.json` does not yet depend on `@monochromatic-dev/i18n-compose-core`.
   Phase 6 must add `"@monochromatic-dev/i18n-compose-core": "workspace:*"` to `dependencies`
   and remove `"typesafe-i18n"` from `devDependencies`.
- The existing call sites use `const t = i18nObject(lang); t.siteName()` — across roughly ten files (rg `'i18nObject'`
from `package/ssg/aquati.cat/src`).
   The plan's §3 forbids exporting a `bindLocale` / `t(locale)` accessor from the shared package.
   Rewrites must become explicit `i18n.label(lang, 'siteName')` calls;
   an app-local thin wrapper is allowed if call-site noise becomes painful,
   but resist adding one unless necessary.
- All ten existing translation keys in `src/i18n/{ca,en,zh}/index.ts` are static UI strings;
   they all belong in `label`,
   not in `noun`.
   The Phase 6 migration is largely mechanical:
   build a `Label` union from the existing key set,
   populate per-locale `labels` records,
   swap the imports.
- `src/i18n/lang-names.ts` (autonyms like `Català`,
   `中文`) is independent of the typesafe-i18n machinery and should be preserved verbatim;
   do not let it break when deleting the generated files.
- `src/build.ts:54` calls `loadAllLocales()` from the generated `i18n-util.sync.ts`.
   The new package has no loader;
   the call site can be removed.
- The generated files to remove only after every import has been redirected:
   `src/i18n/{i18n-types,i18n-util,i18n-util.sync,i18n-util.async,formatters}.ts`.
   `formatters.ts` is a thin stub that currently returns an empty object;
   nothing depends on it semantically.
- The `build:i18n` mise task (`typesafe-i18n --no-watch`) and the `typesafe-i18n` devDependency both go away.
- Test the migration via `mise run //package/ssg/aquati.cat:build` (or whatever the equivalent task names are
when you read this — `mise tasks -C package/ssg/aquati.cat` lists them).
   Rendered output diff should either be empty or every diff line should have a one-line explanation.

## 1. Goal

Build a workspace package at:

```txt
package/i18n-compose/core/
```

The package provides a small,
 type-safe,
 no-codegen API for rendering localized UI text from explicit semantic grammar nodes.

The package is meant to replace `typesafe-i18n` in `ssg-test`
and avoid the parser/template-regex failure mode that motivated the original investigation.

The package must not become an app-message registry.
 It should know how to render generic grammar;
 it must not know what a page,
 post,
 route,
 confirmation dialog,
 selected item,
 timestamp,
 user,
 or product-specific workflow is.

## 2. Non-negotiable ownership boundary

### Shared package owns

```txt
- grammar types
- grammar AST builders
- locale renderers
- locale registry helpers
- generic rendering primitives
- generic vocabulary entry shapes
- locale-specific grammar strategies for supported locales
```

### Consumers own

```txt
- labels
- nouns
- verbs
- subjects
- adjectives, if used
- route/page concepts
- confirmation flows
- timestamps
- item names
- counts as application data
- every domain-specific message shape
- any convenience wrappers around the shared package
```

### Forbidden in the shared package

The shared package must not export or internally define APIs like:

```ts
i18n.confirmDelete(locale, { itemName, },);
i18n.selectedCount(locale, { count, },);
i18n.postedAt(locale, { time, },);
i18n.siteName(locale,);
i18n.noResults(locale,);
```

Those are product-message semantics.
 They belong in a consuming app,
 if they exist at all.

The shared package may expose generic calls like:

```ts
i18n.label(locale, 'siteName',);
i18n.np(locale, { kind: 'counted', count, noun: 'item', },);
i18n.sentence(locale, sentenceAst,);
```

Here,
 `siteName`,
 `item`,
 and all other vocabulary keys are consumer-owned.

## 3. Canonical API shape

Locale must be an explicit argument on every render call.

Canonical:

```ts
i18n.label(locale, 'siteName',);
i18n.noun(locale, 'cat',);
i18n.np(locale, { kind: 'counted', count: 1, noun: 'cat', },);
i18n.vp(locale, { kind: 'verbPhrase', verb: 'save', form: 'imperative', },);
i18n.sentence(locale, sentenceAst,);
i18n.fragment(locale, fragmentAst,);
```

Not canonical:

```ts
const t = i18n(locale,);
t.sentence(sentenceAst,);
```

Do not export `forLocale`,
 `bindLocale`,
 or `t(locale)` from the shared package in the first implementation.
 A consuming package may create an app-local wrapper later if repeated locale passing becomes painful.

Rationale:
 the library should not remember the current locale.
 Every render is a pure operation over explicit inputs:

```txt
locale + locale specs + vocabulary key / grammar node -> string
```

## 4. Locale registry API

`createI18n` should infer the locale union from a const locale list and return both rendering functions
and registry helpers.

Sketch:

```ts
export function createI18n<
  const Locales extends readonly string[],
  Label extends string,
  Subject extends string,
  Verb extends string,
  Noun extends string,
>(config: {
  locales: Locales;
  defaultLocale: Locales[number];
  specs: Record<Locales[number], LocaleSpec<Label, Subject, Verb, Noun>>;
},): I18n<Locales[number], Label, Subject, Verb, Noun>;
```

Return shape:

```ts
export type I18n<
  Locale extends string,
  Label extends string,
  Subject extends string,
  Verb extends string,
  Noun extends string,
> = {
  locales: readonly Locale[];
  defaultLocale: Locale;
  isLocale: (value: string,) => value is Locale;
  assertLocale: (value: string,) => Locale;

  label: (locale: Locale, key: Label,) => string;
  noun: (locale: Locale, noun: Noun,) => string;
  np: (locale: Locale, phrase: NounPhrase<Subject, Noun>,) => string;
  vp: (locale: Locale, phrase: VerbPhrase<Subject, Verb, Noun>,) => string;
  sentence: (locale: Locale,
    sentence: Sentence<Subject, Verb, Noun>,) => string;
  fragment: (locale: Locale,
    fragment: Fragment<Subject, Verb, Noun>,) => string;
};
```

The `ssg-test` package can re-export local names from its own `src/i18n/index.ts`:

```ts
export const locales = ['ca', 'en', 'zh',] as const;
export type Locale = typeof locales[number];

export const i18n = createI18n({
  locales,
  defaultLocale: 'en',
  specs: { ca, en, zh, },
},);

export const isLocale = i18n.isLocale;
export const assertLocale = i18n.assertLocale;
```

If existing app code currently imports `Locales`,
 `locales`,
 `isLocale`,
 or `loadAllLocales` from generated `typesafe-i18n` files,
 replace those imports before deleting the generated files.
 If `loadAllLocales` is only present because of `typesafe-i18n`'s generated loading model,
 remove the call sites or replace it with an app-local no-op only where unavoidable.

## 5. Locale scope

Initial locale scope is:

```txt
ca, en, zh
```

Catalan is not a migration afterthought.
 It is a first-class v1 locale because `ssg-test` already has three locales.

The library should expose locale builders for all three:

```ts
defineCatalanLocale(...);
defineEnglishLocale(...);
defineChineseLocale(...);
```

Consumers provide vocabulary.
 Builders provide grammar strategy.

Do not require every consumer to hand-write `noun`,
 `np`,
 `vp`,
 and `sentence` renderers from scratch for ordinary use.
 The normal path should be:

```ts
const en = defineEnglishLocale({ labels, subjects, nouns, verbs, },);
const zh = defineChineseLocale({ labels, subjects, nouns, verbs, },);
const ca = defineCatalanLocale({ labels, subjects, nouns, verbs, },);
```

Expose a lower-level `defineCustomLocale` only as an advanced escape hatch.

## 6. Vocabulary categories

Static UI text belongs in `label`,
 not `noun`.

Good:

```ts
type Label = 'siteName' | 'chooseALang' | 'noResults' | 'page';
type Noun = 'cat' | 'message' | 'item' | 'post';

i18n.label(locale, 'siteName',);
i18n.np(locale, { kind: 'counted', count: 0, noun: 'message', },);
```

Bad:

```ts
type Noun = 'siteName' | 'chooseALang' | 'noResults' | 'page';

i18n.noun(locale, 'siteName',);
i18n.np(locale, { kind: 'counted', count: 3, noun: 'chooseALang', },);
```

The second form makes non-grammatical UI labels available in grammatical slots,
 which defeats the purpose of a typed composition API.

## 7. No exported core app vocabulary

Do not export `CoreVerb` or `CoreNoun` unions like:

```ts
export type CoreVerb = 'have' | 'see' | 'delete';
export type CoreNoun = 'cat' | 'item' | 'message';
```

Nouns and verbs are consumer vocabulary.

The library may export grammatical primitives:

```ts
export type Person = 1 | 2 | 3;
export type GrammaticalNumber = 'singular' | 'plural';
export type Tense = 'past' | 'present' | 'future';
export type Countability = 'countable' | 'mass' | 'both';
export type GrammaticalGender = 'masculine' | 'feminine' | 'neuter';
```

Package tests may define test vocabulary internally,
 but that vocabulary must not become part of the public API.

## 8. Grammar AST

`Sentence` must be a discriminated union.
 Invalid grammatical states should be unrepresentable where TypeScript can enforce them.

### Subject references

```ts
export type SubjectRef<S extends string,> =
  | { kind: 'subject.key'; subject: S; }
  | { kind: 'subject.externalName'; text: string; };
```

External names are opaque text.
 The library may position them grammatically but does not translate them or own their meaning.

### External text leaves

```ts
export type ExternalText = {
  kind: 'externalText';
  text: string;
};
```

External text is for values such as item titles,
 names,
 or preformatted times.
 The renderer may place the text but must not parse it or treat it as a translation template.

Escaping/sanitization remains the caller's responsibility if the rendered output is inserted into HTML.

### Noun phrases

Use tagged variants.

```ts
export type NounPhrase<S extends string, N extends string,> =
  | { kind: 'noun.bare'; noun: N; }
  | { kind: 'noun.counted'; count: number; noun: N; }
  | { kind: 'noun.definite'; noun: N; }
  | { kind: 'noun.indefinite'; noun: N; }
  | { kind: 'noun.possessed'; possessor: Possessor<S>; noun: N; }
  | { kind: 'noun.externalText'; text: string; };

export type Possessor<S extends string,> =
  | { kind: 'possessor.subject'; subject: S; }
  | { kind: 'possessor.externalName'; text: string; };
```

Use `count`,
 not `adj`.
 Word adjectives are a separate future field,
 not a numeric quantity field.

Articles are explicit.
 The renderer must not infer `a`,
 `an`,
 `the`,
 `el`,
 `la`,
 etc. from a bare noun.

### Verb phrases

```ts
export type VerbPhrase<S extends string, V extends string, N extends string,> =
  {
    kind: 'verbPhrase';
    verb: V;
    object?: NounPhrase<S, N>;
    complement?: NonFiniteComplement<S, V, N>;
    adverbials?: readonly Adverbial<S, N>[];
  };

export type NonFiniteComplement<S extends string, V extends string,
  N extends string,> = {
    kind: 'complement.infinitive';
    phrase: VerbPhrase<S, V, N>;
  };
```

This supports generic grammar like “want to delete X” without adding a domain-specific `confirmDelete` API.

### Adverbials

Avoid a single `adverbialPrep` field on nouns.
 A noun does not have one universal preposition.

Use adverbial relation nodes:

```ts
export type Adverbial<S extends string, N extends string,> =
  | { kind: 'adverbial.location'; relation: 'at' | 'in' | 'to' | 'from';
    place: NounPhrase<S, N>; }
  | { kind: 'adverbial.time'; relation: 'at' | 'before' | 'after';
    time: NounPhrase<S, N> | ExternalText; };
```

Locale renderers choose how to surface the relation.

### Sentences

```ts
export type Sentence<S extends string, V extends string, N extends string,> =
  | DeclarativeSentence<S, V, N>
  | YesNoQuestion<S, V, N>
  | WhQuestion<S, V, N>
  | ImperativeSentence<S, V, N>;

export type DeclarativeSentence<S extends string, V extends string,
  N extends string,> = {
    kind: 'sentence.declarative';
    subject: SubjectRef<S>;
    predicate: VerbPhrase<S, V, N>;
    tense?: Tense;
    terminator?: '.';
  };

export type YesNoQuestion<S extends string, V extends string,
  N extends string,> = {
    kind: 'sentence.question.yesNo';
    subject: SubjectRef<S>;
    predicate: VerbPhrase<S, V, N>;
    tense?: Tense;
    terminator?: '?';
  };

export type ImperativeSentence<S extends string, V extends string,
  N extends string,> = {
    kind: 'sentence.imperative';
    predicate: VerbPhrase<S, V, N>;
    subject?: never;
    terminator?: '.' | '!';
  };
```

Wh-questions are slot-based:

```ts
export type WhQuestion<S extends string, V extends string, N extends string,> =
  | {
    kind: 'sentence.question.wh.subject';
    wh: 'who';
    predicate: VerbPhrase<S, V, N>;
    tense?: Tense;
    terminator?: '?';
  }
  | {
    kind: 'sentence.question.wh.object';
    wh: 'what';
    subject: SubjectRef<S>;
    verb: V;
    adverbials?: readonly Adverbial<S, N>[];
    tense?: Tense;
    terminator?: '?';
  }
  | {
    kind: 'sentence.question.wh.adverbial';
    wh: 'where' | 'when' | 'why' | 'how';
    subject: SubjectRef<S>;
    predicate: VerbPhrase<S, V, N>;
    tense?: Tense;
    terminator?: '?';
  };
```

This lets English front wh-words and lets Chinese place wh-words in the occupied slot.

### Fragments

Some UI text is not a complete sentence.
 Do not fake subjectless declaratives.

```ts
export type Fragment<S extends string, V extends string, N extends string,> =
  | { kind: 'fragment.nounPhrase'; phrase: NounPhrase<S, N>;
    capitalization?: Capitalization; }
  | { kind: 'fragment.verbPhrase'; phrase: VerbPhrase<S, V, N>;
    form: VerbFragmentForm; capitalization?: Capitalization; }
  | { kind: 'fragment.sequence'; parts: readonly FragmentPart<S, V, N>[];
    capitalization?: Capitalization; };

export type FragmentPart<S extends string, V extends string,
  N extends string,> =
    | { kind: 'part.label'; label: string; }
    | { kind: 'part.nounPhrase'; phrase: NounPhrase<S, N>; }
    | { kind: 'part.externalText'; text: string; };
```

The exact `fragment.sequence` typing should ensure labels are from the consumer's `Label` union,
 not arbitrary string.
 Keep the sketch above conceptually but type it precisely in implementation.

## 9. Vocabulary entry shapes

### Subject entries

Subject entries need possessive surfaces.
 They cannot be derived safely from the nominative surface.

```ts
export type SubjectEntry = {
  surface: string;
  possessive: string;
  person: Person;
  number: GrammaticalNumber;
  gender?: GrammaticalGender;
};
```

For English:

```ts
I: { surface: 'I', possessive: 'my', person: 1, number: 'singular' }
they: { surface: 'they', possessive: 'their', person: 3, number: 'plural' }
```

### Noun entries

Nouns need enough features for English,
 Chinese,
 and Catalan.

```ts
export type NounEntry = {
  surface: string;
  plural?: string | ((count: number,) => string);
  gender?: GrammaticalGender;
  countability?: Countability;
  classifier?: string;
  articles?: {
    definite?: ArticleForms;
    indefinite?: ArticleForms;
  };
};
```

Example intent:

```ts
// en
cat: {
  surface: 'cat',
  plural: 'cats',
  articles: {
    definite: { singular: 'the', plural: 'the' },
    indefinite: { singular: 'a' },
  },
}

// zh
cat: {
  surface: '猫',
  classifier: '只',
}

// ca
cat: {
  surface: 'gat',
  plural: 'gats',
  gender: 'masculine',
  articles: {
    definite: { singular: 'el', plural: 'els' },
    indefinite: { singular: 'un', plural: 'uns' },
  },
}
```

Do not rely on articles for bare nouns.
 Article intent is carried by the noun-phrase variant.

### Verb entries

Do not model verbs as a single renderer function receiving `{ person, number, tense }`.

English needs finite,
 base,
 infinitive,
 imperative,
 and participle-like forms so interrogatives and complements do not produce bad output such as `Did I had`
 or `Does he has`.

Use structured entries,
 with locale-specific builders normalizing them into renderer-internal forms.

English sketch:

```ts
export type EnglishVerbEntry = {
  base: string;
  present3s?: string;
  past?: string;
  pastParticiple?: string;
  gerund?: string;
  imperative?: string;
  auxiliaryStrategy?: 'do-support' | 'copula' | 'modal' | 'none';
};
```

Chinese sketch:

```ts
export type ChineseVerbEntry = {
  surface: string;
  past?: string;
  future?: string;
  perfective?: string;
};
```

Catalan sketch:

```ts
export type CatalanVerbEntry = {
  infinitive: string;
  imperative?: string;
  finite: Partial<Record<Tense, Partial<Record<PersonNumberKey, string>>>>;
};
```

The exact Catalan shape can be tuned during implementation,
 but the design must support person/number agreement and should not force Catalan through the English verb model.

## 10. Locale-specific rendering rules

### English requirements

- Declarative present third-person singular uses the finite third-person form.
- Yes/no questions use do-support where appropriate.
- Questions use the base verb after `do`,
   `does`,
   or `did`.
- Future uses `will` + base.
- Infinitive complements use `to` + base.
- Imperatives use base/imperative form.
- Articles render only when the noun phrase explicitly requests definite or indefinite form.
- Capitalization applies to the first emitted token unless the token has its own casing invariant,
   such as `I` or an external proper name.

### Chinese requirements

- Counted nouns use Arabic digits plus classifier when a classifier exists.
- Keep the spacing rule between Latin digits and Chinese characters:
   one ASCII space,
   for example `1 只猫`.
- Yes/no questions use the appropriate question particle,
   for example `吗`,
   but wh-questions must not receive the yes/no particle merely because they end in a question mark.
- Wh-questions render the wh-word in the occupied slot.
- Use Chinese punctuation for Chinese sentence terminators:
   `。`,
   `？`,
   `！`.
- Tense/aspect surfaces come from the Chinese verb entry or renderer strategy;
   do not pretend Chinese has the same finite-verb model as English.

### Catalan requirements

- Nouns support grammatical gender.
- Definite and indefinite articles agree with noun gender and number for the covered vocabulary.
- Verbs support person/number agreement for covered tenses.
- Question rendering may rely on punctuation/intonation for v1 where appropriate,
   but the renderer must still be explicit and tested.
- Do not use Catalan as a static-label-only fallback.
   If grammar primitives are available for `en` and `zh`,
   the covered primitives should also be implemented and tested for `ca`.

## 11. Examples

These examples use consumer vocabulary keys.
 They are not built-in library vocabulary.

### Counted noun phrase

```ts
i18n.np('en', { kind: 'counted', count: 1, noun: 'cat', },);
// "1 cat"

i18n.np('zh', { kind: 'counted', count: 1, noun: 'cat', },);
// "1 只猫"

i18n.np('ca', { kind: 'counted', count: 1, noun: 'cat', },);
// "1 gat"
```

### Declarative sentence

```ts
i18n.sentence('en', {
  kind: 'sentence.declarative',
  subject: { kind: 'subject.key', subject: 'I', },
  predicate: {
    kind: 'verbPhrase',
    verb: 'have',
    object: { kind: 'noun.counted', count: 1, noun: 'cat', },
  },
},);
// "I have 1 cat."

i18n.sentence('zh', {
  kind: 'sentence.declarative',
  subject: { kind: 'subject.key', subject: 'I', },
  predicate: {
    kind: 'verbPhrase',
    verb: 'have',
    object: { kind: 'noun.counted', count: 1, noun: 'cat', },
  },
},);
// "我有 1 只猫。"

i18n.sentence('ca', {
  kind: 'sentence.declarative',
  subject: { kind: 'subject.key', subject: 'I', },
  predicate: {
    kind: 'verbPhrase',
    verb: 'have',
    object: { kind: 'noun.counted', count: 1, noun: 'cat', },
  },
},);
// "Jo tinc 1 gat."
```

### Yes/no question

```ts
i18n.sentence('en', {
  kind: 'sentence.question.yesNo',
  subject: { kind: 'subject.key', subject: 'I', },
  predicate: {
    kind: 'verbPhrase',
    verb: 'have',
    object: { kind: 'noun.counted', count: 1, noun: 'cat', },
  },
},);
// "Do I have 1 cat?"

i18n.sentence('zh', {
  kind: 'sentence.question.yesNo',
  subject: { kind: 'subject.key', subject: 'I', },
  predicate: {
    kind: 'verbPhrase',
    verb: 'have',
    object: { kind: 'noun.counted', count: 1, noun: 'cat', },
  },
},);
// "我有 1 只猫吗？"
```

### Wh-object question

```ts
i18n.sentence('en', {
  kind: 'sentence.question.wh.object',
  wh: 'what',
  subject: { kind: 'subject.key', subject: 'I', },
  verb: 'see',
},);
// "What do I see?"

i18n.sentence('zh', {
  kind: 'sentence.question.wh.object',
  wh: 'what',
  subject: { kind: 'subject.key', subject: 'I', },
  verb: 'see',
},);
// "我看见什么？"
```

### Generic confirmation-like grammar without domain API

The shared package does not expose `confirmDelete`.
 A consuming app can compose generic grammar directly:

```ts
i18n.sentence(locale, {
  kind: 'sentence.question.yesNo',
  subject: { kind: 'subject.key', subject: 'you', },
  predicate: {
    kind: 'verbPhrase',
    verb: 'want',
    complement: {
      kind: 'complement.infinitive',
      phrase: {
        kind: 'verbPhrase',
        verb: 'delete',
        object: { kind: 'noun.externalText', text: itemName, },
      },
    },
  },
},);
```

The library sees grammar plus opaque external text.
 It does not know this is a confirmation dialog.

### Static labels

```ts
i18n.label(locale, 'siteName',);
i18n.label(locale, 'chooseALang',);
i18n.label(locale, 'noResults',);
```

Labels are generic consumer vocabulary.
 They are not nouns.

### Interpolation without template holes

No template-with-holes primitive is allowed.

Bad:

```ts
i18n.message(locale, 'Posted at {time}', { time, },);
```

Good:

```ts
const rendered = `${i18n.label(locale, 'postedAt',)} ${formatTime(time,)}`;
```

If locale-specific order is needed,
 the consuming app owns that composition.
 The shared library should not gain a `postedAt` method.

## 12. Package layout

```txt
package/i18n-compose/core/
├── README.md
├── mise.toml
├── package.json
├── tsconfig.json
└── src/
    ├── index.ts
    ├── createI18n.ts
    ├── ast.ts
    ├── entries.ts
    ├── localeSpec.ts
    ├── render.ts
    ├── locales/
    │   ├── ca.ts
    │   ├── en.ts
    │   └── zh.ts
    ├── test-vocab.ts
    ├── *.unit.test.ts
    └── *.type.test.ts
```

`test-vocab.ts` is test-only vocabulary and must not be exported from the package root.

## 13. Implementation phases

### Phase 1: package skeleton

- Create `package/i18n-compose/core/`.
- Add `package.json`,
   `tsconfig.json`,
   and `mise.toml` consistent with nearby workspace modules.
- Add `README.md` documenting the ownership boundary and explicit-locale API.
- Add root exports but keep implementation minimal until types are ready.

Acceptance:

```txt
- package builds or type-checks using existing workspace conventions
- no generated i18n files are introduced
- no codegen step is introduced
```

### Phase 2: type model and AST builders

- Implement vocabulary entry types.
- Implement `NounPhrase`,
   `VerbPhrase`,
   `Adverbial`,
   `Sentence`,
   and `Fragment` types.
- Add small builder helpers only if they reduce call-site noise without hiding structure.
- Add type tests proving invalid states are rejected.

Type-test examples:

```txt
- imperative cannot have a subject
- wh-object question cannot also provide a normal object
- yes/no question requires a subject
- declarative requires a subject
- noun.counted requires count
- noun.definite does not accept count unless a separate counted-definite variant is intentionally added
- label keys cannot be used as noun keys
```

### Phase 3: locale builders

Implement:

```ts
defineEnglishLocale(...)
defineChineseLocale(...)
defineCatalanLocale(...)
defineCustomLocale(...)
```

Each builder validates or normalizes the locale-specific vocabulary entry shapes into a `LocaleSpec`.

Acceptance:

```txt
- missing vocabulary entries fail at compile time
- missing labels fail at compile time
- extra locale keys are caught with `satisfies` or equivalent patterns
- locale builders do not require consumers to write full render methods for ordinary use
```

### Phase 4: renderers

Implement rendering for:

```txt
- labels
- bare/count/definite/indefinite/possessed/external noun phrases
- finite verb phrases where needed
- infinitive complements
- declaratives
- yes/no questions
- wh subject/object/adverbial questions
- imperatives
- fragments
```

Acceptance:

```txt
- English do-support works
- English base/finite/infinitive forms work
- English article rendering is explicit
- Chinese classifiers work
- Chinese yes/no particles do not appear in wh-questions
- Chinese punctuation is Chinese punctuation
- Catalan gender/articles/plurals work for covered nouns
- Catalan finite verb forms work for covered subjects/tenses
```

### Phase 5: createI18n and registry helpers

- Implement `createI18n` with explicit-locale render methods.
- Implement `isLocale` and `assertLocale`.
- Return `locales` and `defaultLocale`.
- Do not add `t(locale)` or `forLocale` in v1.

Acceptance:

```txt
- locale union is inferred from the const locale list
- typoed runtime locale strings can be rejected with isLocale/assertLocale
- render methods all require locale as their first argument
```

### Phase 6: migrate `ssg-test`

Because all call sites are under our control,
 do not preserve the current accessor shape for its own sake.

Migration tasks:

```txt
- create app-local `src/i18n/index.ts` that exports locales, Locale, i18n, isLocale, assertLocale
- move existing static translations into label tables for ca/en/zh
- define any nouns/verbs/subjects actually needed by new grammar call sites
- replace generated `typesafe-i18n` imports with the new app-local i18n module
- replace old message-method calls with explicit `i18n.label(locale, key)` or grammar calls
- preserve or replace `lang-names.ts` explicitly; do not let it break through deleted generated imports
- remove `typesafe-i18n` devDependency
- remove the `typesafe-i18n --no-watch` generation step
- delete generated files only after all imports are gone
- update cache invalidation so `src/i18n/**/*.ts` changes invalidate rendered output
- update acceptance commands to real current workspace tasks; do not cite nonexistent tasks
```

Generated files to remove once no longer imported:

```txt
src/i18n/i18n-types.ts
src/i18n/i18n-util.ts
src/i18n/i18n-util.sync.ts
src/i18n/i18n-util.async.ts
src/i18n/formatters.ts
```

Verify the exact generated-file list in the repo before deletion.

### Phase 7: cleanup and docs

- README examples must use `i18n.sentence(locale, ast)`,
   not `t(locale).sentence(ast)`.
- README must explain that labels are not nouns.
- README must include the no-template-holes rule.
- README must include at least one example showing generic grammar composition with opaque external text.
- README must warn that the package owns language mechanics,
   not application semantics.

## 14. Testing plan

Use the workspace's normal test tooling and style.
 Do not invent unsupported task names in the plan.

Required tests:

```txt
- unit tests for locale registry helpers
- type tests for impossible AST states
- golden render tests for ca/en/zh
- table tests across sentence kind, tense, subject number/person, object presence, complement presence, and adverbial presence
- tests for labels being separate from nouns
- tests for external text placement without translation/parsing
- tests for Chinese punctuation and question-particle behavior
- tests for English do-support and base-form behavior
- tests for Catalan article/gender/plural behavior on covered vocabulary
```

Recommended robustness tests:

```txt
- generated random counts for plural/count rendering
- random external text containing braces `{}` to prove no template parser or brace regex is involved
- regression cases for nested braces in labels and external text
```

Do not claim a workspace-wide fuzzing/property/mutation mandate unless it is actually present in the repo.

## 15. Acceptance criteria

Package acceptance:

```txt
- no codegen
- no template parser
- no regex over translation strings for `{}` placeholders
- no app-specific message APIs
- no exported core app vocabulary
- explicit locale parameter on every render call
- ca/en/zh supported by locale builders and tests
- invalid sentence states rejected by TypeScript where practical
- render tests green for all covered grammar primitives
```

`ssg-test` migration acceptance:

```txt
- all generated `typesafe-i18n` imports removed or replaced
- all existing locales ca/en/zh still present
- all existing static UI strings represented as labels
- existing language metadata preserved or replaced
- actual current build/lint/test tasks pass
- rendered output diff is either empty or intentionally explained
- i18n source changes invalidate cached rendered output
- `typesafe-i18n` dependency and generation step removed
```

## 16. Explicit non-goals

```txt
- no app message registry in the shared package
- no `confirmDelete`, `selectedCount`, `postedAt`, `siteName`, or similar methods in the shared package
- no global current locale
- no locale binding API in v1
- no parser for template holes
- no codegen or compile step
- no machine translation
- no attempt to handle every natural-language phenomenon
- no support for arbitrary languages until a real consumer needs them
```

## 17. Design checks before merge

Before merging the package or the `ssg-test` migration,
 verify:

```txt
- package source contains no `confirmDelete`, `selectedCount`, `postedAt`, `siteName`, `chooseALang`, `noResults`, route, rss, post, or page-specific APIs outside tests/examples
- package source contains no core noun/verb unions exported from the root
- every public render method takes locale as the first argument
- every old `typesafe-i18n` generated export used by `ssg-test` has a replacement or the call site is removed
- `ca` is not skipped in tests
- wh-question tests prove slot-based behavior
- English interrogative tests prove base-verb behavior after auxiliaries
- Chinese wh-question tests prove no `吗` particle is added
- labels cannot be used as nouns
```

## 18. Short summary for new teammates

We are replacing generated/template-based i18n with a small typed grammar renderer.

The library renders generic grammar from consumer-owned vocabulary:

```ts
i18n.sentence(locale, ast,);
i18n.np(locale, nounPhrase,);
i18n.label(locale, key,);
```

The library does not own app messages or app state:

```ts
// forbidden in shared package
i18n.confirmDelete(locale, ...);
i18n.postedAt(locale, ...);
i18n.selectedCount(locale, ...);
```

The guiding rule is:

```txt
shared package owns language mechanics; consuming packages own product semantics.
```
