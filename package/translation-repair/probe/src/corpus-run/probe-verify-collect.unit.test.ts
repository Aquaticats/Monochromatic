/**
 Tests for how the verify runner asks about each region and keeps the flagged
 ones, over scripted clients in which no model is ever called.

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
import type { ScreenedDefectClaim, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  collectFlagged,
  keepAdmissible,
} from '../../dist/final/node/index.mjs';
import {
  ADDITION_CHECK,
  CONTRADICTED_CHECK,
  NO_DEFECT_CHECK,
  omissionClaimBy,
  omissionClaimsOfRoster,
  OMISSION_CHECK,
  probeScriptedClient,
  type ProbeCheck,
  successiveClients,
  UNANCHORED_CHECK,
} from '@monochromatic-dev/translation-repair-run/ts/introduced-defect-scripted-client.test-fixture.ts';
import { sortedByModelId, } from '@monochromatic-dev/translation-repair-markup/ts/model-order.test-fixture.ts';
import { napCase, } from '../relabel-case.test-fixture.ts';
import { divertingConsoleLog, } from '@monochromatic-dev/translation-repair-test-fixture/ts/corpus-run/console-log-capture.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Builds a client in which every prober casts the same check.

 @param check - check every prober casts on the one region

 @returns Scripted client

 @example
 ```ts
 const client = everyProberCasts({ check: OMISSION_CHECK, },);
 ```
 */
function everyProberCasts({ check, }: { readonly check: ProbeCheck; },): ReturnType<typeof probeScriptedClient> {
  return probeScriptedClient({
    checkFor: function same() {
      return check;
    },
    asked: [],
  },);
}

/**
 Builds a client in which only the first prober of the roster claims damage.

 @returns Scripted client

 @example
 ```ts
 const client = onlyFirstProberClaims();
 ```
 */
function onlyFirstProberClaims(): ReturnType<typeof probeScriptedClient> {
  return probeScriptedClient({
    checkFor: function firstClaims(modelId,) {
      return (modelId === RUN_MODELS.checkerModelIds[0]) ? OMISSION_CHECK : NO_DEFECT_CHECK;
    },
    asked: [],
  },);
}

/**
 Claim with a given admissibility, for the filter.

 @param admissibility - what the screen made of the claim

 @returns Claim carrying it

 @example
 ```ts
 const claim = claimScreenedAs({ admissibility: 'contradicted', },);
 ```
 */
