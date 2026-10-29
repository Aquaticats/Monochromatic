/**
 Tests for the report `score-probe` prints over a run's artifacts and, where
 two flags name them, over a graded repair sheet and its manifest.

 Each case writes its own runs directory, hands it and a command line to
 `printProbeScore` and reads the whole of what was printed, or the refusal
 that stopped it.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { RunJsonUnreadableError, } from '@monochromatic-dev/translation-repair-shared-contract';
import { printProbeScore, } from '../../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { writeScoreArtifacts, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/score-artifacts.test-fixture.ts';
import { builtPipelineDigest, } from '@monochromatic-dev/translation-repair-glossary/ts/corpus-run/built-pipeline-digest.test-fixture.ts';
import {
  probeArtifactText,
  probedRecord,
  probeRegion,
} from '@monochromatic-dev/translation-repair-artifact/ts/corpus-run/score-probe-artifacts.test-fixture.ts';
import {
  catDrawDigest,
  catRepairSheetText,
  catSampleManifestText,
} from '@monochromatic-dev/translation-repair-lane-contest/ts/corpus-run/score-sheets.test-fixture.ts';
import {
  NO_POOL_POLICY,
  UNASKED_REPOSITORY,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts/corpus-run/pool-policy.test-fixture.ts';

/**
 What the pool prints as the pipeline that read it.
 */
const POOL_STAMP = await builtPipelineDigest({ packageEntry: import.meta.resolve('@monochromatic-dev/translation-repair-artifact-pool',), },);

/**
 Draw seed every fixture sheet declares.
 */
const SEED = 'round-cats';

/**
 Corpus commit every fixture sheet declares.
 */
const CORPUS = 'abc1234';

/**
 Issues the fixture manifest orders, the last of which no probe read.
 */
const ISSUES = [
  'adjudicated/nap',
  'adjudicated/chase',
  'adjudicated/purr',
  'adjudicated/wander',
];

/**
 The lines printed before any grade is joined, for the fixture artifact.
 */
const TELEMETRY_LINES = [
  'PROBE entries=1 repairShippedRecords=3 repairUnprobedRecords=0 regions=3 majorityIntroduced=2 '
    + 'minorityIntroduced=0 noneIntroduced=1',
  'CLAIMS added=4 dropped=0 contradicted=0 unanchored=0 degradedRosterRegions=0',
  'REFINEMENT rewrittenSlices=0 majorityIntroduced=0 minorityIntroduced=0 noneIntroduced=0 added=0 '
    + 'dropped=0 contradicted=0 unanchored=0',
  'ROSTER editorOffered=0 editorDegraded=0 editorSilent=0 refineOffered=0 refineDegraded=0 refineSilent=0 '
    + 'entriesWithRewrites=1/1',
  'NOTE rewrittenSlices=0 means no artifact here carries a refinement audit. That is what artifacts written '
    + 'before the lane was audited look like, and it is NOT evidence the lane rewrote nothing: read the ROSTER '
    + 'line to tell the two apart.',
];

/**
 Closing note of an agreement report.
 */
const CLEAN_NOTE = 'NOTE refutedByHuman is the clean number: the human read the same wording and said it breaks '
  + 'nothing nearby, so each one is a correct repair a gate would have discarded. sharedWithHuman is NOT '
  + 'confirmation, since the sheet\'s N fires both for a repair that did not fix its target and for one that '
  + 'broke something.';

/**
 Writes the one artifact the cases read: three shipped records, two of them
 flagged by a majority of probers and one of those rewritten afterwards.

 @param runsDir - runs directory the case owns

 @example
 ```ts
 await writeProbedRun({ runsDir, },);
 ```
 */
