/**
 Tests for the report `score-crosscheck` prints over a run's settled
 artifacts and a judge roster.

 Each case writes its own runs directory, hands it and a roster to
 `printCrosscheck` and reads the whole of what was printed, or the refusal
 that stopped it. The roster is handed in, so a case seats two judges where
 the command seats the run's eight.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { printCrosscheck, } from '../../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import {
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import {
  BISCUIT_ARTIFACT,
  MITTENS_ARTIFACT,
  POUNCE_ARTIFACT,
  tuftArtifact,
  writeScoreArtifacts,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/score-artifacts.test-fixture.ts';
import { builtPipelineDigest, } from '@monochromatic-dev/translation-repair-glossary/ts/corpus-run/built-pipeline-digest.test-fixture.ts';
import {
  NO_POOL_POLICY,
  UNASKED_REPOSITORY,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts/corpus-run/pool-policy.test-fixture.ts';

/**
 What the pool prints as the pipeline that read it.
 */
const POOL_STAMP = await builtPipelineDigest({ packageEntry: import.meta.resolve('@monochromatic-dev/translation-repair-artifact-pool',), },);

/**
 The two judges every case seats.
 */
const ROSTER = [
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
] as const;

/**
 Note about the undecided arm, printed after every arm count.
 */
const UNDECIDED_NOTE = 'NOTE undecided is needs-human, held OUT of every rate rather than filed as control. '
  + 'Rejected means the panel decided against a claim, so a judge can agree or disagree with it; needs-human '
  + 'means the panel declined to decide, and agreement with a verdict never given is undefined. Those claims '
  + 'lean supported on this run, so folding them into control would fill it with claims the panel mostly '
  + 'believed. From repair cache version 34 needs-human also holds a claim a supported majority settled at '
  + 'neutral, which asserts no defect, so the accepted arm no longer carries those claims and reads higher by '
  + 'construction against earlier runs.';

/**
 The floor note, which follows the author table.
 */
const FLOOR_NOTE = '\nNOTE the floor column reads against MIN_JUDGED_CLAIMS=30, which is a provisional guard rather '
  + 'than a calibrated threshold. An author clearing neither arm is not excluded from the run; it simply cannot '
  + 'carry a per-author rate yet.';

/**
 The note on what a crosscheck can bar, which closes the author table.
 */
const BAR_NOTE = 'NOTE this crosscheck can bar a claim\'s AUTHORS and cannot bar its adjudicators: the whole roster '
  + 'sits as critics, panel and judges, so no seat outside the panel exists to bar one with. It measures '
  + 'whether a verdict survives being re-asked without its author, never precision.';

/**
 The table header.
 */
const AUTHOR_HEADER = '\nAUTHOR                                                accepted  control  sole  floor';

/**
 Warning about claims the whole roster proposed, for one claim.
 */
const UNJUDGEABLE_WARNING = 'WARNING 1 claim was proposed by the WHOLE roster, leaving no seat to judge. Such claims '
  + 'are reported here rather than dropped: they are the most corroborated claims in the run, and removing '
  + 'them would lift every rate by hiding exactly the strongest agreement in the population.';

/**
 The line every read prints first, and the pool's two lines.

 @param runsDir - runs directory the case wrote

 @param entries - entries the pool reports, as the pool words them

 @returns Three lines

 @example
 ```ts
 const lines = headLines({ runsDir, entries: '2 entries', },);
 ```
 */
function headLines(
  {
    runsDir,
    entries,
  }: {
    readonly runsDir: string;
    readonly entries: string;
  },
): readonly string[] {
  return [
    `SOURCE ${runsDir}/artifacts`,
    `POOL read by pipeline ${POOL_STAMP}`,
    `POOL ${entries} across 1 pipeline generation`,
  ];
}

