// The ten-second pressure averages one /proc/pressure file reports, read once per line.
// What: a record keyed `<resource>_<label>`, one entry per line of the file, holding the avg10 value as a number.
// Why: the idle-cost measurement records the host's pressure before and after an interval, and read each line's
// average through a pattern, which the repository lint bans where a cursor reaches the same field.
// Tested in pressure-shares.test.ts.
import { isDecimalNumberCharacter } from './scan-characters.ts';

/** Field name that opens the ten-second average of a pressure line. */
const averageLabel = 'avg10=';

/**
 A /proc/pressure line whose ten-second average cannot be read.
 
 The message names the whole line, because the caller prints it into the measurement artifact.
 */
export class PressureLineError extends Error {
  /** Name this error reports, so a log line says which reader refused the text. */
  override name = 'PressureLineError';
}

/**
 Ten-second averages of one pressure file, keyed by the resource and the line's own label.
 
 @param options - `resource` is the file's name, such as `cpu` or `io`; `text` is the whole file.
 
 @returns The averages, for example `{ cpu_some: 0.12, cpu_full: 0 }`.
 
 @throws {PressureLineError} When a line holds no ten-second average.
 
 @example
 ```ts
 pressureShares({ resource: 'cpu', text: 'some avg10=0.12 total=7\n' }); // { cpu_some: 0.12 }
 ```
 */
export function pressureShares({ resource, text }: { resource: string; text: string }): Record<string, number> {
  const shares: Record<string, number> = {};
  for (const line of text.trim().split('\n')) {
    const average = tenSecondAverage(line);
    if (average === undefined) throw new PressureLineError('The pressure line has no avg10 value: ' + line);
    shares[resource + '_' + line.split(' ')[0]] = average;
  }
  return shares;
}

/**
 Ten-second average one pressure line states.
 
 @param line - One line of a /proc/pressure file.
 
 @returns The average as a number, or undefined when the line states none.
 */
function tenSecondAverage(line: string): number | undefined {
  let start = line.indexOf(averageLabel);
  // A label followed by no number is not an average, so the search resumes at the next label.
  while (start >= 0) {
    let cursor = start + averageLabel.length;
    const numberStart = cursor;
    while (isDecimalNumberCharacter(line[cursor] ?? '')) cursor++;
    if (cursor > numberStart) return Number(line.slice(numberStart, cursor));
    start = line.indexOf(averageLabel, start + 1);
  }
  return undefined;
}
