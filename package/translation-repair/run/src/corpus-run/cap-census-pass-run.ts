import {
  open,
  readFile,
} from 'node:fs/promises';

import {
  type CapLogReading,
  readCapLog,
} from './cap-census-read.ts';

//region Cap census pass run
// WHICH LOGS THE CAP CENSUS COUNTS (ledger P7, P10): a log counts only when a
// line among its first `HEAD_LINES` lines opens `START tip=`, since suite,
// build and prototype logs carry fixture `SPEND` lines. Each log is opened
// only far enough to find the marker, and a pass-run log's text is dropped
// once its samples are read, so memory holds samples, not logs.
//
// THE HEAD IS COUNTED IN LINES, NOT BYTES (ledger B963). A pass prints
// one `INCOMPLETE` line for each entry whose page is absent at the pin before
// it prints `START`, and the bytes of those lines grow whenever a field is
// added to them (`at=` took each from 66 to 94), so a window of 4096 bytes lost
// pass logs whose marker had not moved at all. The number of lines before
// `START` is the number of such entries, which no field changes.

/**
 Lines at the head of a log searched for the pass-run marker; the marker must sit among them.

 It exceeds the entries of any pin by a wide margin, and a log that holds no marker costs this many lines to read, not
 its whole text.
 */
const HEAD_LINES = 2_000;

/**
 Line a pass run opens its log with.
 */
const PASS_RUN_MARKER = 'START tip=';

/**
 Whether a line among the first `HEAD_LINES` of a log opens with the pass-run marker.

 Lines are read one at a time and the read stops at the marker or at the bound, so the cost does not depend on the
 size of the log or on the bytes of the lines before the marker. A bare carriage return ends a line here, as it does
 for any line reader.

 @param path - log file

 @returns Whether the marker opens one of the head lines

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

  /**
   Lines read so far.
   */
  let linesRead = 0;
  for await (const line of handle.readLines()) {
    if (line.startsWith(PASS_RUN_MARKER,))
      return true;
    linesRead += 1;
    if (linesRead >= HEAD_LINES)
      return false;
  }
  return false;
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
