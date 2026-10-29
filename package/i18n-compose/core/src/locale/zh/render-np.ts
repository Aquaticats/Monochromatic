/**
 Chinese noun-phrase renderer factory.
 
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
 Renders a counted noun phrase using the noun's classifier when present,
 otherwise just the digit and noun.
 
 @param entry - resolved noun entry
 
 @param count - count attached by the phrase
 
 @returns counted phrase surface, e.g. `1 只猫`
 
 @example
 ```ts
 countedNoun({ entry: { surface: '猫', classifier: '只' }, count: 1 }); // '1 只猫'
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
  if (entry.classifier
    !== undefined)
    return `${String(count,)} ${entry.classifier}${entry.surface}`;
  return `${String(count,)} ${entry.surface}`;
}

/**
 Builds a Chinese noun-phrase renderer closed over the supplied vocab tables.
 
 @param nouns - noun vocabulary keyed by the consumer's `Noun` union
 
 @param subjects - subject vocabulary keyed by the consumer's `Subject` union
 
 @returns render function for noun phrases
 
 @example
 ```ts
 const renderNounPhrase = makeChineseNounPhraseRenderer({ nouns, subjects });
 renderNounPhrase({ kind: 'noun.counted', count: 1, noun: 'cat' }); // '1 只猫'
 ```
 */
export function makeChineseNounPhraseRenderer<S extends string, N extends string,>(
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
   
   @returns possessive surface
   */
  function renderPossessor(p: Possessor<S>,): string {
    if (p.kind
      === 'possessor.subject')
      return subjects[p.subject]
        .possessive;
    return `${p.text}的`;
  }

  /**
   Renders a noun-phrase AST in Chinese.
   
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
       Resolved noun entry validated before classifier rendering.
       */
      const entry = nouns[phrase.noun];
      assertCountableNoun({
        entry,
        noun: phrase.noun,
      },);
      return countedNoun({
        entry,
        count: phrase.count,
      },);
    }
    if (phrase.kind
      === 'noun.definite')
      return `这${nouns[phrase.noun]
        .classifier
        ?? '个'}${nouns[phrase.noun]
          .surface}`;
    if (phrase.kind
      === 'noun.indefinite')
      return `一${nouns[phrase.noun]
        .classifier
        ?? '个'}${nouns[phrase.noun]
          .surface}`;
    if (phrase.kind
      === 'noun.possessed')
      return `${renderPossessor(phrase.possessor,)}${nouns[phrase.noun]
        .surface}`;
    return phrase.text;
  }

  return renderNounPhrase;
}
