/**
 Chinese locale type definitions: verb entry shape and locale-input record.
 
 @module
 */

import type {
  NounEntry,
  SubjectEntry,
} from '../../entries.ts';

/**
 Chinese verb entry. Chinese verbs do not inflect for person, number,
 or tense at the morphological level; tense/aspect is carried by
 surrounding particles (`了` for perfective, `会` for future, etc.). The
 entry stores the bare surface plus optional dedicated forms when the
 caller wants per-tense overrides; the default rendering uses `surface`.
 */
export type ChineseVerbEntry = {
  /**
   Bare surface form, e.g. `有` for `have`, `看见` for `see`, `删除` for `delete`.
   */
  readonly surface: string;
  /**
   Optional dedicated past-tense surface; falls back to `surface + 了`.
   */
  readonly past?: string;
  /**
   Optional dedicated future surface; falls back to `会 + surface`.
   */
  readonly future?: string;
  /**
   Optional perfective surface, used when the renderer wants explicit completion marking.
   */
  readonly perfective?: string;
};

/**
 Input shape accepted by {@link defineChineseLocale}.
 */
export type DefineChineseLocaleInput<
  Label extends string,
  Subject extends string,
  Verb extends string,
  Noun extends string,
  > = {
    readonly labels: Readonly<Record<Label, string>>;
    readonly subjects: Readonly<Record<Subject, SubjectEntry>>;
    readonly nouns: Readonly<Record<Noun, NounEntry>>;
    readonly verbs: Readonly<Record<Verb, ChineseVerbEntry>>;
  };

/**
 Chinese script never recases tokens; the invariant set is empty.
 */
export const ZH_CASE_INVARIANTS: ReadonlySet<string> = new Set();
