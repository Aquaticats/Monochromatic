import { createReadStream, } from 'node:fs';

import {
  parseRunLogRecord,
  readRunLogText,
} from '@monochromatic-dev/translation-repair-glossary/ts';
import {
  type CapLogReading,
  readCapLog,
} from './cap-census-read.ts';

//region Cap census pass run
// WHICH LOGS THE CAP CENSUS COUNTS (ledger P7, P10): a log counts only when the
// first record among its first `HEAD_LINES` lines whose message opens `START tip=` holds a
// commit id a repository could give, since suite, build and prototype logs carry
// fixture `SPEND` lines. Each log is opened only far enough to find the marker,
// and a pass-run log's text is dropped once its samples are read, so memory
// holds samples, not logs.
//
// A LOG IS A RECORD FILE (`typography/src/run-log.ts`): one JSON object per line, the pass's own `START` line the
// message of one of them. The marker is read from the message of a line that is exactly a record, so a line of a record
// file that merely mentions it, or a cut-off line, is not the pass's START.
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
//
// A LINE IS READ UP TO `LINE_BYTES`, NOT WHOLE (ledger B963). A head bounded in
// lines still buffered each line whole, so a log with no line break was read
// entire (18 of the 2412 logs over 1 MiB under the agent root have none in
// their first MiB; three are 13.4 MB browser store files). The head is read in
// blocks of `BLOCK_BYTES`, one byte to a character since only ASCII is looked
// for, and a line that outgrows the bound ends the search: the log is no pass
// run. The longest line up to and including START among the 445 real pass logs
// measured on 2026-10-25 was 378 bytes, so the bound is about 170 times it.

/**
 Lines at the head of a log searched for the pass-run marker; the marker must sit among them.

 A pass prints one `INCOMPLETE` line for each entry whose page is absent at the pin before `START`, so a pin with more
 such entries than this is read as no pass run, and the number is a choice, not a measure of any pin.
 A log that holds no marker costs this many lines to read, not its whole text.
 */
const HEAD_LINES = 2_000;

/**
 Bytes one line of the head may hold; a log with a longer line is no pass run.

 The number is a choice with room: the longest line up to and including START in a real pass log measured 378 bytes.
 Memory while the head is searched is about this many bytes for the line being read and two blocks of the file.
 */
const LINE_BYTES = 65_536;

/**
 Bytes read from the file at a time while the head is searched.
 */
const BLOCK_BYTES = 65_536;

/**
 Line a pass run opens its log with.
 */
const PASS_RUN_MARKER = 'START tip=';

/**
 Sentinel for a line that is no record opening with the pass-run marker.
 */
const NO_MARKER: unique symbol = Symbol('line is no record opening with the pass-run marker',);

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

 @param text - field after `tip=` on a line that opens the marker

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
 The message of a line that is a record opening with the pass-run marker.

 @param line - line of the record file, a cut-off or foreign line included

 @returns Record's message, or {@link NO_MARKER} where the line is no record or its message does not open with the
 marker

 @example
 ```ts
 const message = markerMessageOf({ line: '{"level":"info","message":"START tip=2a33","timestamp":0}', },); // 'START tip=2a33'
 ```
 */
function markerMessageOf({ line, }: { readonly line: string; },): string | typeof NO_MARKER {
  /**
   The line as a record, if it is one.
   */
  const record = parseRunLogRecord({ line, },);
  if (record === 'not-a-record')
    return NO_MARKER;
  return record.message
    .startsWith(PASS_RUN_MARKER,) ? record.message : NO_MARKER;
}

/**
 Whether a line opens a pass's report, which only a pass prints.

 @param line - message of the record

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

 The log is read in blocks of `BLOCK_BYTES`, one byte to a character, and the lines of a block are cut where a line
 feed stands, which is the only line end a record file has: the sink writes each record on one line. The unfinished
 line at the end of a block is held to be completed by the next. The read stops at the first record whose message opens
 with the marker, at the bound in lines, or at a line longer than `LINE_BYTES`, so the cost and the memory do not
 depend on the size of the log, on the bytes of the lines before the marker or on whether the log has a line break at
 all. The first marker decides: a log whose first marker holds no commit id is a suite's, whatever it prints later.

 @param path - log file

 @returns Whether that line holds a commit id a repository could give

 @example
 ```ts
 const isPassRun = await opensWithinHead({ path, },);
 ```
 */
async function opensWithinHead({ path, }: { readonly path: string; },): Promise<boolean> {
  return await (async function searchHead(): Promise<boolean> {
    /**
     Lines read so far.
     */
    let linesRead = 0;

    /**
     The unfinished line at the end of the blocks read so far.
     */
    let unfinished = '';
    for await (const block of createReadStream(
      path,
      {
        encoding: 'latin1',
        highWaterMark: BLOCK_BYTES,
      },
    )) {
      /**
       The text cut at every line feed: the lines it finished, then the line it leaves unfinished.
       */
      const pieces = `${unfinished}${String(block,)}`.split('\n',);
      unfinished = pieces.pop() ?? '';
      for (const line of pieces) {
        if (line.length > LINE_BYTES)
          return false;
        /**
         The record's message, when the line is a record that opens with the marker.
         */
        const message = markerMessageOf({ line, },);
        if (message !== NO_MARKER)
          return startsAPass({ line: message, },);
        linesRead += 1;
        if (linesRead >= HEAD_LINES)
          return false;
      }
      if (unfinished.length > LINE_BYTES)
        return false;
    }
    /**
     The record the last, unterminated line is, when it is one that opens with the marker.
     */
    const lastMessage = markerMessageOf({ line: unfinished, },);
    return (lastMessage !== NO_MARKER) && startsAPass({ line: lastMessage, },);
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
    lines: (await readRunLogText({ path, },)).split('\n',),
  },);
}

//endregion Cap census pass run
