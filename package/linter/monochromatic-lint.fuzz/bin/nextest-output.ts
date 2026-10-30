/**
 Reading the status lines of a nextest run.
 */

/**
 Characters that make a letter run part of a word, so that `FAILED` is not the marker `FAIL`.
 */
const wordCharacters = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_';

/**
 Find what follows the bracket of a nextest status line.

 nextest prints `FAIL [ 0.012s] ( 3/38) <binary id> <test name>`, or `FAIL + LEAK [` when the failed test left a
 child process behind, and `PASS [` for a passing test.

 @param line - one line of nextest output

 @param marker - the status word that starts the line, after any indentation

 @returns the text after the first bracket as the only item, or no item when the line is not a line of that status

 @example
 ```ts
 afterBracket({ line: '    PASS [   0.004s] crate test_name', marker: 'PASS' });
 ```
 */
function afterBracket({
  line,
  marker,
}: {
  readonly line: string;
  readonly marker: string;
}): readonly string[] {
  /**
   The line without indentation.
   */
  const trimmed = line.trim();
  if (!trimmed.startsWith(marker)) return [];
  /**
   What follows the status word.
   */
  const rest = trimmed.slice(marker.length);
  if (wordCharacters.includes(rest.charAt(0)) && (rest !== '')) return [];
  /**
   Position of the bracket.
   */
  const bracket = rest.indexOf('[');
  if (bracket === (-1)) return [];
  return [rest.slice(bracket + 1)];
}

/**
 Name the test that a failing nextest line ends with.

 @param text - the text after the bracket of a `FAIL` line

 @returns the name as the only item, or no item when the line has no name after whitespace

 @example
 ```ts
 failedTestName(' 0.012s] ( 3/38) binary generators::control');
 ```
 */
function failedTestName(text: string): readonly string[] {
  /**
   The text without trailing whitespace.
   */
  const trimmed = text.trimEnd();
  /**
   Position of the whitespace before the last word.
   */
  const space = trimmed.lastIndexOf(' ');
  return space === (-1) ? [] : [trimmed.slice(space + 1)];
}

/**
 Name the failing tests of a nextest run.

 nextest repeats each failure in its closing recap, so the names are deduplicated.

 @param output - standard output and error of the run

 @returns each failing test name once, in the order it first appears

 @example
 ```ts
 failedControls('FAIL [ 0.012s] ( 3/38) binary generators::control');
 ```
 */
export function failedControls(output: string): readonly string[] {
  return [
    ...new Set(output.split('\n')
      .flatMap(function failedOf(line) {
        return afterBracket({
          line,
          marker: 'FAIL',
        });
      })
      .flatMap(failedTestName)),
  ];
}

/**
 Count the passing tests of a nextest run.

 @param output - standard output and error of the run

 @returns the number of `PASS` lines

 @example
 ```ts
 passedControls('PASS [ 0.004s] crate test_name');
 ```
 */
export function passedControls(output: string): number {
  return output.split('\n')
    .flatMap(function passedOf(line) {
      return afterBracket({
        line,
        marker: 'PASS',
      });
    })
    .length;
}
