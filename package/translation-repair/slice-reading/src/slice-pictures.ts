import type { ChunkPair, } from './chunk-document.ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { PairedReading, } from './image-reading-pair.ts';
import { photoReferences, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Slice pictures
// WHICH PICTURES ONE SLICE IS SHOWN, and what a stage is told about them.
//
// THE SLICE'S OWN AND ITS NEIGHBOURS'. Measured over the pinned corpus, 79 of
// 1260 slices name a picture on the source side and NONE names one on the target
// side without also naming it on the source, so the source's own references are
// the whole attachment surface. But `Zha_Ke/1` carries 3652 characters
// transcribing a letter that `Zha_Ke/2` names, one slice later, so a slice shown
// only its own pictures would be shown nothing about the one it transcribes.
//
// ONE SECTION EACH WAY, which is the width the fidelity window already uses and
// the width the phenomenon has: measured 2026-08-18 over 92 entries, every
// relocation pair in the corpus is adjacent and the longest run of flagged
// slices anywhere is three.
//
// A REFUSED READING BECOMES A FINDING, NOT A CAVEAT IN THE PROMPT. A stage told
// "there is a picture here and nobody could read it" is being handed a hedge it
// has to weigh, and it has no way to weigh it. It is told about the pictures
// that WERE read, and the run records the rest where a person reads findings.

/**
 How a picture's readings are labelled for the model shown them.
 */
const PICTURE_HEADING = 'PICTURE';

/**
 Side markers a reader writes at the head of each chat message.

 THE VOCABULARY `READING_INSTRUCTION` ASKS FOR, matched here as text so a
 context carrying any of them is led by the legend that says what they mean.
 */
const SIDE_MARKERS: readonly string[] = [
  '[left]',
  '[right]',
];

/**
 What a stage shown side-marked transcripts is told the markers mean.

 ONCE, AHEAD OF THE BLOCKS, and only when a block carries a marker (class
 thirty-three, 2026-09-16). A chat screenshot is taken by one of its two
 parties, so the right-hand bubbles are that party's own messages on every
 messenger this corpus shows; the reader records the side, and this line is
 what turns a side into a speaker for a judge or translator that never saw
 the picture. Without it Mio16's judges guessed, and swapped the opening
 speakers of a human translation that had them right.
 */
export const SIDE_LEGEND: string = 'In a chat, [right] marks messages sent from the phone that took the '
  + 'screenshot and [left] the other party\'s; [sticker] and [image] mark a picture sent in place of words.';

/**
 What one slice is shown about the pictures around it.

 @example
 ```ts
 const pictures: SlicePictures = { context: '', findings: [], };
 ```
 */
export type SlicePictures = {
  /**
   Readings rendered for a prompt, empty when nothing was corroborated.
   */
  readonly context: string;

  /**
   One line per picture no reading is available for, naming which picture and
   why, in the wording a scorecard can group.
   */
  readonly findings: readonly string[];
};

/**
 Pictures named by one slice and by the slices either side of it.

 @param slices - prepared slice pairs of one entry

 @param slicePosition - POSITION IN `slices`, never a stamped `sliceIndex`

 @returns Asset names in document order, each once

 @throws {@link RangeError} when `slicePosition` is not a position in `slices`,
 since an index stamped elsewhere would silently name no pictures and read as
 a slice that shows none

 @example
 ```ts
 const names = slicePictureNames({ slices, slicePosition, },);
 ```
 */
export function slicePictureNames(
  {
    slices,
    slicePosition,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly slicePosition: number;
  },
): readonly string[] {
  if ((!Number.isInteger(slicePosition,))
    || (slicePosition < 0)
    || (slicePosition >= slices.length)) {
    throw new RangeError(
      `slicePictureNames asked for slice ${String(slicePosition,)} of `
        + `${String(slices.length,)}: not a position in this entry. An index `
        + `stamped elsewhere would name no pictures here, which reads as a `
        + `slice that shows none.`,
    );
  }

  /**
   Slice whose picture window was requested.
   */
  const current = slices[slicePosition];
  if (current?.syntax === 'front-matter')
    return [];

  /**
   Names gathered so far, in order, each once.
   */
  const named = new Set<string>();

  for (const at of [
    slicePosition - 1,
    slicePosition,
    slicePosition + 1,
  ]) {
    /**
     That slice, absent at either end of the document.
     */
    const beside = slices[at];
    if ((beside === undefined) || (beside.syntax === 'front-matter'))
      continue;
    for (const reference of photoReferences({
      text: beside.source
        .text,
    },))
      named.add(reference.assetName,);
  }

  return [...named,];
}

/**
 Renders what is known about one slice's pictures, and names what is not.

 @param slices - prepared slice pairs of one entry

 @param slicePosition - POSITION IN `slices`

 @param readings - what reading produced per asset name, for this entry

 @returns Prompt block for corroborated readings, plus findings for the rest

 @throws {@link RangeError} by way of {@link slicePictureNames}

 @example
 ```ts
 const pictures = slicePictures({ slices, slicePosition, readings, },);
 ```
 */
export function slicePictures(
  {
    slices,
    slicePosition,
    readings,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly slicePosition: number;
    readonly readings: ReadonlyMap<string, PairedReading>;
  },
): SlicePictures {
  /**
   Pictures this slice and its neighbours show.
   */
  const names = slicePictureNames({
    slices,
    slicePosition,
  },);

  /**
   Rendered blocks, one per picture that was read.
   */
  const blocks: string[] = [];

  /**
   Findings, one per picture that was not.
   */
  const findings: string[] = [];

  for (const assetName of names) {
    /**
     What reading this picture produced, absent when the entry's readings were
     never gathered.
     */
    const reading = readings.get(assetName,);
    if (reading === undefined) {
      findings.push(`picture ${assetName}: not read`,);
      continue;
    }
    if (reading.kind === 'unavailable') {
      findings.push(`picture ${assetName}: no reading, ${reading.reason}`,);
      continue;
    }
    if (reading.kind === 'no-text') {
      // WORDED AS A FACT ABOUT THE PICTURE, not about the pipeline. Two thirds
      // of this corpus's pictures are photographs, so a slice showing one is
      // not missing evidence and whoever reads the finding should not go
      // looking for it.
      findings.push(`picture ${assetName}: carries no text`,);
      continue;
    }

    /**
     Each reader's transcription, named by the model that produced it, since
     agreement establishes that two readers describe the same picture rather
     than the same AMOUNT of it: the shorter vouches, the longer informs.
     */
    const transcriptions = reading.readings
      .map(function labelled(one,): string {
        return `${one.modelId}:\n${one.text}`;
      },);
    blocks.push(`${PICTURE_HEADING} ${assetName}\n${transcriptions.join('\n\n',)}`,);
  }

  /**
   Whether any rendered block carries a side marker, which is when the
   legend earns its place.
   */
  const sided = blocks.some(function carriesMarker(block,): boolean {
    return SIDE_MARKERS.some(function inBlock(marker,): boolean {
      return block.includes(marker,);
    },);
  },);

  return {
    context: (sided
      ? [
        SIDE_LEGEND,
        ...blocks,
      ]
      : blocks).join('\n\n',),
    findings,
  };
}

/**
 Every slice's picture block, keyed by the index its consumers read.

 TWO INDEX SPACES MEET HERE, which is the whole reason this exists.
 {@link slicePictures} takes a POSITION, because a window is defined by who sits
 either side in the array. A stage downstream of preparation holds no array: it
 holds rows stamped with `sliceIndex`, and it is known what happens
 when those two are assumed equal by someone holding neither.

 THEY ARE EQUAL, AND ENFORCED SO. `assertSliceIndexing` refuses any preparation
 whose slice at a position is stamped with a different index, so this reads the
 stamp rather than the position and gets the same number by a route that would
 break loudly if the invariant ever did.

 THE FINDINGS ARE DROPPED, deliberately. A refused reading is already reported
 once by the stage that windows it for itself, and a second stage reporting the
 same refusal would have a run count one unread picture twice.

 @param slices - prepared slice pairs of one entry, indexed as prepared

 @param readings - what reading produced per asset name, for this entry

 @returns Picture block per slice, keyed by stamped index, empty where a slice
 neighbours no readable picture

 @throws {@link RangeError} by way of {@link slicePictures}

 @throws Error - when two slices carry one stamped index, which
 `assertSliceIndexing` already forbids and which would otherwise silently drop
 one slice's pictures

 @example
 ```ts
 const contexts = slicePictureContexts({ slices, readings, },);
 ```
 */
export function slicePictureContexts(
  {
    slices,
    readings,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly readings: ReadonlyMap<string, PairedReading>;
  },
): ReadonlyMap<number, string> {
  /**
   One pair per slice, stamped index against rendered block.
   */
  const entries = slices.map(function nameSlicePictures(
    slice,
    slicePosition,
  ): readonly [
    number,
    string,
  ] {
    /**
     Block this slice would be shown, windowed positionally.
     */
    const rendered = slicePictures({
      slices,
      slicePosition,
      readings,
    },);

    return [
      slice.target
        .sliceIndex,
      rendered.context,
    ];
  },);

  /**
   Those pairs as a lookup.
   */
  const contexts = new Map(entries,);

  // BELT OVER BRACES, and the braces are `assertSliceIndexing`. A `Map` keeps
  // the last of any duplicate key without a word, and the loss would surface as
  // a producer shown the wrong slice's pictures rather than as a failure, which
  // is the quietest possible way to be wrong about what a passage depicts.
  if (contexts.size !== entries.length) {
    throw new Error(
      `slice pictures: ${String(entries.length,)} ${
        wordForCount({
          count: entries.length,
          one: 'slice',
          many: 'slices',
        },)
      } ${
        wordForCount({
          count: entries.length,
          one: 'carries',
          many: 'carry',
        },)
      } ${
        String(contexts.size,)
      } distinct stamped ${
        wordForCount({
          count: contexts.size,
          one: 'index',
          many: 'indices',
        },)
      }, so at least two name one slice and one slice's pictures were dropped`,
    );
  }

  return contexts;
}

//endregion Slice pictures
