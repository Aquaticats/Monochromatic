import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Archive composite lines
// Split out of `archive-composite-revision.ts` when the composite's evidence (ledger B716) took it over its line
// budget: the line arithmetic and the words a log line says it in.

/**
 Lines of a text, each with the line ending that closes it.

 @param text - block or revision

 @returns Lines whose concatenation is the text

 @example
 ```ts
 linesOf({ text: 'a\nb', },); // ['a\n', 'b']
 ```
 */
export function linesOf({ text, }: { readonly text: string; },): readonly string[] {
  /**
   Text cut at its line feeds, each piece without the one that closed it.
   */
  const pieces = text.split('\n',);
  /**
   Every piece that a line feed closed, with it put back.
   */
  const closed = pieces
    .slice(
    0,
    -1,
    )
    .map(function withEnding(piece,): string {
      return `${piece}\n`;
    },);
  // A text that ends in a line feed has no line after it.
  return text.endsWith('\n',) ? closed : [
    ...closed,
    pieces.at(-1,) ?? '',
  ];
}

/**
 A count with its noun, as a log line says it.

 @param count - how many

 @param one - noun for one

 @param many - noun for any other count

 @returns The count and its noun

 @example
 ```ts
 counted({ count: 2, one: 'line', many: 'lines', },); // '2 lines'
 ```
 */
export function counted(
  {
    count,
    one,
    many,
  }: {
    readonly count: number;
    readonly one: string;
    readonly many: string;
  },
): string {
  return `${String(count,)} ${
    wordForCount({
      count,
      one,
      many,
    },)
  }`;
}

/**
 Line numbers as a log line says them, counting from one.

 @param indexes - zero-based line positions

 @returns "line 4" for one, "lines 1, 2" for several

 @example
 ```ts
 numbered({ indexes: [0, 1,], },); // 'lines 1, 2'
 ```
 */
export function numbered({ indexes, }: { readonly indexes: readonly number[]; },): string {
  return `${
    wordForCount({
      count: indexes.length,
      one: 'line',
      many: 'lines',
    },)
  } ${
    indexes.map(function oneBased(index,): string {
      return String(index + 1,);
    },)
      .join(', ',)
  }`;
}

/**
 Positions of the lines two revisions both change.

 @param left - first revision's changed positions

 @param right - second revision's changed positions

 @returns Shared positions in ascending order

 @example
 ```ts
 sharedLines({ left: [0, 2,], right: [2,], },); // [2]
 ```
 */
export function sharedLines(
  {
    left,
    right,
  }: {
    readonly left: readonly number[];
    readonly right: readonly number[];
  },
): readonly number[] {
  return left.filter(function inBoth(index,): boolean {
    return right.includes(index,);
  },);
}

//endregion Archive composite lines
