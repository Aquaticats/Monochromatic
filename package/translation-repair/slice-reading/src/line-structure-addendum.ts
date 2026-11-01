import { BILINGUAL_LINE_CLAUSE, } from '@monochromatic-dev/translation-repair-house-rule/ts';

//region Line structure addendum
// Turns the computed line-structure fact into the sentence the editor is given.
// Split from `repair-chunk` because it is a pure text decision and that file is
// at its line budget.
//
// DECIDED BY THE CALLER, on the enclosing CHUNK, not here on the slice. See
// `buildEditorAddendum` for why the unit matters more than the predicate.

/**
 Instruction added when the source slice is line-structured.

 STATED AS A FACT rather than as a condition the model must evaluate. The
 standing rule already asks the editor to recognise line-structured text
 itself, and an attempt to write that same recognition as a heuristic failed
 its positive control, so leaving the judgement to a model is not obviously
 safer than measuring it.

 SPEAKS ABOUT THE ORIGINAL, because that is the side the predicate reads. An
 earlier wording opened `This region's CURRENT TEXT IS line-structured`, which
 on the case this exists for is false: `Toka_ls`'s verse chunk is 21 source
 blocks at median 22 against 18 target blocks at median 101. It told the
 editor something untrue about the text in front of it, and then asked for one
 output line per INPUT line, which on an already-merged translation asks for
 the merged shape to be preserved.

 NAMES THE OBSERVED FAILURE rather than only the desired shape. The editor did
 not merely reflow `Toka_ls`: it replaced three correctly translated lines with
 invented text, one of them with a correct translation of a DIFFERENT line.
 Forbidding reflow would not have covered that, so both are stated.
 */
const LINE_STRUCTURE_RULE = 'This region\'s ORIGINAL IS line-structured: each '
  + 'original line is a unit. Keep every existing line in place and in order, '
  + 'recast only within a line, and never invent a line, drop a line, or fill '
  + `one line with content belonging to another. ${BILINGUAL_LINE_CLAUSE}`;

/**
 Builds the editor rule addendum for one slice.

 TAKES THE VERDICT, DOES NOT COMPUTE IT, because the slice is the wrong unit to
 compute it on. `isLineStructured` needs at least five blocks before it will
 answer anything but false, and slicing cuts a verse section into pieces
 smaller than that. Measured on `Toka_ls`: the verse chunk is line-structured
 at 21 blocks, median 22, and subdivides into seven slices of which ONE still
 trips the predicate. Four of the other six sit at medians 20, 22, 23 and 29,
 squarely inside the verse range, and fail only for want of a fifth block.

 So the caller decides on the enclosing chunk and every slice carved from it
 inherits the answer. Subdivision is the pipeline's own choice and cannot
 change whether the original is verse.

 @param baseAddendum - configured addendum, possibly empty

 @param lineStructured - whether the enclosing chunk's ORIGINAL is
 line-structured; measured on the source because the translation may already
 have merged the lines that make it verse, and on `Toka_ls` the Chinese chunk
 sits at median 22 against the English rendering's 101

 @returns Addendum, empty when there is nothing to add

 @example
 ```ts
 const addendum = buildEditorAddendum({ baseAddendum: '', lineStructured: true, },);
 ```
 */
export function buildEditorAddendum(
  {
    baseAddendum,
    lineStructured,
  }: {
    readonly baseAddendum: string;
    readonly lineStructured: boolean;
  },
): string {
  return [
    baseAddendum,
    lineStructured ? LINE_STRUCTURE_RULE : '',
  ]
    .filter(function isPresent(part,): boolean {
      return part !== '';
    },)
    .join('\n',);
}

//endregion Line structure addendum
