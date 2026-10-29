/**
 End-to-end test of the repair-provenance chain: slice outcomes into issue
 records, records into artifact JSON, artifact JSON back into grading
 candidates, candidates into both sheets.

 Every other suite fixes its own fixture at one module's boundary, so all of
 them can agree with the code they test and disagree with each other. This one
 runs the real path a corpus entry takes, which is the only place a field
 renamed on one side and read on the other actually shows up.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { PatchOperation, } from '@monochromatic-dev/translation-repair-page-pass';
import type {
  AdjudicatedIssue,
  ClaimPanelReading,
  EditableEnvelope,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  collectRepairRegions,
  type IssueCheckerReading,
  parseGradedRepairSheet,
} from '@monochromatic-dev/translation-repair-adjudication';
import {
  buildIssueRecords,
  type ChunkRepairOutcome,
  formatRepairSheet,
} from '@monochromatic-dev/translation-repair-assembly';
import {
  assertSourceBytes,
  classifyBand,
  extractGradingCandidate,
  formatGradingSheet,
  type SizeBand,
} from '@monochromatic-dev/translation-repair-house-rule';
import { parseSettledArtifact, } from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Bands a raw byte count, stating the unit explicitly.

 These cases probe the band BOUNDARIES, which are byte counts by
 definition and cannot be produced from text, so the assertion is the
 honest way to reach `classifyBand` rather than a cast around its guard.

 @param count - UTF-8 byte length under test

 @returns Band that count falls in

 @example
 ```ts
 expect(bandAt(1_842,),).toBe('small',);
 ```
 */
function bandAt(count: number,): SizeBand {
  assertSourceBytes(count,);
  return classifyBand({ sourceBytes: count, },);
}


/**
 Translation before repair.
 */
const TARGET_TEXT = 'The cat is doing the sleeping. She wakes at dusk.';

/**
 Envelope content the editor replaced.
 */
const REPLACED = 'The cat is doing the sleeping.';

/**
 Replacement the accuracy stage wrote.
 */
const REPLACEMENT = 'The cat is asleep.';

/**
 Slice text after applying that replacement.
 */
const PATCHED_TEXT = 'The cat is asleep. She wakes at dusk.';

/**
 Weighted mass behind this fixture's claim, as the panel summed it.
 */
const PANEL_TALLY = {
  supported: 2,
  unsupported: 0,
  ambiguous: 0,
  sourceDefect: 0,
  abstain: 1,
};

/**
 Ballots behind {@link PANEL_TALLY}, one panelist having abstained.

 THE ABSTENTION IS THE POINT. It leaves a ballot, while a panelist whose
 reply never arrived leaves none, and a reader holding only the sums cannot
 tell those apart at all.
 */
const PANEL_READING = {
  ballots: [
    {
      panelistId: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
      vote: 'supported' as const,
      weight: 1,
    },
    {
      panelistId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
      vote: 'supported' as const,
      weight: 1,
    },
    {
      panelistId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
      vote: 'abstain' as const,
      weight: 1,
    },
  ],
  configuredPanelists: 4,
  tally: PANEL_TALLY,
} as const satisfies ClaimPanelReading;

/**
 Accepted issue the replacement was written for.
 */
const ISSUE: AdjudicatedIssue = {
  issueId: 'adjudicated/nap',
  status: 'accepted' as const,
  severity: 'major' as const,
  claims: [
    {
      claimId: 'claim/nap',
      claim: {
        category: 'style/awkward-phrasing' as const,
        severity: 'major' as const,
        summary: 'Progressive gloss reads as machine output.',
        spans: [
          {
            side: 'source' as const,
            nodeId: 'block/0',
            nodeHash: hashContent({ content: '猫猫在睡觉。', },),
            startOffset: 0,
            endOffset: 6,
            quotedText: '猫猫在睡觉。',
          },
          {
            side: 'target' as const,
            nodeId: 'block/0',
            nodeHash: hashContent({ content: TARGET_TEXT, },),
            startOffset: 0,
            endOffset: REPLACED.length,
            quotedText: REPLACED,
          },
        ],
      },
    },
  ],
  tallies: { 'claim/nap': PANEL_TALLY, },

  // THE BALLOTS BEHIND THAT TALLY. Two supported and one abstained, which
  // the five numbers alone cannot say.
  readings: { 'claim/nap': PANEL_READING, },
};

