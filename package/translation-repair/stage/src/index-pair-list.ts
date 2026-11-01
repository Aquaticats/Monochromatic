import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { IndexPair, } from './pair-agreement.ts';

//region Index pair list
// The shape every pairing answer and its cache record share: a list of
// correspondences, each naming one index on each side. The block pairing
// reader (`pair-blocks-wire.ts`), the section pairing reader
// (`pair-sections-read.ts`) and the slice cache (`slice-cache-store.ts`) each
// kept their own copy of this test (audit area six, 2026-09-28); a cache that
// accepted what the wire refuses would hand a stage a pairing no model gave.

/**
 Whether a parsed value is a list of correspondences.

 SHAPE ONLY. What a pairing must satisfy against the blocks or sections it
 describes is each reader's own question.

 @param value - candidate list, still unknown in type

 @returns Whether it is an array whose every entry names two integer indices

 @example
 ```ts
 isIndexPairList([{ source: 0, target: 0, },],); // true
 isIndexPairList([{ source: 0, target: 1.5, },],); // false
 ```
 */
export function isIndexPairList(value: unknown,): value is readonly IndexPair[] {
  if (!Array.isArray(value,))
    return false;
  return value.every(function isPair(entry: unknown,): boolean {
    if (!isJsonRecord(entry,))
      return false;
    if (!('source' in entry))
      return false;
    if (!('target' in entry))
      return false;

    /**
     Candidate indices, still unknown in type.
     */
    const {
      source,
      target,
    } = entry;
    return Number.isInteger(source,) && Number.isInteger(target,);
  },);
}

/**
 Whether a parsed model reply has the shape of a pairing: an object whose
 `pairs` is a list of correspondences.

 The block and section pairing readers each kept this outer check around
 their own copy of the list test (audit area six, 2026-09-28); both name
 their own wire type and delegate here.

 @param value - parsed model reply

 @returns Whether it carries a `pairs` list of integer index pairs

 @example
 ```ts
 isIndexPairingWire({ pairs: [], },); // true
 isIndexPairingWire({ pairs: 'none', },); // false
 ```
 */
export function isIndexPairingWire(
  value: unknown,
): value is { readonly pairs: readonly IndexPair[]; } {
  if (!isJsonRecord(value,))
    return false;
  if (!('pairs' in value))
    return false;
  return isIndexPairList(value.pairs,);
}

//endregion Index pair list
