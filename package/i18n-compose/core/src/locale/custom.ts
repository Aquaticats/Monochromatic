/**
 `defineCustomLocale` is an escape hatch for locales not covered by
 {@link defineEnglishLocale}, {@link defineChineseLocale}, or {@link defineCatalanLocale}.
 
 Consumers writing a custom builder accept full responsibility for the
 locale's grammar strategy; the library only enforces that the resulting
 spec implements every render method.
 
 @module
 */

import type { LocaleSpec, } from '../locale-spec.ts';

/**
 Identity wrapper that surfaces a custom-built {@link LocaleSpec} under
 the same name shape as the built-in `define*Locale` family.
 
 The function does no extra work; it exists so call sites read the same
 way regardless of whether the locale comes from a built-in builder or
 a hand-rolled one.
 
 @param spec - manually constructed locale spec
 
 @returns same spec, typed
 
 @example
 ```ts
 const fr = defineCustomLocale({
   renderLabel: (key) => frenchLabels[key],
   renderNoun: (key) => frenchNouns[key].surface,
   renderNounPhrase: (phrase) => { ... },
   renderVerbPhrase: (phrase) => { ... },
   renderSentence: (sentence) => { ... },
   renderFragment: (fragment) => { ... },
 });
 ```
 */
export function defineCustomLocale<
  Label extends string,
  Subject extends string,
  Verb extends string,
  Noun extends string,
  >(
  spec: LocaleSpec<Label, Subject, Verb, Noun>,
  ): LocaleSpec<Label, Subject, Verb, Noun> {
  return spec;
}
