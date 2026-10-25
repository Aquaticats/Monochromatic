import {
  open,
  readFile,
} from 'node:fs/promises';

import {
  type CapLogReading,
  readCapLog,
} from './cap-census-read.ts';

//region Cap census pass run
// WHICH LOGS THE CAP CENSUS COUNTS (ledger P7, P10): a log counts only when the
// first line among its first `HEAD_LINES` lines that opens `START tip=` holds a
// commit id a repository could give, since suite, build and prototype logs carry
// fixture `SPEND` lines. Each log is opened only far enough to find the marker,
// and a pass-run log's text is dropped once its samples are read, so memory
// holds samples, not logs.
//
// A REAL COMMIT ID IS WHAT TELLS A PASS FROM A SUITE (ledger B963). A test suite
// runs fixture passes in its own process and prints their START lines into its
// log, at line 267 to 1900 of it, so once the head grew to 2000 lines the marker
// alone admitted 515 suite logs as pass runs, which moved the report's count of
// pass-run logs by 47 percent. Every suite log measured carries `tip=` and forty
// `f` digits, a placeholder no repository gives, while a real pass prints the
// commit id it read, forty hexadecimal digits that are not one digit repeated.
// Over the 1104 logs the head admitted on 2026-10-25, the 552 that hold test
// runner PASS lines all carry the placeholder, 107 probe logs of the re-check
// carry forty `a` digits, and the other 445 carry a varied id and hold no PASS
// line. The test is made of what a commit id is and not of the fixture's
// spelling, and its limit is that a fixture printing the repository's own head,
// or a varied invented id, would pass it.
//
// THE HEAD IS COUNTED IN LINES, NOT BYTES (ledger B963). A pass prints
// one `INCOMPLETE` line for each entry whose page is absent at the pin before
// it prints `START`, and the bytes of those lines grow whenever a field is
// added to them (`at=` took each from 66 to 94), so a window of 4096 bytes lost
// pass logs whose marker had not moved at all. The number of lines before
// `START` is the number of such entries, which no field changes.

/**
 Lines at the head of a log searched for the pass-run marker; the marker must sit among them.

 A pass prints one `INCOMPLETE` line for each entry whose page is absent at the pin before `START`, so a pin with more
 such entries than this is read as no pass run, and the number is a choice, not a measure of any pin.
 A log that holds no marker costs this many lines to read, not its whole text.
 */
const HEAD_LINES = 2_000;

/**
 Line a pass run opens its log with.
 */
const PASS_RUN_MARKER = 'START tip=';

/**
 Digits a commit id holds.
 */
const COMMIT_ID_LENGTH = 40;

/**
 Characters a commit id is written in.
 */
const COMMIT_ID_DIGITS = '0123456789abcdef';

/**
 Whether a text is a commit id a repository could give.

 A COMMIT ID IS FORTY LOWERCASE HEXADECIMAL DIGITS THAT ARE NOT ONE DIGIT REPEATED. A repository does not give such an
 id; a test suite writes one as the placeholder of a fixture pass (ledger B963).

 @param text - the field after `tip=` on a line that opens the marker

 @returns Whether it is forty hexadecimal digits of more than one kind

 @example
 ```ts
 const real = isCommitId({ text: 'f'.repeat(40,), },); // => false
 ```
 */
function isCommitId({ text, }: { readonly text: string; },): boolean {
  if (text.length !== COMMIT_ID_LENGTH)
    return false;
  for (const character of text) {
    if (!COMMIT_ID_DIGITS.includes(character,))
      return false;
  }
  return new Set(text,).size > 1;
}

/**
 Whether a line opens a pass's report, which only a pass prints.

 @param line - a line of the log

 @returns Whether it opens with the pass-run marker followed by a commit id

 @example
 ```ts
 const starts = startsAPass({ line: 'START tip=2a336603bf5d1f0a8c7e94b1d62e0a3f77c9d815 pipeline=...', },); // => true
 ```
 */
function startsAPass({ line, }: { readonly line: string; },): boolean {
  /**
   The line after the marker.
   */
  const afterMarker = line.slice(PASS_RUN_MARKER.length,);

  /**
   The field that follows the marker, up to the first space.
   */
  const [text = '',] = afterMarker.split(
    ' ',
    1,
  );
  return isCommitId({ text, },);
}

/**
 Whether the first line among the first `HEAD_LINES` of a log that opens with the pass-run marker is a pass's START.

 Lines are read one at a time and the read stops at the first line that opens with the marker or at the bound, so the
 cost does not depend on the size of the log or on the bytes of the lines before the marker. The first marker decides:
 a log whose first marker holds no commit id is a suite's, whatever it prints later. A bare carriage return ends a line
 here, as it does for any line reader.

 @param path - log file

 @returns Whether that line holds a commit id a repository could give

 @example
 ```ts
 const isPassRun = await opensWithinHead({ path, },);
 ```
 */
async function opensWithinHead({ path, }: { readonly path: string; },): Promise<boolean> {
  /**
   The open log, closed when the search ends.
   */
  await using handle = await open(path,);

  return await (async function searchHead(): Promise<boolean> {
    /**
     Lines read so far.
     */
    let linesRead = 0;
    for await (const line of handle.readLines()) {
      if (line.startsWith(PASS_RUN_MARKER,))
        return startsAPass({ line, },);
      linesRead += 1;
      if (linesRead >= HEAD_LINES)
        return false;
    }
    return false;
  })();
}

/**
 Reads one log's samples when it is a pass-run log.

 @param path - log file

 @returns Its samples and the lines it left out for their stamp, or that it
 is no pass-run log

 @example
 ```ts
 const read = await capCensusPassRunReading({ path, },);
 ```
 */
export async function capCensusPassRunReading(
  { path, }: { readonly path: string; },
): Promise<CapLogReading | 'not-a-pass-run'> {
  if (!(await opensWithinHead({ path, },)))
    return 'not-a-pass-run';
  return readCapLog({
    lines: (await readFile(
      path,
      'utf8',
    )).split('\n',),
  },);
}

//endregion Cap census pass run
