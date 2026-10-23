import {
  type Logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import { commentRegionsOf, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  type SliceReplacement,
  spliceSlices,
} from '@monochromatic-dev/translation-repair-page-pass/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  type PageGrammarReading,
  readPageGrammar,
} from './page-grammar.ts';
import type { PublishDefect, } from './publish-defects.ts';
import { shippableReplacements, } from './shippable-replacements.ts';
import { textOnPage, } from './source-ships.ts';
import {
  type WouldShipSlice,
  type WouldShipSource,
  wouldShipTextPerSlice,
} from './would-ship-text.ts';

//region Source ships compose
// THE PAGE THAT CARRIES THE ORIGINAL OF AN UNTRANSLATED PASSAGE, COMPOSED UNDER THE GRAMMAR. The owner ruled on
// 2026-10-23 that the page carries the original of such a passage; the original is corpus text the strict MDX grammar
// the site compiles with may refuse (a bare brace, a tag nothing closes), and a page it refuses never ships
// (`page-grammar.ts`). Before this file that stopped the entry after the hours it had cost, with neither page nor
// artifact written and the same text waiting for the retry. A run always ships (the owner, 2026-09-27), so the page is
// composed in steps and each is read by the grammar before the next. The base case is the page with each source run
// written as the artifact records it, and it is the page whenever it parses. Otherwise:
//
// 1. Only the source runs that do not parse as a text of their own, and those holding an HTML comment that never
//    closes, are written as literal text (`source-ships-encoding.ts`). A run is a contiguous run of slices shipping their original, the unit the note
//    stands before, so a tag opened in one slice and closed in the next is read as the pair it is.
// 2. If the page still does not parse, every source run is written as literal text.
// 3. If it still does not, the page ships without its source runs, the silent gap the pages had before the ruling, and
//    the defect `page-source-unparseable` names the slices.
//
// Each step that is taken writes one warn line naming the step and the slices. A step whose page equals the page last
// tried changed nothing and is not taken. A run holding a comment that never closes is one no grammar reading can
// vouch for: the package masks everything after such an opener, so a page that "parses" with it hides the rest of the
// page from the package's own parser, and it never ships as written. The original differs from the artifact's record only on these paths; with
// the page parsing as recorded, nothing here touches a byte of it.

/**
 Names a step of the composition.

 `as-written` is the page with every original as recorded, `encoded-failing-runs` the runs that do not parse alone
 written as literal text, `encoded-every-run` all of them, and `dropped` the page without any.
 */
export type SourcePageStep = 'as-written' | 'dropped' | 'encoded-every-run' | 'encoded-failing-runs';

/**
 What the grammar says of a text, handed to the composition so a case can stand in for it.
 */
export type GrammarReader = (arguments_: { readonly pageText: string; },) => PageGrammarReading;

/**
 A page composed, and how.
 */
export type ComposedPage = {
  /**
   The page.
   */
  readonly pageText: string;

  /**
   The replacements the page was spliced from.
   */
  readonly replacements: readonly SliceReplacement[];

  /**
   The step that produced it.
   */
  readonly step: SourcePageStep;

  /**
   Slices whose original the step wrote as literal text or left off the page, none for a page with every original as
   recorded.
   */
  readonly slices: readonly number[];

  /**
   Defects the composition reports, the page without its originals among them.
   */
  readonly defects: readonly PublishDefect[];
};

/**
 One step of the composition after the base case.
 */
type Step = {
  /**
   Name the composition returns when this step makes the page.
   */
  readonly step: SourcePageStep;

  /**
   Number printed in the warn line, of three.
   */
  readonly number: number;

  /**
   What the warn line says the step does.
   */
  readonly does: string;

  /**
   What the warn line calls the page this step makes, when the next step names the page it found refused.
   */
  readonly makes: string;

  /**
   Slices the step writes as literal text.
   */
  readonly encoded: readonly number[];

  /**
   Whether originals are on the page at all.
   */
  readonly includeSource: boolean;
};

/**
 The page last tried, what the warn line calls it and what the grammar said of it.
 */
type LastTried = {
  /**
   The page.
   */
  pageText: string;

  /**
   What the warn line calls it.
   */
  what: string;

  /**
   What the warn line says is wrong with it: the grammar's refusal, or a comment nothing closes.
   */
  problem: string;
};

/**
 Refusal when the composition ends without a page, which the last step makes impossible.

 @example
 ```ts
 throw new SourcePageCompositionError({ entryId: 'BookshopCat', },);
 ```
 */
export class SourcePageCompositionError extends Error {
  /**
   Message contains the entry id and nothing of the page.
   */
  readonly messageNamesOnly: true = true;

  /**
   Entry whose page was being composed.
   */
  readonly entryId: string;

