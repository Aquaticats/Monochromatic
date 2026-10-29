/**
 Tests for the sentinel probe's walk over corpus entries: which entries it
 probes, in which order, what it prints for each, and what it asks for before
 anything is spent, over a throwaway corpus and a scripted repair.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider';
import type { RepairModels, } from '@monochromatic-dev/translation-repair-assembly';
import {
  probeCorpusEntries,
  type ProbedResult,
} from '../../dist/final/node/index.mjs';
import { SEAT_SYNTHETIC_VISION_NO_OPENROUTER, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';
import {
  keptInertClient,
  makeProbeCorpus,
  refusalOf,
} from '@monochromatic-dev/translation-repair-corpus-pass/ts/corpus-run/probes-b-built-command.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Roster handed to every scripted repair, only ever compared by identity.
 */
const MODELS: RepairModels = {
  criticModelIds: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
  panelModelIds: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
  editorModelIds: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
  judgeModelIds: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
  checkerModelIds: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
};

/**
 Result every scripted repair returns.
 */
const QUIET_RESULT: ProbedResult = {
  status: 'unchanged',
  issues: [],
  findings: [],
};

/**
 Deadline handed to every scripted repair.
 */
const CALL_TIMEOUT_MS = 1_234;

/**
 What a scripted repair was asked.
 */
type RepairAsk = {
  /**
   Original text it was given.
   */
  readonly sourceText: string;

  /**
   Translation text it was given.
   */
  readonly targetText: string;

  /**
   Whether the client it was given is the one the run built.
   */
  readonly sharedClient: boolean;

  /**
   Whether the roster it was given is the run's own.
   */
  readonly ownModels: boolean;

  /**
   Deadline it was given.
   */
  readonly perCallTimeoutMs: number;
};

/**
 Corpus of three invented entries, one of which lacks its translation.
 */
const FILES = {
  'people/Anilovr/page.md': 'zh Anilovr\n',
  'people/Anilovr/page.en.md': 'en Anilovr\n',
  'people/Aniloviraw/page.md': 'zh Aniloviraw\n',
  'people/Aniloviraw/page.en.md': 'en Aniloviraw\n',
  'people/Mittens/page.md': 'zh Mittens\n',
} as const;

/**
 Probes over the invented corpus with a scripted repair and a clock reading
 5, 12, 20, 33 and so on, so every duration is known.

 @param typed - ids the operator wrote

 @param repair - what the repair does

 @returns Clients built, repairs asked and lines printed

 @example
 ```ts
 const probed = await probeWith({ typed: ['Anilovr',], repair, sinon, },);
 ```
 */
async function probeWith(
  {
    typed,
    repair,
    sinon,
  }: {
    readonly typed: readonly string[];
    readonly repair: (ask: RepairAsk,) => Promise<ProbedResult>;
    readonly sinon: Parameters<typeof divertingConsoleLog>[0]['sinon'];
  },
): Promise<{
  readonly built: number;
  readonly lines: readonly string[];
  readonly commitSha: string;
  readonly refusal: string;
}> {
  await using corpus = await makeProbeCorpus({ files: FILES, },);
  using printed = divertingConsoleLog({ sinon, },);

  /**
   Clients the run built.
   */
  const clients: ModelClient[] = [];

  /**
   Clock readings still to hand out.
   */
  const readings = [0, 5, 12, 20, 33, 47, 60, 80,];

  /**
   Walks the entries, as the case under test asks.
   */
  async function walk(): Promise<void> {
    await probeCorpusEntries({
      line: lineOf({
        command: 'sentinel-probe',
        typed,
      },),
      pin: corpus.pin,
      newClient: function newClient(): ModelClient {
        return keptInertClient({ clients, },);
      },
      repair: async function scripted(input,): Promise<ProbedResult> {
        return await repair({
          sourceText: input.sourceText,
          targetText: input.targetText,
          sharedClient: input.client === clients[0],
          ownModels: input.models === MODELS,
          perCallTimeoutMs: input.perCallTimeoutMs,
        },);
      },
      models: MODELS,
      perCallTimeoutMs: CALL_TIMEOUT_MS,
      now: function tick(): number {
        return readings.shift() ?? 0;
      },
      stageSettings: SCRIPTED_STAGE_SETTINGS,
    },);
  }

  /**
   What the walk refused with, empty when it ran.
   */
  const refusal = await refusalOf({ run: walk, },);
  return {
    built: clients.length,
    lines: printed.lines,
    commitSha: corpus.commitSha,
    refusal,
  };
}

