import {
  type CorpusPin,
  isMissingCorpusObject,
  listCorpusPeople,
  readCorpusFile,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';

import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { reportingRefusals, } from './cli-refusal.ts';
import {
  censusEntryOf,
  reportRepeatCensus,
} from './repeat-census-report.ts';
import type { EntryCensus, } from './repeat-census-summary.ts';
import { readRunCorpusPinSetting, } from './run-corpus-pin.ts';

//region Repeat census
// The repeated-wording census as a command: the wiring only. The line reading is
// `repeat-census-lines.ts`, what is printed and when the command declines is
// `repeat-census-report.ts`, and the corpus pin is the one every corpus read
// resolves against. READ-ONLY and keyless: it reads git objects at the pin and
// calls no model.

/**
 Reads the page of every entry at the pin, leaving out an entry without one.

 @param pin - corpus clone and commit

 @returns Entry id and page text for each entry that has a page

 @example
 ```ts
 const pages = await readPages({ pin, },);
 ```
 */
async function readPages(
  {
    pin,
  }: {
    readonly pin: CorpusPin;
  },
): Promise<readonly {
  readonly id: string;
  readonly text: string
}[]> {
  /**
   Entry ids at the pin.
   */
  const ids = await listCorpusPeople({ pin, },);
  /**
   Entry pages that exist at the pin, in the listing's order.
   */
  const pages = await allInInputOrder({
    members: ids.map(async function readPage(id,): Promise<
      readonly {
        readonly id: string;
        readonly text: string
      }[]
      > {
      try {
      /**
       Page text at the pin, with CRLF folded.
       */
        const text = await readCorpusFile({
          pin,
          relPath: `people/${id}/page.md`,
        },);
        return [{
          id,
          text,
        },];
      } catch (error) {
        if (isMissingCorpusObject(error,))
          return [];
        throw error;
      }
    },),
  },);
  return pages.flat();
}

/**
 Reads the pinned corpus, runs the census over each page, and prints the summary
 whole; then declines, so the command exits non-zero, when a positive control
 read false or could not be read.

 @param pin - corpus clone and commit

 @throws {@link Error} when a corpus read fails for a reason other than a missing page

 @throws {@link StatedRefusalError} when a positive control of the census did not hold, after the whole summary is printed

 @example
 ```ts
 await runRepeatCensus({ pin, },);
 ```
 */
async function runRepeatCensus(
  {
    pin,
  }: {
    readonly pin: CorpusPin;
  },
): Promise<void> {
  /**
   Entry pages at the pin.
   */
  const pages = await readPages({ pin, },);

  /**
   Census of every page.
   */
  const entries: EntryCensus[] = pages.map(function censusOf(page,): EntryCensus {
    return censusEntryOf({
      id: page.id,
      text: page.text,
    },);
  },);
  reportRepeatCensus({ entries, },);
}

if (import.meta.main)
  await reportingRefusals({
    what: 'repeat-census',
    argv: process.argv,
    env: process.env,
    run: function runRepeatCensusAtPin(): Promise<void> {
      /**
       Corpus clone and commit this run reads, read inside the refusal boundary.
       */
      const { pin, } = readRunCorpusPinSetting({ env: process.env, },);
      return runRepeatCensus({ pin, },);
    },
  },);

//endregion Repeat census