async function writeProbedRun({ runsDir, }: { readonly runsDir: string; },): Promise<void> {
  await writeScoreArtifacts({
    runsDir,
    artifacts: {
      'Whiskers.json': probeArtifactText({
        entryId: 'Whiskers',
        issues: [
          probedRecord({
            issueId: 'adjudicated/nap',
            refined: false,
            regions: [probeRegion({ envelopeId: 'envelope/nap', issueIds: ['adjudicated/nap',], corroborated: 2, },),],
          },),
          probedRecord({
            issueId: 'adjudicated/chase',
            refined: true,
            regions: [probeRegion({ envelopeId: 'envelope/chase', issueIds: ['adjudicated/chase',], corroborated: 2, },),],
          },),
          probedRecord({
            issueId: 'adjudicated/purr',
            refined: false,
            regions: [probeRegion({ envelopeId: 'envelope/purr', issueIds: ['adjudicated/purr',], corroborated: 0, },),],
          },),
        ],
        findings: [],
      },),
    },
  },);
}

/**
 Writes a repair sheet and its manifest into a runs directory.

 @param runsDir - runs directory the case owns

 @param grades - what the grader wrote at each position

 @param digested - whether both files carry the draw fingerprint

 @returns Paths of the sheet and the manifest

 @example
 ```ts
 const paths = await writeGradedDraw({ runsDir, grades: ['Y',], digested: false, },);
 ```
 */
async function writeGradedDraw(
  {
    runsDir,
    grades,
    digested,
  }: {
    readonly runsDir: string;
    readonly grades: readonly string[];
    readonly digested: boolean;
  },
): Promise<{ readonly sheet: string; readonly manifest: string; }> {
  /**
   Where the two files go.
   */
  const paths = {
    sheet: join(
      runsDir,
      'repair-sheet.md',
    ),
    manifest: join(
      runsDir,
      'manifest.json',
    ),
  };
  await writeFile(
    paths.sheet,
    catRepairSheetText({
      seed: SEED,
      corpusSha: CORPUS,
      ...(digested ? { drawDigest: catDrawDigest({ seed: SEED, corpusSha: CORPUS, count: ISSUES.length, issueIds: ISSUES, },), } : {}),
      grades,
    },),
    'utf8',
  );
  await writeFile(
    paths.manifest,
    catSampleManifestText({
      seed: SEED,
      corpusSha: CORPUS,
      count: ISSUES.length,
      digested,
      issueIds: ISSUES,
    },),
    'utf8',
  );
  return paths;
}

