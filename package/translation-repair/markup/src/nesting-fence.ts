import {
  characterRunEnd,
  isOneOf,
} from './nesting-line-lexing.ts';

//region Nesting fence
// READS A LINE AS A FENCE, in the one way the scan may skip what a fence holds:
// only a fence the parser certainly reads, and the closing line the grammar
// closes it by or a looser one, since closing early only counts more.

/**
 Fewest fence characters in a run that opens or closes a fenced block.
 */
const FENCE_MINIMUM = 3;

/**
 A run of fence characters that opens or closes a fenced block.
 */
export type FenceRun = {
  /**
   Character the run is made of, the empty string for no fence.
   */
  readonly character: string;

  /**
   How many of them there are, zero for no fence.
   */
  readonly length: number;
};

/**
 The reading of a line that is no fence.
 */
export const NO_FENCE: FenceRun = {
  character: '',
  length: 0,
};

/**
 Reads a line as a fence marker.

 @param line - one line with its container markers and indentation removed

 @returns The fence run it begins with, or none

 @example
 ```ts
 fenceOf({ line: '```ts', },);
 // => { character: '`', length: 3, }
 ```
 */
export function fenceOf({ line, }: { readonly line: string; },): FenceRun {
  /**
   Character the fence is made of.
   */
  const character = line[0] ?? '';
  if (!isOneOf({
    character,
    set: '`~',
  },))
    return NO_FENCE;
  /**
   Index after the run of that character.
   */
  const end = characterRunEnd({
    line,
    from: 0,
    set: character,
  },);
  if (end < FENCE_MINIMUM)
    return NO_FENCE;
  return {
    character,
    length: end,
  };
}

/**
 Reads a line as the opening of a fence the parser certainly reads.

 CERTAIN MEANS THE LINE BEGINS WITH THE FENCE. A fence after any blank or any
 container marker may sit inside a quotation or a list item that a later line
 leaves, and a fence indented four columns is indented code, and where it
 interrupts nothing the parser's reading depends on columns this scan does
 not keep; the scan counts the lines after those. A backtick fence whose info
 string holds a backtick is no fence (`micromark-core-commonmark@2.0.4`
 `lib/code-fenced.js`, `info` and `meta`), so it is none here.

 @param line - one whole line, container markers and indentation included

 @returns The fence run the line opens a block with, or none

 @example
 ```ts
 certainFenceOf({ line: '```ts', },);
 // => { character: '`', length: 3, }
 certainFenceOf({ line: '```a`b', },);
 // => { character: '', length: 0, }
 ```
 */
export function certainFenceOf({ line, }: { readonly line: string; },): FenceRun {
  /**
   The run the line begins with.
   */
  const run = fenceOf({ line, },);
  /**
   Whether the info string and the meta after the run hold a backtick.
   */
  const infoHoldsBacktick = line
    .slice(run.length,)
    .includes('`',);
  if ((run.character === '`') && infoHoldsBacktick)
    return NO_FENCE;
  return run;
}

//endregion Nesting fence
