import { CANADIAN_PAIRS, } from './canadian-spelling-pairs.ts';
import {
  IZE_ENDINGS,
  IZE_STEMS,
  OUR_ENDINGS,
  OUR_STEMS,
} from './canadian-spelling-stems.ts';

//region Canadian spelling words
// The spelling map the Canadian pass reads, built from the stem families and
// the explicit pairs, and the small word sets that decide whether a word
// stands in prose, opens a sentence or names someone.

/**
 Every listed spelling to its Canadian form, lower case: each "-or" stem with
 each ending that keeps the "u", each "-ise" stem with each ending, and the
 explicit pairs.
 */
export const CANADIAN_SPELLINGS: ReadonlyMap<string, string> = new Map([
  ...OUR_STEMS.flatMap(function ourForms(stem,): readonly (readonly [
    string,
    string,
  ])[] {
    return OUR_ENDINGS.map(function ourForm(ending,): readonly [
      string,
      string,
    ] {
      return [
        `${stem}r${ending}`,
        `${stem}ur${ending}`,
      ];
    },);
  },),
  ...IZE_STEMS.flatMap(function izeForms(stem,): readonly (readonly [
    string,
    string,
  ])[] {
    return IZE_ENDINGS.map(function izeForm(ending,): readonly [
      string,
      string,
    ] {
      return [
        `${stem}is${ending}`,
        `${stem}iz${ending}`,
      ];
    },);
  },),
  ...CANADIAN_PAIRS,
],);

/**
 "Mum" as Canadian English writes it (the house policy: "mom, not mum").
 Lower case, "mum" also means silent ("keep mum"), so it is respelled only
 after a possessive; capitalised, it is a form of address and always "Mom".
 */
export const MOTHER_WORDS: ReadonlyMap<string, string> = new Map([
  [
    'mum',
    'mom',
  ],
  [
    'mums',
    'moms',
  ],
],);

/**
 Possessives after which "mum" can only mean a mother.
 */
export const POSSESSIVE_DETERMINERS: ReadonlySet<string> = new Set([
  'my',
  'your',
  'his',
  'her',
  'our',
  'their',
],);

/**
 Titles whose closing period is no sentence end, so a capital after them
 starts a name ("Mr. Gray").
 */
export const NAME_TITLES: ReadonlySet<string> = new Set([
  'Mr',
  'Mrs',
  'Ms',
  'Mx',
  'Dr',
  'Prof',
  'St',
  'Mt',
  'Jr',
  'Sr',
  'No',
  'vs',
  'etc',
],);

/**
 Listed words that, capitalised, are more often a name than a sentence's
 first word: "Gray" is a surname, and "Id" is Freud's.
 */
export const CAPITALISED_EXCLUDED: ReadonlySet<string> = new Set([
  'gray',
  'grays',
  'id',
],);

/**
 Words that name the psychoanalytic frame, in lower case: where a text
 carries one, a bare "id" is Freud's and keeps its spelling (ledger H14).
 */
export const FREUDIAN_WORDS: ReadonlySet<string> = new Set([
  'ego',
  'egos',
  'superego',
  'freud',
  'freudian',
  'psychoanalysis',
  'psychoanalytic',
],);

/**
 Words a title-case heading writes in lower case.
 */
export const TITLE_SMALL_WORDS: ReadonlySet<string> = new Set([
  'a',
  'an',
  'and',
  'as',
  'at',
  'but',
  'by',
  'for',
  'from',
  'in',
  'nor',
  'of',
  'on',
  'or',
  'the',
  'to',
  'vs',
  'with',
],);

//endregion Canadian spelling words
