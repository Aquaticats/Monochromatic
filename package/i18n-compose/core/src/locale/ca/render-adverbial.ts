/**
 Catalan adverbial renderer factory.
 
 @module
 */

import type {
  Adverbial,
  ExternalText,
  NounPhrase,
} from '../../ast.ts';

/**
 Picks the Catalan preposition for a location relation.
 
 @param relation - location relation from the AST
 
 @returns surface preposition
 */
function locationPreposition(relation: 'at' | 'in' | 'to' | 'from',): string {
  if (relation === 'at')
    return 'a';
  if (relation === 'in')
    return 'en';
  if (relation === 'to')
    return 'a';
  return 'de';
}

/**
 Picks the Catalan preposition/phrase for a time relation.
 
 @param relation - time relation from the AST
 
 @returns surface preposition
 */
function timePreposition(relation: 'at' | 'before' | 'after',): string {
  if (relation === 'at')
    return 'a';
  if (relation === 'before')
    return 'abans de';
  return 'després de';
}

/**
 Builds a Catalan adverbial renderer that consumes already-built noun-phrase rendering.
 
 @param renderNounPhrase - noun-phrase render function
 
 @returns render function for adverbial clusters
 
 @example
 ```ts
 const renderAdverbials = makeCatalanAdverbialRenderer({ renderNounPhrase });
 ```
 */
export function makeCatalanAdverbialRenderer<S extends string, N extends string,>(
  {
    renderNounPhrase,
  }: {
    readonly renderNounPhrase: (phrase: NounPhrase<S, N>,) => string;
  },
): (advs?: readonly Adverbial<S, N>[],) => string {
  /**
   Renders a time operand handling both noun-phrase and external-text variants.
   
   @param operand - operand AST node
   
   @returns rendered surface
   */
  function renderTimeOperand(operand: NounPhrase<S, N> | ExternalText,): string {
    return operand.kind
      === 'externalText' ? operand.text : renderNounPhrase(operand,);
  }

  /**
   Renders a single adverbial node using Catalan prepositions.
   
   @param adv - adverbial AST node
   
   @returns rendered surface
   */
  function renderAdverbial(adv: Adverbial<S, N>,): string {
    if (adv.kind
      === 'adverbial.location')
      return `${locationPreposition(adv.relation,)} ${renderNounPhrase(adv.place,)}`;
    return `${timePreposition(adv.relation,)} ${renderTimeOperand(adv.time,)}`;
  }

  /**
   Renders an adverbial cluster as a single space-joined string.
   
   @param advs - optional adverbial list
   
   @returns joined surface, or empty string for empty input
   */
  function renderAdverbials(
    advs?: readonly Adverbial<S, N>[],
  ): string {
    if ((advs === undefined) || (advs.length
      === 0))
      return '';
    return advs
      .map(function renderOne(adv,): string {
        return renderAdverbial(adv,);
      },)
      .join(' ',);
  }

  return renderAdverbials;
}
