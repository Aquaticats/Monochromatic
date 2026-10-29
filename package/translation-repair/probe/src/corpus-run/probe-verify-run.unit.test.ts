/**
 Tests for the verify run over scripted gatherers and clients, in which no
 model is ever called and no corpus clone is read.

 CAPTURING `console.log` IS PROCESS-WIDE, which is why this file runs at
 `concurrency: 1`.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary';
import { RUN_MODELS, } from '@monochromatic-dev/translation-repair-run';
import {
  formatVerifyManifest,
  formatVerifySheet,
  type RelabelCase,
  runProbeVerify,
  VERIFY_BLIND_NOTE,
  type VerifyItem,
  verifyOpening,
  verifyWrote,
} from '../../dist/final/node/index.mjs';
import {
  NO_DEFECT_CHECK,
  omissionClaimsOfRoster,
  OMISSION_CHECK,
  probeScriptedClient,
  successiveClients,
} from '@monochromatic-dev/translation-repair-run/ts/introduced-defect-scripted-client.test-fixture.ts';
import { napCase, } from '../relabel-case.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
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
 Records what the gatherers were handed and answers with the given regions.
 */
type Gathering = {
  /**
   Arguments of every call to the damaged gatherer.
   */
  readonly damagedCalls: unknown[];

  /**
   Arguments of every call to the control gatherer.
   */
  readonly controlCalls: unknown[];

  /**
   The two gatherers, as the run takes them.
   */
  readonly gather: Parameters<typeof runProbeVerify>[0]['gather'];
};

/**
 Builds gatherers that answer with fixed regions and keep what they were asked.

 @param damaged - regions the damaged gatherer answers with

 @param controls - regions the control gatherer answers with

 @returns Gatherers and their recorded calls

 @example
 ```ts
 const gathering = scriptedGathering({ damaged: [DAMAGED_CASE,], controls: [], },);
 ```
 */
function scriptedGathering(
  {
    damaged,
    controls,
  }: {
    readonly damaged: readonly RelabelCase[];
    readonly controls: readonly RelabelCase[];
  },
): Gathering {
  /**
   Calls kept for the damaged gatherer.
   */
  const damagedCalls: unknown[] = [];

  /**
   Calls kept for the control gatherer.
   */
  const controlCalls: unknown[] = [];
  return {
    damagedCalls,
    controlCalls,
    gather: {
      damaged: function gatherDamaged(call,) {
        damagedCalls.push(call,);
        return Promise.resolve(damaged,);
      },
      controls: function gatherControls(call,) {
        controlCalls.push(call,);
        return Promise.resolve(controls,);
      },
    },
  };
}

/**
 A verify manifest with each item's claimants sorted. The claimants are listed in the order the probers answered, from a bench the
 prompt rotates, which moved when the house tense rule changed the prompt, so both manifests are read in one order.

 @param text - manifest JSON

 @returns Manifest JSON with every item's claimants sorted

 @example
 ```ts
 const sorted = claimantsSorted({ text: '{"items":[{"claimants":["b","a"]}]}', });
 ```
 */
