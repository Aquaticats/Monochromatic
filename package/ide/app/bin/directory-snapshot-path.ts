// The directory one refresh-log line reports having read, taken from the `path=` field it prints.
// What: the run of characters after that field up to the first whitespace, or undefined when the line names none.
// Why: the idle-cost measurement counts the distinct directories the IDE listed, and asked for this field through
// a pattern, which the repository lint bans where a cursor reaches the same field.
// Tested in directory-snapshot-path.test.ts.
import { isWhitespace } from './scan-characters.ts';

/** Field name that opens the directory a listing line reports. */
const pathLabel = 'path=';

/**
 Directory a listing line reports.
 
 @param line - One line of the IDE's refresh log.
 
 @returns The path as the line spells it, or undefined when the line names none.
 
 @example
 ```ts
 listedDirectoryPath('read directory snapshot path=/a/b folders=2'); // '/a/b'
 ```
 */
export function listedDirectoryPath(line: string): string | undefined {
  let start = line.indexOf(pathLabel);
  // A field followed by whitespace names no directory, so the search resumes at the next field.
  while (start >= 0) {
    let cursor = start + pathLabel.length;
    const pathStart = cursor;
    while (cursor < line.length && !isWhitespace(line[cursor] ?? '')) {
      cursor++;
    }
    if (cursor > pathStart) return line.slice(pathStart, cursor);
    start = line.indexOf(pathLabel, start + 1);
  }
  return undefined;
}
