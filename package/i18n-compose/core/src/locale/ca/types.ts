/**
 Catalan locale type definitions: verb entry shape and locale-input record.
 
 @module
 */

import type {
  NounEntry,
  SubjectEntry,
} from '../../entries.ts';
import type {
  PersonNumberKey,
  Tense,
} from '../../grammar-primitives.ts';

/**
 Catalan verb entry. Catalan inflects for person, number, and tense, so
 a single `(person, number, tense) => string` shape would lose the
 distinction between regular and irregular forms; instead, finite forms
 live in a sparse map indexed by `Tense` and {@link PersonNumberKey}.
 */
export type CatalanVerbEntry = {
  /**
   Infinitive form, e.g. `tenir`. Used by complements and as fallback head.
   */
  readonly infinitive: string;
  /**
   Imperative surface; defaults to second-person singular present finite when unset.
   */
  readonly imperative?: string;
  /**
   Sparse table of finite forms by tense and person/number. A `Map` models
   the genuine sparseness directly (absent keys are simply not set) instead
   of `Partial<Record>`, which fakes optionality at the type level. Missing
   entries throw at render time rather than producing wrong agreement.
   */
  readonly finite: ReadonlyMap<Tense, ReadonlyMap<PersonNumberKey, string>>;
};

/**
 Input shape accepted by {@link defineCatalanLocale}.
 */
export type DefineCatalanLocaleInput<
  Label extends string,
  Subject extends string,
  Verb extends string,
  Noun extends string,
  > = {
    readonly labels: Readonly<Record<Label, string>>;
    readonly subjects: Readonly<Record<Subject, SubjectEntry>>;
    readonly nouns: Readonly<Record<Noun, NounEntry>>;
    readonly verbs: Readonly<Record<Verb, CatalanVerbEntry>>;
  };

/**
 Catalan does not pin any tokens out of the casing pipeline; the invariant set is empty.
 */
export const CA_CASE_INVARIANTS: ReadonlySet<string> = new Set();
