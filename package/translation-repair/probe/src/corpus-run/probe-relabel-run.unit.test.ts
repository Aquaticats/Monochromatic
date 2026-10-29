/**
 Tests for the relabel run over scripted gatherers and clients, in which no
 model is ever called and no corpus clone is read.

 CAPTURING `console.log` IS PROCESS-WIDE, which is why this file runs at
 `concurrency: 1`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type RelabelCase,
  relabelNotes,
  runProbeRelabel,
} from '../../dist/final/node/index.mjs';
import {
  NO_DEFECT_CHECK,
  probeScriptedClient,
  successiveClients,
} from '@monochromatic-dev/translation-repair-run/ts/introduced-defect-scripted-client.test-fixture.ts';
import { napCase, } from '../relabel-case.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Pin naming a clone that exists nowhere, since the scripted gatherers read none.
 */
const PIN = {
  cloneDir: '/cats/corpus-clone',
  commitSha: 'b'.repeat(40,),
};

/**
 Region a human read as damaged.
 */
const DAMAGED_CASE = napCase({
  entryId: 'tabby',
  positions: [2,],
  recorded: 'corroborated=0',
},);

/**
 Region the same entry's reader did not flag.
 */
const CONTROL_CASE = napCase({
  entryId: 'tabby',
  positions: [],
  recorded: 'not probed',
},);

/**
 Builds the gatherers of a run, keeping what each was handed.

 @param damaged - regions the damaged gatherer answers with

 @param controls - regions the control gatherer answers with

 @param damagedCalls - log of the damaged gatherer's arguments

 @param controlCalls - log of the control gatherer's arguments

 @returns Two gatherers, as the run takes them

 @example
 ```ts
 const gather = gatherers({ damaged: [], controls: [], damagedCalls: [], controlCalls: [], },);
 ```
 */
function gatherers(
  {
    damaged,
    controls,
    damagedCalls,
    controlCalls,
  }: {
    readonly damaged: readonly RelabelCase[];
    readonly controls: readonly RelabelCase[];
    readonly damagedCalls: unknown[];
    readonly controlCalls: unknown[];
  },
): Parameters<typeof runProbeRelabel>[0]['gather'] {
  return {
    damaged: function gatherDamaged(call,) {
      damagedCalls.push(call,);
      return Promise.resolve(damaged,);
    },
    controls: function gatherControls(call,) {
      controlCalls.push(call,);
      return Promise.resolve(controls,);
    },
  };
}

/**
 Builder over enough quiet clients for the given number of probes.

 @param probes - how many probes the run will make

 @returns Builder and count of what it handed out

 @example
 ```ts
 const { newClient, built, } = quietClients({ probes: 3, },);
 ```
 */
