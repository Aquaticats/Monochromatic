import { resolveRealGit as resolveGit, } from '@monochromatic-dev/git-executable-core/ts';
import {
  allInInputOrder,
  refusalText,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  alignDocumentSections,
  isLineStructured,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  type CorpusPin,
  isMissingCorpusObject,
  listCorpusPeople,
  readCorpusFile,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  parseDocument,
  type RepairDocument,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { subdivideChunkPair, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { pickSpreadSample, } from './bench-draw.ts';

//region Bench sample
// The slices a roster-width bench runs, drawn the same way every time.
//
// Deterministic because the whole point is comparing widths: if each width saw
// a different sample, the decline rates would differ by sample and nothing
// could be read off the comparison. The draw is a stride over slices ordered by
// SOURCE SIZE, which spreads the sample across the size range rather than
// concentrating it wherever the corpus happens to be dense.
//
// Spends no quota. Reads the pinned corpus only.

/**
 One slice the bench translates, with the facts a stage call needs.

 @example
 ```ts
 const slice: BenchSlice = { entryId: 'Mittens', index: 3, ... };
 ```

 @internal
 */
export type BenchSlice = {
  /**
   Entry this slice was cut from.
   */
  readonly entryId: string;

  /**
   Position within that entry's slices, so a row can be traced back.
   */
  readonly index: number;

  /**
   Original passage to render.
   */
  readonly sourceText: string;

  /**
   Translation as it stands, blank when the archive has none here.
   */
  readonly incumbentText: string;

  /**
   Whether the line-structure rule governs this slice, inherited from its
   chunk exactly as `repairTranslation` inherits it.
   */
  readonly lineStructured: boolean;
};

/**
 A step of cutting an entry that failed: the line saying why the entry was
 skipped, held as data so the lines print in the order the corpus lists the
 entries once every entry has settled.

 @example
 ```ts
 const skipped: SkippedStep = { kind: 'skipped', line: 'BENCH skipping Mittens: people/Mittens/page.md could not be parsed: refused by Error', };
 ```
 */
type SkippedStep = {
  readonly kind: 'skipped';
  readonly line: string;
};

/**
 What one step of cutting an entry came to: its value, or the skip line.

 @example
 ```ts
 const outcome: StepOutcome<number> = { kind: 'done', value: 3, };
 ```
 */
type StepOutcome<Value,> =
  | {
    readonly kind: 'done';
    readonly value: Value;
  }
    | SkippedStep;

/**
 One page of an entry, read and parsed.

 @example
 ```ts
 const page: ParsedPage = { text, document: parseDocument({ text, },), };
 ```
 */
type ParsedPage = {
  /**
   Page text, which slicing cuts at the offsets the parse records.
   */
  readonly text: string;

  /**
   The page parsed, which alignment pairs by section.
   */
  readonly document: RepairDocument;
};

/**
 What a skip line says where aligning the pair or cutting it into slices
 failed. The throws found there (in `chunk-document.ts`, `slice-pair.ts`,
 `line-structure.ts` and the modules they call) each guard an invariant of the
 aligner or the slicer, and none says which page broke it, so the line says
 so rather than naming one.

 NO INPUT IS KNOWN TO REACH IT, AS MEASURED ON 2026-10-06. A bounded probe
 over generated input handed `sliceListedEntries` three fixed seeds of 2,000
 cat-themed page pairs each, drawn from a grammar of headings, paragraphs,
 lists, quotes, fences, tables, comments, footnotes, math, line-structured
 blocks, lone tags and braces, with front matter, carriage returns and byte
 order marks, against English pages carrying the original's sections, another
 count of them, some dropped, or extra ones shuffled in. 5,773 pairs passed
 this step, 2,363 of them cut into 7,326 slices, and none ended on this line;
 the 227 pages built with front matter no parser accepts each printed their
 parse line, which shows the probe read every line the draw printed. The line
 stays for a broken invariant, which then skips the entry rather than ending
 the draw.
 */
const PAIR_STEP_FAILED = 'the pair could not be aligned or cut into slices, and which page caused it is not known';

/**
 Builds the outcome of an entry skipped at one step.

 @param entryId - entry the line names first, since a reader scans a draw's skip lines by entry

 @param where - page and step that failed on it, which a reader fixing the corpus looks for; or
 the step and that no one page is known, where the failure names none

 @param error - what the step raised, printed through `refusalText` so the
 line repeats a marked class's message whole and names any other class alone

 @returns Skipped step, carrying the line whole so the draw can print it in listing order once
 every entry has settled

 @example
 ```ts
 const skipped = skippedAt({ entryId: 'Mittens', where: 'people/Mittens/page.md could not be read', error, },);
 ```
 */
function skippedAt(
  {
    entryId,
    where,
    error,
  }: {
    readonly entryId: string;
    readonly where: string;
    readonly error: unknown;
  },
): SkippedStep {
  return {
    kind: 'skipped',
    line: `BENCH skipping ${entryId}: ${where}: ${refusalText({ error, },)}`,
  };
}

/**
 Runs one step of cutting an entry, holding its failure as the entry's skip
 line.

 @param entryId - entry a failure's line names

 @param where - what a failure's line says failed, known before the step runs
 so the line never has to guess it from the failure

 @param step - step's work, run inside the catch so a failure of it skips
 this entry rather than ending the whole draw

 @returns Step's value, or the skip line held as data so the draw goes on to the next entry

 @example
 ```ts
 const parsed = stepOf({ entryId: 'Mittens', where: 'people/Mittens/page.md could not be parsed', step: function parse() { return parseDocument({ text, },); }, },);
 ```
 */
function stepOf<Value,>(
  {
    entryId,
    where,
    step,
  }: {
    readonly entryId: string;
    readonly where: string;
    readonly step: () => Value;
  },
): StepOutcome<Value> {
  try {
    return {
      kind: 'done',
      value: step(),
    };
  }
  catch (error) {
    return skippedAt({
      entryId,
      where,
      error,
    },);
  }
}

/**
 Reads one page of an entry, holding a page absent at a commit the clone
 holds as the entry's skip line naming the page.

 ONLY THAT ONE FAILURE IS A SKIP. A page absent at a held commit is an entry
 missing one side, an ordinary state of this corpus that the census reports
 too. A read that failed any other way (an object git cannot produce, a
 commit the clone lacks, a clone git cannot open, a failure that is no corpus
 read refusal at all) says nothing about the entry and something about the
 clone or this process, so it refuses the draw: skipped, it dropped the entry
 from a sample that then compared widths over a corpus it never read whole.

 @param entryId - entry a failure's line names

 @param pin - draw's clone and commit, resolved once by `sampleBenchSlices`
 so every page of one draw is read at one revision through one git binary

 @param relPath - page read, which a failure's line names

 @param readFile - reads one page at the pin, passed in so a case scripts which refusal ends first

 @returns Page's text, or the skip line naming it, held as data so the pair is reported in input
 order whichever page failed first

 @throws Whatever the read failed with, unchanged, unless it is a page absent
 at a commit the clone holds (`isMissingCorpusObject`)

 @example
 ```ts
 const read = await textOf({ entryId: 'Mittens', pin, relPath: 'people/Mittens/page.md', readFile: readCorpusFile, },);
 ```
 */
async function textOf(
  {
    entryId,
    pin,
    relPath,
    readFile,
  }: {
    readonly entryId: string;
    readonly pin: CorpusPin;
    readonly relPath: string;
    readonly readFile: typeof readCorpusFile;
  },
): Promise<StepOutcome<string>> {
  try {
    return {
      kind: 'done',
      value: await readFile({
        pin,
        relPath,
      },),
    };
  }
  catch (error) {
    if (!isMissingCorpusObject(error,))
      throw error;
    return skippedAt({
      entryId,
      where: `${relPath} could not be read`,
      error,
    },);
  }
}

/**
 Reads and parses one page of an entry, holding a failure of either as the
 entry's skip line naming the page and the step.

 @param entryId - entry a failure's line names

 @param pin - draw's clone and commit, resolved once by `sampleBenchSlices`
 so every page of one draw is read at one revision through one git binary

 @param relPath - page read, which a failure's line names

 @param readFile - reads one page at the pin, passed in so a case scripts which refusal ends first

 @returns Page read and parsed, or the skip line of the step that failed, held as data so the
 pair is reported in input order whichever page failed first

 @example
 ```ts
 const page = await pageOf({ entryId: 'Mittens', pin, relPath: 'people/Mittens/page.md', readFile: readCorpusFile, },);
 ```
 */
async function pageOf(
  {
    entryId,
    pin,
    relPath,
    readFile,
  }: {
    readonly entryId: string;
    readonly pin: CorpusPin;
    readonly relPath: string;
    readonly readFile: typeof readCorpusFile;
  },
): Promise<StepOutcome<ParsedPage>> {
  /**
   The page's text, or why it could not be read.
   */
  const read = await textOf({
    entryId,
    pin,
    relPath,
    readFile,
  },);
  if (read.kind === 'skipped')
    return read;

  return stepOf({
    entryId,
    where: `${relPath} could not be parsed`,
    step: function parsed(): ParsedPage {
      return {
        text: read.value,
        document: parseDocument({ text: read.value, },),
      };
    },
  },);
}

/**
 Cuts one entry's two parsed pages into slices.

 @param entryId - entry each slice carries, so a row of the bench can be traced back to it

 @param source - original page, whose sections give each slice the passage to render

 @param target - English page, aligned with the original so each slice carries the translation as it stands

 @returns Every slice of that entry

 @example
 ```ts
 const slices = pairSlices({ entryId: 'Mittens', source, target, },);
 ```
 */
function pairSlices(
  {
    entryId,
    source,
    target,
  }: {
    readonly entryId: string;
    readonly source: ParsedPage;
    readonly target: ParsedPage;
  },
): readonly BenchSlice[] {
  /**
   Aligned sections, exactly as the pipeline pairs them.
   */
  const alignment = alignDocumentSections({
    source: source.document,
    target: target.document,
  },);

  /**
   Slices accumulated across this entry's sections.
   */
  const slices: BenchSlice[] = [];
  for (const pair of alignment.pairs) {
    /**
     Whether the whole section reads as line-structured, which its slices
     inherit.
     */
    const chunkGoverns = isLineStructured({ text: pair.source
      .text, },);
    for (
      const slice of subdivideChunkPair({
        pair,
        sourceText: source.text,
        targetText: target.text,
        baseIndex: slices.length,
      },)
    ) {
      slices.push({
        entryId,
        index: slices.length,
        sourceText: slice.source
          .text,
        incumbentText: slice.target
          .text,
        lineStructured: chunkGoverns
          || isLineStructured({ text: slice.source
            .text, },),
      },);
    }
  }

  return slices;
}

/**
 Cuts one entry into slices, or says why it cannot.

 An entry missing one side, a page absent at a commit the clone holds, is
 simply not sampled: the census reports the same gap. A read that failed any
 other way refuses the draw (`textOf`).

 @param entryId - corpus entry, whose two pages are read under `people/<entryId>/`

 @param pin - draw's clone and commit, resolved once by `sampleBenchSlices`
 so every page of one draw is read at one revision through one git binary

 @param readFile - reads one page at the pin, passed in so a case scripts which refusal ends first

 @returns Every slice of that entry, or its skip line: of two pages that
 fail, the original's failure, an absent page or a parse, regardless of which
 ended first

 @throws Whatever a page's read failed with, unchanged, unless the page is
 absent at a commit the clone holds (`textOf`); of two such failures, the
 one first in input order

 @example
 ```ts
 const outcome = await sliceEntry({ entryId: 'Mittens', pin, readFile: readCorpusFile, },);
 ```
 */
async function sliceEntry(
  {
    entryId,
    pin,
    readFile,
  }: {
    readonly entryId: string;
    readonly pin: CorpusPin;
    readonly readFile: typeof readCorpusFile;
  },
): Promise<StepOutcome<readonly BenchSlice[]>> {
  /**
   Both sides at the pin, each read and parsed, in input order.
   */
  const [source, target,] = await allInInputOrder({
    members: [
      pageOf({
        entryId,
        pin,
        relPath: `people/${entryId}/page.md`,
        readFile,
      },),
      pageOf({
        entryId,
        pin,
        relPath: `people/${entryId}/page.en.md`,
        readFile,
      },),
    ],
  },);
  if (source.kind === 'skipped')
    return source;
  if (target.kind === 'skipped')
    return target;

  return stepOf({
    entryId,
    where: PAIR_STEP_FAILED,
    step: function slicesOfPair(): readonly BenchSlice[] {
      return pairSlices({
        entryId,
        source: source.value,
        target: target.value,
      },);
    },
  },);
}

/**
 Prints one skip line on the terminal, which is where a bench draw's progress goes.

 @param line - skip line as `skippedAt` built it and `sliceListedEntries` reports it, printed whole since
 it already names the entry, the page and the failure

 @example
 ```ts
 printSkipLine('BENCH skipping Mittens: people/Mittens/page.md could not be read: refused by Error',);
 ```
 */
function printSkipLine(line: string,): void {
  console.log(line,);
}

/**
 Cuts every listed entry into slices, reporting each entry it cannot cut.

 An entry missing one side is not a bench failure: the census reports the
 same gap, and refusing to draw a sample over it would make the bench depend
 on corpus completeness it does not need. A read that failed any other way
 is no gap in the corpus, and refuses the draw (`textOf`).

 @param entryIds - entries at the pin, in the order the corpus lists them

 @param pin - draw's clone and commit, resolved once by `sampleBenchSlices`
 so every page of one draw is read at one revision through one git binary

 @param readFile - reads one page at the pin, passed in so a case scripts which refusal ends first

 @param report - takes each skip line, passed in so a case reads the lines a
 draw prints and their order; a draw prints them on the terminal

 @returns Every slice of every entry cut, in listing order

 @throws Whatever a page's read failed with, unchanged, unless the page is
 absent at a commit the clone holds (`textOf`); of several such failures,
 the first in listing order, and within an entry the original's

 @example
 ```ts
 const all = await sliceListedEntries({ entryIds, pin, readFile: readCorpusFile, report: console.log, },);
 ```

 @internal
 */
export async function sliceListedEntries(
  {
    entryIds,
    pin,
    readFile,
    report,
  }: {
    readonly entryIds: readonly string[];
    readonly pin: CorpusPin;
    readonly readFile: typeof readCorpusFile;
    readonly report: (line: string,) => void;
  },
): Promise<readonly BenchSlice[]> {
  /**
   Every entry sliced, or the line that says why it was skipped: a page
   absent at a commit the clone holds, a page that does not parse, or a pair
   that cannot be aligned or cut. Any other read failure refuses here, the
   first in listing order.

   The line names the entry, the page and the step that failed on it, or says
   that no one page is known, then the failure through `refusalText`: a
   corpus read's marked message names the commit, the path and the kind, and
   any other failure names its class alone. No cut stands in for knowing what
   the text holds.
   */
  const outcomes = await allInInputOrder({
    members: entryIds.map(function sliceOne(entryId,): Promise<StepOutcome<readonly BenchSlice[]>> {
      return sliceEntry({
        entryId,
        pin,
        readFile,
      },);
    },),
  },);

  // Printed here, in listing order, rather than by each entry as its reads
  // ended: the order the disk answered in is no property of the corpus.
  for (const outcome of outcomes) {
    if (outcome.kind === 'skipped')
      report(outcome.line,);
  }

  return outcomes.flatMap(function slicesOf(outcome,): readonly BenchSlice[] {
    return (outcome.kind === 'done') ? outcome.value : [];
  },);
}

/**
 Draws the bench sample across the whole pinned corpus.

 @param count - slices wanted; fewer come back only when the corpus holds
 fewer

 @param pin - corpus clone and commit to read: the pin its entry reads in a run, a
 throwaway clone in a test, since the real one is unlicensed. REQUIRED: a
 default read the clone for any caller that left it out (ledger M43, X24)

 @returns Sample ordered by source size, smallest first

 @throws {@link StatedRefusalError} when the pinned corpus yields no slice at
 all, since a bench drawn over nothing would report widths as
 indistinguishable while having compared them on no work

 @throws Whatever a page's read failed with, unchanged, unless the page is
 absent at a commit the clone holds, since only that is an entry missing one
 side; a draw over a clone missing an object, or one git cannot read, would
 otherwise compare widths over a corpus it never read whole

 @example
 ```ts
 const sample = await sampleBenchSlices({ count: 12, pin, },);
 ```

 @internal
 */
export async function sampleBenchSlices(
  {
    count,
    pin,
  }: {
    readonly count: number;
    readonly pin: CorpusPin;
  },
): Promise<readonly BenchSlice[]> {
  /**
   Own the batch's revision and resolve its native executable before concurrent reads.
   Self-shim detection decodes candidate files; repeating it per page can exhaust the heap.
   */
  const resolvedPin: CorpusPin = {
    ...pin,
    gitPath: pin.gitPath ?? await resolveGit(),
  };
  /**
   Entries at the pin, in the order the corpus lists them.
   */
  const entryIds = await listCorpusPeople({ pin: resolvedPin, },);

  /**
   Every slice of every readable entry.
   */
  const all = await sliceListedEntries({
    entryIds,
    pin: resolvedPin,
    readFile: readCorpusFile,
    report: printSkipLine,
  },);
  // A STATED REFUSAL, since the remedy is the operator's: the clone or the
  // commit the run reads is not the corpus they meant. As a plain error both
  // commands that draw here printed it as a fault in themselves, at exit 5
  // under frames.
  if (all.length === 0) {
    throw new StatedRefusalError({
      says: `the corpus at the pin yields no slice to sample: ${String(entryIds.length,)} ${
        wordForCount({
          count: entryIds.length,
          one: 'entry is',
          many: 'entries are',
        },)
      } listed there and none could be sliced, so a bench drawn over it would compare on no work; check the `
        + 'clone and the commit this run reads',
    },);
  }

  return pickSpreadSample({
    slices: all,
    count,
  },);
}

//endregion Bench sample