/**
 Envelope cut from that issue's target-side evidence.
 */
const ENVELOPE: EditableEnvelope = {
  envelopeId: 'envelope/nap',
  startOffset: 0,
  endOffset: REPLACED.length,
  baseText: REPLACED,
  baseHash: hashContent({ content: REPLACED, },),
  issueIds: [ISSUE.issueId,],
};

/**
 Operation the apply gate accepted.
 */
const OPERATION: PatchOperation = {
  envelopeId: ENVELOPE.envelopeId,
  baseHash: ENVELOPE.baseHash,
  newText: REPLACEMENT,
};

/**
 Checker round behind this fixture's verdict: resolved two to one.

 Cat-themed invention. Model ids come from the catalog because that union is
 closed.
 */
const CHECKER_READING = {
  ballots: [
    {
      modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
      verdict: 'fixed',
      wroteTheText: false,
    },
    {
      modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
      verdict: 'fixed',
      wroteTheText: false,
    },
    {
      modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
      verdict: 'not-fixed',
      wroteTheText: false,
    },
  ],
  configuredCheckers: 3,
  tally: {
    fixed: 2,
    notFixed: 1,
    worse: 0,
    resolved: true,
    regressed: false,
  },
} as const satisfies IssueCheckerReading;

/**
 What the same three said when asked again about the REFINED text.

 DELIBERATELY NOT THE SAME SHAPE as `CHECKER_READING`, and one voice changed
 its answer between them. That is the state the second field exists to keep:
 two rounds rule on the same issue id, so a reader holding one merged record
 could not tell which round said what, and the dissent that the deciding round
 carried would look as though it had never happened.
 */
const RECHECK_READING = {
  ballots: [
    {
      modelId: SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
      verdict: 'fixed',
      wroteTheText: false,
    },
    {
      modelId: SEAT_SYNTHETIC_TEXT_EVERYWHERE,
      verdict: 'fixed',
      wroteTheText: false,
    },
    {
      modelId: SEAT_HYPER_OPENROUTER_UNMEASURED,
      verdict: 'fixed',
      wroteTheText: false,
    },
  ],
  configuredCheckers: 3,
  tally: {
    fixed: 3,
    notFixed: 0,
    worse: 0,
    resolved: true,
    regressed: false,
  },
} as const satisfies IssueCheckerReading;

/**
 Builds the settled slice outcome the way `repairChunk` returns it.

 @param accuracyPatchSelected - whether the patched candidate won

 @returns Outcome the driver flattens

 @example
 ```ts
 const outcome = settledOutcome({ accuracyPatchSelected: true, },);
 ```
 */
function settledOutcome(
  { accuracyPatchSelected, }: { readonly accuracyPatchSelected: boolean; },
): ChunkRepairOutcome {
  return {
    sliceIndex: 0,
    repairedText: accuracyPatchSelected ? PATCHED_TEXT : TARGET_TEXT,
    changed: accuracyPatchSelected,
    issues: [ISSUE,],
    resolvedIssueIds: accuracyPatchSelected ? [ISSUE.issueId,] : [],
    candidateResolvedIssueIds: [ISSUE.issueId,],
    // A REAL ROUND, two to one, because the point of carrying it is that a
    // one-vote majority and a unanimous one stop looking alike on disk.
    checkerReadings: { [ISSUE.issueId]: CHECKER_READING, },
    recheckReadings: { [ISSUE.issueId]: RECHECK_READING, },
    repairRegions: collectRepairRegions({
      envelopes: [ENVELOPE,],
      applied: [OPERATION,],
    },),
    accuracyPatchSelected,
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
    heardCritics: 6,
    heardCriticIds: [],
    claimAttributions: [],
    findings: [],
    silentStages: [],
  };
}

