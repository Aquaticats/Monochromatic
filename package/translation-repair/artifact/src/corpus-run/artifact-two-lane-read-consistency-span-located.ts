import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  carriesOrCutsMarkup,
  ConsistencyInvariantError,
  type ConsistencyRendering,
  holdsEdgeApostrophe,
  isSingleMark,
  locateWords,
  type UnlocatedReason,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import { rendersAsNothing, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  artifactGenerationReadsLineEndingsAsCodePadding,
  type TwoLaneArtifactGeneration,
} from './artifact-two-lane-contract.ts';
import type { ArtifactConsistencyRendering, } from './artifact-two-lane-consistency.ts';

//region Artifact consistency span located
// A RECORDED RENDERING OF THE SIXTEENTH SHAPE READ AGAINST THE BALLOTS. The
// quote of a located place is the words the place's English holds where the
// majority's quotes stand, so it is not compared with a judge's string: its
// folded words (`consistency-fold.ts`) are compared with the folded words a
// strict majority of the usable ballots quoted. The English the span stands in
// is not in the file, so "the words stand exactly once there" is the pass's
// check alone, as it is for the first shape. The single marks a generation
// seventeen place records beside its span (`markBefore`, `markAfter`) are read
// as recorded for the same reason: nothing in the file holds the English they
// stood in to check them against, and the occurrence reader
// (`artifact-two-lane-read-consistency-occurrence.ts`) has already checked that
// each is one single mark.
//
// THE SLOT IS FOUND WITH THE DEFINITIONS THE PLACE RECORDS, from generation eighteen. The pass finds a slot inside its span with the
// link and footnote definitions of the whole page, and the file holds no page to rebuild them from (the head of
// `artifact-two-lane-read-consistency-span.ts` says why, ledger B661), so the pass records the definitions that move the slot
// (`definitions`, ledger B665) and the slot is located here with them. A rendering of generation seventeen or earlier records none, and
// its slot is located as the span's own text reads: a shortcut reference in the span whose definition stands in another slice is
// plain text there, and a recorded slot at its joint can be refused although the pass found it, as the recount can expect a slot the
// pass could not find. That is the limit of those generations, which are not written any more.
//
// A GENERATION SEVENTEEN QUOTE MAY END OR BEGIN WITH ONE APOSTROPHE the ballots'
// folded words lack: the span takes in the proven apostrophe beside the words
// (`the parents’`, `’round the`; `consistency-proven-apostrophes.ts`), and the record
// stores the span as the quote with no new field. The reader accepts such a quote and
// checks what the file can show: at most one of the three apostrophe-like marks at
// each edge, directly beside a letter or a digit, never the opening curly single mark
// (`edgeMarksContradictedBySpan`). A READER HOLDS NO ENGLISH AND CANNOT VERIFY THAT THE
// MARK IS PROVEN, that no single quotation character of the page's prose could pair with it, any more
// than it can verify that the words stand once in the English; that is the pass's
// check alone. Generations fifteen and sixteen are read as before, with neither
// the check nor the planner's reading of the mark.

/**
 The recorded rendering read as the counter's located rendering.

 @param rendering - recorded rendering, located

 @param path - where the rendering sits, for the parse error

 @param generation - generation the section was written under, which says whether line endings pad the inline code of the words when
 a recorded slot is located in them (from nineteen) or a literal space alone does

 @returns The counter's form, with the recorded offset and, for a frame that
 has one, the slot and its offset in the words, and the definitions it was located with; the slot is located with the recorded
 definitions, and with none of the page's where the rendering records none (the head of this file says why)

 @throws {@link ArtifactParseError} when a recorded slot does not stand exactly
 once in the words of its quote

 @throws {@link ConsistencyInvariantError} when the rendering records no offset,
 which every rendering of the sixteenth shape does

 @example
 ```ts
 const located = asLocatedBySpan({ rendering, path, generation: 19, },);
 ```
 */
export function asLocatedBySpan(
  {
    rendering,
    path,
    generation,
  }: {
    readonly rendering: Extract<ArtifactConsistencyRendering, { readonly kind: 'located'; }>;
    readonly path: string;
    readonly generation: TwoLaneArtifactGeneration;
  },
): Extract<ConsistencyRendering, { readonly kind: 'located'; }> {
  if (rendering.start === undefined) {
    throw new ConsistencyInvariantError({ invariant:
      `the located rendering at ${path} records no offset, and the reader of the sixteenth shape requires one`, },);
  }
  /**
   The counter's form of a rendering without a slot.
   */
  const bare: Extract<ConsistencyRendering, { readonly kind: 'located'; }> = {
    kind: 'located',
    quote: rendering.quote,
    core: rendering.quote,
    start: rendering.start,
    ...((rendering.markBefore === undefined) ? {} : { markBefore: rendering.markBefore, }),
    ...((rendering.markAfter === undefined) ? {} : { markAfter: rendering.markAfter, }),
    ...((rendering.definitions === undefined) ? {} : { definitions: rendering.definitions, }),
  };
  if (rendering.slot === undefined)
    return bare;
  /**
   Where the recorded slot stands in the words.
   */
  const inside = locateWords({
    haystack: rendering.quote,
    words: rendering.slot,
    apostrophes: 'outside',
    codePadding: artifactGenerationReadsLineEndingsAsCodePadding({ generation, },) ? 'spaces-and-line-endings' : 'spaces',
    ...((rendering.definitions === undefined) ? {} : { definitions: rendering.definitions, }),
  },);
  if (inside.kind !== 'once') {
    throw new ArtifactParseError({
      path,
      reason: 'a slot that stands exactly once in the words of its quote',
    },);
  }
  return {
    ...bare,
    slot: rendering.slot,
    slotStart: inside.start,
  };
}