function claimScreenedAs(
  { admissibility, }: { readonly admissibility: ScreenedDefectClaim['admissibility']; },
): ScreenedDefectClaim {
  return {
    ...omissionClaimBy({ modelId: 'cat-house/tabbyscribe-2', },),
    admissibility,
  };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: keepAdmissible.name,
      concurrency: 1,
      children: [
        it({
          name: 'KEEPS only the claims the screen corroborated or found removal-corroborated, in the order given',
          fn: async () => {
            expect(keepAdmissible({
              claims: [
                claimScreenedAs({ admissibility: 'contradicted', },),
                claimScreenedAs({ admissibility: 'corroborated', },),
                claimScreenedAs({ admissibility: 'unanchored', },),
                claimScreenedAs({ admissibility: 'pre-existing', },),
                claimScreenedAs({ admissibility: 'removal-corroborated', },),
              ],
            },),).toEqual([
              claimScreenedAs({ admissibility: 'corroborated', },),
              claimScreenedAs({ admissibility: 'removal-corroborated', },),
            ],);
          },
        },),

        it({
          name: 'KEEPS nothing from no claims',
          fn: async () => {
            expect(keepAdmissible({ claims: [], },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: collectFlagged.name,
      concurrency: 1,
      children: [
        it({
          name: 'ASKS nothing and prints nothing for no cases, building no client',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             Builder that would hand out a client, and a count of the clients it handed out.
             */
            const { newClient, built, } = successiveClients({ clients: [everyProberCasts({ check: NO_DEFECT_CHECK, },),], },);

            expect(await collectFlagged({
              cases: [],
              kind: 'damaged',
              newClient,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },),).toEqual([],);
            expect(printed.lines,).toEqual([],);
            expect(built(),).toBe(0,);
          },
        },),

        it({
          name: 'KEEPS the flagged regions with their claims and counts one claim in the singular, several in the plural, '
            + 'and none in the plural',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             Regions probed in order: two claimants, one claimant, none.
             */
            const cases = [
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
              napCase({
                entryId: 'mittens',
                positions: [11,],
                recorded: 'c',
              },),
            ];

            /**
             One client per region, in region order.
             */
            const { newClient, } = successiveClients({
              clients: [
                everyProberCasts({ check: OMISSION_CHECK, },),
                onlyFirstProberClaims(),
                everyProberCasts({ check: NO_DEFECT_CHECK, },),
              ],
            },);

            /**
             Items the run kept.
             */
            const items = await collectFlagged({
              cases,
              kind: 'damaged',
              newClient,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            /**
             Claims every prober of the roster made, in roster order.
             */
            const everyClaim = omissionClaimsOfRoster();

            expect(printed.lines,).toEqual([
              `VERIFY damaged tabby ${String(everyClaim.length,)} admissible claims`,
              'VERIFY damaged whiskers 1 admissible claim',
              'VERIFY damaged mittens 0 admissible claims',
            ],);
            // The claims are listed in the order the probers answered, from a bench the prompt rotates (it moved when the house
            // tense rule changed the prompt), so each item's claims are compared sorted by prober.
            expect(items.map(function claimsSorted(item,) {
              return {
                ...item,
                claims: sortedByModelId({ items: item.claims, },),
              };
            },),).toEqual([
              {
                relabelCase: cases[0],
                claims: sortedByModelId({ items: everyClaim, },),
                kind: 'damaged',
              },
              {
                relabelCase: cases[1],
                claims: everyClaim.slice(
                  0,
                  1,
                ),
                kind: 'damaged',
              },
            ],);
          },
        },),

        it({
          name: 'LABELS the items and the lines of the control set as control',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             The one control region.
             */
            const control = napCase({
              entryId: 'tabby',
              positions: [],
              recorded: 'not probed',
            },);

            /**
             Items the run kept.
             */
            const items = await collectFlagged({
              cases: [control,],
              kind: 'control',
              newClient: successiveClients({ clients: [everyProberCasts({ check: ADDITION_CHECK, },),], },).newClient,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(printed.lines,).toEqual([
              `VERIFY control tabby ${String(RUN_MODELS.checkerModelIds.length,)} admissible claims`,
            ],);
            expect(items.map(function kindOf(item,) {
              return item.kind;
            },),).toEqual(['control',],);
          },
        },),

        it({
          name: 'DROPS a region whose only claims the screen contradicted or found unanchored, counting it as none',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             Regions probed in order: contradicted claims, unanchored claims.
             */
            const cases = [
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

            /**
             Items the run kept.
             */
            const items = await collectFlagged({
              cases,
              kind: 'damaged',
              newClient: successiveClients({
                clients: [
                  everyProberCasts({ check: CONTRADICTED_CHECK, },),
                  everyProberCasts({ check: UNANCHORED_CHECK, },),
                ],
              },).newClient,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(items,).toEqual([],);
            expect(printed.lines,).toEqual([
              'VERIFY damaged tabby 0 admissible claims',
              'VERIFY damaged whiskers 0 admissible claims',
            ],);
          },
        },),

        it({
          name: 'BUILDS one client per region, so no region is answered from another region\'s reply',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            /**
             Builder and the count of clients it handed out.
             */
            const { newClient, built, } = successiveClients({
              clients: [
                everyProberCasts({ check: NO_DEFECT_CHECK, },),
                everyProberCasts({ check: NO_DEFECT_CHECK, },),
              ],
            },);

            await collectFlagged({
              cases: [
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
              ],
              kind: 'damaged',
              newClient,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(built(),).toBe(2,);
            expect(printed.lines.length,).toBe(2,);
          },
        },),
      ],
    },),
  ],
},);