/**
 Runs the whole chain a corpus entry takes: outcome to records, records
 through the artifact JSON the pass writes, artifact back to candidates.

 @param accuracyPatchSelected - whether the patched candidate won

 @returns Candidates the sheets render

 @example
 ```ts
 const candidates = throughArtifact({ accuracyPatchSelected: true, },);
 ```
 */
function throughArtifact(
  { accuracyPatchSelected, }: { readonly accuracyPatchSelected: boolean; },
) {
  /**
   Whole-document issue report, exactly as the driver builds it.
   */
  const issues = buildIssueRecords({
    outcomes: [settledOutcome({ accuracyPatchSelected, },),],
    blocked: false,
  },);

  /**
   Artifact text exactly as the corpus pass writes it to disk.

   Serialized and re-read rather than passed as an object on purpose, and NOT
   a deep clone the way `structuredClone` would be: JSON drops what
   `structuredClone` keeps, and every optional field on a repair record
   (`introducedDefects`, `finalSliceText`) is exactly the kind of thing that
   survives a clone and vanishes through a file. The disk boundary is the
   thing under test.
   */
  const onDisk = JSON.stringify({
    id: 'Kitten',
    status: 'repaired',
    acceptedCount: 1,
    issues,
    repairedText: accuracyPatchSelected ? PATCHED_TEXT : TARGET_TEXT,
  },);

  /**
   Artifact read back out of that text.
   */
  const artifact: unknown = JSON.parse(onDisk,);

  /**
   Accepted issues read back out.
   */
  const parsed = parseSettledArtifact({ value: artifact, },);

  return parsed.acceptedIssues
    .map(function toCandidate(accepted,) {
      /**
       What the artifact recorded about this issue's repair.
       */
      const reading = accepted.repair;

      return extractGradingCandidate({
        issue: accepted.issue,
        entryId: parsed.id,
        band: bandAt(2_000,),
        ...(reading.kind === 'unrecorded'
          ? {}
          : { repair: reading.repair, }),
      });
    },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'repair provenance end to end',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'carries a shipped repair from the slice outcome through the '
            + 'artifact onto the repair sheet, with the replacement intact',
          fn: async () => {
            const candidates = throughArtifact({ accuracyPatchSelected: true, },);
            expect(candidates,).toHaveLength(1,);
            expect(candidates[0]?.repair?.disposition,).toBe('shipped',);
            expect(candidates[0]?.repair?.regions[0]?.editorAfter,)
              .toBe(REPLACEMENT,);

            /** Repair sheet over the round-tripped candidate. */
            const sheet = formatRepairSheet({
              sample: candidates,
              seed: 'cat-seed',
              corpusSha: 'sha/1',
              drawDigest: 'digest-of-this-draw',
            },);
            expect(sheet.includes(REPLACEMENT,),).toBe(true,);
            expect(sheet.includes('- repair grade: [ ]',),).toBe(true,);

            // The claim that justifies omitting finalSliceText, checked against the
            // text the run actually returns rather than against a fixture constant.
            expect(candidates[0]?.repair?.finalSliceText,).toBeUndefined();
            expect(PATCHED_TEXT.includes(REPLACEMENT,),).toBe(true,);
          },
        },),

        it({
          name: 'reads its own graded sheet back, closing the loop between the '
            + 'formatter and the parser rather than testing each against a fixture '
            + 'of the other\'s shape',
          fn: async () => {
            const candidates = throughArtifact({ accuracyPatchSelected: true, },);

            /** Sheet exactly as the runbook hands it to the grader. */
            const blank = formatRepairSheet({
              sample: candidates,
              seed: 'cat-seed',
              corpusSha: 'sha/1',
              drawDigest: 'digest-of-this-draw',
            },);

            /** Same sheet with the one box filled in, as a grader leaves it. */
            const graded = blank.replace(
              '- repair grade: [ ]',
              '- repair grade: [Y, the tense now matches]',
            );

            /** Verdicts read back off the grader's file. */
            const items = parseGradedRepairSheet({ text: graded, },);
            expect(items,).toHaveLength(candidates.length,);
            expect(items[0]?.verdict,).toBe('fixes',);
            expect(items[0]?.note,).toBe('the tense now matches',);

            // The ungraded sheet must read as unscored, or a blank run would look
            // like a graded one whose repairs all failed.
            expect(
              parseGradedRepairSheet({ text: blank, },)[0]
                ?.verdict,
            ).toBe('unscored',);
          },
        },),

        it({
          name: 'carries a rejected repair through as not-selected, so the sheet '
            + 'shows what was attempted and asks for no grade',
          fn: async () => {
            const candidates = throughArtifact({ accuracyPatchSelected: false, },);
            expect(candidates[0]?.repair?.disposition,).toBe('not-selected',);

            /** Repair sheet over the rejected repair. */
            const sheet = formatRepairSheet({
              sample: candidates,
              seed: 'cat-seed',
              corpusSha: 'sha/1',
              drawDigest: 'digest-of-this-draw',
            },);
            expect(sheet.includes(REPLACEMENT,),).toBe(true,);
            expect(sheet.includes('- repair grade: [ ]',),).toBe(false,);
            expect(sheet.includes('counts against coverage',),).toBe(true,);
          },
        },),

        it({
          name: 'leaves the detection sheet identical whether a repair shipped or '
            + 'not, which is what makes the two numbers independent',
          fn: async () => {
            /** Detection sheet where the repair shipped. */
            const shipped = formatGradingSheet({
              sample: throughArtifact({ accuracyPatchSelected: true, },),
              seed: 'cat-seed',
              bar: 0.9,
              corpusSha: 'sha/1',
              drawDigest: 'digest-of-this-draw',
            },);

            /** Detection sheet where it did not. */
            const rejected = formatGradingSheet({
              sample: throughArtifact({ accuracyPatchSelected: false, },),
              seed: 'cat-seed',
              bar: 0.9,
              corpusSha: 'sha/1',
              drawDigest: 'digest-of-this-draw',
            },);
            expect(shipped,).toBe(rejected,);
            expect(shipped.includes(REPLACEMENT,),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: 'checker round across the disk boundary',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CARRIES every ballot and the seated roster through the artifact JSON, because '
            + '`resolved` alone cannot say whether the checkers were unanimous or one vote apart, and a '
            + 'field that survives a clone can still vanish through a file',
          fn: async () => {
            /**
             Issue report as bytes. Written and re-read rather than cloned: JSON
             drops what `structuredClone` keeps, and an optional field is exactly
             what survives a clone and vanishes through a file.
             */
            const onDisk = JSON.stringify(buildIssueRecords({
              outcomes: [settledOutcome({ accuracyPatchSelected: true, },),],
              blocked: false,
            },),);

            /** Those bytes read back. */
            const readBack: unknown = JSON.parse(onDisk,);
            if (!Array.isArray(readBack,))
              throw new Error('issue report should read back as a list',);

            /** First record, as it came off disk. */
            const [record,] = readBack as readonly { readonly checkerReading?: unknown; readonly resolved?: unknown; }[];
            expect(record?.resolved,).toBe(true,);
            expect(record?.checkerReading,).toEqual(CHECKER_READING,);
          },
        },),

        it({
          name: 'RECORDS NOTHING for an issue no checker ruled on, so an absent reading never reads as '
            + 'agreement',
          fn: async () => {
            /** Outcome whose checker round said nothing about this issue. */
            const silent = {
              ...settledOutcome({ accuracyPatchSelected: true, },),
              checkerReadings: {},
              recheckReadings: {},
            };

            /** Report over that outcome, as bytes. */
            const onDisk = JSON.stringify(buildIssueRecords({
              outcomes: [silent,],
              blocked: false,
            },),);

            /** Those bytes read back. */
            const readBack: unknown = JSON.parse(onDisk,);
            if (!Array.isArray(readBack,))
              throw new Error('issue report should read back as a list',);
            const [record,] = readBack as readonly { readonly checkerReading?: unknown; }[];
            expect(record?.checkerReading,).toBe(undefined,);
          },
        },),

        it({
          name: 'KEEPS the refinement recheck as its own field rather than folding it into the deciding '
            + 'round, since both rule on the same issue id and only the first one `resolved` rests on',
          fn: async () => {
            /** Issue report as bytes, carrying both rounds. */
            const onDisk = JSON.stringify(buildIssueRecords({
              outcomes: [settledOutcome({ accuracyPatchSelected: true, },),],
              blocked: false,
            },),);

            /** Those bytes read back. */
            const readBack: unknown = JSON.parse(onDisk,);
            if (!Array.isArray(readBack,))
              throw new Error('issue report should read back as a list',);

            /** First record, as it came off disk. */
            const [record,] = readBack as readonly {
              readonly checkerReading?: unknown;
              readonly recheckReading?: unknown;
            }[];
            expect(record?.checkerReading,).toEqual(CHECKER_READING,);
            expect(record?.recheckReading,).toEqual(RECHECK_READING,);

            // THE POINT OF TWO FIELDS. One voice answered differently across the
            // rounds, and a merged record could not report that at all.
            expect(record?.recheckReading,).not.toEqual(record?.checkerReading,);
          },
        },),

        it({
          name: 'RECORDS NOTHING for a slice the naturalness lane never rewrote, which is most of them, '
            + 'so an absent recheck never reads as a rewrite that held',
          fn: async () => {
            /** Outcome the refinement lane bought no second round on. */
            const unrefined = {
              ...settledOutcome({ accuracyPatchSelected: true, },),
              recheckReadings: {},
            };

            /** Report over that outcome, as bytes. */
            const onDisk = JSON.stringify(buildIssueRecords({
              outcomes: [unrefined,],
              blocked: false,
            },),);

            /** Those bytes read back. */
            const readBack: unknown = JSON.parse(onDisk,);
            if (!Array.isArray(readBack,))
              throw new Error('issue report should read back as a list',);
            const [record,] = readBack as readonly {
              readonly checkerReading?: unknown;
              readonly recheckReading?: unknown;
            }[];
            expect(record?.recheckReading,).toBe(undefined,);

            // The deciding round is untouched by the lane never running.
            expect(record?.checkerReading,).toEqual(CHECKER_READING,);
          },
        },),

        it({
          name: 'CARRIES THE ADJUDICATION PANEL\'S BALLOTS through the same file, since the accept gate decides whether an issue exists at all and its five weighted numbers cannot say who voted or who abstained',
          fn: async () => {
            /** Issue report as bytes. */
            const onDisk = JSON.stringify(buildIssueRecords({
              outcomes: [settledOutcome({ accuracyPatchSelected: true, },),],
              blocked: false,
            },),);

            /** Those bytes read back. */
            const readBack: unknown = JSON.parse(onDisk,);
            if (!Array.isArray(readBack,))
              throw new Error('issue report should read back as a list',);

            /** First record, as it came off disk. */
            const [record,] = readBack as readonly {
              readonly issue?: {
                readonly readings?: Readonly<Record<string, unknown>>;
                readonly tallies?: Readonly<Record<string, unknown>>;
              };
            }[];
            expect(record?.issue?.readings?.['claim/nap'],).toEqual(PANEL_READING,);

            // KEYED AS THE TALLIES ARE, so nothing has to guess the pairing.
            expect(Object.keys(record?.issue?.readings ?? {},),).toStrictEqual(
              Object.keys(record?.issue?.tallies ?? {},),
            );
          },
        },),
      ],
    },),
  ],
},);
