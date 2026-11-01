import { RENDERING_GLOSSARY, } from '../dist/final/node/index.mjs';

//region Rendering glossary seeded
// WHETHER A TERM IS SEEDED IN THE REAL RENDERING GLOSSARY, read against the
// package's own production glossary rather than a test-authored list.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. Several rendering-glossary tests kept
// their own copy of this guard; all now import it from here.

/**
 Whether a term is present in the real rendering glossary.

 @param term - term to look up

 @returns Whether the glossary seeds it

 @example
 ```ts
 const seeded = isSeeded('fandom',);
 ```
 */
function isSeeded(term: string,): boolean {
  return RENDERING_GLOSSARY.some(function isTerm(entry,): boolean {
    return entry.term === term;
  },);
}

/**
 The terms of a list the real rendering glossary seeds, in list order.

 @param terms - terms to look up

 @returns Those the glossary seeds

 @example
 ```ts
 const seeded = seededAmong({ terms: ['fandom', 'catnip',], },);
 ```
 */
export function seededAmong({ terms, }: { readonly terms: readonly string[]; },): readonly string[] {
  return terms.filter(function seeded(term,): boolean {
    return isSeeded(term,);
  },);
}

//endregion Rendering glossary seeded