function quietClients({ probes, }: { readonly probes: number; },): ReturnType<typeof successiveClients> {
  return successiveClients({
    clients: Array.from(
      { length: probes, },
      function quiet(): ReturnType<typeof probeScriptedClient> {
        return probeScriptedClient({
          checkFor: function none() {
            return NO_DEFECT_CHECK;
          },
          asked: [],
        },);
      },
    ),
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: runProbeRelabel.name,
      concurrency: 1,
      children: [
        it({
          name: 'REBUILDS both sets from the manifest and the artifacts in the runs directory it is handed, probes the '
            + 'damaged set then the control set, and closes with the two notes',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             Arguments each gatherer was handed.
             */
            const damagedCalls: unknown[] = [];
            const controlCalls: unknown[] = [];

            /**
             Builder over three quiet clients for each of the two cases.
             */
            const { newClient, built, } = quietClients({ probes: 6, },);

            await runProbeRelabel({
              dir: '/cats/runs',
              pin: PIN,
              production: 'withheld',
              newClient,
              gather: gatherers({
                damaged: [DAMAGED_CASE,],
                controls: [CONTROL_CASE,],
                damagedCalls,
                controlCalls,
              },),
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            /**
             Manifest path both gatherers were handed.
             */
            const manifestPath = '/cats/runs/sample-manifest-milestone-three-precision-round-three.json';
            expect(damagedCalls,).toEqual([
              {
                manifestPath,
                pin: PIN,
                runsDir: '/cats/runs',
              },
            ],);
            expect(controlCalls,).toEqual([
              {
                manifestPath,
                damaged: [DAMAGED_CASE,],
                pin: PIN,
                runsDir: '/cats/runs',
              },
            ],);
            expect(built(),).toBe(6,);

            /**
             Lines the run printed, by their opening words.
             */
            const heads = printed.lines.map(function head(line,): string {
              return line.split('  ',)[0] ?? '';
            },);
            expect(printed.lines.slice(
              0,
              2,
            ),).toEqual([
              'RELABEL rebuilt 1 distinct damaged region',
              'RELABEL gathered 1 unflagged control region',
            ],);
            expect(heads.filter(function isHeading(line,): boolean {
              return line.startsWith('RELABEL tabby',);
            },),).toEqual([
              'RELABEL tabby positions=2 issuesServed=1 beforeChars=53 afterChars=15',
              'RELABEL tabby positions= issuesServed=1 beforeChars=53 afterChars=15',
            ],);
            expect(printed.lines.slice(-2,),).toEqual(relabelNotes({
              production: 'withheld',
              other: 'rendered',
            },),);
            expect(printed.lines.length,).toBe(2 + (2 * 5) + 2,);
          },
        },),

        it({
          name: 'PRINTS only the two counts and the notes and builds no client when both sets are empty',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             Builder that would hand out a client, and a count of what it handed out.
             */
            const { newClient, built, } = quietClients({ probes: 1, },);

            await runProbeRelabel({
              dir: '/cats/runs',
              pin: PIN,
              production: 'rendered',
              newClient,
              gather: gatherers({
                damaged: [],
                controls: [],
                damagedCalls: [],
                controlCalls: [],
              },),
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(built(),).toBe(0,);
            expect(printed.lines,).toEqual([
              'RELABEL rebuilt 0 distinct damaged regions',
              'RELABEL gathered 0 unflagged control regions',
              ...relabelNotes({
                production: 'rendered',
                other: 'withheld',
              },),
            ],);
          },
        },),

        it({
          name: 'PROBES several damaged regions then several controls, in the order they were gathered',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             Regions of each set, in gathering order.
             */
            const damaged = [
              napCase({
                entryId: 'tabby',
                positions: [2,],
                recorded: 'a',
              },),
              napCase({
                entryId: 'whiskers',
                positions: [7,],
                recorded: 'b',
              },),
            ];
            const controls = [
              napCase({
                entryId: 'mittens',
                positions: [],
                recorded: 'c',
              },),
              napCase({
                entryId: 'tabby',
                positions: [],
                recorded: 'd',
              },),
            ];

            await runProbeRelabel({
              dir: '/cats/runs',
              pin: PIN,
              production: 'withheld',
              newClient: quietClients({ probes: 12, },).newClient,
              gather: gatherers({
                damaged,
                controls,
                damagedCalls: [],
                controlCalls: [],
              },),
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(printed.lines.filter(function isHeading(line,): boolean {
              return line.startsWith('RELABEL ',) || line.startsWith('  run-recorded',);
            },),).toEqual([
              'RELABEL rebuilt 2 distinct damaged regions',
              'RELABEL gathered 2 unflagged control regions',
              'RELABEL tabby positions=2 issuesServed=1 beforeChars=53 afterChars=15',
              '  run-recorded  a',
              'RELABEL whiskers positions=7 issuesServed=1 beforeChars=53 afterChars=15',
              '  run-recorded  b',
              'RELABEL mittens positions= issuesServed=1 beforeChars=53 afterChars=15',
              '  run-recorded  c',
              'RELABEL tabby positions= issuesServed=1 beforeChars=53 afterChars=15',
              '  run-recorded  d',
            ],);
          },
        },),
      ],
    },),
  ],
},);
