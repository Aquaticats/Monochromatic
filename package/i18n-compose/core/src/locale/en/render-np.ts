/**
 English noun-phrase renderer factory.
 
 @module
 */

import type {
  NounPhrase,
  Possessor,
} from '../../ast.ts';
import { assertCountableNoun, } from '../../countability.ts';
import type {
  NounEntry,
  SubjectEntry,
} from '../../entries.ts';

/**
 Returns the plural surface for a noun under a specific count.
 
 @param entry - resolved noun entry
 
 @param count - count requesting the form
 
 @returns plural surface; falls back to `surface + s` when no plural is supplied
 
 @example
 ```ts
 nounPlural({ entry: { surface: 'cat', plural: 'cats' }, count: 2 }); // 'cats'
 ```
 */
function nounPlural(
  {
    entry,
    count,
  }: {
    readonly entry: NounEntry;
    readonly count: number;
  },
): string {
  if (entry.plural
    === undefined)
    return `${entry.surface}s`;
  if ((typeof entry.plural) === 'string')
    return entry.plural;
  return entry.plural(count,);
}

/**
 Picks singular vs plural surface for a counted noun.
 
 @param entry - noun entry
 
 @param count - count attached by the phrase
 
 @returns singular surface when count is 1, plural otherwise
 
 @example
 ```ts
 countedNoun({ entry: { surface: 'cat', plural: 'cats' }, count: 3 }); // 'cats'
 ```
 */
function countedNoun(
  {
    entry,
    count,
  }: {
    readonly entry: NounEntry;
    readonly count: number;
  },
): string {
  return count === 1 ? entry.surface : nounPlural({
    entry,
    count,
  },);
}

/**
 Builds an English noun-phrase renderer closed over the supplied vocab tables.
 
 @param nouns - noun vocabulary keyed by the consumer's `Noun` union
 
 @param subjects - subject vocabulary keyed by the consumer's `Subject` union
 
 @returns render function for noun phrases
 
 @example
 ```ts
 const renderNounPhrase = makeEnglishNounPhraseRenderer({ nouns, subjects });
 renderNounPhrase({ kind: 'noun.counted', count: 1, noun: 'cat' }); // '1 cat'
 ```
 */
export function makeEnglishNounPhraseRenderer<S extends string, N extends string,>(
  {
    nouns,
    subjects,
  }: {
    readonly nouns: Readonly<Record<N, NounEntry>>;
    readonly subjects: Readonly<Record<S, SubjectEntry>>;
  },
): (phrase: NounPhrase<S, N>,) => string {
  /**
   Renders a possessor surface for `noun.possessed`.
   
   @param p - possessor AST node
   
   @returns possessive surface to drop into the rendered phrase
   */
  function renderPossessor(p: Possessor<S>,): string {
    if (p.kind
      === 'possessor.subject')
      return subjects[p.subject]
        .possessive;
    return `${p.text}'s`;
  }

  /**
   Renders a noun-phrase AST in English.
   
   @param phrase - noun-phrase AST
   
   @returns rendered surface
   */
  function renderNounPhrase(phrase: NounPhrase<S, N>,): string {
    if (phrase.kind
      === 'noun.bare')
      return nouns[phrase.noun]
        .surface;
    if (phrase.kind
      === 'noun.counted') {
      /**
       Resolved noun entry validated before numeric rendering.
       */
      const entry = nouns[phrase.noun];
      assertCountableNoun({
        entry,
        noun: phrase.noun,
      },);
      /**
       Counted noun head: singular at count 1, plural otherwise.
       */
      const head = countedNoun({
        entry,
        count: phrase.count,
      },);
      return `${String(phrase.count,)} ${head}`;
    }
    if (phrase.kind
      === 'noun.definite') {
      /**
       Definite-article singular form, defaults to `the` when the entry omits it.
       */
      const article = nouns[phrase.noun]
        .articles
        ?.definite
        ?.singular
        ?? 'the';
      return `${article} ${nouns[phrase.noun]
        .surface}`;
    }
    if (phrase.kind
      === 'noun.indefinite') {
      /**
       Indefinite-article singular form, defaults to `a` (no a/an inference).
       */
      const article = nouns[phrase.noun]
        .articles
        ?.indefinite
        ?.singular
        ?? 'a';
      return `${article} ${nouns[phrase.noun]
        .surface}`;
    }
    if (phrase.kind
      === 'noun.possessed')
      return `${renderPossessor(phrase.possessor,)} ${nouns[phrase.noun]
        .surface}`;
    return phrase.text;
  }

  return renderNounPhrase;
}
