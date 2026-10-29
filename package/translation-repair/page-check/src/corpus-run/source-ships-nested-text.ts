import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import type { TextEdit, } from './source-ships-ranges.ts';

//region Source ships nested text
// THE CHARACTERS OF A NESTED BLOCK AND WHERE AN EDIT OF IT ENDS in the whole text (`source-ships-nested.ts`).

/**
 The characters of a text at the indices.

 @param text - text

 @param indices - indices of the characters, in order

 @returns Characters joined

 @example
 ```ts
 const body = charactersAt({ text: 'abc', indices: [0, 2,], },);
 ```
 */
export function charactersAt(
  {
    text,
    indices,
  }: {
    readonly text: string;
    readonly indices: readonly number[];
  },
): string {
  return indices
    .map(function characterAt(at,): string {
      return text.charAt(at,);
    },)
    .join('',);
}

/**
 Where an edit of a nested text ends in the whole text.

 An edit that takes a line ending away (a line joined to the one before it) takes with it what the nested text gave up at
 the start of the next line, the marks of a block quote and the indentation, which would otherwise stand in the middle of
 the joined line as characters of the text.

 @param source - whole text

 @param indices - indices of the whole text that make up the nested text, in order

 @param edit - edit at the offsets of the nested text

 @returns Offset in the whole text after the last character the edit replaces

 @example
 ```ts
 const end = endOfEdit({ source, indices: [4, 5, 8,], edit: { start: 1, end: 2, replacement: ' ', }, },);
 ```
 */
export function endOfEdit(
  {
    source,
    indices,
    edit,
  }: {
    readonly source: string;
    readonly indices: readonly number[];
    readonly edit: TextEdit;
  },
): number {
  /**
   Index of the last character the edit replaces.
   */
  const last = nonNullishOrThrow(indices[edit.end - 1],);
  /**
   Index of the first character of the nested text after it, absent at the end of the nested text.
   */
  const next = indices[edit.end];
  /**
   Whether the last character it replaces is a line ending.
   */
  const endsLine = source.charAt(last,) === '\n';
  /**
   What the edit writes in place of what it replaces.
   */
  const { replacement, } = edit;
  /**
   Whether it ends with a line ending of its own.
   */
  const writesBreak = replacement.endsWith('\n',);
  /**
   Whether the edit takes a line ending away and writes none in its place.
   */
  const takesBreak = endsLine && (!writesBreak);
  if (takesBreak && (next !== undefined))
    return next;
  return last + 1;
}

//endregion Source ships nested text
