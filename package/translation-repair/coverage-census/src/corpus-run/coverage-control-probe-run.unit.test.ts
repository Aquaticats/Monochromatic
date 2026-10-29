/**
 Tests for the coverage control probe's run: the order it asks for things in,
 what it hands the control, what it prints, and what it refuses, over a
 throwaway corpus and a scripted control.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider';
import {
  type CoverageControlResult,
  runCoverageControl,
} from '../../dist/final/node/index.mjs';
import { capturingLoggerPair, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { SEAT_SYNTHETIC_VISION_NO_OPENROUTER, } from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { lineOf, } from '@monochromatic-dev/translation-repair-run/ts/corpus-run/command-line.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import {
  keptInertClient,
  makeProbeCorpus,
  refusalOf,
} from '@monochromatic-dev/translation-repair-corpus-pass/ts/corpus-run/probes-b-built-command.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Original of three sections against a translation of one.
 */
const THREE_SECTIONS = '## 一\n\n猫一。\n\n## 二\n\n猫二。\n\n## 三\n\n猫三。\n';

/**
 Translation of one section.
 */
const ONE_SECTION = '## One\n\nCat one.\n';

/**
 Roster handed to every scripted control.
 */
const ROSTER = [SEAT_SYNTHETIC_VISION_NO_OPENROUTER,] as const;

/**
 Deadline handed to every scripted control.
 */
const EXCHANGE_TIMEOUT_MS = 4_321;

/**
 What a scripted control was asked.
 */
type ControlAsk = {
  /**
   Where each case it was given sits.
   */
  readonly wheres: readonly string[];

  /**
   Whether the client is the one the run built.
   */
  readonly sharedClient: boolean;

  /**
   Roster it was given.
   */
  readonly modelIds: readonly string[];

  /**
   Deadline it was given.
   */
  readonly exchangeTimeoutMs: number;

  /**
   Whether its cancellation had fired.
   */
  readonly aborted: boolean;

  /**
   Whether its logger is the one the run was given.
   */
  readonly ownLogger: boolean;
};

/**
 Runs the probe over the given corpus files with a scripted control.

 @param files - corpus files by path

 @param typed - arguments the operator wrote

 @returns Clients built, what the control was asked, lines printed and what
 the run refused with, empty when it ran

 @example
 ```ts
 const ran = await runOver({ files, typed: [], sinon, },);
 ```
 */
async function runOver(
  {
    files,
    typed,
    sinon,
  }: {
    readonly files: Readonly<Record<string, string>>;
    readonly typed: readonly string[];
    readonly sinon: Parameters<typeof divertingConsoleLog>[0]['sinon'];
  },
): Promise<{
  readonly built: number;
  readonly asked: readonly ControlAsk[];
  readonly lines: readonly string[];
  readonly refusal: string;
}> {
  await using corpus = await makeProbeCorpus({ files, },);
  using printed = divertingConsoleLog({ sinon, },);
  const { logger, } = capturingLoggerPair();

  /**
   Clients the run built.
   */
  const clients: ModelClient[] = [];

  /**
   What the control was asked.
   */
  const asked: ControlAsk[] = [];

  /**
   Scripted control: keeps what it was asked and says it held.

   @param input - what the run handed the control

   @returns Result of one damaged case that held

   @example
   ```ts
   const result = await scripted(input,);
   ```
   */
  function scripted(input: Parameters<Parameters<typeof runCoverageControl>[0]['holds']>[0],): Promise<CoverageControlResult> {
    asked.push({
      wheres: input.cases.map(function whereOf({ where, },): string {
        return where;
      },),
      sharedClient: input.client === clients[0],
      modelIds: input.modelIds,
      exchangeTimeoutMs: input.exchangeTimeoutMs,
      aborted: input.signal.aborted,
      ownLogger: input.l === logger,
    },);
    return Promise.resolve({
      held: true,
      sawAbsenceOnTarget: 1,
      sawAbsenceOnDecoy: 0,
      decoysTaken: 1,
      rows: [{
        where: 'Mittens section 0',
        before: 'carried',
        after: 'absent',
        absentBefore: 0,
        absentAfter: 3,
        decoy: 'carried',
        absentAfterDecoy: 0,
        decoyAt: 12,
        removedSpans: 1,
        removedChars: 20,
      },],
      refusals: [],
    },);
  }

  /**
   Runs the probe, as the case under test asks.
   */
  async function walk(): Promise<void> {
    await runCoverageControl({
      line: lineOf({
        command: 'coverage-control-probe',
        typed,
      },),
      pin: corpus.pin,
      newClient: function newClient(): ModelClient {
        return keptInertClient({ clients, },);
      },
      holds: scripted,
      roster: ROSTER,
      exchangeTimeoutMs: EXCHANGE_TIMEOUT_MS,
      l: logger,
      stageSettings: SCRIPTED_STAGE_SETTINGS,
    },);
  }

  /**
   What the run refused with, empty when it ran.
   */
  const refusal = await refusalOf({ run: walk, },);
  return {
    built: clients.length,
    asked,
    lines: printed.lines,
    refusal,
  };
}

