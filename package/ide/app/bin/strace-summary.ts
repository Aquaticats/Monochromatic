// The columns of one `strace --summary-only` row, read in one pass so the counters of the idle-cost measurement can
// take the columns they need.
// What: the words of a row, with no empty word for the whitespace between or around them, and the shape of the
// columns a call row states.
// Why: the measurement collected the row through `trim().split(/\s+/)` and tested two of its columns through
// patterns, and the repository lint bans a pattern where a walk over the row reads the same text.
// Tested in strace-summary.test.ts.
import { isDecimalNumberCharacter, isWhitespace } from './scan-characters.ts';

/**
 Columns one summary row holds.
 
 A row that holds only whitespace answers no columns, where a pattern split answered one empty column; the caller
 asks for at least the five columns a call row states, so neither answer reaches its counters.
 
 @param line - One line of `strace --summary-only` output.
 
 @returns The columns of that line, in the order strace printed them.
 
 @example
 ```ts
 reportedColumns(' 65.42    0.000123      4    123  openat'); // ['65.42', '0.000123', '4', '123', 'openat']
 ```
 */
export function reportedColumns(line: string): string[] {
  const columns: string[] = [];
  let cursor = 0;
  while (cursor < line.length) {
    while (cursor < line.length && isWhitespace(line[cursor] ?? '')) cursor++;
    const columnStart = cursor;
    while (cursor < line.length && !isWhitespace(line[cursor] ?? '')) cursor++;
    if (cursor > columnStart) columns.push(line.slice(columnStart, cursor));
  }
  return columns;
}

/**
 Whether a column states a plain decimal number strace prints.
 
 @param column - One column of a summary row.
 
 @returns Whether every character of the column is an ASCII digit or the decimal point, and there is one.
 
 @example
 ```ts
 isPrintedDecimal('0.000123'); // true
 isPrintedDecimal('1e3'); // false
 ```
 */
export function isPrintedDecimal(column: string): boolean {
  if (column.length === 0) return false;
  for (const character of column) {
    if (!isDecimalNumberCharacter(character)) return false;
  }
  return true;
}
