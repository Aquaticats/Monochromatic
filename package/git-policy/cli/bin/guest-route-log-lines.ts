/**
 The two questions every guest driver asks about the log an arm wrote: whether a word begins one of its lines, and
 which of its expected markers it lacks.

 The drivers asked both with a regular expression until this module held the scan: this repository bans a regular
 expression outside a justified site (`no-restricted-syntax/no-regex`), and a scan of the lines answers the same
 question, because `^` in a multiline pattern means the start of a line and nothing else.
 */

/**
 Whether some line of one text begins with a prefix.

 The first line is a line like every other one, and a last line with no newline after it counts too: the scan reads the
 start of each line, which is where a multiline `^` matched.

 @param text - the log to scan

 @param prefix - the word a line has to begin with

 @returns whether one line of the text begins with the prefix

 @example
 ```ts
 hasLineStartingWith({ text: 'ok\nSKIPPED: no ZFS here', prefix: 'SKIPPED' });
 ```
 */
export function hasLineStartingWith({
  text,
  prefix,
}: {
  readonly text: string;
  readonly prefix: string;
},): boolean {
  return text
    .split('\n',)
    .some(function beginsWithThePrefix(line: string,): boolean {
      return line
        .startsWith(prefix,);
    },);
}

/**
 The markers of one list that a log does not hold.

 A driver lists the lines a passing arm prints, one per claim the arm makes, and refuses a run whose log lacks one:
 an arm that returned without running is then a failed run rather than a green suite.

 @param text - the log to scan

 @param markers - the lines a passing run prints, in the order the driver lists them

 @returns the markers the log lacks, in the order the driver listed them

 @example
 ```ts
 const missing = missingMarkers({ text: pulled, markers: EXPECTED_MARKERS });
 ```
 */
export function missingMarkers({
  text,
  markers,
}: {
  readonly text: string;
  readonly markers: readonly string[];
},): readonly string[] {
  return markers
    .filter(function wasNotPrinted(marker: string,): boolean {
      return !text
        .includes(marker,);
    },);
}