await describe({
  name: printCrosscheck.name,
  concurrency: 1,
  children: [
    it({
      name: 'PRINTS the population, the arms, the status breakdown, each author\'s row and the closing notes, '
        + 'and the warning for a claim the whole roster proposed',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-crosscheck-run-', },);
        await writeScoreArtifacts({
          runsDir: scratch.path,
          artifacts: {
            'Biscuit.json': BISCUIT_ARTIFACT,
            'Mittens.json': MITTENS_ARTIFACT,
          },
        },);

        await printCrosscheck({
          policy: NO_POOL_POLICY,
          repository: UNASKED_REPOSITORY,
          runsDir: scratch.path,
          roster: ROSTER,
        },);

        expect(printed.lines,).toStrictEqual([
          ...headLines({ runsDir: scratch.path, entries: '2 entries', },),
          'POPULATION entries=2 withoutAttribution=1 judgeable=2 unjudgeable=1 legacyClaims=0 joinFailures=0',
          'ARMS accepted=1 control=1 undecided=0',
          UNDECIDED_NOTE,
          'NON-ACCEPTED BY STATUS rejected=1',
          AUTHOR_HEADER,
          'hf:Qwen/Qwen3.8-27B                                          1        0     1  neither',
          'hf:moonshotai/Kimi-K3                                        0        1     1  neither',
          FLOOR_NOTE,
          BAR_NOTE,
          UNJUDGEABLE_WARNING,
        ],);
      },
    },),

    it({
      name: 'SAYS no entry carries attribution, after counting the legacy claims, where every entry predates it',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-crosscheck-run-', },);
        await writeScoreArtifacts({
          runsDir: scratch.path,
          artifacts: {
            'Pounce.json': POUNCE_ARTIFACT,
            'Mittens.json': MITTENS_ARTIFACT,
          },
        },);

        await printCrosscheck({
          policy: NO_POOL_POLICY,
          repository: UNASKED_REPOSITORY,
          runsDir: scratch.path,
          roster: ROSTER,
        },);

        expect(printed.lines,).toStrictEqual([
          ...headLines({ runsDir: scratch.path, entries: '2 entries', },),
          'POPULATION entries=2 withoutAttribution=2 judgeable=0 unjudgeable=0 legacyClaims=1 joinFailures=0',
          'ARMS accepted=0 control=0 undecided=0',
          UNDECIDED_NOTE,
          'NOTE no entry carries attribution yet, so no claim can have its author barred and there is nothing to '
            + 'crosscheck. Entries settled before attribution existed record no proposer.',
        ],);
      },
    },),

    it({
      name: 'WARNS of a claim on an attributed entry that no attribution holds, and names no status where every '
        + 'judgeable claim was accepted',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-crosscheck-run-', },);
        await writeScoreArtifacts({
          runsDir: scratch.path,
          artifacts: {
            'Tuft.json': tuftArtifact({
              claims: [
                {
                  claimId: 'issue/nap',
                  proposers: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
                },
              ],
              issues: [
                {
                  status: 'accepted',
                  claimIds: ['issue/nap',],
                },
                {
                  status: 'accepted',
                  claimIds: ['issue/ghost',],
                },
              ],
            },),
          },
        },);

        await printCrosscheck({
          policy: NO_POOL_POLICY,
          repository: UNASKED_REPOSITORY,
          runsDir: scratch.path,
          roster: ROSTER,
        },);

        expect(printed.lines,).toStrictEqual([
          ...headLines({ runsDir: scratch.path, entries: '1 entry', },),
          'POPULATION entries=1 withoutAttribution=0 judgeable=1 unjudgeable=0 legacyClaims=0 joinFailures=1',
          'WARNING 1 claim sits on entries that DO carry attribution yet have no proposer recorded. That is the two '
            + 'records disagreeing about claim identity, not a quiet critic, and it is reported apart from the legacy '
            + 'count so it cannot hide inside an expected number.',
          'ARMS accepted=1 control=0 undecided=0',
          UNDECIDED_NOTE,
          'NON-ACCEPTED BY STATUS none',
          AUTHOR_HEADER,
          'hf:Qwen/Qwen3.8-27B                                          1        0     1  neither',
          FLOOR_NOTE,
          BAR_NOTE,
        ],);
      },
    },),

    it({
      name: 'SAYS every claim was proposed by the whole roster, and warns of it, where no claim has a judge to seat',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-crosscheck-run-', },);
        await writeScoreArtifacts({
          runsDir: scratch.path,
          artifacts: {
            'Tuft.json': tuftArtifact({
              claims: [
                {
                  claimId: 'issue/purr',
                  proposers: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER, SEAT_SYNTHETIC_VISION_WITHHELD,],
                },
              ],
              issues: [
                {
                  status: 'accepted',
                  claimIds: ['issue/purr',],
                },
              ],
            },),
          },
        },);

        await printCrosscheck({
          policy: NO_POOL_POLICY,
          repository: UNASKED_REPOSITORY,
          runsDir: scratch.path,
          roster: ROSTER,
        },);

        expect(printed.lines,).toStrictEqual([
          ...headLines({ runsDir: scratch.path, entries: '1 entry', },),
          'POPULATION entries=1 withoutAttribution=0 judgeable=0 unjudgeable=1 legacyClaims=0 joinFailures=0',
          'ARMS accepted=0 control=0 undecided=0',
          UNDECIDED_NOTE,
          'NOTE every attributed claim was proposed by the whole roster, so no claim has a judge to seat and there '
            + 'is nothing to crosscheck.',
          UNJUDGEABLE_WARNING,
        ],);
      },
    },),

    it({
      name: 'SAYS the entries carry attribution but no issue names an attributed claim, where nothing joins',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-crosscheck-run-', },);
        await writeScoreArtifacts({
          runsDir: scratch.path,
          artifacts: {
            'Tuft.json': tuftArtifact({
              claims: [
                {
                  claimId: 'issue/nap',
                  proposers: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
                },
              ],
              issues: [],
            },),
          },
        },);

        await printCrosscheck({
          policy: NO_POOL_POLICY,
          repository: UNASKED_REPOSITORY,
          runsDir: scratch.path,
          roster: ROSTER,
        },);

        expect(printed.lines,).toStrictEqual([
          ...headLines({ runsDir: scratch.path, entries: '1 entry', },),
          'POPULATION entries=1 withoutAttribution=0 judgeable=0 unjudgeable=0 legacyClaims=0 joinFailures=0',
          'ARMS accepted=0 control=0 undecided=0',
          UNDECIDED_NOTE,
          'NOTE the entries carry attribution, but none of their issues names an attributed claim, so there is '
            + 'nothing to crosscheck.',
        ],);
      },
    },),

    it({
      name: 'NAMES the artifact that could not be read before the population, with its verb in the singular',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-crosscheck-run-', },);
        await writeScoreArtifacts({
          runsDir: scratch.path,
          artifacts: {
            'Biscuit.json': BISCUIT_ARTIFACT,
            'Broken.json': '{"id": "Bro',
          },
        },);

        await printCrosscheck({
          policy: NO_POOL_POLICY,
          repository: UNASKED_REPOSITORY,
          runsDir: scratch.path,
          roster: ROSTER,
        },);

        expect(printed.lines.slice(0, 8,),).toStrictEqual([
          `SOURCE ${scratch.path}/artifacts`,
          'POOL malformed Broken.json: could not read Broken.json as JSON (SyntaxError at byte 11)',
          `POOL read by pipeline ${POOL_STAMP}`,
          'POOL 1 entry across 1 pipeline generation',
          'POOL   1 artifact unreadable, passed through to be reported as malformed: Broken',
          'WARNING 1 artifact could not be read and is in NEITHER population this report counts, so every count '
            + 'is over the rest:',
          '  Broken.json: could not read Broken.json as JSON (SyntaxError at byte 11)',
          'POPULATION entries=1 withoutAttribution=0 judgeable=2 unjudgeable=1 legacyClaims=0 joinFailures=0',
        ],);
      },
    },),

    it({
      name: 'REFUSES in its own words a runs directory with no artifacts directory, after naming the source',
      fn: async (ctx) => {
        using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
        await using scratch = await scratchDir({ prefix: 'score-crosscheck-run-', },);

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
          await printCrosscheck({
            policy: NO_POOL_POLICY,
            repository: UNASKED_REPOSITORY,
            runsDir,
            roster: ROSTER,
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
  ],
},);
