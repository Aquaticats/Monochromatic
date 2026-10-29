/**
 Tests for the join of human repair grades to probe readings that
 `score-probe` prints.

 A wrong join does not fail, it mislabels, so a case holds the join by
 position, the refusal of a sheet and a manifest of different lengths, the
 count of rewritten slices and the lines printed.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { TelemetryProbeReading, } from '@monochromatic-dev/translation-repair-lane-contest';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import {
  countRefinedJoined,
  joinProbeGrades,
  printProbeAgreement,
} from '../../dist/final/node/index.mjs';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';

/**
 Reading of a chunk the probe heard three probers on and found nothing in.
 */
const QUIET_READING: TelemetryProbeReading = {
  heardProbers: 3,
  configuredProbers: 3,
  regions: [],
};

/**
 Manifest of two issues.
 */
const MANIFEST = {
  seed: 'round-cats',
  corpusSha: 'abc1234',
  generation: { kind: 'unrecorded', reason: 'fixture', },
  items: [
    {
      position: 1,
      entryId: 'Whiskers',
      issueId: 'adjudicated/nap',
    },
    {
      position: 2,
      entryId: 'Mittens',
      issueId: 'adjudicated/chase',
    },
  ],
} as const;

await describe({
  name: 'score-probe-agreement',
  concurrency: 1,
  children: [
    describe({
      name: joinProbeGrades.name,
      concurrency: 1,
      children: [
        it({
          name: 'PAIRS each position\'s verdict with the reading of its issue, and leaves an unprobed issue bare',
          fn: async () => {
            expect(joinProbeGrades({
              manifest: MANIFEST,
              graded: [
                { index: 1, verdict: 'fixes', note: '', },
                { index: 2, verdict: 'does-not-fix', note: '', },
              ],
              byIssueId: new Map([['adjudicated/nap', QUIET_READING,],],),
            },),).toStrictEqual([
              {
                verdict: 'fixes',
                reading: QUIET_READING,
              },
              { verdict: 'does-not-fix', },
            ],);
          },
        },),

        it({
          name: 'REFUSES in its own words a sheet shorter than its manifest, naming both counts, the sheet\'s in the plural',
          fn: async () => {
            /**
             What joining two items to one row raised.
             */
            const refusal = caught(function joinsShortSheet(): unknown {
              return joinProbeGrades({
                manifest: {
                  ...MANIFEST,
                  items: [MANIFEST.items[0],],
                },
                graded: [
                  { index: 1, verdict: 'fixes', note: '', },
                  { index: 2, verdict: 'fixes', note: '', },
                ],
                byIssueId: new Map(),
              },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              'StatedRefusalError: sheet and manifest disagree about the draw: sheet has 2 items, manifest has 1. '
                + 'Joining them by position would mislabel every verdict after the first divergence.',
            );
          },
        },),

        it({
          name: 'REFUSES in its own words a sheet of one item against a longer manifest, the item in the singular',
          fn: async () => {
            /**
             What joining one item to two rows raised.
             */
            const refusal = caught(function joinsLongManifest(): unknown {
              return joinProbeGrades({
                manifest: MANIFEST,
                graded: [{ index: 1, verdict: 'fixes', note: '', },],
                byIssueId: new Map(),
              },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              'StatedRefusalError: sheet and manifest disagree about the draw: sheet has 1 item, manifest has 2. '
                + 'Joining them by position would mislabel every verdict after the first divergence.',
            );
          },
        },),
      ],
    },),

    describe({
      name: countRefinedJoined.name,
      concurrency: 1,
      children: [
        it({
          name: 'COUNTS the manifest positions whose issue the naturalness lane rewrote',
          fn: async () => {
            expect(countRefinedJoined({
              manifest: MANIFEST,
              refinedIssueIds: new Set(['adjudicated/chase', 'adjudicated/other',],),
            },),).toBe(1,);
          },
        },),

        it({
          name: 'COUNTS none where no issue was rewritten',
          fn: async () => {
            expect(countRefinedJoined({
              manifest: MANIFEST,
              refinedIssueIds: new Set(),
            },),).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: printProbeAgreement.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS the counts and the clean-number note where no joined position was rewritten',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printProbeAgreement({
              agreement: {
                joined: 3,
                probeFlagged: 2,
                refutedByHuman: 1,
                sharedWithHuman: 1,
                flaggedUnscored: 0,
                unflaggedFailures: 1,
              },
              refinedJoined: 0,
            },);

            expect(printed.lines,).toStrictEqual([
              'AGREEMENT joined=3 probeFlagged=2 refutedByHuman=1 sharedWithHuman=1 flaggedUnscored=0 '
                + 'unflaggedFailures=1 refinedJoined=0',
              'NOTE refutedByHuman is the clean number: the human read the same wording and said it breaks '
                + 'nothing nearby, so each one is a correct repair a gate would have discarded. sharedWithHuman is NOT '
                + 'confirmation, since the sheet\'s N fires both for a repair that did not fix its target and for one '
                + 'that broke something.',
            ],);
          },
        },),

        it({
          name: 'PRINTS the rewritten-slice note between the counts and the clean-number note, over the remaining joined',
          fn: async (ctx) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);

            printProbeAgreement({
              agreement: {
                joined: 3,
                probeFlagged: 2,
                refutedByHuman: 1,
                sharedWithHuman: 1,
                flaggedUnscored: 0,
                unflaggedFailures: 1,
              },
              refinedJoined: 1,
            },);

            expect(printed.lines.length,).toBe(3,);
            expect(printed.lines[0],).toBe(
              'AGREEMENT joined=3 probeFlagged=2 refutedByHuman=1 sharedWithHuman=1 flaggedUnscored=0 '
                + 'unflaggedFailures=1 refinedJoined=1',
            );
            expect(printed.lines[1],).toBe(
              'NOTE refinedJoined counts positions where the naturalness lane rewrote the slice AFTER the probe ran. '
                + 'There the probe judged the accuracy stage\'s wording while the repair sheet asked the human to grade '
                + 'the RETURNED wording, so those rows compare two different texts and belong in neither column as '
                + 'evidence about the probe. Read the other counts over the remaining 2.',
            );
          },
        },),
      ],
    },),
  ],
},);
