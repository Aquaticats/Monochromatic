/**
 Chinese adverbial renderer factory.
 
 @module
 */

import type {
  Adverbial,
  ExternalText,
  NounPhrase,
} from '../../ast.ts';

/**
 Locative coverb chosen for a Chinese `adverbial.location` relation.
 
 @param relation - location relation from the AST
 
 @returns surface coverb string
 */
function locativeCoverb(relation: 'at' | 'in' | 'to' | 'from',): string {
  if (relation === 'to')
    return '到';
  if (relation === 'from')
    return '从';
  return '在';
}

/**
 Builds a Chinese adverbial renderer that consumes already-built noun-phrase rendering.
 
 @param renderNounPhrase - noun-phrase render function
 
 @returns render function for adverbial clusters; returns empty string for empty input
 
 @example
 ```ts
 const renderAdverbials = makeChineseAdverbialRenderer({ renderNounPhrase });
 ```
 */
export function makeChineseAdverbialRenderer<S extends string, N extends string,>(
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
   Renders a single adverbial node using Chinese coverbs/particles.
   
   @param adv - adverbial AST node
   
   @returns rendered surface
   */
  function renderAdverbial(adv: Adverbial<S, N>,): string {
    if (adv.kind
      === 'adverbial.location')
      return `${locativeCoverb(adv.relation,)}${renderNounPhrase(adv.place,)}`;
    if (adv.relation
      === 'before')
      return `${renderTimeOperand(adv.time,)}之前`;
    if (adv.relation
      === 'after')
      return `${renderTimeOperand(adv.time,)}之后`;
    return `在${renderTimeOperand(adv.time,)}`;
  }

  /**
   Renders an adverbial cluster as a single joined string.
   
   @param advs - optional adverbial list
   
   @returns concatenated surface, or empty string for empty input
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
      .join('',);
  }

  return renderAdverbials;
}
