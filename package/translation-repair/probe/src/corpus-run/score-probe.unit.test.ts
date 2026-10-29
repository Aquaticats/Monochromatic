/**
 Tests for the map that joins a graded sheet position to a probe verdict.

 This is the join the gate's probe comparison rests on: sheet position to
 issue id through the manifest, then issue id to reading here. A wrong answer
 does not fail, it mislabels, and every count downstream still looks ordinary.

 The defect this pins was live. The map used to be built from each reading's
 `regions[].issueIds`, which names every issue a region serves, and one
 replacement can serve several accepted issues. A shared envelope therefore
 appeared in the readings of every record it served, and the last one indexed
 won. Ownership now comes from the record itself.

 @module
 */

import { join, } from 'node:path';

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  indexReadingsByIssue,
  ProbeIssueIndexError,
} from '@monochromatic-dev/translation-repair-run';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { writeScoreArtifacts, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/score-artifacts.test-fixture.ts';
import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { PROBE_ENTRY, } from './built-entry.test-fixture.ts';
import { builtPipelineDigest, } from '@monochromatic-dev/translation-repair-glossary/ts/corpus-run/built-pipeline-digest.test-fixture.ts';
import {
  CONFLICTING_POOL_SAYS,
  CONFLICTING_POOL_VARIABLES,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts/corpus-run/pool-policy.test-fixture.ts';
import {
  probeArtifactText,
  probedRecord,
  probeRegion,
} from '@monochromatic-dev/translation-repair-artifact/ts/corpus-run/score-probe-artifacts.test-fixture.ts';

/**
 Built command under test.
 */
const COMMAND = 'score-probe';

/**
 What the pool prints as the pipeline that read it.
 */
const POOL_STAMP = await builtPipelineDigest({ packageEntry: PROBE_ENTRY, },);

/**
 Note printed when no refinement audit is carried.
 */
const REFINEMENT_NOTE = 'NOTE rewrittenSlices=0 means no artifact here carries a refinement audit. That is what '
  + 'artifacts written before the lane was audited look like, and it is NOT evidence the lane rewrote '
  + 'nothing: read the ROSTER line to tell the two apart.';

/**
 Note printed when the probe is not scored against grades.
 */
const UNSCORED_NOTE = 'NOTE majorityIntroduced counts regions a gate WOULD have blocked, not regions that were '
  + 'damaged. Pass --repair-sheet PATH --manifest PATH to score it against the human grades.';

/**
 Builds a region tally naming the issues it serves.

 @param envelopeId - envelope the region replaced

 @param issueIds - every issue this one region serves

 @returns Tally shaped as a reading carries it

 @example
 ```ts
 const tally = catTally({ envelopeId: 'envelope/nap', issueIds: [], },);
 ```
 */
function catTally(
  {
    envelopeId,
    issueIds,
  }: {
    readonly envelopeId: string;
    readonly issueIds: readonly string[];
  },
) {
  return {
    envelopeId,
    issueIds,
    corroborated: 0,
    removalCorroborated: 0,
    contradicted: 0,
    unanchored: 0,
    preExisting: 0,
    noneFound: 3,
    uncertain: 0,
    claims: [],
  };
}

/**
 Builds a probe reading over the given regions.

 @param regions - screened tallies for the regions serving this issue

 @returns Reading shaped as a record carries it

 @example
 ```ts
 const reading = catReading({ regions: [], },);
 ```
 */
function catReading(
  { regions, }: { readonly regions: readonly ReturnType<typeof catTally>[]; },
) {
  return {
    heardProbers: 3,
    configuredProbers: 3,
    regions,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: indexReadingsByIssue.name,
      children: [
        it({
          name: 'maps each issue to the reading its OWN record carried, even when '
            + 'one shared envelope serves both issues and names both. This is the '
            + 'ordinary case rather than a rare collision, because envelopes merge '
            + 'overlapping evidence, and reading ownership off the region lists '
            + 'would hand one issue the other record\'s verdict',
          fn: async () => {
            /**
             Envelope serving both issues, as a merged replacement does.
             */
            const shared = catTally({
              envelopeId: 'envelope/shared',
              issueIds: [
                'adjudicated/nap',
                'adjudicated/chase',
              ],
            },);
            /**
             Reading of the record about the napping issue.
             */
            const napReading = catReading({ regions: [shared,], },);
            /**
             Reading of the record about the chasing issue, distinguishable by an
             extra region so the two are not interchangeable.
             */
            const chaseReading = catReading({
              regions: [
                shared,
                catTally({
                  envelopeId: 'envelope/chase',
                  issueIds: ['adjudicated/chase',],
                },),
              ],
            },);

            /**
             Join built from records rather than from region lists.
             */
            const byIssueId = indexReadingsByIssue({
              owned: [
                {
                  issueId: 'adjudicated/nap',
                  reading: napReading,
                  refined: false,
                },
                {
                  issueId: 'adjudicated/chase',
                  reading: chaseReading,
                  refined: false,
                },
              ],
            },);

            expect(byIssueId.get('adjudicated/nap',),).toBe(napReading,);
            expect(byIssueId.get('adjudicated/chase',),).toBe(chaseReading,);
            expect(byIssueId.size,).toBe(2,);
          },
        },),

        it({
          name: 'THROWS when two records claim one issue id, rather than keeping '
            + 'the last. The id is the identity the whole join rests on, so a '
            + 'duplicate means a sheet position could carry another record\'s '
            + 'verdict, and silently overwriting is exactly the failure that '
            + 'produces confident wrong numbers',
          fn: async () => {
            /**
             What indexesDuplicate raised, read for its class as well as its wording.
             */
            const refusalOfIndexesDuplicate = caught(function indexesDuplicate() {
              indexReadingsByIssue({
                owned: [
                  {
                    issueId: 'adjudicated/nap',
                    reading: catReading({ regions: [], },),
                    refined: false,
                  },
                  {
                    issueId: 'adjudicated/nap',
                    refined: false,
                    reading: catReading({
                      regions: [
                        catTally({
                          envelopeId: 'envelope/other',
                          issueIds: ['adjudicated/nap',],
                        },),
                      ],
                    },),
                  },
                ],
              },);
            },);

            expect(refusalOfIndexesDuplicate,).toBeInstanceOf(ProbeIssueIndexError,);
            expect((refusalOfIndexesDuplicate as Error).message,).toContain('adjudicated/nap',);
          },
        },),

        it({
          name: 'accepts the SAME reading offered twice for one issue without '
            + 'throwing, since that is repetition rather than conflict and '
            + 'refusing it would turn a harmless duplicate into a failed run',
          fn: async () => {
            /**
             One reading offered under the same id twice.
             */
            const reading = catReading({ regions: [], },);

            /**
             Join over the repeated pair.
             */
            const byIssueId = indexReadingsByIssue({
              owned: [
                {
                  issueId: 'adjudicated/nap',
                  reading,
                  refined: false,
                },
                {
                  issueId: 'adjudicated/nap',
                  reading,
                  refined: false,
                },
              ],
            },);

            expect(byIssueId.size,).toBe(1,);
            expect(byIssueId.get('adjudicated/nap',),).toBe(reading,);
          },
        },),

        it({
          name: 'returns an empty map for no readings, which is what a run whose '
            + 'probe never fired looks like and is not a fault',
          fn: async () => {
            expect(indexReadingsByIssue({ owned: [], },).size,).toBe(0,);
          },
        },),
      ],
    },),

    describe({
      name: 'score-probe as built',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'PRINTS the telemetry, with the note that no gate is scored, for a run whose artifact carries a probe, and exits 0',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'score-probe-', },);
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

            /**
             What the command wrote.
             */
            const run = await runBuiltCommand({
              command: COMMAND,
              packageEntry: PROBE_ENTRY,
              args: [],
              env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
            },);

            expect(run.code,).toBe(0,);
            expect(run.stderr,).toBe('',);
            expect(run.stdout,).toBe([
              `SOURCE ${scratch.path}/artifacts`,
              `POOL read by pipeline ${POOL_STAMP}`,
              'POOL 1 entry across 1 pipeline generation',
              'PROBE entries=1 repairShippedRecords=1 repairUnprobedRecords=0 regions=1 majorityIntroduced=1 '
                + 'minorityIntroduced=0 noneIntroduced=0',
              'CLAIMS added=2 dropped=0 contradicted=0 unanchored=0 degradedRosterRegions=0',
              'REFINEMENT rewrittenSlices=0 majorityIntroduced=0 minorityIntroduced=0 noneIntroduced=0 added=0 '
                + 'dropped=0 contradicted=0 unanchored=0',
              'ROSTER editorOffered=0 editorDegraded=0 editorSilent=0 refineOffered=0 refineDegraded=0 '
                + 'refineSilent=0 entriesWithRewrites=0/1',
              REFINEMENT_NOTE,
              UNSCORED_NOTE,
              '',
            ].join('\n',),);
          },
        },),

        it({
          name: 'REFUSES A REQUIRED COMMIT AND A MIXED POOL ASKED FOR TOGETHER in its own words, read from the '
            + 'environment it runs in, exiting 6 after the source line',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'score-probe-', },);
            await writeScoreArtifacts({
              runsDir: scratch.path,
              artifacts: {},
            },);

            /**
             What the command wrote.
             */
            const run = await runBuiltCommand({
              command: COMMAND,
              packageEntry: PROBE_ENTRY,
              args: [],
              env: {
                TRANSLATION_REPAIR_RUNS_DIR: scratch.path,
                ...CONFLICTING_POOL_VARIABLES,
              },
            },);

            expect(run,).toEqual({
              code: 6,
              stdout: `SOURCE ${scratch.path}/artifacts\n`,
              stderr: `score-probe: ${CONFLICTING_POOL_SAYS}\n`,
            },);
          },
        },),

        it({
          name: 'REFUSES --repair-sheet written without --manifest in its own words, exiting 6 before reading anything',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'score-probe-', },);

            /**
             What the command wrote.
             */
            const run = await runBuiltCommand({
              command: COMMAND,
              packageEntry: PROBE_ENTRY,
              args: [
                '--repair-sheet',
                join(
                  scratch.path,
                  'sheet.md',
                ),
              ],
              env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
            },);

            expect(run.code,).toBe(6,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(
              'score-probe: --repair-sheet and --manifest score the probe against the human grades together; '
                + 'name both, or neither for the telemetry alone\n',
            );
          },
        },),

        it({
          name: 'REFUSES a flag written with no value in its own words, exiting 6 before reading anything',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'score-probe-', },);

            /**
             What the command wrote.
             */
            const run = await runBuiltCommand({
              command: COMMAND,
              packageEntry: PROBE_ENTRY,
              args: ['--manifest',],
              env: { TRANSLATION_REPAIR_RUNS_DIR: scratch.path, },
            },);

            expect(run.code,).toBe(6,);
            expect(run.stdout,).toBe('',);
            expect(run.stderr,).toBe(
              'score-probe: --manifest needs a value written after it. Usage: score-probe [--repair-sheet '
                + '<graded repair sheet>] [--manifest <repair manifest>]\n',
            );
          },
        },),
      ],
    },),
  ],
},);