await describe({
  name: runCoverageControl.name,
  children: [
    it({
      name: 'HANDS the control the passages, the one client, the roster, the deadline, an unfired signal and the logger, '
        + 'and prints the offer then the reading',
      fn: async (ctx) => {
        const ran = await runOver({
          files: {
            'people/Mittens/page.md': THREE_SECTIONS,
            'people/Mittens/page.en.md': ONE_SECTION,
          },
          typed: [],
          sinon: ctx.sinon,
        },);

        expect(ran,).toEqual({
          built: 1,
          asked: [{
            wheres: ['Mittens section 0', 'Mittens section 1', 'Mittens section 2',],
            sharedClient: true,
            modelIds: ROSTER,
            exchangeTimeoutMs: EXCHANGE_TIMEOUT_MS,
            aborted: false,
            ownLogger: true,
          },],
          lines: [
            'COVERAGE control offering 3 cases to a roster of 1',
            'COVERAGE control HELD over 1 damaged case: absence votes appeared on 1 targeted cut '
              + 'and on 0 of 1 equally large cut taken elsewhere',
            'COVERAGE control 0 cases could not be damaged: 0 because the roster never called them covered, '
              + '0 because what the roster anchored on was the whole page',
            'The roster voted absence once the rendering it pointed at was gone, so an absence '
              + 'vote is reachable and a run that produced none is reporting the corpus rather '
              + 'than the instrument.',
          ],
          refusal: '',
        },);
      },
    },),
    it({
      name: 'WALKS only the entries named after --only',
      fn: async (ctx) => {
        const ran = await runOver({
          files: {
            'people/Mittens/page.md': THREE_SECTIONS,
            'people/Mittens/page.en.md': ONE_SECTION,
            'people/Tabby/page.md': THREE_SECTIONS,
            'people/Tabby/page.en.md': ONE_SECTION,
          },
          typed: ['--only', 'Tabby',],
          sinon: ctx.sinon,
        },);

        expect(ran.asked.map(function wheresOf({ wheres, },): readonly string[] {
          return wheres;
        },),).toEqual([['Tabby section 0', 'Tabby section 1', 'Tabby section 2',],],);
      },
    },),
    it({
      name: 'REFUSES a corpus offering no passage, after building the client and before printing or asking the control',
      fn: async (ctx) => {
        const ran = await runOver({
          files: {
            'people/Whiskers/page.md': '## 一\n\n猫一。\n',
            'people/Whiskers/page.en.md': ONE_SECTION,
          },
          typed: [],
          sinon: ctx.sinon,
        },);

        expect(ran,).toEqual({
          built: 1,
          asked: [],
          lines: [],
          refusal: 'StatedRefusalError: coverage control probe refused: no walked entry offered a passage the aligners '
            + 'declined to pair, so there was nothing to ask the roster about',
        },);
      },
    },),
    it({
      name: 'BUILDS the client before reading the entry filter, so an entry the corpus lacks is refused only after it',
      fn: async (ctx) => {
        const ran = await runOver({
          files: {
            'people/Mittens/page.md': THREE_SECTIONS,
            'people/Mittens/page.en.md': ONE_SECTION,
          },
          typed: ['--only', 'Nobody',],
          sinon: ctx.sinon,
        },);

        expect(ran,).toEqual({
          built: 1,
          asked: [],
          lines: [],
          refusal: 'StatedRefusalError: --only asks for "Nobody", which the corpus at the pin does not hold',
        },);
      },
    },),
  ],
},);
