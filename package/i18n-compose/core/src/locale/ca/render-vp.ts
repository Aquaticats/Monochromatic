/**
 Catalan verb-phrase renderer factory and finite-verb form helpers.
 
 @module
 */

import type { SubjectAgreement, } from '../../agreement.ts';
import type {
  Adverbial,
  NounPhrase,
  VerbPhrase,
} from '../../ast.ts';
import {
  personNumberKey,
  type Tense,
} from '../../grammar-primitives.ts';
import { joinTokens, } from '../../render-helpers.ts';
import type { CatalanVerbEntry, } from './types.ts';

/**
 Resolves the finite verb surface for a tense and subject agreement.
 
 @param entry - Catalan verb entry
 
 @param key - verb key (used only in error messages)
 
 @param tense - sentence tense
 
 @param agreement - subject person/number
 
 @returns finite verb surface
 
 @throws Error when the entry lacks a form for this tense or person/number combination
 
 @example
 ```ts
 finiteVerbSurface({ entry, key: 'have', tense: 'present', agreement: { person: 1, number: 'singular' } });
 ```
 */
export function finiteVerbSurface(
  {
    entry,
    key,
    tense,
    agreement,
  }: {
    readonly entry: CatalanVerbEntry;
    readonly key: string;
    readonly tense: Tense;
    readonly agreement: SubjectAgreement;
  },
): string {
  /**
   Tense-specific subtable.
   */
  const tenseTable = entry
    .finite
    .get(tense,);
  if (tenseTable === undefined)
    throw new Error(`Catalan verb '${key}' has no finite forms for tense '${tense}'`,);
  /**
   Joined person/number lookup key.
   */
  const pn = personNumberKey({
    person: agreement.person,
    number: agreement.number,
  },);
  /**
   Finite surface for the lookup key.
   */
  const surface = tenseTable.get(pn,);
  if (surface === undefined) {
    throw new Error(
      `Catalan verb '${key}' has no finite form for tense '${tense}' at ${pn}`,
    );
  }
  return surface;
}

/**
 Builds a Catalan verb-phrase renderer.
 
 @param verbs - verb vocabulary keyed by the consumer's `Verb` union
 
 @param renderNounPhrase - noun-phrase render function
 
 @param renderAdverbials - adverbial cluster render function
 
 @returns render function for verb phrases
 
 @example
 ```ts
 const renderVerbPhrase = makeCatalanVerbPhraseRenderer({ verbs, renderNounPhrase, renderAdverbials });
 ```
 */
export function makeCatalanVerbPhraseRenderer<
  S extends string,
  V extends string,
  N extends string,
  >(
  {
    verbs,
    renderNounPhrase,
    renderAdverbials,
  }: {
    readonly verbs: Readonly<Record<V, CatalanVerbEntry>>;
    readonly renderNounPhrase: (phrase: NounPhrase<S, N>,) => string;
    readonly renderAdverbials: (
      advs?: readonly Adverbial<S, N>[],
    ) => string;
  },
  ): (phrase: VerbPhrase<S, V, N>,) => string {
  /**
   Renders a verb-phrase AST in Catalan using the infinitive form.
   
   @param phrase - verb-phrase AST
   
   @returns rendered surface
   */
  function renderVerbPhrase(phrase: VerbPhrase<S, V, N>,): string {
    /**
     Infinitive head for non-finite verb-phrase rendering.
     */
    const head = verbs[phrase.verb]
      .infinitive;
    /**
     Rendered object surface; empty string when absent.
     */
    const object = phrase.object
      === undefined
      ? ''
      : renderNounPhrase(phrase.object,);
    /**
     Rendered complement (bare infinitive phrase); empty string when absent.
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
      head,
      object,
      complement,
      adverbials,
    ],);
  }

  return renderVerbPhrase;
}