  /**
   @param entryId - affected entry
   */
  public constructor({ entryId, }: { readonly entryId: string; },) {
    super(`entry ${entryId}: the composition of the page ended without a page, which its last step cannot do`,);
    this.name = 'SourcePageCompositionError';
    this.entryId = entryId;
  }
}

/**
 Groups the slices that ship their original into runs.

 @param slices - every slice's reading, in comparison-row order

 @returns Slice indices of each run, in page order; none when no slice ships its original

 @example
 ```ts
 const runs = sourceRunsOf({ slices: wouldShipTextPerSlice({ artifact, },), },);
 ```
 */
export function sourceRunsOf(
  { slices, }: { readonly slices: readonly WouldShipSlice[]; },
): readonly (readonly number[])[] {
  /**
   Runs found so far, the last of which a continuing slice joins.
   */
  const runs: number[][] = [];
  for (const slice of slices) {
    /**
     What the slice ships.
     */
    const { reading, } = slice;
    if (reading.kind !== 'source-ships')
      continue;
    if (reading.opensRun) {
      runs.push([slice.sliceIndex,],);
      continue;
    }
    /**
     The run this slice continues.
     */
    const run = nonNullishOrThrow(runs.at(-1,),);
    run.push(slice.sliceIndex,);
  }
  return runs;
}

/**
 Text a run of slices would make of a page of its own, as the page carries it.

 @param run - slice indices of the run

 @param slices - every slice's reading, in comparison-row order

 @returns Note and originals, one blank line apart

 @example
 ```ts
 const text = runTextOf({ run: [88, 89,], slices, },);
 ```
 */
function runTextOf(
  {
    run,
    slices,
  }: {
    readonly run: readonly number[];
    readonly slices: readonly WouldShipSlice[];
  },
): string {
  /**
   What each slice of the run puts on the page.
   */
  const texts = slices.flatMap(function textOf(slice,): readonly string[] {
    /**
     What the slice ships.
     */
    const { reading, } = slice;
    return (run.includes(slice.sliceIndex,) && (reading.kind === 'source-ships'))
      ? [textOnPage({ reading, },),]
      : [];
  },);
  return `${texts.join('\n\n',)}\n`;
}

/**
 Whether the text a run puts on the page opens an HTML comment that never closes.

 @param run - slice indices of the run

 @param slices - every slice's reading, in comparison-row order

 @returns Whether the run's own text has such an opener

 @example
 ```ts
 const open = holdsOpenComment({ run: [88, 89,], slices, },);
 ```
 */
function holdsOpenComment(
  {
    run,
    slices,
  }: {
    readonly run: readonly number[];
    readonly slices: readonly WouldShipSlice[];
  },
): boolean {
  return commentRegionsOf({
    text: runTextOf({
      run,
      slices,
    },),
  },)
    .some(function neverCloses({ terminated, },): boolean {
      return !terminated;
    },);
}

/**
 Names slices the way every line of the composition prints them.

 @param indices - slice indices

 @returns `[1, 3]`

 @example
 ```ts
 const named = namedSlices({ indices: [1, 3,], },);
 ```
 */
function namedSlices({ indices, }: { readonly indices: readonly number[]; },): string {
  return `[${indices.join(', ',)}]`;
}

/**
 Composes the page of one entry, the originals of its untranslated passages included, so that it parses.

 @param artifact - settled entry, read for what each slice would carry

 @param slices - pairs the entry was prepared into, carrying the spans to write into

 @param archiveText - whole archive English the entry started from

 @param entryId - person entry being published, named in the lines the composition writes

 @param l - logger, tagged by the caller with this entry

 @param readGrammar - what the grammar says of a text, the strict MDX grammar the site compiles with by default

 @returns Page, the step that produced it and the defects to report; a page carrying no original is composed without
 reading it against the grammar at all

 @throws {@link SourcePageCompositionError} when the steps end without a page, which the last step makes impossible

 @example
 ```ts
 const composed = composeSourcePage({ artifact, slices, archiveText, entryId, l, },);
 ```
 */
