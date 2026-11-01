/**
 Tests that the naturalness PHASE computes the neighbouring window and
 hands it to the damage probe inside each slice's settlement.

 WHY THIS IS SEPARATE FROM THE SHEET TEST. `introduced-defect-wire` already
 renders a window it is HANDED, and the accuracy lane already hands it one.
 Whether the REFINEMENT lane computes one and passes it over is a different
 question, and it is the one that once went wrong: this lane's probe was
 called with no window at all for as long as it existed, so the naturalness
 lane's auditor reasoned about a slice alone while the accuracy lane's auditor
 reasoned about one in context, and their findings were never comparable.

 The failure mode is invisible to every other kind of test. The window is an
 optional property spread into an object literal, TypeScript does not
 excess-property-check a spread, and the probe decides nothing, so a lane that
 never passed it compiled, linted, and passed its own suite while asking the
 models a strictly smaller question.

 NO NETWORK. The client is a stub scripting the rewriter, the judges, the
 retention recheck and the probe, and recording every probe sheet.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type {
  ChunkRepairOutcome,
  RepairModels,
} from '@monochromatic-dev/translation-repair-assembly';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { runRefinePhase, } from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger for the phase under test.
 */
const l = tagged({ tag: 'refine-window-threading-test', },);

/**
 Markers no prompt constant and no other fixture contains, so a match in a
 sheet can only have come from the passage that carries it.

 Both sides of each slice are marked separately. The window renders the
 neighbours' ORIGINAL and their ARCHIVE ENGLISH into two different blocks, and
 a single marker per slice could not tell a threaded source window from a
 threaded incumbent one.
 */
const MARK = {
  previousSource: 'ZQPREVSRC',
  previousArchive: 'ZQPREVEN',
  middleSource: 'ZQMIDSRC',
  middleArchive: 'ZQMIDEN',
  nextSource: 'ZQNEXTSRC',
  nextArchive: 'ZQNEXTEN',
} as const;

/**
 Long single-line paragraph, which is the shape the lane finds refinable.

 @param mark - marker identifying which slice this is

 @returns Paragraph carrying that marker

 @example
 ```ts
 const text = paragraph({ mark: MARK.middleArchive, },);
 ```
 */
function paragraph({ mark, }: { readonly mark: string; },): string {
  return `The cat is doing the sunbathing on the windowsill in every afternoon ${mark}, and when the light is moving across the floor she is following it without any hurry at all.`;
}

/**
 Invented zh original of one slice.

 @param mark - marker identifying which slice this is

 @returns Original carrying that marker

 @example
 ```ts
 const text = original({ mark: MARK.middleSource, },);
 ```
 */
function original({ mark, }: { readonly mark: string; },): string {
  return `猫猫每天下午都在窗台上晒太阳 ${mark}。`;
}

/**
 Smoother rendering the scripted rewriter returns for every slice.

 CARRIES NO MARKER on purpose. It lands in the region blocks of every sheet,
 so a marker in it would appear in each slice's sheet and defeat attribution.
 */
const SMOOTH_TEXT =
  'The cat sunbathes on the windowsill every afternoon, and when the light moves across the floor she follows it without hurry.';

/**
 Roster with the lane on, refiners disjoint from checkers as the phase
 requires.
 */
const MODELS: RepairModels = {
  criticModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  panelModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  editorModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  judgeModelIds: [
    SEAT_HYPER_OPENROUTER_VISION_EDITOR,
    SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
    SEAT_SYNTHETIC_VISION_WITHHELD,
    SEAT_HYPER_OPENROUTER_UNMEASURED,
  ],
  refinerModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  checkerModelIds: [
    SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
    SEAT_SYNTHETIC_VISION_WITHHELD,
    SEAT_HYPER_OPENROUTER_UNMEASURED,
  ],
};

/**
 Separator between slices in the assembled translation.
 */
const GAP = '\n\n';

/**
 Builds the slice list and the document its offsets address.

 @param marks - source and archive marker per slice, in document order

 @returns Slices plus the assembled translation

 @example
 ```ts
 const prepared = prepare({ marks: [{ source: MARK.middleSource, archive: MARK.middleArchive, },], },);
 ```
 */
