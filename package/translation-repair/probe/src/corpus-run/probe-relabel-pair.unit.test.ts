/**
 Tests for how the relabel runner probes one case under its three
 conditions, over scripted clients in which no model is ever called.

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

import { RUN_MODELS, } from '@monochromatic-dev/translation-repair-run';
import {
  otherDisclosure,
  probePair,
} from '../../dist/final/node/index.mjs';
import {
  ADDITION_CHECK,
  NO_DEFECT_CHECK,
  OMISSION_CHECK,
  probeScriptedClient,
  type ProbeCheck,
  successiveClients,
} from '@monochromatic-dev/translation-repair-run/ts/introduced-defect-scripted-client.test-fixture.ts';
import { napCase, } from '../relabel-case.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Heading the rendered prompt gives the list of issues, which a withheld or
 absent prompt does not carry.
 */
const RENDERED_HEADING = 'PRE-EXISTING DEFECTS THIS EDIT TARGETED';

/**
 Builds a client in which every prober casts the same check, keeping what it
 was sent.

 @param check - check every prober casts on the one region

 @param prompts - log the client keeps the user sheets in

 @returns Scripted client

 @example
 ```ts
 const client = everyProberCasts({ check: OMISSION_CHECK, prompts: [], },);
 ```
 */
function everyProberCasts(
  {
    check,
    prompts,
  }: {
    readonly check: ProbeCheck;
    readonly prompts: string[];
  },
): ReturnType<typeof probeScriptedClient> {
  return probeScriptedClient({
    checkFor: function same() {
      return check;
    },
    asked: [],
    prompts,
  },);
}

/**
 Whether a sheet carries the heading the rendered prompt gives the issue list.

 @param sheet - user sheet one probe was sent

 @returns Whether the heading is there

 @example
 ```ts
 const rendered = rendersTheList({ sheet, },);
 ```
 */
function rendersTheList({ sheet, }: { readonly sheet: string; },): boolean {
  return sheet.includes(RENDERED_HEADING,);
}

/**
 Whether every sheet of each group carries the issue-list heading.

 @param sheetsOfEach - user sheets each probe was sent, one group per probe

 @returns One flag per group

 @example
 ```ts
 const flags = listRenderedIn({ sheetsOfEach: [productionSheets, otherSheets,], },);
 ```
 */
function listRenderedIn({ sheetsOfEach, }: { readonly sheetsOfEach: readonly (readonly string[])[]; },): readonly boolean[] {
  return sheetsOfEach.map(function listRendered(sheets,): boolean {
    return sheets.every(function carries(sheet,): boolean {
      return rendersTheList({ sheet, },);
    },);
  },);
}

/**
 Claim lines one probe prints, one per prober in roster order.

 @param admissibility - what the screen made of each claim

 @param category - category each prober named

 @returns Lines, without newlines

 @example
 ```ts
 const lines = claimLinesOf({ admissibility: 'corroborated', category: 'accuracy/addition', },);
 ```
 */
function claimLinesOf(
  {
    admissibility,
    category,
  }: {
    readonly admissibility: string;
    readonly category: string;
  },
): readonly string[] {
  return RUN_MODELS.checkerModelIds
    .map(function line(modelId,): string {
      return `    ${modelId} ${admissibility} (${category})`;
    },)
    .toSorted();
}

/**
 Printed lines with each run of claim lines (four spaces in) sorted. The probers' claims print in the order of a bench the prompt
 rotates, which moved when the house tense rule changed the prompt, so each run is compared in one order.

 @param lines - what the probe printed

 @returns Lines with every run of claim lines sorted

 @example
 ```ts
 const sorted = claimRunsSorted({ lines: ['RELABEL cat', '    b x', '    a x', ], },);
 ```
 */
function claimRunsSorted({ lines, }: { readonly lines: readonly string[]; },): readonly string[] {
  /**
   Lines read so far, the open run of claim lines kept apart until it ends.
   */
  const sorted: string[] = [];
  const run: string[] = [];
  for (const line of lines) {
    if (line.startsWith('    ',)) {
      run.push(line,);
    } else {
      sorted.push(...run.toSorted(), line,);
      run.length = 0;
    }
  }
  sorted.push(...run.toSorted(),);
  return sorted;
}

/**
 Models of the roster that a prober count of one is read against.
 */