function claimantsSorted({ text, }: { readonly text: string; },): string {
  /**
   The manifest read back.
   */
  const parsed = JSON.parse(text,) as { readonly items: readonly { readonly claimants: readonly string[]; }[]; };
  return JSON.stringify(
    {
      ...parsed,
      items: parsed.items.map(function sortedItem(item,) {
        return {
          ...item,
          claimants: item.claimants.toSorted(),
        };
      },),
    },
    undefined,
    2,
  );
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: verifyOpening.name,
      concurrency: 1,
      children: [
        it({
          name: 'SAYS one control region in the singular and counts the damaged set apart',
          fn: async () => {
            expect(verifyOpening({
              damaged: [
                DAMAGED_CASE,
                DAMAGED_CASE,
              ],
              controls: [CONTROL_CASE,],
            },),).toBe('VERIFY probing 2 damaged and 1 control region, issues withheld',);
          },
        },),

        it({
          name: 'SAYS no control region in the plural',
          fn: async () => {
            expect(verifyOpening({
              damaged: [DAMAGED_CASE,],
              controls: [],
            },),).toBe('VERIFY probing 1 damaged and 0 control regions, issues withheld',);
          },
        },),
      ],
    },),

    describe({
      name: verifyWrote.name,
      concurrency: 1,
      children: [
        it({
          name: 'SAYS one item in the singular and names the sheet in the directory',
          fn: async () => {
            expect(verifyWrote({
              count: 1,
              dir: '/cats/runs',
            },),).toBe('VERIFY wrote 1 item to /cats/runs/probe-verify-sheet.md',);
          },
        },),

        it({
          name: 'SAYS several items, and none, in the plural',
          fn: async () => {
            expect(verifyWrote({
              count: 2,
              dir: '/cats/runs',
            },),).toBe('VERIFY wrote 2 items to /cats/runs/probe-verify-sheet.md',);
            expect(verifyWrote({
              count: 0,
              dir: '/cats/runs',
            },),).toBe('VERIFY wrote 0 items to /cats/runs/probe-verify-sheet.md',);
          },
        },),
      ],
    },),

    describe({
      name: runProbeVerify.name,
      concurrency: 1,
      children: [
        it({
          name: 'GATHERS both sets from the manifest and the artifacts in the runs directory it is handed and the '
            + 'pin, probes each, and writes the blind sheet and its manifest',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            await using scratch = await scratchDir({ prefix: 'probe-verify-run-', },);
            /**
             Gatherers answering one damaged and one control region.
             */
            const gathering = scriptedGathering({
              damaged: [DAMAGED_CASE,],
              controls: [CONTROL_CASE,],
            },);

            /**
             Builder over one client per region: both omit the clause.
             */
            const { newClient, built, } = successiveClients({
              clients: [
                probeScriptedClient({
                  checkFor: function omits() {
                    return OMISSION_CHECK;
                  },
                  asked: [],
                },),
                probeScriptedClient({
                  checkFor: function omits() {
                    return OMISSION_CHECK;
                  },
                  asked: [],
                },),
              ],
            },);

            await runProbeVerify({
              dir: scratch.path,
              pin: PIN,
              newClient,
              gather: gathering.gather,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            /**
             Items the sheet and the manifest were written from.
             */
            const items: readonly VerifyItem[] = [
              {
                relabelCase: DAMAGED_CASE,
                claims: omissionClaimsOfRoster(),
                kind: 'damaged',
              },
              {
                relabelCase: CONTROL_CASE,
                claims: omissionClaimsOfRoster(),
                kind: 'control',
              },
            ];

            /**
             Manifest path both gatherers were handed.
             */
            const manifestPath = `${scratch.path}/sample-manifest-milestone-three-precision-round-three.json`;
            expect(gathering.damagedCalls,).toEqual([
              {
                manifestPath,
                pin: PIN,
                runsDir: scratch.path,
              },
            ],);
            expect(gathering.controlCalls,).toEqual([
              {
                manifestPath,
                damaged: [DAMAGED_CASE,],
                pin: PIN,
                runsDir: scratch.path,
              },
            ],);
            expect(built(),).toBe(2,);
            expect(printed.lines,).toEqual([
              'VERIFY probing 1 damaged and 1 control region, issues withheld',
              `VERIFY damaged tabby ${String(RUN_MODELS.checkerModelIds.length,)} admissible claims`,
              `VERIFY control tabby ${String(RUN_MODELS.checkerModelIds.length,)} admissible claims`,
              `VERIFY wrote 2 items to ${scratch.path}/probe-verify-sheet.md`,
              VERIFY_BLIND_NOTE,
            ],);
            expect(
              await readFile(
                join(
                  scratch.path,
                  'probe-verify-sheet.md',
                ),
                'utf8',
              ),
            ).toBe(formatVerifySheet({ items, },),);
            /**
             The manifest as the run wrote it, and as the formatter writes it for the same items.
             */
            const manifestText = await readFile(
              join(
                scratch.path,
                'probe-verify-manifest.json',
              ),
              'utf8',
            );
            const formatted = formatVerifyManifest({ items, },);
            expect(claimantsSorted({ text: manifestText, },),).toBe(claimantsSorted({ text: formatted, },),);
            // THE FORMAT ON DISK, beside the content compare that reads both manifests with the claimants in one order (n166, Z3):
            // the file is the formatter's own text, JSON with an indent of two spaces and no newline after it, so a writer that drops
            // the indent or adds a newline fails. The formatter's output is its own canonical form, which makes the same read of the
            // file mean the same thing.
            expect(JSON.stringify(JSON.parse(formatted,), undefined, 2,),).toBe(formatted,);
            expect(JSON.stringify(JSON.parse(manifestText,), undefined, 2,),).toBe(manifestText,);
          },
        },),

        it({
          name: 'WRITES an empty sheet and says none flagged when no region draws a claim',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            await using scratch = await scratchDir({ prefix: 'probe-verify-run-', },);

            await runProbeVerify({
              dir: scratch.path,
              pin: PIN,
              newClient: successiveClients({
                clients: [
                  probeScriptedClient({
                    checkFor: function none() {
                      return NO_DEFECT_CHECK;
                    },
                    asked: [],
                  },),
                ],
              },).newClient,
              gather: scriptedGathering({
                damaged: [DAMAGED_CASE,],
                controls: [],
              },).gather,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(printed.lines,).toEqual([
              'VERIFY probing 1 damaged and 0 control regions, issues withheld',
              'VERIFY damaged tabby 0 admissible claims',
              `VERIFY wrote 0 items to ${scratch.path}/probe-verify-sheet.md`,
              VERIFY_BLIND_NOTE,
            ],);
            expect(
              await readFile(
                join(
                  scratch.path,
                  'probe-verify-manifest.json',
                ),
                'utf8',
              ),
            ).toBe(formatVerifyManifest({ items: [], },),);
          },
        },),
        it({
          name: 'REFUSES a sheet already in the runs directory before gathering or asking anything, naming its path',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            await using scratch = await scratchDir({ prefix: 'probe-verify-run-', },);
            await writeFile(
              join(
                scratch.path,
                'probe-verify-sheet.md',
              ),
              '# a grader\'s work\n',
              'utf8',
            );
            /**
             Gatherers that would answer, and the calls they were given.
             */
            const gathering = scriptedGathering({
              damaged: [DAMAGED_CASE,],
              controls: [],
            },);

            /**
             Builder that would hand out a client, and a count of what it handed out.
             */
            const { newClient, built, } = successiveClients({
              clients: [
                probeScriptedClient({
                  checkFor: function omits() {
                    return OMISSION_CHECK;
                  },
                  asked: [],
                },),
              ],
            },);

            /**
             What the run refused with.
             */
            const refusal = await rejectionOf(async function runsOverATakenSheet(): Promise<unknown> {
              return await runProbeVerify({
                dir: scratch.path,
                pin: PIN,
                newClient,
                gather: gathering.gather,
                stageSettings: SCRIPTED_STAGE_SETTINGS,
              },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              `StatedRefusalError: ${scratch.path}/probe-verify-sheet.md already exists; grade or move it before rerunning, `
                + 'since a rerun would replace a grader\'s work',
            );
            expect(built(),).toBe(0,);
            expect(gathering.damagedCalls,).toEqual([],);
            expect(printed.lines,).toEqual([],);
            expect(
              await readFile(
                join(
                  scratch.path,
                  'probe-verify-sheet.md',
                ),
                'utf8',
              ),
            ).toBe('# a grader\'s work\n',);
          },
        },),

        it({
          name: 'REFUSES a manifest already in the runs directory before asking anything, naming its path',
          fn: async (ctx,) => {
            using printed = divertingConsoleLog({ sinon: ctx.sinon, },);
            await using scratch = await scratchDir({ prefix: 'probe-verify-run-', },);
            await writeFile(
              join(
                scratch.path,
                'probe-verify-manifest.json',
              ),
              '{"items":[]}',
              'utf8',
            );
            /**
             Builder that would hand out a client, and a count of what it handed out.
             */
            const { newClient, built, } = successiveClients({ clients: [], },);

            /**
             What the run refused with.
             */
            const refusal = await rejectionOf(async function runsOverATakenManifest(): Promise<unknown> {
              return await runProbeVerify({
                dir: scratch.path,
                pin: PIN,
                newClient,
                gather: scriptedGathering({
                  damaged: [DAMAGED_CASE,],
                  controls: [],
                },).gather,
                stageSettings: SCRIPTED_STAGE_SETTINGS,
              },);
            },);

            expect(refusal,).toBeInstanceOf(StatedRefusalError,);
            expect(String(refusal,),).toBe(
              `StatedRefusalError: ${scratch.path}/probe-verify-manifest.json already exists; grade or move it before rerunning, `
                + 'since a rerun would replace a grader\'s work',
            );
            expect(built(),).toBe(0,);
            expect(printed.lines,).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
