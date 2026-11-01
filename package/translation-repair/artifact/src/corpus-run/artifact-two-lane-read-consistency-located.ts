import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  carriesOrCutsMarkup,
  type ConsistencyRendering,
  occurrencesOf,
  quoteCore,
  type UnlocatedReason,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import { rendersAsNothing, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { ArtifactConsistencyRendering, } from './artifact-two-lane-consistency.ts';

//region Artifact consistency located
// A RECORDED LOCATED RENDERING READ AS THE COUNTER'S, for the recount of what
// moving words over it would do, as a generation fifteen section recorded it
// (a judge's string; the sixteenth shape is read in
// `artifact-two-lane-read-consistency-span.ts`). The English the quote was
// located in is not in the file, so its offset is left at zero and the recount
// compares words only.

/**
 The recorded rendering of a place read as the counter's located rendering, for
 the recount of what moving words over it would do.

 @param rendering - recorded rendering, located

 @param path - where the rendering sits, for the parse error

 @returns The counter's form, its offset in the English left at zero since the
 recount compares words only

 @throws {@link ArtifactParseError} when a frame's slot does not stand once in
 the quote's words

 @example
 ```ts
 const located = asLocated({ rendering, path, },);
 ```
 */
export function asLocated(
  {
    rendering,
    path,
  }: {
    readonly rendering: Extract<ArtifactConsistencyRendering, { readonly kind: 'located'; }>;
    readonly path: string;
  },
): Extract<ConsistencyRendering, { readonly kind: 'located'; }> {
  /**
   The words the quote keeps.
   */
  const core = quoteCore({ quote: rendering.quote, },);
  if (rendering.slot === undefined) {
    return {
      kind: 'located',
      quote: rendering.quote,
      core,
      start: 0,
    };
  }
  /**
   Where the slot stands in the core.
   */
  const found = occurrencesOf({
    haystack: core,
    needle: rendering.slot,
  },);
  if (found.kind !== 'once') {
    throw new ArtifactParseError({
      path,
      reason: 'a slot that stands exactly once in the words of its quote',
    },);
  }
  return {
    kind: 'located',
    quote: rendering.quote,
    core,
    start: 0,
    slot: rendering.slot,
    slotStart: found.at,
  };
}

/**
 Whether the usable ballots contradict a reason given for a place that was not
 located. A reason is contradicted when the ballots say what it denies, and
 also when it could not have been reached: a quote reason beside no agreed
 quote, a slot reason in a repeated run or beside no agreed slot.

 @param reason - the recorded reason

 @param quotes - the quote a strict majority gave, none when no text had a majority

 @param slots - the slot a strict majority gave beside that quote, none when no text had a majority

 @param framed - whether the group is a frame with a slot

 @returns Whether the ballots say what the reason denies, or the reason names a step the pass never reached

 @example
 ```ts
 const wrong = reasonContradicted({ reason: 'no-agreement', quotes: ['the cat naps',], slots: [], framed: false, },);
 ```
 */
export function reasonContradicted(
  {
    reason,
    quotes,
    slots,
    framed,
  }: {
    readonly reason: UnlocatedReason;
    readonly quotes: readonly string[];
    readonly slots: readonly string[];
    readonly framed: boolean;
  },
): boolean {
  if (reason === 'not-asked')
    return true;
  if (reason === 'no-english')
    return false;
  /**
   The quote a strict majority gave, absent when no text had one.
   */
  const [quote,] = quotes;
  /**
   The slot a strict majority gave beside it.
   */
  const [slot,] = slots;
  if (reason === 'no-agreement')
    return quote !== undefined;
  if (quote === undefined)
    return true;
  /**
   The words the quote keeps.
   */
  const core = quoteCore({ quote, },);
  if (reason === 'empty')
    return !rendersAsNothing({ text: core, },);
  if (rendersAsNothing({ text: core, },))
    return true;
  if ((reason === 'not-found') || (reason === 'ambiguous')
    || (reason === 'overlaps-another'))
    return false;
  if (reason === 'markup') {
    return !(carriesOrCutsMarkup({ text: quote, },)
      || (framed && (slot !== undefined)
        && carriesOrCutsMarkup({ text: slot, },)));
  }
  // A SLOT REASON comes after a quote that stood once and held its markup in pairs.
  if ((!framed) || carriesOrCutsMarkup({ text: quote, },))
    return true;
  if (reason === 'slot-no-agreement')
    return slot !== undefined;
  if (slot === undefined)
    return true;
  if (reason === 'slot-empty')
    return !rendersAsNothing({ text: slot, },);
  if (rendersAsNothing({ text: slot, },))
    return true;
  /**
   Where the slot stands in the quote's words.
   */
  const inCore = occurrencesOf({
    haystack: core,
    needle: slot,
  },);
  if (reason === 'slot-not-found')
    return inCore.kind !== 'none';
  if (reason === 'slot-ambiguous')
    return inCore.kind !== 'many';
  return slot !== core;
}

/**
 Whether the words of a located rendering are ones the pass would have located:
 they show a reader something, hold their markup in pairs, and in a frame leave
 words of the frame beside the slot.

 @param rendering - recorded rendering, located

 @param framed - whether the group is a frame with a slot

 @returns Whether the pass would have left the place unlocated instead

 @example
 ```ts
 const wrong = locatedContradicted({ rendering, framed: false, },);
 ```
 */
export function locatedContradicted(
  {
    rendering,
    framed,
  }: {
    readonly rendering: Extract<ArtifactConsistencyRendering, { readonly kind: 'located'; }>;
    readonly framed: boolean;
  },
): boolean {
  /**
   The words the quote keeps.
   */
  const core = quoteCore({ quote: rendering.quote, },);
  if (rendersAsNothing({ text: core, },) || carriesOrCutsMarkup({ text: rendering.quote, },))
    return true;
  if ((!framed) || (rendering.slot === undefined))
    return false;
  return rendersAsNothing({ text: rendering.slot, },) || (rendering.slot === core)
    || carriesOrCutsMarkup({ text: rendering.slot, },);
}

//endregion Artifact consistency located
