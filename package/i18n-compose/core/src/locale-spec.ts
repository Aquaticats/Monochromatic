/**
 {@link LocaleSpec} is the uniform output produced by every locale builder.
 
 Builders accept locale-specific vocabulary entry shapes (e.g.
 {@link EnglishVerbEntry}) and emit this single shape so {@link createI18n} can
 dispatch render calls without knowing which builder produced the spec.
 
 @module
 */

import type {
  Fragment,
  NounPhrase,
  Sentence,
  VerbPhrase,
} from './ast.ts';

/**
 Function property type with method-level bivariance preserved deliberately.
 
 Locale specs are stored under a loose {@link AnyLocaleSpec} constraint inside
 {@link createI18n}, and concrete specs with literal vocabulary keys must remain
 assignable to that upper bound while still exposing property signatures.
 TypeScript only models that safe legacy method variance through method
 signatures, so the helper isolates the method-signature exception in one
 reusable type instead of repeating methods on the public {@link LocaleSpec} shape.
 */
type BivariantRenderer<Parameter, Output,> = {
  /**
   Method-signature variance anchor used only through indexed access below.
   */
  // oxlint-disable-next-line typescript/method-signature-style -- method syntax is required here to preserve LocaleSpec's historical bivariant assignability while public fields stay property signatures.
  bivarianceHack(parameter: Parameter,): Output;
}['bivarianceHack'];

/**
 Render-time interface every locale builder produces.
 
 Each method takes the same AST shape regardless of locale; the
 differences live inside the closure the builder captures.
 
 The generics flow from the consumer's vocabulary unions so that an
 AST referencing a noun key the locale spec does not define is rejected
 at compile time.
 */
export type LocaleSpec<
  Label extends string,
  Subject extends string,
  Verb extends string,
  Noun extends string,
  > = {
  /**
   Resolves a static UI label key to its rendered surface.
   */
    readonly renderLabel: BivariantRenderer<Label, string>;
  /**
   Resolves a bare noun key to its rendered surface, without any article or count.
   */
    readonly renderNoun: BivariantRenderer<Noun, string>;
  /**
   Renders a complete noun phrase AST.
   */
    readonly renderNounPhrase: BivariantRenderer<NounPhrase<Subject, Noun>, string>;
  /**
   Renders a complete verb phrase AST, using the locale's tense and agreement strategy.
   */
    readonly renderVerbPhrase: BivariantRenderer<VerbPhrase<Subject, Verb, Noun>, string>;
  /**
   Renders a sentence AST (declarative, yes/no, wh, or imperative).
   */
    readonly renderSentence: BivariantRenderer<Sentence<Subject, Verb, Noun>, string>;
  /**
   Renders a fragment AST (sub-sentence text such as headings or button labels).
   */
    readonly renderFragment: BivariantRenderer<Fragment<Label, Subject, Verb, Noun>, string>;
  };

/**
 Extracts the `Label` type parameter from a {@link LocaleSpec}.
 */
export type LabelOf<Spec,> = Spec extends LocaleSpec<infer L, string, string, string> ? L
  : never;

/**
 Extracts the `Subject` type parameter from a {@link LocaleSpec}.
 */
export type SubjectOf<Spec,> = Spec extends LocaleSpec<string, infer S, string, string>
  ? S
  : never;

/**
 Extracts the `Verb` type parameter from a {@link LocaleSpec}.
 */
export type VerbOf<Spec,> = Spec extends LocaleSpec<string, string, infer V, string> ? V
  : never;

/**
 Extracts the `Noun` type parameter from a {@link LocaleSpec}.
 */
export type NounOf<Spec,> = Spec extends LocaleSpec<string, string, string, infer N> ? N
  : never;