await describe({
  name: probeCorpusEntries.name,
  children: [
    it({
      name: 'PROBES one named entry through one client, handing the repair both texts, the roster and the deadline',
      fn: async (ctx) => {
        /**
         Everything the repair was asked.
         */
        const asked: RepairAsk[] = [];
        const probed = await probeWith({
          typed: ['Anilovr',],
          sinon: ctx.sinon,
          repair: function record(ask,): Promise<ProbedResult> {
            asked.push(ask,);
            return Promise.resolve(QUIET_RESULT,);
          },
        },);

        expect(probed.built,).toBe(1,);
        expect(asked,).toEqual([{
          sourceText: 'zh Anilovr\n',
          targetText: 'en Anilovr\n',
          sharedClient: true,
          ownModels: true,
          perCallTimeoutMs: CALL_TIMEOUT_MS,
        },],);
        expect(probed.lines,).toEqual([
          `PROBE start corpus=${probed.commitSha} targets=Anilovr`,
          'PROBE Anilovr status=unchanged issues=0 accepted=0 repairs=none refinedIssues=0 findings=0 ms=5',
          'PROBE done',
        ],);
      },
    },),
    it({
      name: 'PROBES several named entries in the order the corpus lists them, not the order typed, sharing the one client',
      fn: async (ctx) => {
        const probed = await probeWith({
          typed: ['Anilovr', 'Aniloviraw',],
          sinon: ctx.sinon,
          repair: function quiet(): Promise<ProbedResult> {
            return Promise.resolve(QUIET_RESULT,);
          },
        },);

        expect(probed.built,).toBe(1,);
        expect(probed.lines,).toEqual([
          `PROBE start corpus=${probed.commitSha} targets=Aniloviraw,Anilovr`,
          'PROBE Aniloviraw status=unchanged issues=0 accepted=0 repairs=none refinedIssues=0 findings=0 ms=5',
          'PROBE Anilovr status=unchanged issues=0 accepted=0 repairs=none refinedIssues=0 findings=0 ms=8',
          'PROBE done',
        ],);
      },
    },),
    it({
      name: 'PROBES the default sentinels when no id is named',
      fn: async (ctx) => {
        const probed = await probeWith({
          typed: [],
          sinon: ctx.sinon,
          repair: function quiet(): Promise<ProbedResult> {
            return Promise.resolve(QUIET_RESULT,);
          },
        },);

        expect(probed.lines[0],).toBe(`PROBE start corpus=${probed.commitSha} targets=Aniloviraw,Anilovr`,);
        expect(probed.lines.length,).toBe(4,);
      },
    },),
    it({
      name: 'PRINTS an ERROR line for an entry whose translation is absent and goes on to the next',
      fn: async (ctx) => {
        const probed = await probeWith({
          typed: ['Anilovr', 'Mittens',],
          sinon: ctx.sinon,
          repair: function quiet(): Promise<ProbedResult> {
            return Promise.resolve(QUIET_RESULT,);
          },
        },);

        expect(probed.lines,).toEqual([
          `PROBE start corpus=${probed.commitSha} targets=Anilovr,Mittens`,
          'PROBE Anilovr status=unchanged issues=0 accepted=0 repairs=none refinedIssues=0 findings=0 ms=5',
          `PROBE Mittens status=ERROR ms=8 error=corpus read failed for ${probed.commitSha}:people/Mittens/page.en.md `
            + '(missing-object); the commit has no such path: check the path, or pin a commit that has it.',
          'PROBE done',
        ],);
      },
    },),
    it({
      name: 'PRINTS an ERROR line for an entry whose repair throws, naming the error and never its message',
      fn: async (ctx) => {
        const probed = await probeWith({
          typed: ['Anilovr',],
          sinon: ctx.sinon,
          repair: function hiss(): Promise<ProbedResult> {
            return Promise.reject(new RangeError('secret hiss',),);
          },
        },);

        expect(probed.lines,).toEqual([
          `PROBE start corpus=${probed.commitSha} targets=Anilovr`,
          'PROBE Anilovr status=ERROR ms=5 error=refused by RangeError',
          'PROBE done',
        ],);
      },
    },),
    it({
      name: 'REFUSES ids the corpus lacks before building a client, asking the repair or printing a line',
      fn: async (ctx) => {
        const probed = await probeWith({
          typed: ['Anilovr', 'Tom', 'Felix',],
          sinon: ctx.sinon,
          repair: function quiet(): Promise<ProbedResult> {
            return Promise.resolve(QUIET_RESULT,);
          },
        },);

        expect(probed.built,).toBe(0,);
        expect(probed.lines,).toEqual([],);
        expect(probed.refusal,).toBe(
          'StatedRefusalError: sentinel-probe asks for "Tom", "Felix", which the corpus at the pin does not hold',
        );
      },
    },),
  ],
},);