export function composeSourcePage(
  {
    artifact,
    slices,
    archiveText,
    entryId,
    l,
    readGrammar = readPageGrammar,
  }: {
    readonly artifact: WouldShipSource;
    readonly slices: readonly ChunkPair[];
    readonly archiveText: string;
    readonly entryId: string;
    readonly l: Logger;
    readonly readGrammar?: GrammarReader;
  },
): ComposedPage {
  /**
   Logger naming this function.
   */
  const log = tagged({
    l,
    tag: composeSourcePage.name,
  },);
  /**
   What each slice would carry.
   */
  const readings = wouldShipTextPerSlice({ artifact, },);
  /**
   Slices of each run of slices shipping their original.
   */
  const runs = sourceRunsOf({ slices: readings, },);
  /**
   Every slice shipping its original.
   */
  const everySource = runs.flat();

  /**
   Composes the page with the given slices written as literal text, or without any original.

   @param encoded - slices to write as literal text

   @param includeSource - whether originals are on the page at all

   @returns Page and what it was spliced from

   @example
   ```ts
   const page = pageWith({ encoded: [], includeSource: true, },);
   ```
   */
  function pageWith(
    {
      encoded,
      includeSource,
    }: {
      readonly encoded: readonly number[];
      readonly includeSource: boolean;
    },
  ): Pick<ComposedPage, 'pageText' | 'replacements'> {
    /**
     What each slice contributes.
     */
    const replacements = shippableReplacements({
      artifact,
      includeSource,
      encodedSlices: new Set(encoded,),
    },);
    return {
      pageText: spliceSlices({
        targetText: archiveText,
        slices,
        replacements,
      },),
      replacements,
    };
  }

  /**
   The page with every original as the artifact records it.
   */
  const written = pageWith({
    encoded: [],
    includeSource: true,
  },);
  /**
   What the grammar says of it, which is not asked where no slice ships its original.
   */
  const firstReading: PageGrammarReading = (runs.length === 0)
    ? { kind: 'parses', }
    : readGrammar({ pageText: written.pageText, },);
  /**
   Slices of the runs holding a comment that never closes, which the grammar cannot vouch for.
   */
  const openComments = runs
    .filter(function opensComment(run,): boolean {
      return holdsOpenComment({
        run,
        slices: readings,
      },);
    },)
    .flat();
  if ((firstReading.kind === 'parses') && (openComments.length === 0)) {
    return {
      ...written,
      step: 'as-written',
      slices: [],
      defects: [],
    };
  }

  /**
   Slices of the runs that do not parse as a text of their own or hold a comment that never closes.
   */
  const failingAlone = runs
    .filter(function refusedAlone(run,): boolean {
      return holdsOpenComment({
        run,
        slices: readings,
      },)
        || (readGrammar({
          pageText: runTextOf({
            run,
            slices: readings,
          },),
        },)
          .kind
          === 'refused');
    },)
    .flat();
  /**
   Steps in order.
   */
  const steps: readonly Step[] = [
    {
      step: 'encoded-failing-runs',
      number: 1,
      does: 'writing the source runs that do not parse alone as literal text',
      makes: 'the page with the source runs that do not parse alone written as literal text',
      encoded: failingAlone,
      includeSource: true,
    },
    {
      step: 'encoded-every-run',
      number: 2,
      does: 'writing every source run as literal text',
      makes: 'the page with every source run written as literal text',
      encoded: everySource,
      includeSource: true,
    },
    {
      step: 'dropped',
      number: 3,
      does: 'shipping the page without its source runs',
      makes: 'the page without its source runs',
      encoded: [],
      includeSource: false,
    },
  ];
  /**
   The page last tried, what it is called and why the grammar refused it.
   */
  const last: LastTried = {
    pageText: written.pageText,
    what: 'the page with its source text as written',
    problem: (firstReading.kind === 'refused')
      ? `does not parse (${firstReading.refusal})`
      : `holds an HTML comment that never closes, slices ${namedSlices({ indices: openComments, },)}`,
  };
  for (const attempt of steps) {
    /**
     The page this step makes.
     */
    const candidate = pageWith({
      encoded: attempt.encoded,
      includeSource: attempt.includeSource,
    },);
    // A STEP THAT CHANGES NOTHING IS NOT TAKEN: the page is the one the grammar has just refused. The last step is
    // taken whatever it changes, since it is the one that always ships.
    if ((candidate.pageText === last.pageText) && (attempt.step !== 'dropped'))
      continue;
    /**
     Slices the step touches, in page order.
     */
    const touched = (attempt.step === 'dropped') ? everySource : attempt.encoded;
    log.warn(
      `entry ${entryId}: ${last.what} ${last.problem}; step ${String(attempt.number,)} of 3: ${
        attempt.does
      }, slices ${namedSlices({ indices: touched, },)}`,
    );
    if (attempt.step === 'dropped') {
      return {
        ...candidate,
        step: attempt.step,
        slices: touched,
        defects: [
          {
            check: 'page-source-unparseable',
            message: `entry ${entryId} ships without the source text of slices ${
              namedSlices({ indices: touched, },)
            }, since the page did not parse with it as written or as literal text`,
          },
        ],
      };
    }
    /**
     What the grammar says of the page this step makes.
     */
    const reading = readGrammar({ pageText: candidate.pageText, },);
    if (reading.kind === 'parses') {
      return {
        ...candidate,
        step: attempt.step,
        slices: touched,
        defects: [],
      };
    }
    last.pageText = candidate.pageText;
    last.what = attempt.makes;
    last.problem = `does not parse (${reading.refusal})`;
  }
  throw new SourcePageCompositionError({ entryId, },);
}

//endregion Source ships compose
