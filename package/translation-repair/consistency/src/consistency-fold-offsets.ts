import { normalizePunctuation, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Consistency fold offsets
// THE FOLD THAT KEEPS ITS OFFSETS. The locator (`consistency-fold.ts`), the reading of the
// marks that join a word (`consistency-word-joints.ts`, `consistency-closing-marks.ts`) and the
// proof of an apostrophe (`consistency-proven-apostrophes.ts`) read an English in one fold: the
// evidence fold of `quote-normalize.ts`, a run of whitespace as one space, a blank line kept
// apart from a space because it is a paragraph boundary. A run of whitespace folds to one or two
// units, so the fold is not one unit to one, and it carries a map from each folded unit back to
// the stretch of the original it stands for. THIS FILE IS THE ONE PLACE THE FOLD IS MADE, so a
// reader of the folded English and a reader of the original name the same marks.

/**
 A text in the fold, with the offsets of the original each folded unit stands for.

 @example
 ```ts
 const folded: OffsetFoldedText = foldWithOffsets({ text: 'girl,\nwasn’t  she', },);
 ```
 */
export type OffsetFoldedText = {
  /**
   The folded text.
   */
  readonly folded: string;

  /**
   Offset in the original where the original stretch of each folded UTF-16
   unit starts.
   */
  readonly starts: readonly number[];

  /**
   Exclusive offset in the original where it ends.
   */
  readonly ends: readonly number[];

  /**
   The stretch of the original each folded UTF-16 unit stands for: the
   character before the fold, which tells a corner bracket and an opening
   curly mark from the straight mark they fold to.
   */
  readonly originals: readonly string[];
};

/**
 Whether a character is whitespace.

 @param character - one whole character

 @returns Whether it is whitespace

 @example
 ```ts
 const space = isWhitespace({ character: '\n', },); // true
 ```
 */
function isWhitespace({ character, }: { readonly character: string; },): boolean {
  // TRIM CHANGES A SINGLE CHARACTER ONLY WHEN IT IS WHITESPACE, which is the
  // question here (spacing); the character is not compared with the empty string.
  return character.trim() !== character;
}

/**
 Units a blank line folds to: two line breaks, so the folded text read again
 holds a blank line again.
 */
const BLANK_LINE = [
  '\n',
  '\n',
] as const;

/**
 Units any other run of whitespace folds to.
 */
const SPACE = [' ',] as const;

/**
 Folds a text and keeps the offsets of the original each folded unit stands
 for. A run of whitespace holding two line breaks or more is a blank line, a
 paragraph boundary, and folds to two line breaks; any other run folds to one
 space. Every unit of a run stands for the whole run.

 @param text - text in the original

 @returns The folded text and the offsets of the original each folded unit stands for

 @example
 ```ts
 const { folded, } = foldWithOffsets({ text: 'girl,\nwasn’t  she', },); // "girl, wasn't she"
 ```
 */
export function foldWithOffsets({ text, }: { readonly text: string; },): OffsetFoldedText {
  /**
   Folded units, each with the original stretch it stands for.
   */
  const units: {
    readonly unit: string;
    readonly start: number;
    readonly end: number
  }[] = [];
  /**
   Whitespace run being read: where it began and ended, and how many line breaks it holds.
   */
  const run = {
    open: false,
    start: 0,
    end: 0,
    breaks: 0,
  };
  /**
   Closes the whitespace run being read, folded to one unit.
   */
  function closeRun(): void {
    if (!run.open)
      return;
    for (const unit of (run.breaks >= 2) ? BLANK_LINE : SPACE) {
      units.push({
        unit,
        start: run.start,
        end: run.end,
      },);
    }
    run.open = false;
  }
  /**
   Offset in the original reached so far.
   */
  const reached = { at: 0, };
  for (const character of normalizePunctuation({ text, },)) {
    if (isWhitespace({ character, },)) {
      if (!run.open) {
        run.open = true;
        run.start = reached.at;
        run.breaks = 0;
      }
      run.end = reached.at + character.length;
      if (character === '\n')
        run.breaks += 1;
    }
    else {
      closeRun();
      for (
        let index = 0;
        index < character.length;
        index += 1
      ) {
        units.push({
          unit: character.charAt(index,),
          start: reached.at,
          end: reached.at + character.length,
        },);
      }
    }
    reached.at += character.length;
  }
  closeRun();
  return {
    folded: units.map(function unitOf({ unit, },): string {
      return unit;
    },)
      .join('',),
    starts: units.map(function startOf({ start, },): number {
      return start;
    },),
    ends: units.map(function endOf({ end, },): number {
      return end;
    },),
    originals: units.map(function originalOf({
      start,
      end,
    },): string {
      return text.slice(
        start,
        end,
      );
    },),
  };
}

/**
 The offsets in the fold of the units that stand at some offsets of the original.

 @param offsets - offsets in the original, such as the delimiters of its emphasis, which the fold keeps one unit to one
 (a character that is no white space)

 @param folded - the original in the fold

 @returns The offsets of the folded units whose stretch of the original starts at one of them; none where `offsets` holds none

 @example
 ```ts
 const inFold = offsetsInFold({ offsets: new Set([2,],), folded: foldWithOffsets({ text: 'a *b*', },), },); // holds 2
 ```
 */
export function offsetsInFold(
  {
    offsets,
    folded,
  }: {
    readonly offsets: ReadonlySet<number>;
    readonly folded: OffsetFoldedText;
  },
): ReadonlySet<number> {
  /**
   The offsets found, mutated only inside this function.
   */
  const found = new Set<number>();
  // An empty set of offsets names no unit, so the fold is not walked.
  if (offsets.size === 0)
    return found;
  /**
   Where in the original each folded unit starts.
   */
  const { starts, } = folded;
  starts.forEach(function isNamed(
    start,
    unit,
  ): void {
    if (offsets.has(start,))
      found.add(unit,);
  },);
  return found;
}

/**
 A text in the fold as the characters the edges of a span are read over: the folded characters, one whole character each, and
 for each the character as it was written, which tells a backslash escape from a backslash before a curly mark the fold turned
 straight.

 @param folded - the text in the fold, with the character each unit stood for

 @returns The folded characters and the written ones, a pair of lists of the same length

 @example
 ```ts
 const { originals, } = writtenPointsOf({ folded: foldWithOffsets({ text: 'a\\’', },), },); // ['a', '\\', '’']
 ```
 */
export function writtenPointsOf(
  { folded, }: { readonly folded: OffsetFoldedText; },
): {
  readonly points: readonly string[];
  readonly originals: readonly string[];
} {
  /**
   The folded text, one whole character each.
   */
  const points = Array.from(folded.folded,);
  /**
   The written character of each folded character, built in order and mutated only inside this function.
   */
  const originals: string[] = [];
  /**
   Offset in the folded text of the character being read, mutated only inside this function.
   */
  const reading = { unit: 0, };
  for (const point of points) {
    originals.push(folded.originals[reading.unit] ?? point,);
    reading.unit += point.length;
  }
  return {
    points,
    originals,
  };
}

//endregion Consistency fold offsets