const ROSTER_SIZE = RUN_MODELS.checkerModelIds.length;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: otherDisclosure.name,
      concurrency: 1,
      children: [
        it({
          name: 'ANSWERS rendered when production sends withheld',
          fn: async () => {
            expect(otherDisclosure({ production: 'withheld', },),).toBe('rendered',);
          },
        },),

        it({
          name: 'ANSWERS withheld when production sends rendered',
          fn: async () => {
            expect(otherDisclosure({ production: 'rendered', },),).toBe('withheld',);
          },
        },),
      ],
    },),

    describe({
      name: probePair.name,
      concurrency: 1,
      children: [
        it({
          name: 'PRINTS the case heading, then each condition with its claims before its counts, in production, '
            + 'other and absent order',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             User sheets of the production probe, the other prompt and the absent arm.
             */
            const productionSheets: string[] = [];
            const otherSheets: string[] = [];
            const absentSheets: string[] = [];

            await probePair({
              relabelCase: napCase({
                entryId: 'tabby',
                positions: [2,],
                recorded: 'corroborated=0',
              },),
              production: 'withheld',
              newClient: successiveClients({
                clients: [
                  everyProberCasts({
                    check: OMISSION_CHECK,
                    prompts: productionSheets,
                  },),
                  everyProberCasts({
                    check: NO_DEFECT_CHECK,
                    prompts: otherSheets,
                  },),
                  everyProberCasts({
                    check: ADDITION_CHECK,
                    prompts: absentSheets,
                  },),
                ],
              },).newClient,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            /**
             Probers that answered each probe, which is every prober of the roster.
             */
            const heard = `heard=${String(ROSTER_SIZE,)}/${String(ROSTER_SIZE,)}`;

            expect(claimRunsSorted({ lines: printed.lines, },),).toEqual([
              'RELABEL tabby positions=2 issuesServed=1 beforeChars=53 afterChars=15',
              '  run-recorded  corroborated=0',
              ...claimLinesOf({
                admissibility: 'removal-corroborated',
                category: 'accuracy/omission',
              },),
              `  issues-withheld ${heard} corroborated=0 removal=${String(ROSTER_SIZE,)} contradicted=0 `
                + 'unanchored=0 preExisting=0 none=0 uncertain=0',
              `  issues-rendered ${heard} corroborated=0 removal=0 contradicted=0 unanchored=0 preExisting=0 `
                + `none=${String(ROSTER_SIZE,)} uncertain=0`,
              ...claimLinesOf({
                admissibility: 'corroborated',
                category: 'accuracy/addition',
              },),
              `  issues-absent ${heard} corroborated=${String(ROSTER_SIZE,)} removal=0 contradicted=0 `
                + 'unanchored=0 preExisting=0 none=0 uncertain=0',
            ],);
            expect(
              listRenderedIn({
                sheetsOfEach: [
                  productionSheets,
                  otherSheets,
                  absentSheets,
                ],
              },),
            ).toEqual([
              false,
              true,
              false,
            ],);
          },
        },),

        it({
          name: 'SENDS the list rendered, then withheld, then no list when production renders, so the absent arm never renders a heading',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             User sheets of the production probe, the other prompt and the absent arm.
             */
            const productionSheets: string[] = [];
            const otherSheets: string[] = [];
            const absentSheets: string[] = [];

            await probePair({
              relabelCase: napCase({
                entryId: 'tabby',
                positions: [],
                recorded: 'not probed',
              },),
              production: 'rendered',
              newClient: successiveClients({
                clients: [
                  everyProberCasts({
                    check: NO_DEFECT_CHECK,
                    prompts: productionSheets,
                  },),
                  everyProberCasts({
                    check: NO_DEFECT_CHECK,
                    prompts: otherSheets,
                  },),
                  everyProberCasts({
                    check: NO_DEFECT_CHECK,
                    prompts: absentSheets,
                  },),
                ],
              },).newClient,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(
              printed.lines.filter(function isCondition(line,): boolean {
                return line.startsWith('  issues-',);
              },).map(function label(line,): string {
                return line.split(' ',)[2] ?? '';
              },),
            ).toEqual([
              'issues-rendered',
              'issues-withheld',
              'issues-absent',
            ],);
            expect(
              listRenderedIn({
                sheetsOfEach: [
                  productionSheets,
                  otherSheets,
                  absentSheets,
                ],
              },),
            ).toEqual([
              true,
              false,
              false,
            ],);
          },
        },),

        it({
          name: 'BUILDS one fresh client per probe, three for one case',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             Builder and the count of clients it handed out.
             */
            const { newClient, built, } = successiveClients({
              clients: [
                everyProberCasts({
                  check: NO_DEFECT_CHECK,
                  prompts: [],
                },),
                everyProberCasts({
                  check: NO_DEFECT_CHECK,
                  prompts: [],
                },),
                everyProberCasts({
                  check: NO_DEFECT_CHECK,
                  prompts: [],
                },),
              ],
            },);

            await probePair({
              relabelCase: napCase({
                entryId: 'tabby',
                positions: [2,],
                recorded: 'corroborated=0',
              },),
              production: 'withheld',
              newClient,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(built(),).toBe(3,);
            expect(printed.lines.length,).toBe(5,);
          },
        },),
      ],
    },),
  ],
},);