/**
 Whether the recorded words of a located rendering hold at an edge a single mark
 the pass could not have put there. The words a quote names carry no single mark at
 their edges, and the only one a generation seventeen span takes in is a proven
 apostrophe (`consistency-proven-apostrophes.ts`): one of the three apostrophe-like
 marks with a letter or a digit directly inside it, at most one at each edge. The
 opening curly single mark, two marks in a row, and a mark with a space or any other
 character inside it are marks the pass leaves outside the span. THIS CHECK CANNOT
 SAY THAT THE MARK IS PROVEN: that depends on the single quotation characters of the
 prose of the place's English, which the file does not hold.

 @param rendering - recorded rendering, located

 @returns Whether the words start or end with a single mark that is no apostrophe beside a letter or a digit

 @example
 ```ts
 const wrong = edgeMarksContradictedBySpan({ rendering, },);
 ```
 */
export function edgeMarksContradictedBySpan(
  { rendering, }: { readonly rendering: Extract<ArtifactConsistencyRendering, { readonly kind: 'located'; }>; },
): boolean {
  /**
   The whole characters of the recorded words.
   */
  const points = Array.from(rendering.quote,);
  return ([
    'start',
    'end',
  ] as const).some(function refusesEdge(side,): boolean {
    return isSingleMark({ character: ((side === 'start') ? points[0] : points.at(-1,)) ?? '', },)
      && (!holdsEdgeApostrophe({
        text: rendering.quote,
        side,
      },));
  },);
}

/**
 Whether the usable ballots contradict a reason given for a place that was not
 located. A reason is contradicted when the ballots say what it denies, and
 also when it could not have been reached: a reason for a quote beside no agreed
 quote.

 @param reason - the recorded reason

 @param words - the folded words a strict majority quoted, none when no words had a majority

 @returns Whether the ballots say what the reason denies, or the reason names a step the pass never reached

 @example
 ```ts
 const wrong = reasonContradictedBySpan({ reason: 'no-agreement', words: ['the cat naps',], },);
 ```
 */
export function reasonContradictedBySpan(
  {
    reason,
    words,
  }: {
    readonly reason: UnlocatedReason;
    readonly words: readonly string[];
  },
): boolean {
  if (reason === 'not-asked')
    return true;
  if (reason === 'no-english')
    return false;
  /**
   The folded words a strict majority quoted, absent when none had one.
   */
  const [agreed,] = words;
  if (reason === 'no-agreement')
    return agreed !== undefined;
  if (agreed === undefined)
    return true;
  if (reason === 'empty')
    return !rendersAsNothing({ text: agreed, },);
  if (rendersAsNothing({ text: agreed, },))
    return true;
  // WHERE THE WORDS STAND IN THE ENGLISH, AND WHAT THE ENGLISH HOLDS THERE, IS
  // NOT IN THE FILE: not-found, ambiguous, markup and overlaps-another cannot
  // be contradicted, and the slot reasons are refused when the reason is read.
  return !((reason === 'not-found') || (reason === 'ambiguous')
    || (reason === 'markup')
    || (reason === 'overlaps-another'));
}

/**
 Whether the words of a located rendering are ones the pass would have located:
 they show a reader something, hold their markup in pairs, and a slot, where
 there is one, shows something, is not all of the words and holds its markup in
 pairs.

 @param rendering - recorded rendering, located

 @returns Whether the pass would have left the place unlocated, or without a slot, instead

 @example
 ```ts
 const wrong = locatedContradictedBySpan({ rendering, },);
 ```
 */
export function locatedContradictedBySpan(
  { rendering, }: { readonly rendering: Extract<ArtifactConsistencyRendering, { readonly kind: 'located'; }>; },
): boolean {
  if (rendersAsNothing({ text: rendering.quote, },) || carriesOrCutsMarkup({ text: rendering.quote, },))
    return true;
  if (rendering.slot === undefined)
    return false;
  return rendersAsNothing({ text: rendering.slot, },) || (rendering.slot === rendering.quote)
    || carriesOrCutsMarkup({ text: rendering.slot, },);
}

//endregion Artifact consistency span located
