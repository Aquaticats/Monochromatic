/**
 Locale-agnostic grammatical primitives shared across every locale builder.
 
 @module
 */

/**
 Grammatical person.
 
 `1` is the speaker, `2` is the addressee, `3` is anyone else.
 Locales pick the surface form (English `I/you/she`, Catalan `jo/tu/ella`, Chinese `我/你/她`)
 from a {@link SubjectEntry} keyed by the consumer's subject identifier;
 the numeric person is what verb agreement reads in locales that require it.
 */
export type Person = 1 | 2 | 3;

/**
 Grammatical number.
 
 Distinguishes singular from plural for subject agreement, article selection,
 and noun pluralization in locales that mark either category.
 */
export type GrammaticalNumber = 'singular' | 'plural';

/**
 Sentence-level tense.
 
 Kept deliberately narrow: declaratives, questions, and complements in v1
 only need past/present/future. Aspect and mood are out of scope; if a
 locale needs perfective or progressive forms it carries them in its own
 verb-entry shape, not here.
 */
export type Tense = 'past' | 'present' | 'future';

/**
 Noun countability.
 
 `countable` accepts a numeric count and a plural form (English `cat/cats`);
 `mass` rejects `noun.counted` because the AST carries only a bare numeric
 count; `both` (e.g. English `chicken` as meat vs. as an animal) defers
 to the noun-phrase variant at the call site to decide which behavior applies.
 */
export type Countability = 'countable' | 'mass' | 'both';

/**
 Grammatical gender.
 
 `masculine` and `feminine` cover Catalan; `neuter` is included for
 completeness when a future locale needs it. English uses `neuter` by
 default for inanimate nouns; Chinese ignores the field entirely.
 */
export type GrammaticalGender = 'masculine' | 'feminine' | 'neuter';

/**
 Joined `person + number` key used by inflection tables.
 
 Catalan verb forms are stored as a sparse map indexed by this key,
 so the entry shape is the same across regular and irregular verbs
 and missing slots throw at render time rather than silently producing
 an empty string.
 
 @example
 Catalan present indicative of `tenir` for first-person singular: `'1s'`.
 */
export type PersonNumberKey =
  | '1s'
    | '2s'
    | '3s'
    | '1p'
    | '2p'
    | '3p';

/**
 Builds a {@link PersonNumberKey} from a person and number pair.
 
 @param person - grammatical person
 
 @param number - grammatical number
 
 @returns joined key suitable for indexing inflection tables
 
 @example
 ```ts
 personNumberKey({ person: 1, number: 'singular' }); // '1s'
 personNumberKey({ person: 3, number: 'plural' });   // '3p'
 ```
 */
export function personNumberKey(
  {
    person,
    number,
  }: {
    readonly person: Person;
    readonly number: GrammaticalNumber;
  },
): PersonNumberKey {
  /**
   Number shorthand: `s` for singular, `p` for plural.
   */
  const numberCode = number === 'singular' ? 's' : 'p';
  return `${person}${numberCode}` as PersonNumberKey;
}

/**
 Capitalization mode applied to the first emitted token of a render result.
 
 `preserve` leaves the raw surface unchanged; `firstLetter` uppercases the
 first character of the first token, except for tokens with a casing
 invariant (English `I`, external proper names) which always preserve
 their surface form.
 */
export type Capitalization = 'preserve' | 'firstLetter';

/**
 Non-finite verb form used inside a {@link Fragment} that does not
 grow into a full sentence.
 
 Kept disjoint from the finite-verb tables: an `imperative` fragment is
 rendered as a directive ("Save"), an `infinitive` fragment as `to save`
 in English or `desar` in Catalan, and a `gerund` fragment as `saving`
 or the language's nearest equivalent.
 */
export type VerbFragmentForm = 'imperative' | 'infinitive' | 'gerund';
