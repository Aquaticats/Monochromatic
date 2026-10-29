/**
 Vocabulary entry shapes shared across locales.
 
 Subjects and nouns are locale-agnostic-enough that one structural shape
 works for English, Chinese, and Catalan. Verbs vary per locale and live
 in their respective locale modules.
 
 @module
 */

import type {
  Countability,
  GrammaticalGender,
  GrammaticalNumber,
  Person,
} from './grammar-primitives.ts';

/**
 Singular and plural surface forms for a single article variety.
 
 Used by both `definite` and `indefinite` slots on a {@link NounEntry}.
 Either field may be omitted when the locale does not surface that form
 for the given noun (English mass nouns have no `indefinite.plural`,
 for example).
 */
export type ArticleForms = {
  /**
   Article form combined with a singular noun, e.g. English `a`, Catalan `el`.
   */
  readonly singular?: string;
  /**
   Article form combined with a plural noun, e.g. Catalan `els`; English `the`.
   */
  readonly plural?: string;
};

/**
 Vocabulary entry for a subject pronoun or proper name.
 
 The entry must carry the possessive surface explicitly: in English `I`
 does not derive `my` and `they` does not derive `their` from any rule
 the library could safely encode without a per-locale possessive table.
 Locales that do not use possessives may leave the field equal to the
 nominative surface; the renderer does not invent forms.
 
 @example
 English first-person singular:
 ```ts
 { surface: 'I', possessive: 'my', person: 1, number: 'singular' }
 ```
 
 @example
 English third-person plural:
 ```ts
 { surface: 'they', possessive: 'their', person: 3, number: 'plural' }
 ```
 */
export type SubjectEntry = {
  /**
   Nominative surface form, e.g. English `I`, Catalan `jo`, Chinese `我`.
   */
  readonly surface: string;
  /**
   Possessive surface form, e.g. English `my`, Catalan `meu`, Chinese `我的`.
   */
  readonly possessive: string;
  /**
   Grammatical person used by verb-agreement lookups.
   */
  readonly person: Person;
  /**
   Grammatical number used by verb-agreement lookups.
   */
  readonly number: GrammaticalNumber;
  /**
   Optional gender; Catalan adjectives and articles consult this when present.
   */
  readonly gender?: GrammaticalGender;
};

/**
 Pluralization strategy for a noun.
 
 A string is the regular plural surface used for any count other than 1.
 A function receives the count and returns the surface, letting a noun
 encode irregular forms (English `child` to `children`) or count-sensitive
 forms (Catalan number/gender agreement when needed) without exposing
 the count to the renderer.
 */
export type NounPlural =
  | string
    | ((count: number,) => string);

/**
 Vocabulary entry for a common noun.
 
 The shape covers all three v1 locales: English supplies `plural` and
 `articles`; Chinese supplies `classifier`; Catalan supplies `gender`,
 `plural`, and `articles`. Fields the locale does not surface stay
 undefined.
 
 @example
 English `cat`:
 ```ts
 {
   surface: 'cat',
   plural: 'cats',
   articles: {
     definite: { singular: 'the', plural: 'the' },
     indefinite: { singular: 'a' },
   },
 }
 ```
 
 @example
 Chinese `cat`:
 ```ts
 { surface: '猫', classifier: '只' }
 ```
 
 @example
 Catalan `cat`:
 ```ts
 {
   surface: 'gat',
   plural: 'gats',
   gender: 'masculine',
   articles: {
     definite: { singular: 'el', plural: 'els' },
     indefinite: { singular: 'un', plural: 'uns' },
   },
 }
 ```
 */
export type NounEntry = {
  /**
   Singular bare surface form, used for `noun.bare` and as the base for derivations.
   */
  readonly surface: string;
  /**
   Plural surface or a function producing one from a count.
   */
  readonly plural?: NounPlural;
  /**
   Grammatical gender, consulted by gender-sensitive locales (Catalan).
   */
  readonly gender?: GrammaticalGender;
  /**
   Countability constraint, validated before rendering `noun.counted` phrases.
   */
  readonly countability?: Countability;
  /**
   Classifier surface, used by Chinese-style counted phrases.
   */
  readonly classifier?: string;
  /**
   Article tables, used by `noun.definite` and `noun.indefinite`.
   */
  readonly articles?: {
    readonly definite?: ArticleForms;
    readonly indefinite?: ArticleForms;
  };
};
