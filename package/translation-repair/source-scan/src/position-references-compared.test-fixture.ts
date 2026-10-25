/**
 How the position-references scan (ledger D33) reads a word of the compared list after a position: whether a gap alone
 joins two words, and whether a word that is also a verb, a clause opener or a floating quantifier is the thing
 compared. Split out of the scan so each file stays under the line limit.

 @module
 */

import type { Word, } from './position-references-text.test-fixture.ts';
import {
  COMPARED_NOT_BEFORE_BARE_VERB,
  COMPARED_ONLY_BEFORE_NAMED_THING,
  COMPARED_UNLESS_OBJECT_FOLLOWS,
  JOIN_GAP,
  OBJECT_OPENERS,
  QUANTIFIED_NAMES,
  VERBS_AFTER_THAT,
} from './position-references-vocabulary.test-fixture.ts';

//region Compared words

/**
 Whether two words stand next to each other with nothing but a joining gap
 between them.

 @param flat - joined text

 @param left - earlier word

 @param right - later word

 @returns Whether they join

 @example
 ```ts
 adjoins({ flat, left, right, },);
 ```
 */
export function adjoins(
  {
    flat,
    left,
    right,
  }: {
    readonly flat: string;
    readonly left: Word;
    readonly right: Word;
  },
): boolean {
  if (right.start <= left.end)
    return false;
  for (let at = left.end; at < right.start; at += 1) {
    if (!JOIN_GAP.has(flat.charAt(at,),))
      return false;
  }
  return true;
}

/**
 Whether a word is followed by another that nothing but a joining gap separates from it.

 @param flat - joined text

 @param words - every word of it

 @param at - index of the first word

 @returns Whether a next word joins it

 @example
 ```ts
 wordJoinsNext({ flat, words, at, },);
 ```
 */
function wordJoinsNext(
  {
    flat,
    words,
    at,
  }: {
    readonly flat: string;
    readonly words: readonly Word[];
    readonly at: number;
  },
): boolean {
  /**
   The first word.
   */
  const word = words[at];
  /**
   The word after it.
   */
  const next = words[at + 1];
  return (word !== undefined)
    && (next !== undefined)
    && adjoins({
      flat,
      left: word,
      right: next,
    },);
}

/**
 Whether the word after a word, joined to it by a gap alone, belongs to a set.

 @param flat - joined text

 @param words - every word of it

 @param at - index of the first word

 @param set - words to look for

 @returns Whether a next word joins it and belongs to the set

 @example
 ```ts
 joinedWordIn({ flat, words, at, set: OBJECT_OPENERS, },);
 ```
 */
function joinedWordIn(
  {
    flat,
    words,
    at,
    set,
  }: {
    readonly flat: string;
    readonly words: readonly Word[];
    readonly at: number;
    readonly set: ReadonlySet<string>;
  },
): boolean {
  return wordJoinsNext({
    flat,
    words,
    at,
  },)
    && set.has(words[at + 1]
      ?.text
      ?? '',);
}

/**
 Whether a word of the compared list, standing right after a position, is the thing compared, judged with the word
 after it.

 TWO KINDS OF WORD ARE READ WITH THE WORD AFTER THEM (re-check 8, nit A). A word that is a verb or opens a clause
 (`zero`, `that`) compares unless a determiner, a quantifier or a pronoun follows, since a verb takes an object and a
 clause begins with one; `that` followed by a verb of `VERBS_AFTER_THAT` is a pointer too ("the lines below that write
 it"). A quantifier that floats away from its noun (`all`, `both`) compares only when a determiner, a word of
 `QUANTIFIED_NAMES` follows or no word joins it, since "the cases below all pass" names no thing. Any other word
 of the list compares.

 @param flat - joined text

 @param words - every word of it

 @param at - index of the compared word

 @returns Whether the compared word is read as the object of a comparison

 @example
 ```ts
 readsAsComparedObject({ flat, words, at, },);
 ```
 */
export function readsAsComparedObject(
  {
    flat,
    words,
    at,
  }: {
    readonly flat: string;
    readonly words: readonly Word[];
    readonly at: number;
  },
): boolean {
  /**
   The compared word.
   */
  const compared = words[at];
  if (compared === undefined)
    return false;
  /**
   Whether the next word, when one joins, opens an object or a clause.
   */
  const opens = joinedWordIn({
    flat,
    words,
    at,
    set: OBJECT_OPENERS,
  },);
  if (
    COMPARED_NOT_BEFORE_BARE_VERB.has(compared.text,)
      && joinedWordIn({
        flat,
        words,
        at,
        set: VERBS_AFTER_THAT,
      },)
  )
    return false;
  if (COMPARED_UNLESS_OBJECT_FOLLOWS.has(compared.text,))
    return !opens;
  if (COMPARED_ONLY_BEFORE_NAMED_THING.has(compared.text,))
    return (!wordJoinsNext({
      flat,
      words,
      at,
    },))
      || opens
      || joinedWordIn({
        flat,
        words,
        at,
        set: QUANTIFIED_NAMES,
      },);
  return true;
}

//endregion Compared words
