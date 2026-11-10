// The microsecond count a log record states it took, read from the line the record printed.
// What: the digits after `elapsed_us=`, as the number of microseconds that record reports, and absence when the line
// states none.
// Why: the bundle checks print how long unpacking a parser took and how long reusing one took, and compare them
// between starts, so a line that states no count must read as no measurement rather than as zero.
// The read is one linear pass and is tested in logged-elapsed-time.test.ts.

/**
 The field a record states its microseconds with.
 */
const elapsedField = 'elapsed_us=';

/**
 Whether a character is an ASCII digit, the only kind the count is written with.
 *
 @param character - one character of the line.
 *
 @returns whether the character is in `0` to `9`.
 */
const isDigit = (character: string): boolean => (character >= '0') && (character <= '9');

/**
 The microseconds the first `elapsed_us=` field of `line` states, or absence when the line states none.
 *
 The leftmost field that states digits is the one read, as a pattern without the global flag finds the leftmost match
 and stops: a field written without digits after it is skipped, as the pattern's own search skipped it.
 *
 @param line - one line of a session's output.
 *
 @returns The stated microseconds, or absence when the line states none.
 */
export const loggedElapsedMicroseconds = (line: string): number | undefined => {
  for (let field = line.indexOf(elapsedField); field >= 0; field = line.indexOf(elapsedField, field + 1)) {
    const digitsStart = field + elapsedField.length;
    let digitsEnd = digitsStart;
    while (digitsEnd < line.length && isDigit(line[digitsEnd] ?? '')) {
      digitsEnd++;
    }
    if (digitsEnd > digitsStart) {
      return Number(line.slice(digitsStart, digitsEnd));
    }
  }
  return undefined;
};