function prepare(
  {
    marks,
  }: {
    readonly marks: readonly { readonly source: string; readonly archive: string; }[];
  },
): { readonly slices: readonly ChunkPair[]; readonly targetText: string; } {
  /**
   Archive wording of each slice, in document order.
   */
  const targets = marks.map(function toTarget(mark,): string {
    return paragraph({ mark: mark.archive, },);
  },);

  /**
   Assembled translation the slice offsets address.
   */
  const documentText = targets.join(GAP,);

  return {
    targetText: documentText,
    slices: marks.map(function toSlice(mark, index,): ChunkPair {
      /**
       Original of this slice.
       */
      const sourceText = original({ mark: mark.source, },);

      /**
       Archive wording of this slice.
       */
      const targetText = targets[index] ?? '';

      /**
       Where this slice starts in the assembled translation.

       FOUND BY SEARCH RATHER THAN ACCUMULATED. Every paragraph carries its
       own marker, so each is unique in the document and the search cannot
       land on the wrong one; a running total would be a second statement of
       the same fact, able to disagree with the text it describes.
       */
      const startOffset = documentText.indexOf(targetText,);

      return {
        source: {
          sliceIndex: index,
          text: sourceText,
          startOffset: 0,
          endOffset: sourceText.length,
          nodes: [],
        },
        target: {
          sliceIndex: index,
          text: targetText,
          startOffset,
          endOffset: startOffset + targetText.length,
          nodes: [],
        },
      };
    },),
  };
}

/**
 Builds one settled accuracy outcome for a slice.

 @param sliceIndex - slice this outcome belongs to

 @param repairedText - what the accuracy pass settled, which the lane rewrites

 @returns Outcome the phase refines

 @example
 ```ts
 const outcome = settledOutcome({ sliceIndex: 0, repairedText, },);
 ```
 */
function settledOutcome(
  {
    sliceIndex,
    repairedText,
  }: {
    readonly sliceIndex: number;
    readonly repairedText: string;
  },
): ChunkRepairOutcome {
  return {
    sliceIndex,
    repairedText,
    changed: false,
    issues: [],
    resolvedIssueIds: [],
    candidateResolvedIssueIds: [],
    // No checker round in this fixture, so nothing was said about any issue.
    checkerReadings: {},
    recheckReadings: {},
    repairRegions: [],
    accuracyPatchSelected: false,
    refined: false,
    rounds: [],
    droppedDeclaredNames: [],
    // Hand-written fixture text, so nothing here has a model author.
    authorship: {
      perIssue: {},
      everyIssue: [],
    },
    nonTranslationVotes: 0,
    nonTranslationContradicted: false,
    nonTranslationStanding: false,
    heardCritics: 1,
    heardCriticIds: [],
    claimAttributions: [],
    findings: [],
    silentStages: [],
  };
}

/**
 Where the window begins in a probe sheet.

 The sheet renders the slice under review first and the neighbours after it,
 so splitting here separates what a sheet is ABOUT from what it was given as
 context. Skipping the split produces a false pass: every sheet mentions its
 neighbours' markers once the window is threaded, so "this sheet contains the
 previous marker" is true of the previous slice's own sheet too.
 */
const NEARBY_FENCE = 'NEARBY ORIGINAL';

/**
 One probe sheet, split into the pair under review and the window.
 */
type ProbeSheet = {
  /**
   Everything before the nearby fence: the slice being audited.
   */
  readonly reviewed: string;

  /**
   The nearby fence onwards, empty on a slice standing alone.
   */
  readonly window: string;
};

/**
 Runs the phase and returns every sheet its damage probe asked.

 @param marks - source and archive marker per slice, in document order

 @returns Distinct probe sheets, split at the fence, in the order they were
 first asked

 @example
 ```ts
 const sheets = await probeSheets({ marks, },);
 ```
 */
