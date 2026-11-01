# @monochromatic-dev/i18n-compose-core

Type-safe,
 no-codegen i18n composition library.

Renders localized UI text from explicit semantic grammar nodes for `ca`,
 `en`,
 and `zh`.
The library owns language mechanics;
 the consumer owns application semantics.

## Ownership boundary

The shared package owns:

- grammar AST types
- vocabulary entry shapes (subjects,
   nouns,
   verbs with locale-specific forms)
- locale renderers and locale builders
- locale registry helpers (`isLocale`,
   `assertLocale`)
- generic rendering primitives

The consumer owns:

- labels (static UI strings)
- nouns,
   verbs,
   subjects (and any adjective vocabulary)
- route/page concepts,
   confirmation flows,
   item names,
   timestamps
- every domain-specific message shape

The shared package never exports product-message APIs such as `confirmDelete`,
`selectedCount`,
 `postedAt`,
 `siteName`,
 or `noResults`.
Those belong in the consuming app,
 if they exist at all.

## Shared vocabulary across locales

Every locale spec passed to `createI18n` must expose the same label,
 subject,
verb,
 and noun keys.
The package rejects mismatched spec records at the factory boundary,
so adding `cat` to English without adding `cat` to Catalan and Chinese fails type checking.

```ts
const en = defineEnglishLocale({ labels, subjects, nouns, verbs, },);
const zh = defineChineseLocale({ labels, subjects, nouns, verbs, },);
const ca = defineCatalanLocale({ labels, subjects, nouns, verbs, },);

createI18n({
  locales: ['ca', 'en', 'zh',] as const,
  defaultLocale: 'en',
  specs: { ca, en, zh, },
},);
```

This keeps every public render method safe for every supported locale:
`i18n.noun(locale, 'cat')` cannot type-check unless all configured specs know `cat`.

## Explicit locale on every call

Every render call takes the locale as its first argument:

```ts
i18n.label(locale, 'siteName',);
i18n.noun(locale, 'cat',);
i18n.np(locale, { kind: 'noun.counted', count: 1, noun: 'cat', },);
i18n.vp(locale, { kind: 'verbPhrase', verb: 'save', },);
i18n.sentence(locale, sentenceAst,);
i18n.fragment(locale, fragmentAst,);
```

The library does not remember the current locale.
Every render is a pure operation over explicit inputs.
No `forLocale`,
 `bindLocale`,
 or `t(locale)` accessor is exported.

## Labels are not nouns

Static UI strings such as `siteName`,
 `noResults`,
 or `chooseALang`
belong in the `label` vocabulary,
 not the `noun` vocabulary.
Putting them in `noun` makes them available in grammatical slots
(counted,
 definite,
 possessed),
which defeats the purpose of a typed composition API.

```ts
type Label = 'siteName' | 'chooseALang' | 'noResults' | 'page';
type Noun = 'cat' | 'message' | 'item' | 'post';
```

## No template holes

The library does not expose a string-with-placeholders primitive.
Locale-specific ordering is the consumer's job:

```ts
const rendered = `${i18n.label(locale, 'postedAt',)} ${formatTime(time,)}`;
```

No `i18n.message(locale, 'Posted at {time}', { time })` and no `postedAt` method.

## Generic grammar composition

A consuming app composes generic grammar nodes directly;
the library does not know what a confirmation,
 route,
 or product workflow is.
Nested complements preserve the full nested phrase,
including objects,
 complements,
 and adverbials.

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

## Countability validation

`NounEntry.countability` controls whether a noun may appear in `noun.counted`.
`countability: 'mass'` rejects numeric counted phrases in every built-in locale,
because `noun.counted` carries only a bare number and a noun key.
Model measured amounts as countable measure nouns or pass a preformatted phrase through `noun.externalText`.

```ts
const nouns = {
  water: {
    surface: 'water',
    countability: 'mass',
  },
};

i18n.np('en', { kind: 'noun.counted', count: 2, noun: 'water', },); // throws
```

## English auxiliary strategies

English verb entries may set `auxiliaryStrategy` for question and complement construction.
Omitting the field uses `do-support`.

- `do-support` emits `do`,
   `does`,
   or `did`,
   then keeps the lexical verb in base form.
- `copula` fronts the finite verb itself,
   such as `Are you ready?`.
- `modal` fronts the modal surface and renders nested complements bare,
   such as `Can you save?`.
- `none` skips do-insertion and renders complements bare for caller-supplied special cases.

```ts
const verbs = {
  can: {
    base: 'can',
    auxiliaryStrategy: 'modal',
  },
  save: {
    base: 'save',
    present3s: 'saves',
    past: 'saved',
  },
};

i18n.sentence('en', {
  kind: 'sentence.question.yesNo',
  subject: { kind: 'subject.key', subject: 'you', },
  predicate: {
    kind: 'verbPhrase',
    verb: 'can',
    complement: {
      kind: 'complement.infinitive',
      phrase: { kind: 'verbPhrase', verb: 'save', },
    },
  },
},); // Can you save?
```

## Locale scope

Initial scope:
 `ca`,
 `en`,
 `zh`.
 All three are first-class v1 locales.

```ts
import {
  createI18n,
  defineCatalanLocale,
  defineChineseLocale,
  defineEnglishLocale,
} from '@monochromatic-dev/i18n-compose-core';

const en = defineEnglishLocale({ labels, subjects, nouns, verbs, },);
const zh = defineChineseLocale({ labels, subjects, nouns, verbs, },);
const ca = defineCatalanLocale({ labels, subjects, nouns, verbs, },);

export const locales = ['ca', 'en', 'zh',] as const;

export const i18n = createI18n({
  locales,
  defaultLocale: 'en',
  specs: { ca, en, zh, },
},);
```

`defineCustomLocale` is available as an escape hatch when the supplied
English/Chinese/Catalan builders are insufficient.

## Non-goals

- no app message registry in the shared package
- no global current locale
- no parser for template holes
- no codegen step
- no machine translation
- no support for arbitrary languages until a real consumer needs them
