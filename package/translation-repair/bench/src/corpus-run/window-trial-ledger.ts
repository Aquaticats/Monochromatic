import {
  appendFile,
  mkdir,
  open,
} from 'node:fs/promises';
import { dirname, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  contextRoot,
  readTextOrEmptyIfMissing,
  rethrowUnlessMissingPath,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import { errorName, } from '@monochromatic-dev/translation-repair-markup/ts';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { writeFileAtomic, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region Window trial ledger
// What a window trial has already bought, kept on disk as it is bought.
//
// WHY IT EXISTS. The window trial judges each flagged slice three times, roughly 1760 real
// exchanges. The slice cache does not help: it is keyed and read by the DOCUMENT
// DRIVER, and this trial calls the stage directly, so nothing resumes. A run
// that dies at hour three would otherwise restart from zero.
//
// ONE LINE PER COMPLETED ARM, appended the moment it completes rather than
// batched at the end, because the failure this guards against is the process
// not reaching the end.
//
// WHAT MAKES A LINE SKIPPABLE is the protocol it was bought under, not just
// which slice it describes. A trial re-run after the rosters, the corpus pin or
// the code generation moved is asking a different question, and resuming across
// that boundary would silently mix two experiments into one tally. That is the
// same defect the slice cache's generation marker exists to prevent, and it is
// worse here because the output is a measurement rather than a document.

/**
 One completed arm of one slice's trial.

 @example
 ```ts
 const row: WindowTrialRow = { protocol: 'abc123', entryId: 'Mittens', sliceIndex: 7, arm: 'wide', sliceClass: 'relocation', shipped: true, decision: 'judged', winnerText: '...', judgesHeard: 6, judgesSeated: 6, position: 2, };
 ```
 */
export type WindowTrialRow = {
  /**
   Digest of everything this arm was bought under: rosters, corpus pin, code
   generation, and the trial's own version.

   A row whose protocol differs from the current one is NOT resumed, and is
   not deleted either: it is another experiment's evidence and the file is
   append-only.
   */
  readonly protocol: string;

  /**
   Corpus entry the slice belongs to.
   */
  readonly entryId: string;

  /**
   Slice position within that entry's preparation.
   */
  readonly sliceIndex: number;

  /**
   Which arm this row is: the two narrow runs are what the run-to-run band is
   measured from, so they are distinguishable rather than pooled.
   */
  readonly arm: string;

  /**
   Class the displacement screen flagged this slice as, or the control label.
   */
  readonly sliceClass: string;

  /**
   Whether this arm replaced the archive's wording.
   */
  readonly shipped: boolean;

  /**
   How the round ended, so a decline is distinguishable from a keep.
   */
  readonly decision: string;

  /**
   Winning text, kept because two arms can both replace and choose
   differently, which a boolean cannot show.
   */
  readonly winnerText: string;

  /**
   Judges whose ballot arrived and validated.

   RECORDED BECAUSE A LOST VOICE LOOKS LIKE A KEPT ARCHIVE. The fan-out retries
   to a quorum of half the roster and then proceeds, so an arm three judges
   timed out on is written as an ordinary decision. The wide arm sends the
   longest sheets under the same deadline, which means degradation lands
   asymmetrically on exactly the arm under test: unrecorded, it would read as
   the window making judges conservative.
   */
  readonly judgesHeard: number;

  /**
   Judges the arm seated, which {@link WindowTrialRow.judgesHeard} is read
   against.
   */
  readonly judgesSeated: number;

  /**
   Which of this slice's three calls this arm was, zero-based.

   RECORDED BECAUSE THE POSITION IS ASSIGNED, not fixed. The wide arm used to
   be third on every slice, which aliased it onto anything that drifts across
   a slice's calls; it now sits at a position derived from the slice, and this
   field is what lets the effect of position be estimated rather than assumed
   away.
   */
  readonly position: number;
};

/**
 Identity of one arm, which is what resumption skips on.

 THE ARM IS PART OF IT, so the two narrow runs of one slice are distinct keys.
 Pooling them would erase the run-to-run band, which is the only thing the
 narrow-to-wide difference can be read against.

 ENCODED RATHER THAN JOINED. Every caller must build the key through this
 function, because a key is only useful if two builders agree on it and a
 hand-joined one silently does not: an earlier version of this file joined on a
 NUL byte while the runner joined on a space, which renders identically in most
 readers and matched nothing, so every resumed run re-bought arms it already
 held. Encoding also means an entry id containing the separator cannot forge
 another slice's key.

 @param row - arm to identify, or enough of one

 @returns Key unique to this protocol, entry, slice and arm together

 @example
 ```ts
 const key = trialKey({ row, },);
 ```
 */
export function trialKey(
  { row, }: {
    readonly row: Pick<WindowTrialRow, 'protocol' | 'entryId' | 'sliceIndex' | 'arm'>;
  },
): string {
  return JSON.stringify([
    row.protocol,
    row.entryId,
    row.sliceIndex,
    row.arm,
  ],);
}

/**
 Whether a parsed line is a usable row.

 @param value - parsed JSON of one line

 @returns True when every field this ledger reads is present and typed

 @example
 ```ts
 if (isWindowTrialRow(parsed,)) rows.push(parsed,);
 ```
 */
function isWindowTrialRow(value: unknown,): value is WindowTrialRow {
  return isJsonRecord(value,)
    && ((typeof value.protocol) === 'string')
    && ((typeof value.entryId) === 'string')
    && ((typeof value.sliceIndex) === 'number')
    && ((typeof value.arm) === 'string')
    && ((typeof value.sliceClass) === 'string')
    && ((typeof value.shipped) === 'boolean')
    && ((typeof value.decision) === 'string')
    && ((typeof value.winnerText) === 'string')
    && ((typeof value.judgesHeard) === 'number')
    && ((typeof value.judgesSeated) === 'number')
    && ((typeof value.position) === 'number');
}

/**
 Logger root for the ledger, under the window trial's own tag.
 */
const l = contextRoot({ tag: 'window-trial', },);

/**
 A ledger line that ends in a newline and does not parse.

 NAMES THE FILE AND THE LINE NUMBER, NEVER THE LINE. A row carries the winning
 text of its arm, which is corpus wording, and the parser's own refusal quotes
 the text it stopped in, so a refusal repeating either could carry a passage
 into a log. Until 2026-10-05 the parser's `SyntaxError` left here as it was.

 @example
 ```ts
 throw new TrialLedgerLineError({ path, line: 7, failure: 'SyntaxError', },);
 ```
 */
class TrialLedgerLineError extends Error {
  /**
   Declares this message safe to forward: it names the file, a line number
   and the class of the parse refusal, and repeats none of the line.
   */
  readonly messageNamesOnly: true = true;

  /**
   Names the line by its number and says what could have written it and what
   an operator can do, since no later read gets past it.

   @param path - ledger file the line sits in

   @param line - one-based number of the line, as an editor counts it

   @param failure - class name of the parse refusal, never its message
   */
  constructor(
    {
      path,
      line,
      failure,
    }: {
      readonly path: string;
      readonly line: number;
      readonly failure: string;
    },
  ) {
    super(
      `window trial ledger ${path}: line ${String(line,)} ends in a newline and does not parse as JSON `
        + `(${failure}). A kill mid-append leaves only an unterminated last line, which the next append removes, `
        + 'so this line was written by something else: a build that appended a row onto such a fragment, or two '
        + 'runs appending at once. What it bought cannot be read from here. Remove that line by hand to buy its '
        + 'arms again, or move the ledger aside to start a fresh one.',
    );
    this.name = 'TrialLedgerLineError';
  }
}

/**
 What one ledger line turned out to hold.
 */
type LineReading =
  | {
    /**
     A completed arm, every field this ledger reads present and typed.
     */
    readonly kind: 'row';

    /**
     That arm.
     */
    readonly row: WindowTrialRow;
  }
    | {
    /**
     Whole JSON that is no row of this build's shape, which is what a row an
     older build wrote under other field names reads as.
     */
      readonly kind: 'not-a-row';
    }
    | {
    /**
     Text the parser refused.
     */
      readonly kind: 'unparsed';

    /**
     Class name of the refusal, never its message, which quotes the line.
     */
      readonly failure: string;
    };

/**
 Reads one line of a ledger.

 @param line - line without its newline

 @returns Row it holds, or which of the two ways it holds none

 @example
 ```ts
 const reading = readLine({ line: '{"protocol":"abc123"}', },);
 ```
 */
function readLine({ line, }: { readonly line: string; },): LineReading {
  try {
    /**
     Parsed line, guarded before it is trusted.
     */
    const parsed: unknown = JSON.parse(line,);
    return isWindowTrialRow(parsed,)
      ? {
        kind: 'row',
        row: parsed,
      }
      : { kind: 'not-a-row', };
  }
  catch (error) {
    // THE CLASS NAME IS ALL THAT LEAVES HERE. The parser's message quotes the
    // text it refused, and a ledger line carries an arm's winning text.
    return {
      kind: 'unparsed',
      failure: errorName({ error, },),
    };
  }
}

/**
 One non-blank line of a ledger, with where it sits.
 */
type PlacedLine = {
  /**
   One-based number of the line, as an editor counts it.
   */
  readonly number: number;

  /**
   Whether a newline ends it. Only the text after the last newline of the
   file has none, and `appendTrialRow` writes every row with one, so a line
   without is what a kill mid-append leaves.
   */
  readonly ended: boolean;

  /**
   What the line holds.
   */
  readonly reading: LineReading;
};

/**
 Reads every non-blank line of a ledger's text.

 @param text - whole ledger

 @returns Lines in append order, each with its number and whether it ends

 @example
 ```ts
 const lines = placedLines({ text, },);
 ```
 */
function placedLines({ text, }: { readonly text: string; },): readonly PlacedLine[] {
  /**
   Every line, the last being whatever follows the final newline: nothing
   where the ledger ends at a line boundary.
   */
  const lines = text.split('\n',);

  return lines.flatMap(function toPlaced(
    line,
    lineIndex,
  ): readonly PlacedLine[] {
    if (line.trim() === '')
      return [];
    return [
      {
        number: lineIndex + 1,
        ended: lineIndex < (lines.length - 1),
        reading: readLine({ line, },),
      },
    ];
  },);
}

/**
 What one read of a ledger found, with what it left out.
 */
type TrialLedgerAccount = {
  /**
   Completed arms, in the order they were appended.
   */
  readonly rows: readonly WindowTrialRow[];

  /**
   Whole lines that are no row of this build's shape, which no tally counts.

   A ROW AN OLDER BUILD WROTE IS ONE. The row gained its judge counts and its
   position, and its slice index was renamed, so rows written before each of
   those lack a field this reader requires. They stay in the file, which is
   append-only, and each was bought under a protocol no later run resumes.
   */
  readonly leftOut: number;

  /**
   Whether the ledger ends in a line a kill mid-append cut short, whose arm is
   not among the rows and is bought again.
   */
  readonly tornTail: boolean;
};

/**
 Reads a ledger and accounts for every line it did not take as a row.

 A TORN LAST LINE IS EXPECTED rather than exceptional: the process this guards
 against is one killed mid-append, so the text after the last newline may be a
 fragment. It is left out, said, and its arm is re-bought, which costs one
 arm; `appendTrialRow` removes the fragment before it appends.

 A LINE THAT ENDS IN A NEWLINE AND DOES NOT PARSE IS REFUSED, wherever it
 sits. A single cut-off append cannot leave one, so it was written by
 something this ledger does not allow for: two runs appending at once, or a
 build from before 2026-10-05, whose resumed run appended its first row onto
 the fragment and so wrote one unreadable line holding both.

 A WHOLE LINE THAT IS NO ROW IS LEFT OUT AND COUNTED, never refused and never
 dropped without a word: older builds wrote rows of other shapes into this
 same file.

 @param path - ledger file

 @returns Rows in the order they were appended, how many whole lines were no
 row, and whether the last line was torn; no rows where the file is absent

 @throws {@link TrialLedgerLineError} when a line that ends in a newline does
 not parse, naming its number and none of its text

 @example
 ```ts
 const { rows, leftOut, tornTail, } = await accountTrialLedger({ path, },);
 ```
 */
export async function accountTrialLedger(
  { path, }: { readonly path: string; },
): Promise<TrialLedgerAccount> {
  /**
   Logger pre-tagged with this function's name.
   */
  const al = tagged({
    tag: accountTrialLedger.name,
    l,
  },);

  /**
   Every non-blank line, read. An absent ledger holds none: that is the
   ordinary state before the first arm is bought.
   */
  const placed = placedLines({ text: await readTextOrEmptyIfMissing({ path, },), },);

  /**
   Lines the parser refused, each with whether a newline ends it.
   */
  const unparsed = placed.flatMap(function toUnparsed(one,): readonly {
    readonly number: number;
    readonly ended: boolean;
    readonly failure: string;
  }[] {
    /**
     What this line holds.
     */
    const { reading, } = one;
    if (reading.kind !== 'unparsed')
      return [];
    return [
      {
        number: one.number,
        ended: one.ended,
        failure: reading.failure,
      },
    ];
  },);

  /**
   Earliest refused line that a newline ends, which no cut-off append leaves.
   */
  const unreadable = unparsed.find(function isEnded(one,): boolean {
    return one.ended;
  },);
  if (unreadable !== undefined)
    throw new TrialLedgerLineError({
      path,
      line: unreadable.number,
      failure: unreadable.failure,
    },);

  /**
   Refused text after the last newline, which is what a kill mid-append leaves.
   */
  const torn = unparsed.find(function isUnended(one,): boolean {
    return !one.ended;
  },);
  if (torn !== undefined)
    al.warn(
      `${path}: line ${String(torn.number,)}, the last, has no newline and does not parse (${torn.failure}), as a `
        + 'kill mid-append leaves it; its arm is not counted and is bought again, and the next append removes '
        + 'the fragment',
    );

  /**
   Whole lines that are no row of this build's shape.
   */
  const leftOut = placed.filter(function isNoRow(one,): boolean {
    /**
     What this line holds.
     */
    const { reading, } = one;
    return reading.kind === 'not-a-row';
  },);

  /**
   Earliest of them, absent where every whole line is a row.
   */
  const [firstLeftOut,] = leftOut;
  if (firstLeftOut !== undefined)
    al.warn(
      `${path}: left out ${String(leftOut.length,)} whole ${
        wordForCount({
          count: leftOut.length,
          one: 'line that is',
          many: 'lines that are',
        },)
      } no trial row of this build's shape, the earliest at line ${
        String(firstLeftOut.number,)
      }; a build from before a row field was added or renamed wrote such rows, each under a protocol no later run `
        + 'resumes, and no line is removed from the ledger',
    );

  return {
    rows: placed.flatMap(function toRow(one,): readonly WindowTrialRow[] {
      /**
       What this line holds.
       */
      const { reading, } = one;
      return (reading.kind === 'row') ? [reading.row,] : [];
    },),
    leftOut: leftOut.length,
    tornTail: torn !== undefined,
  };
}

/**
 Reads every completed arm from a ledger, for a caller that prints no account
 of what the read left out. `accountTrialLedger` does the reading and says
 through its logger what it left out and why.

 @param path - ledger file

 @returns Rows in the order they were appended, empty when the file is absent

 @throws {@link TrialLedgerLineError} when a line that ends in a newline does
 not parse, naming its number and none of its text

 @example
 ```ts
 const rows = await readTrialLedger({ path, },);
 ```
 */
export async function readTrialLedger(
  { path, }: { readonly path: string; },
): Promise<readonly WindowTrialRow[]> {
  /**
   Rows, with what was left out of them already said.
   */
  const { rows, } = await accountTrialLedger({ path, },);
  return rows;
}

/**
 Last character of a file, read without reading the rest.

 @param path - file to read

 @returns Its last byte as text, empty where the file is empty or absent

 @throws Whatever opening or reading it raised other than its absence

 @example
 ```ts
 const ended = (await lastByteOf({ path, },)) === '\n';
 ```
 */
async function lastByteOf({ path, }: { readonly path: string; },): Promise<string> {
  try {
    /**
     The open file, closed once its last byte is read.
     */
    await using handle = await open(path,);

    /**
     Bytes the file holds.
     */
    const { size, } = await handle.stat();
    if (size === 0)
      return '';

    /**
     Buffer the last byte is read into.
     */
    const buffer = Buffer.alloc(1,);

    /**
     Bytes the read filled.
     */
    const { bytesRead, } = await handle.read(
      buffer,
      0,
      1,
      size - 1,
    );
    return buffer.toString(
      'utf8',
      0,
      bytesRead,
    );
  }
  catch (error) {
    rethrowUnlessMissingPath({ error, },);
    return '';
  }
}

/**
 Makes a ledger end at a line boundary, so the row appended next starts a line
 of its own.

 WHY AN APPEND HAS TO LOOK FIRST. A kill mid-append leaves text with no
 newline after it, and `appendFile` writes straight after whatever is there.
 Until 2026-10-05 the first row a resumed run appended was joined onto that
 fragment: the line did not parse, so the row was lost and its arm bought
 again, and once a second row followed it the unreadable line was no longer
 the last and every later read of the ledger threw.

 A FRAGMENT IS REMOVED, by writing the ledger again without it. It recorded an
 arm that never completed, which is bought again whether the fragment stays
 or goes. A WHOLE LINE THAT LOST ONLY ITS NEWLINE is given one and keeps what
 it bought.

 ONE WRITER IS ASSUMED, as everywhere in this file: a second run appending at
 the same moment could have its unfinished row taken for a fragment.

 @param path - ledger file, absent before the first arm is bought

 @example
 ```ts
 await endAtLineBoundary({ path, },);
 ```
 */
async function endAtLineBoundary({ path, }: { readonly path: string; },): Promise<void> {
  /**
   Last character of the ledger, empty before the first arm is bought.
   */
  const last = await lastByteOf({ path, },);

  // THE ORDINARY CASE COSTS ONE BYTE: a ledger every append ended with its
  // newline needs nothing, and only one a kill cut short is read whole.
  if ((last === '') || (last === '\n'))
    return;

  /**
   Whole ledger, read only because it does not end in a newline.
   */
  const text = await readTextOrEmptyIfMissing({ path, },);

  /**
   Where the text after the last newline begins: the start of the file where
   it holds no newline at all.
   */
  const tailAt = text.lastIndexOf('\n',) + 1;

  /**
   That text, which no newline ends.
   */
  const tail = text.slice(tailAt,);
  if (tail.trim() === '')
    return;

  /**
   Logger pre-tagged with this function's name.
   */
  const el = tagged({
    tag: endAtLineBoundary.name,
    l,
  },);

  /**
   One-based number of the unended line: one more than the newlines before it.
   */
  const number = text
    .slice(
    0,
    tailAt,
    )
    .split('\n',)
    .length;

  /**
   What that line holds.
   */
  const reading = readLine({ line: tail, },);
  if (reading.kind === 'unparsed') {
    // ATOMIC, so a kill during this write leaves the ledger as it was, with
    // every completed row in it.
    await writeFileAtomic({
      path,
      text: text.slice(
        0,
        tailAt,
      ),
    },);
    el.warn(
      `${path}: removed line ${String(number,)}, a last line a kill mid-append cut short (${reading.failure}), so `
        + 'the row appended now starts a line of its own; the arm that line was recording is bought again',
    );
    return;
  }

  await appendFile(
    path,
    '\n',
  );
  el.info(
    `${path}: line ${String(number,)}, the last, was whole and had no newline; ended it before appending, and it `
      + 'keeps what it holds',
  );
}

/**
 Appends one completed arm, on a line of its own.

 CALLED THE MOMENT THE ARM COMPLETES, never batched. What this protects
 against is the process not reaching the end, so anything held in memory to
 write later is exactly what is lost.

 THE LEDGER'S LAST BYTE IS READ BEFORE EACH APPEND, to see that it ends at a
 line boundary (`endAtLineBoundary`); the whole ledger is read only where it
 does not, which is once after a kill.

 @param path - ledger file

 @param row - arm that completed

 @example
 ```ts
 await appendTrialRow({ path, row, },);
 ```
 */
export async function appendTrialRow(
  {
    path,
    row,
  }: {
    readonly path: string;
    readonly row: WindowTrialRow;
  },
): Promise<void> {
  await mkdir(
    dirname(path,),
    { recursive: true, },
  );
  await endAtLineBoundary({ path, },);
  await appendFile(
    path,
    `${JSON.stringify(row,)}\n`,
  );
}

/**
 Arms already bought under one protocol, as keys a runner skips on.

 ROWS FROM ANOTHER PROTOCOL ARE IGNORED rather than removed. They were bought
 under different rosters, a different corpus pin or different code, so they
 answer a different question; counting them would mix two experiments, and
 deleting them would discard evidence this run has no claim over.

 @param rows - every row the ledger holds

 @param protocol - digest this run is buying under

 @returns Keys of arms this run may skip

 @example
 ```ts
 const done = completedArms({ rows, protocol, },);
 ```
 */
export function completedArms(
  {
    rows,
    protocol,
  }: {
    readonly rows: readonly WindowTrialRow[];
    readonly protocol: string;
  },
): ReadonlySet<string> {
  return new Set(rows
    .filter(function underThisProtocol(row,): boolean {
      return row.protocol === protocol;
    },)
    .map(function toKey(row,): string {
      return trialKey({ row, },);
    },),);
}

//endregion Window trial ledger
