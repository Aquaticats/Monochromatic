/**
 English locale type definitions: verb entry shape and locale-input record.
 
 @module
 */

import type {
  NounEntry,
  SubjectEntry,
} from '../../entries.ts';

/**
 English verb entry. Locale-specific shape: do-support requires the base
 form for auxiliaries, declarative third-person singular requires the
 `present3s` form, past tense the `past` form, and so on. The single
 `(person, number, tense) => string` shape would produce `Did I had`
 for past questions because the renderer would call into the past form
 after `did` instead of using the base.
 */
export type EnglishVerbEntry = {
  /**
   Base form, e.g. `save`. Used by imperatives, infinitives, and post-auxiliary positions.
   */
  readonly base: string;
  /**
   Third-person singular present finite form, e.g. `saves`. Falls back to `base + s`.
   */
  readonly present3s?: string;
  /**
   Simple past form, e.g. `saved`. Falls back to `base + ed`.
   */
  readonly past?: string;
  /**
   Past participle, e.g. `saved`. Falls back to `past` if unset.
   */
  readonly pastParticiple?: string;
  /**
   Gerund form, e.g. `saving`. Falls back to `base + ing`.
   */
  readonly gerund?: string;
  /**
   Imperative surface; defaults to `base`.
   */
  readonly imperative?: string;
  /**
   Strategy for question and complement construction.
   
   `do-support` is the default for ordinary lexical verbs and emits
   `do`/`does`/`did` plus the base verb in questions. `copula` fronts
   the finite verb itself, e.g. `Are you ready?`. `modal` fronts the
   base modal and renders nested complements bare, e.g. `Can you save?`.
   `none` also skips do-insertion and bare-renders complements for entries
   that need caller-supplied surfaces outside the built-in strategies.
   */
  readonly auxiliaryStrategy?: 'do-support' | 'copula' | 'modal' | 'none';
};

/**
 Input passed to {@link defineEnglishLocale}.
 
 Vocabulary categories must be exhaustively keyed to the consumer's
 unions; missing entries fail at compile time via `satisfies`.
 */
export type DefineEnglishLocaleInput<
  Label extends string,
  Subject extends string,
  Verb extends string,
  Noun extends string,
  > = {
  /**
   Static UI label table keyed by the consumer's `Label` union.
   */
    readonly labels: Readonly<Record<Label, string>>;
  /**
   Subject vocabulary keyed by the consumer's `Subject` union.
   */
    readonly subjects: Readonly<Record<Subject, SubjectEntry>>;
  /**
   Noun vocabulary keyed by the consumer's `Noun` union.
   */
    readonly nouns: Readonly<Record<Noun, NounEntry>>;
  /**
   Verb vocabulary keyed by the consumer's `Verb` union.
   */
    readonly verbs: Readonly<Record<Verb, EnglishVerbEntry>>;
  };

/**
 Tokens never recased by sentence-case fixup; English `I` is the canonical case.
 */
export const EN_CASE_INVARIANTS: ReadonlySet<string> = new Set(['I',],);
