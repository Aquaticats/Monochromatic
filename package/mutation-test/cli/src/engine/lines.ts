/**
 Offset to line and column conversion for mutant reporting.
 
 @example
 ```ts
 const table = lineStarts('a\nb');
 positionAt({ table, offset: 2 });
 // { line: 2, column: 0 }
 ```
 */

/**
 Computes start offsets of every line in source order.
 
 @param source - Source text.
 
 @returns Ascending offsets, first entry always zero.
 
 @example
 ```ts
 lineStarts('a\nb');
 // [0, 2]
 ```
 */
export function lineStarts(source: string,): readonly number[] {
  return (function scanNewlines(): readonly number[] {
    /**
     Accumulated line start offsets.
     */
    const starts: number[] = [0,];
    /**
     UTF-16 offset of the next newline; spreading source into code points
     would drift offsets after astral characters.
     */
    let newlineAt = source.indexOf('\n',);

    while (newlineAt !== (-1)) {
      starts.push(newlineAt + 1,);
      newlineAt = source.indexOf(
        '\n',
        newlineAt + 1,
      );
    }

    return starts;
  })();
}

/**
 Converts one offset to a one-based line and zero-based column.
 
 Binary search over line starts keeps conversion cheap when a file
 produces many mutants.
 
 @param options - Line start table and target offset.
 
 @returns One-based line, zero-based column.
 
 @example
 ```ts
 positionAt({ table: lineStarts('a\nb'), offset: 2 });
 // { line: 2, column: 0 }
 ```
 */
export function positionAt(options: {
  readonly table: readonly number[];
  readonly offset: number;
},): {
  readonly line: number;
  readonly column: number;
} {
  /**
   Index of the line containing the offset, by binary search.
   */
  const lineIndex = (function searchLine(): number {
    /**
     Inclusive lower search bound.
     */
    let low = 0;
    /**
     Inclusive upper search bound.
     */
    let high = options.table
      .length
      - 1;

    while (low < high) {
      /**
       Midpoint candidate line index.
       */
      const mid = Math.ceil((low + high) / 2,);
      /**
       Line start offset at the midpoint candidate.
       */
      const midStart = options.table[mid];

      if ((midStart === undefined) || (midStart > options.offset))
        high = mid - 1;
      else
        low = mid;
    }

    return low;
  })();
  /**
   Start offset of the containing line.
   */
  const lineStart = options.table[lineIndex] ?? 0;

  return {
    line: lineIndex + 1,
    column: options.offset - lineStart,
  };
}
