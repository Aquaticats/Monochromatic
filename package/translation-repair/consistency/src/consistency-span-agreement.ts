import {
  codePointLength,
  compareCodePoints,
} from '@monochromatic-dev/translation-repair-markup/ts';
import {
  type ContainmentEdges,
  containsWords,
} from './consistency-contains-words.ts';
import { foldedWordsOf, } from './consistency-fold.ts';
import {
  type QuotesByLabel,
  quotesByLabelOf,
} from './consistency-majority.ts';
import type { ConsistencyVoice, } from './consistency-count-types.ts';

//region Consistency span agreement
// WHICH WORDS OF A PLACE THE JUDGES' QUOTES AGREE ON, when they quoted different spans of it (the lead's ruling of 2026-10-15 on
// the run of TianqiChen66625, group 2: three judges quoted three nested spans of the chosen place, and no one text was a strict
// majority of the quotes, so the chosen place was located by none and the group was lost to `chosen-unlocated`).
//
// A QUOTE IS EVIDENCE FOR EVERY QUOTE IT CONTAINS: the longer quote `wanted to stay` supports the shorter candidate `to stay`, and the
// shorter supports no longer one. A candidate span is one of the place's quotes, read in the fold (`consistency-fold.ts`), that the
// quotes of more than half of all the usable ballots contain, itself included, at the edges of words; the count is over every
// usable ballot and not over the quotes of the place, so a ballot that gave no quote of the place counts against every candidate,
// as it did when one text had to be given by a strict majority of the usable ballots. The place is located by the longest candidate that stands exactly once in its
// English; a candidate that stands nowhere or more than once is passed over for the next longest. The slot words of a frame are
// agreed by the same containment (`agreedSlotWordsOf`, `consistency-majority.ts`), through the one test of it
// (`consistency-contains-words.ts`). THE RISK, which the owner may veto: the span located is the
// longest one a majority can stand behind, so it can be SHORTER than the words a judge meant to carry (group 2's first place
// is located from "wanted", leaving out the "She also" a judge's quote began with), and a splice of it then moves fewer words
// than that judge named. Two candidates of one length are told apart by their code points, so the same ballots always give the
// same span.

/**
 The folded words of the quotes of a place that the quotes of more than half of all the usable ballots contain, longest first;
 a ballot that quoted nothing of the place counts against every candidate.

 @param voices - usable ballots

 @param label - the place's label

 @param quotesByLabel - the quote entries of these ballots by label, which a caller that asks many places reads once for the
 group; read from the ballots where absent

 @param edges - which joints of a longer word bar a quote from containing another: the locator's test of a joint, or none but the
 letters beside the match, as a recount of an artifact written before the locator's test counted reads it; the locator's where absent

 @returns The candidate spans, longest first and by code points among those of one length; none when the ballots quoted nothing
 for the place or no quote is contained in the quotes of more than half of the ballots; a blank quote is a candidate only when
 a strict majority of the ballots gave a blank one

 @example
 ```ts
 const candidates = containedQuoteWordsOf({ voices, label: 'OCCURRENCE 1', },);
 ```
 */
export function containedQuoteWordsOf(
  {
    voices,
    label,
    quotesByLabel = quotesByLabelOf({ voices, },),
    edges = 'locator',
  }: {
    readonly voices: readonly ConsistencyVoice[];
    readonly label: string;
    readonly quotesByLabel?: QuotesByLabel;
    readonly edges?: ContainmentEdges;
  },
): readonly string[] {
  /**
   Every quote naming the place, in the fold.
   */
  const quoted = (quotesByLabel.get(label,) ?? [])
    .map(function foldedOf(entry,): string {
      return foldedWordsOf({ text: entry.quote, },);
    },);
  return [...new Set(quoted,),]
    .filter(function isContainedByMajority(candidate,): boolean {
      /**
       How many of the quotes contain the candidate.
       */
      const containing = quoted.filter(function contains(outer,): boolean {
        return containsWords({
          outer,
          inner: candidate,
          edges,
        },);
      },)
        .length;
      return (containing * 2) > voices.length;
    },)
    .toSorted(function longestFirst(
      left,
      right,
    ): number {
      return (codePointLength({ text: right, },) - codePointLength({ text: left, },)) || compareCodePoints({
        left,
        right,
      },);
    },);
}

//endregion Consistency span agreement
