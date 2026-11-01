/**
 Tests for the `score-crosscheck` runner: the built command over a runs
 directory a case wrote.

 The runner reads settled artifacts and prints the population a judge
 crosscheck would run over, so its as-built cases run to the end of the report
 on a fixture and read the whole of what it printed. Every child is started
 by `runBuiltCommand`, whose environment carries no variable ending in `_API_KEY`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import {
  BISCUIT_ARTIFACT,
  MITTENS_ARTIFACT,
  writeScoreArtifacts,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/score-artifacts.test-fixture.ts';
import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { PROBE_ENTRY, } from './built-entry.test-fixture.ts';
import { builtPipelineDigest, } from '@monochromatic-dev/translation-repair-glossary/ts/corpus-run/built-pipeline-digest.test-fixture.ts';
import {
  CONFLICTING_POOL_SAYS,
  CONFLICTING_POOL_VARIABLES,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts/corpus-run/pool-policy.test-fixture.ts';

/**
 Built command under test.
 */
const COMMAND = 'score-crosscheck';

/**
 What the pool prints as the pipeline that read it.
 */
const POOL_STAMP = await builtPipelineDigest({ packageEntry: PROBE_ENTRY, },);

/**
 Floor note and the note on what a crosscheck can bar, which close every
 report over a non-empty population.
 */
const CLOSING_NOTES = [
  '',
  'NOTE the floor column reads against MIN_JUDGED_CLAIMS=30, which is a provisional guard rather than a '
    + 'calibrated threshold. An author clearing neither arm is not excluded from the run; it simply cannot carry '
    + 'a per-author rate yet.',
  'NOTE this crosscheck can bar a claim\'s AUTHORS and cannot bar its adjudicators: the whole roster sits as '
    + 'critics, panel and judges, so no seat outside the panel exists to bar one with. It measures whether a '
    + 'verdict survives being re-asked without its author, never precision.',
];

/**
 Note about the undecided arm, printed whenever the population is not empty
 or the arms are counted.
 */
const UNDECIDED_NOTE = 'NOTE undecided is needs-human, held OUT of every rate rather than filed as control. '
  + 'Rejected means the panel decided against a claim, so a judge can agree or disagree with it; needs-human '
  + 'means the panel declined to decide, and agreement with a verdict never given is undefined. Those claims '
  + 'lean supported on this run, so folding them into control would fill it with claims the panel mostly '
  + 'believed. From repair cache version 34 needs-human also holds a claim a supported majority settled at '
  + 'neutral, which asserts no defect, so the accepted arm no longer carries those claims and reads higher by '
  + 'construction against earlier runs.';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'score-crosscheck as built',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REPORTS the population, the arms, the status breakdown and each author\'s row, and exits 0',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'score-crosscheck-', },);
            await writeScoreArtifacts({
              runsDir: scratch.path,
              artifacts: {
                'Biscuit.json': BISCUIT_ARTIFACT,
                'Mittens.json': MITTENS_ARTIFACT,
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
              'POOL 2 entries across 1 pipeline generation',
              'POPULATION entries=2 withoutAttribution=1 judgeable=3 unjudgeable=0 legacyClaims=0 joinFailures=0',
              'ARMS accepted=1 control=1 undecided=1',
              UNDECIDED_NOTE,
              'NON-ACCEPTED BY STATUS rejected=1 needs-human=1',
              '',
              'AUTHOR                                                accepted  control  sole  floor',
              'hf:Qwen/Qwen3.8-27B                                          1        0     1  neither',
              'hf:moonshotai/Kimi-K3                                        0        1     1  neither',
              ...CLOSING_NOTES,
              '',
            ].join('\n',),);
          },
        },),

        it({
          name: 'REFUSES A REQUIRED COMMIT AND A MIXED POOL ASKED FOR TOGETHER in its own words, read from the '
            + 'environment it runs in, exiting 6 after the source line',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'score-crosscheck-', },);
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
              stderr: `score-crosscheck: ${CONFLICTING_POOL_SAYS}\n`,
            },);
          },
        },),
      ],
    },),
  ],
},);