await describe({
  name: printProbeScore.name,
  concurrency: 1,
  children: [
    it({
      name: 'PRINTS the source, the telemetry and the note that no grade is joined where no flag names a sheet',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-probe-run-', },);
        await writeProbedRun({ runsDir: scratch.path, },);

        await printProbeScore({
          policy: NO_POOL_POLICY,
          repository: UNASKED_REPOSITORY,
          runsDir: scratch.path,
          line: lineOf({ command: 'score-probe', typed: [], },),
        },);

        expect(printed.lines,).toStrictEqual([
          `SOURCE ${scratch.path}/artifacts`,
          `POOL read by pipeline ${POOL_STAMP}`,
          'POOL 1 entry across 1 pipeline generation',
          ...TELEMETRY_LINES,
          'NOTE majorityIntroduced counts regions a gate WOULD have blocked, not regions that were damaged. '
            + 'Pass --repair-sheet PATH --manifest PATH to score it against the human grades.',
        ],);
      },
    },),

    it({
      name: 'PRINTS the joint counts, the refined-slice note and the clean-number note where both flags name a '
        + 'sheet and a manifest that carry no draw digest',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-probe-run-', },);
        await writeProbedRun({ runsDir: scratch.path, },);
        const draw = await writeGradedDraw({
          runsDir: scratch.path,
          grades: ['Y', 'N', 'N', 'Y',],
          digested: false,
        },);

        await printProbeScore({
          policy: NO_POOL_POLICY,
          repository: UNASKED_REPOSITORY,
          runsDir: scratch.path,
          line: lineOf({
            command: 'score-probe',
            typed: [
              '--repair-sheet',
              draw.sheet,
              '--manifest',
              draw.manifest,
            ],
          },),
        },);

        expect(printed.lines.slice(3,),).toStrictEqual([
          ...TELEMETRY_LINES,
          'NOTE sheet and manifest agree on seed and corpus pin, but one of them carries no draw digest, so this '
            + 'join rests on the file names and the header rather than on the items. Draws taken before the digest '
            + 'existed read this way, and are scoreable; a NEW draw reading this way means the digest was dropped '
            + 'somewhere and the pairing is unproven.',
          'AGREEMENT joined=3 probeFlagged=2 refutedByHuman=1 sharedWithHuman=1 flaggedUnscored=0 '
            + 'unflaggedFailures=1 refinedJoined=1',
          'NOTE refinedJoined counts positions where the naturalness lane rewrote the slice AFTER the probe ran. '
            + 'There the probe judged the accuracy stage\'s wording while the repair sheet asked the human to grade '
            + 'the RETURNED wording, so those rows compare two different texts and belong in neither column as '
            + 'evidence about the probe. Read the other counts over the remaining 2.',
          CLEAN_NOTE,
        ],);
      },
    },),

    it({
      name: 'PRINTS no binding note and no refined-slice note where the draw digest binds the pair and no rewritten '
        + 'slice is joined',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-probe-run-', },);
        await writeScoreArtifacts({
          runsDir: scratch.path,
          artifacts: {
            'Whiskers.json': probeArtifactText({
              entryId: 'Whiskers',
              issues: [
                probedRecord({
                  issueId: 'adjudicated/nap',
                  refined: false,
                  regions: [probeRegion({ envelopeId: 'envelope/nap', issueIds: ['adjudicated/nap',], corroborated: 2, },),],
                },),
              ],
              findings: [],
            },),
          },
        },);
        const draw = await writeGradedDraw({
          runsDir: scratch.path,
          grades: ['Y', 'Y', 'Y', 'Y',],
          digested: true,
        },);

        await printProbeScore({
          policy: NO_POOL_POLICY,
          repository: UNASKED_REPOSITORY,
          runsDir: scratch.path,
          line: lineOf({
            command: 'score-probe',
            typed: [
              '--repair-sheet',
              draw.sheet,
              '--manifest',
              draw.manifest,
            ],
          },),
        },);

        expect(printed.lines.slice(-2,),).toStrictEqual([
          'AGREEMENT joined=1 probeFlagged=1 refutedByHuman=1 sharedWithHuman=0 flaggedUnscored=0 '
            + 'unflaggedFailures=0 refinedJoined=0',
          CLEAN_NOTE,
        ],);
        expect(printed.lines.length,).toBe(10,);
      },
    },),

    it({
      name: 'REFUSES --repair-sheet written without --manifest, before naming the source',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-probe-run-', },);

        /**
         What reporting with half the pair raised.
         */
        const refusal = await rejectionOf(async function reportsSheetAlone(): Promise<void> {
          await printProbeScore({
            policy: NO_POOL_POLICY,
            repository: UNASKED_REPOSITORY,
            runsDir: scratch.path,
            line: lineOf({ command: 'score-probe', typed: ['--repair-sheet', 'sheet.md',], },),
          },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          'StatedRefusalError: --repair-sheet and --manifest score the probe against the human grades together; '
            + 'name both, or neither for the telemetry alone',
        );
        expect(printed.lines,).toStrictEqual([],);
      },
    },),

    it({
      name: 'REFUSES --manifest written without --repair-sheet, before naming the source',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-probe-run-', },);

        /**
         What reporting with the other half of the pair raised.
         */
        const refusal = await rejectionOf(async function reportsManifestAlone(): Promise<void> {
          await printProbeScore({
            policy: NO_POOL_POLICY,
            repository: UNASKED_REPOSITORY,
            runsDir: scratch.path,
            line: lineOf({ command: 'score-probe', typed: ['--manifest', 'manifest.json',], },),
          },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          'StatedRefusalError: --repair-sheet and --manifest score the probe against the human grades together; '
            + 'name both, or neither for the telemetry alone',
        );
        expect(printed.lines,).toStrictEqual([],);
      },
    },),

    it({
      name: 'REFUSES in its own words a runs directory with no artifacts directory, after naming the source',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-probe-run-', },);

        /**
         Runs directory no pass ever wrote into.
         */
        const runsDir = join(
          scratch.path,
          'nowhere',
        );

        /**
         What reporting it raised.
         */
        const refusal = await rejectionOf(async function reportsAbsentRun(): Promise<void> {
          await printProbeScore({
            policy: NO_POOL_POLICY,
            repository: UNASKED_REPOSITORY,
            runsDir,
            line: lineOf({ command: 'score-probe', typed: [], },),
          },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          `StatedRefusalError: cannot list ${runsDir}/artifacts (ENOENT): name a runs directory that holds an `
            + '`artifacts` directory with TRANSLATION_REPAIR_RUNS_DIR',
        );
        expect(printed.lines,).toStrictEqual([`SOURCE ${runsDir}/artifacts`,],);
      },
    },),

    it({
      name: 'REFUSES in its own words a repair sheet that is not there, naming its path, ENOENT and --repair-sheet',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-probe-run-', },);
        await writeProbedRun({ runsDir: scratch.path, },);
        const draw = await writeGradedDraw({
          runsDir: scratch.path,
          grades: ['Y', 'N', 'N', 'Y',],
          digested: false,
        },);

        /**
         Sheet nobody wrote.
         */
        const sheet = join(
          scratch.path,
          'nowhere.md',
        );

        /**
         What reporting it raised.
         */
        const refusal = await rejectionOf(async function reportsAbsentSheet(): Promise<void> {
          await printProbeScore({
            policy: NO_POOL_POLICY,
            repository: UNASKED_REPOSITORY,
            runsDir: scratch.path,
            line: lineOf({
              command: 'score-probe',
              typed: [
                '--repair-sheet',
                sheet,
                '--manifest',
                draw.manifest,
              ],
            },),
          },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          `StatedRefusalError: cannot read the graded repair sheet at ${sheet} (ENOENT); name the sheet that `
            + 'exists with --repair-sheet',
        );
        expect(printed.lines.length,).toBe(8,);
      },
    },),

    it({
      name: 'REFUSES a manifest that is not there, naming its base name and ENOENT',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-probe-run-', },);
        await writeProbedRun({ runsDir: scratch.path, },);
        const draw = await writeGradedDraw({
          runsDir: scratch.path,
          grades: ['Y', 'N', 'N', 'Y',],
          digested: false,
        },);

        /**
         What reporting a manifest nobody wrote raised.
         */
        const refusal = await rejectionOf(async function reportsAbsentManifest(): Promise<void> {
          await printProbeScore({
            policy: NO_POOL_POLICY,
            repository: UNASKED_REPOSITORY,
            runsDir: scratch.path,
            line: lineOf({
              command: 'score-probe',
              typed: [
                '--repair-sheet',
                draw.sheet,
                '--manifest',
                join(
                  scratch.path,
                  'nowhere.json',
                ),
              ],
            },),
          },);
        },);

        expect(refusal,).toBeInstanceOf(RunJsonUnreadableError,);
        expect(String(refusal,),).toBe('RunJsonUnreadableError: could not read nowhere.json as JSON (ENOENT)',);
        expect(printed.lines.length,).toBe(8,);
      },
    },),

    it({
      name: 'REFUSES a sheet and a manifest of different lengths in its own words after the telemetry',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-probe-run-', },);
        await writeProbedRun({ runsDir: scratch.path, },);
        const draw = await writeGradedDraw({
          runsDir: scratch.path,
          grades: ['Y', 'N',],
          digested: false,
        },);

        /**
         What reporting a short sheet raised.
         */
        const refusal = await rejectionOf(async function reportsShortSheet(): Promise<void> {
          await printProbeScore({
            policy: NO_POOL_POLICY,
            repository: UNASKED_REPOSITORY,
            runsDir: scratch.path,
            line: lineOf({
              command: 'score-probe',
              typed: [
                '--repair-sheet',
                draw.sheet,
                '--manifest',
                draw.manifest,
              ],
            },),
          },);
        },);

        expect(refusal,).toBeInstanceOf(StatedRefusalError,);
        expect(String(refusal,),).toBe(
          'StatedRefusalError: sheet and manifest disagree about the draw: sheet has 2 items, manifest has 4. '
            + 'Joining them by position would mislabel every verdict after the first divergence.',
        );
        expect(printed.lines.length,).toBe(9,);
      },
    },),
  ],
},);
