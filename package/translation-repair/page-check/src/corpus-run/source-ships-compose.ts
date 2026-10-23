import {
  type Logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';
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
import { withoutSourceRecordAt, } from './source-ships.ts';
import { runsUnfitForCompany, } from './source-ships-company.ts';
import {
  droppedRunSets,
  type GrammarReader,
  holdsOpenComment,
  namedSlices,
  runTextOf,
  sourceRunsOf,
} from './source-ships-runs.ts';
import {
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
// 1. Only the source runs that do not parse as a text of their own, those holding an HTML comment that never closes,
//    and those that parse alone and cannot stand beside another (a fence mark indented four spaces or more; a name an
//    earlier run still declares, `source-ships-company.ts`), are written as literal text
//    (`source-ships-encoding.ts`). A run is a contiguous run of slices shipping their original, the unit the note
//    stands before, so a tag opened in one slice and closed in the next is read as the pair it is.
// 2. If the page still does not parse, every source run is written as literal text.
// 3. If it still does not, the page ships without the source runs no writing makes parse (the runs the grammar refuses
//    even as literal text, `source-ships-runs.ts`), and where that does not help or no such run is found, without
//    every source run, the silent gap the pages had before the ruling. The defect `page-source-unparseable` names the
//    slices left off, and the page is weighed against the artifact without those slices' originals
//    (`ComposedPage.weighed`), the same sets a page on disk is judged against (`page-agreement.ts`).
//
// Each step that is taken writes one warn line naming the step and the slices. A step whose page equals the page last
// tried changed nothing and is not taken. A run holding a comment that never closes is one no grammar reading can
// vouch for: the package masks everything after such an opener, so a page that "parses" with it hides the rest of the
// page from the package's own parser, and it never ships as written. The original differs from the artifact's record
// only on these paths; with the page parsing as recorded, nothing here touches a byte of it.

/**
 Names a step of the composition.

 `as-written` is the page with every original as recorded, `encoded-failing-runs` the runs that do not parse alone,
 hold a comment that never closes or cannot stand beside another written as literal text, `encoded-every-run` all of
 them, and `dropped` the page without any.
 */
export type SourcePageStep = 'as-written' | 'dropped' | 'encoded-every-run' | 'encoded-failing-runs';

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

  /**
   The artifact the page is weighed against: the settled one itself, or without its record of the originals the page
   was shipped without, whose absence `page-source-unparseable` already names.
   */
  readonly weighed: WouldShipSource;
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
   Composes the page with the given slices written as literal text and the given slices left off it.

   @param encoded - slices to write as literal text

   @param dropped - slices whose original is not on the page

   @returns Page, what it was spliced from and the artifact it is weighed against

   @example
   ```ts
   const page = pageWith({ encoded: [], dropped: [], },);
   ```
   */
  function pageWith(
    {
      encoded,
      dropped,
    }: {
      readonly encoded: readonly number[];
      readonly dropped: readonly number[];
    },
  ): Pick<ComposedPage, 'pageText' | 'replacements' | 'weighed'> {
    /**
     The artifact the page is read from: the settled one, or without its record of the originals left off.
     */
    const weighed = withoutSourceRecordAt({
      artifact,
      slices: dropped,
    },);
    /**
     What each slice contributes.
     */
    const replacements = shippableReplacements({
      artifact: weighed,
      includeSource: true,
      encodedSlices: new Set(encoded,),
    },);
    return {
      pageText: spliceSlices({
        targetText: archiveText,
        slices,
        replacements,
      },),
      replacements,
      weighed,
    };
  }

  /**
   The page with every original as the artifact records it.
   */
  const written = pageWith({
    encoded: [],
    dropped: [],
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
   Runs that parse alone and cannot stand beside another: those with a fence mark indented four spaces or more and
   those redeclaring a name.
   */
  const unfit = runsUnfitForCompany({
    runs,
    slices: readings,
  },);
  /**
   Slices of the runs that do not parse as a text of their own, hold a comment that never closes, or cannot stand beside
   another.
   */
  const failingAlone = runs
    .filter(function refusedAlone(run,): boolean {
      return unfit.includes(run,) || holdsOpenComment({
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
   Steps that write originals as literal text, in order.
   */
  const steps: readonly Step[] = [
    {
      step: 'encoded-failing-runs',
      number: 1,
      does: 'writing the source runs that do not parse alone, or cannot stand beside another, as literal text',
      makes: 'the page with the source runs that do not parse alone, or cannot stand beside another, written as '
        + 'literal text',
      encoded: failingAlone,
    },
    {
      step: 'encoded-every-run',
      number: 2,
      does: 'writing every source run as literal text',
      makes: 'the page with every source run written as literal text',
      encoded: everySource,
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
      dropped: [],
    },);
    // A STEP THAT CHANGES NOTHING IS NOT TAKEN: the page is the one the grammar has just refused.
    if (candidate.pageText === last.pageText)
      continue;
    log.warn(
      `entry ${entryId}: ${last.what} ${last.problem}; step ${String(attempt.number,)} of 3: ${
        attempt.does
      }, slices ${namedSlices({ indices: attempt.encoded, },)}`,
    );
    /**
     What the grammar says of the page this step makes.
     */
    const reading = readGrammar({ pageText: candidate.pageText, },);
    if (reading.kind === 'parses') {
      return {
        ...candidate,
        step: attempt.step,
        slices: attempt.encoded,
        defects: [],
      };
    }
    last.pageText = candidate.pageText;
    last.what = attempt.makes;
    last.problem = `does not parse (${reading.refusal})`;
  }

  // THE THIRD STEP SHIPS WITHOUT THE RUNS NO WRITING MAKES PARSE, then without every run. It is taken whatever it
  // changes, since it is the one that always ships, and the sets are the ones a page on disk is judged against.
  /**
   Sets of slices to leave off, the last of which is every run.
   */
  const dropSets = droppedRunSets({
    artifact,
    readGrammar,
  },);
  for (const [
    at,
    dropped,
  ] of dropSets.entries()) {
    /**
     The page without these slices, the ones failing alone and still on it written as literal text.
     */
    const candidate = pageWith({
      encoded: failingAlone.filter(function stillOnPage(slice,): boolean {
        return !dropped.includes(slice,);
      },),
      dropped,
    },);
    // A SET BEFORE THE LAST IS TAKEN ONLY WHEN THE PAGE WITHOUT IT PARSES.
    if (at < (dropSets.length - 1)) {
      /**
       What the grammar says of the page without these slices.
       */
      const without = readGrammar({ pageText: candidate.pageText, },);
      if (without.kind === 'refused')
        continue;
    }
    log.warn(
      `entry ${entryId}: ${last.what} ${last.problem}; step 3 of 3: ${
        (dropped.length === everySource.length)
          ? 'shipping the page without its source runs'
          : 'shipping the page without the source runs no writing makes parse'
      }, slices ${namedSlices({ indices: dropped, },)}`,
    );
    return {
      ...candidate,
      step: 'dropped',
      slices: dropped,
      defects: [
        {
          check: 'page-source-unparseable',
          message: `entry ${entryId} ships without the source text of slices ${
            namedSlices({ indices: dropped, },)
          }, since the page did not parse with it as written or as literal text`,
        },
      ],
    };
  }
  throw new SourcePageCompositionError({ entryId, },);
}

//endregion Source ships compose
