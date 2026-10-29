/**
 Chinese verb-phrase renderer factory and tense surface helpers.
 
 @module
 */

import type {
  Adverbial,
  NounPhrase,
  VerbPhrase,
} from '../../ast.ts';
import type { Tense, } from '../../grammar-primitives.ts';
import { joinTokens, } from '../../render-helpers.ts';
import type { ChineseVerbEntry, } from './types.ts';

/**
 Picks the verb surface for a tense in Chinese.
 
 @param entry - resolved verb entry
 
 @param tense - sentence tense
 
 @returns tense-marked verb surface
 
 @example
 ```ts
 verbSurfaceForTense({ entry: { surface: '看见' }, tense: 'past' }); // '看见了'
 ```
 */
export function verbSurfaceForTense(
  {
    entry,
    tense,
  }: {
    readonly entry: ChineseVerbEntry;
    readonly tense: Tense;
  },
): string {
  if (tense === 'past')
    return entry.past
      ?? `${entry.surface}了`;
  if (tense === 'future')
    return entry.future
      ?? `会${entry.surface}`;
  return entry.surface;
}

/**
 Builds a Chinese verb-phrase renderer.
 
 @param verbs - verb vocabulary keyed by the consumer's `Verb` union
 
 @param renderNounPhrase - noun-phrase render function
 
 @param renderAdverbials - adverbial cluster render function
 
 @returns render function for verb phrases
 
 @example
 ```ts
 const renderVerbPhrase = makeChineseVerbPhraseRenderer({ verbs, renderNounPhrase, renderAdverbials });
 ```
 */
export function makeChineseVerbPhraseRenderer<
  S extends string,
  V extends string,
  N extends string,
  >(
  {
    verbs,
    renderNounPhrase,
    renderAdverbials,
  }: {
    readonly verbs: Readonly<Record<V, ChineseVerbEntry>>;
    readonly renderNounPhrase: (phrase: NounPhrase<S, N>,) => string;
    readonly renderAdverbials: (
      advs?: readonly Adverbial<S, N>[],
    ) => string;
  },
  ): (phrase: VerbPhrase<S, V, N>,) => string {
  /**
   Renders a verb-phrase AST in Chinese using the bare surface.
   
   @param phrase - verb-phrase AST
   
   @returns rendered surface
   */
  function renderVerbPhrase(phrase: VerbPhrase<S, V, N>,): string {
    /**
     Verb base surface; tense decoration handled by callers that own tense context.
     */
    const verb = verbs[phrase.verb]
      .surface;
    /**
     Rendered object surface; empty string when absent.
     */
    const object = phrase.object
      === undefined
      ? ''
      : renderNounPhrase(phrase.object,);
    /**
     Rendered complement; empty string when absent.
     */
    const complement = phrase.complement
      === undefined
      ? ''
      : renderVerbPhrase(phrase.complement
        .phrase,);
    /**
     Rendered adverbial cluster; empty string when none.
     */
    const adverbials = renderAdverbials(phrase.adverbials,);
    return joinTokens([
      adverbials,
      verb,
      object,
      complement,
    ],);
  }

  return renderVerbPhrase;
}