async function probeSheets(
  {
    marks,
  }: {
    readonly marks: readonly { readonly source: string; readonly archive: string; }[];
  },
): Promise<readonly ProbeSheet[]> {
  const { slices, targetText, } = prepare({ marks, },);

  /**
   User sheet of every probe exchange, in order.
   */
  const probed: string[] = [];

  /**
   Client scripting each stage by the schema it asks for.
   */
  const client: ModelClient = {
    chatText: async () => {
      throw new Error('chatText unused by the phase',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Stage name from the structured-output constraint.
       */
      const stage = request.responseFormat
        ?.json_schema
        .name
        ?? '';

      /**
       User prompt of this exchange.
       */
      const asked = request.messages.at(-1,);
      const content = (asked === undefined) ? '' : messageText({ message: asked, },);
      if (stage === 'introduced_defect_report')
        probed.push(content,);

      /**
       Scripted reply for the stage.
       */
      const scripted: unknown = stage === 'refine_report'
        ? {
          rewrites: [
            {
              paragraph: 1,
              newText: SMOOTH_TEXT,
            },
          ],
        }
        : stage === 'candidate_ballot'
          ? {
            best: 1,
            assessment: 'scripted',
          }
          : stage === 'introduced_defect_report'
            ? {
              checks: [
                {
                  region: 1,
                  verdict: 'no-introduced-defect-found',
                  category: '',
                  severity: '',
                  evidence: '',
                  omittedText: '',
                  verdictReason: '',
                },
              ],
            }
            : { checks: [], };
      if (!request.validate(scripted,))
        throw new Error(`stub script failed the ${stage} guard`,);
      return {
        kind: 'ok',
        value: scripted,
        rawText: JSON.stringify(scripted,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by the phase',);
    },
  };

  await runRefinePhase({
    client,
    targetText,
    slices,
    outcomes: slices.map(function toOutcome(slice,): ChunkRepairOutcome {
      return settledOutcome({
        sliceIndex: slice.target
          .sliceIndex,
        repairedText: slice.target
          .text,
      },);
    },),
    models: MODELS,
    declaredNames: [],
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);

  // DISTINCT SHEETS, because the probe asks EVERY prober in the roster the same
  // sheet and the roster holds three. Keeping the duplicates would make the
  // one-sheet-per-slice attribution refuse on a run that behaved
  // perfectly, and loosening that attribution instead would give up the check
  // that a sheet belongs to the slice it claims.
  return [...new Set(probed,),].map(function split(sheet,): ProbeSheet {
    /**
     Where the window begins, absent on a slice standing alone.
     */
    const at = sheet.indexOf(NEARBY_FENCE,);
    if (at === (-1))
      return {
        reviewed: sheet,
        window: '',
      };
    return {
      reviewed: sheet.slice(0, at,),
      window: sheet.slice(at,),
    };
  },);
}

/**
 The one sheet whose reviewed half carries a marker.

 @param sheets - split sheets from {@link probeSheets}

 @param marker - marker identifying the slice

 @returns That slice's sheet

 @throws When no sheet or more than one reviews it, which means the phase
 asked a different set of questions than this test describes

 @example
 ```ts
 const sheet = about({ sheets, marker: MARK.middleSource, },);
 ```
 */
function about(
  {
    sheets,
    marker,
  }: {
    readonly sheets: readonly ProbeSheet[];
    readonly marker: string;
  },
): ProbeSheet {
  /**
   Sheets reviewing that slice, which must be exactly one.
   */
  const own = sheets.filter(function reviewsIt(sheet,): boolean {
    return sheet.reviewed
      .includes(marker,);
  },);
  if (own.length !== 1) {
    throw new Error(
      `expected exactly one probe sheet reviewing ${marker}, found ${String(own.length,)}`,
    );
  }
  return own[0] as ProbeSheet;
}

/**
 Slices of the three-slice fixture, in document order.
 */
const THREE = [
  {
    source: MARK.previousSource,
    archive: MARK.previousArchive,
  },
  {
    source: MARK.middleSource,
    archive: MARK.middleArchive,
  },
  {
    source: MARK.nextSource,
    archive: MARK.nextArchive,
  },
] as const;

await describe({
  name: 'refinement damage probe window',
  children: [
    it({
      name: 'SHOWS THE PROBE BOTH NEIGHBOURS of the slice it is auditing, on '
        + 'both sides of the window: the original of the passages either side '
        + 'and the archive English of those same two. Without them this lane '
        + 'audits a rewrite alone while the accuracy lane audits one in '
        + 'context, and a fluency rewrite that drops a phrase because the '
        + 'paragraph next door already said it reads as a deletion',
      fn: async () => {
        const sheets = await probeSheets({ marks: THREE, },);

        /**
         Sheet auditing the middle slice, the only one with a neighbour on
         each side.
         */
        const middle = about({
          sheets,
          marker: MARK.middleSource,
        },);

        expect(middle.window,).toContain(MARK.previousSource,);
        expect(middle.window,).toContain(MARK.nextSource,);
        expect(middle.window,).toContain(MARK.previousArchive,);
        expect(middle.window,).toContain(MARK.nextArchive,);
      },
    },),

    it({
      name: 'ASKS A SLICE STANDING ALONE exactly what it was asked before the '
        + 'window existed, with no nearby block at all. This is the control '
        + 'the other case needs: a fence rendered unconditionally would '
        + 'satisfy every assertion of that case while carrying nothing',
      fn: async () => {
        const sheets = await probeSheets({
          marks: [
            {
              source: MARK.middleSource,
              archive: MARK.middleArchive,
            },
          ],
        },);

        expect(about({
          sheets,
          marker: MARK.middleSource,
        },).window,).toBe('',);
      },
    },),
  ],
},);
