/**
 Tests for how one drawn slice's refusal is handled.

 WHAT THESE PIN is that a slice which cannot be tried does not end the run. A
 refusal leaves no ledger row, so a walk that aborted on one would redraw the
 same slice on every resumption, reach it, and die at it again, never getting
 to the slices behind it. The distinction matters more than it looks: a
 refusal and a completed slice that owed nothing are both empty, and only one
 of them is a fault.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  runPick,
  trialKey,
} from '../../dist/final/node/index.mjs';
import { warningRecordingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  statusFailureLogText,
  statusFailureOf,
} from '@monochromatic-dev/translation-repair-provider/ts/provider-status-failure.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { slicePairOf, } from '@monochromatic-dev/translation-repair-slice-reading/ts/corpus-run/slice-pair-of.test-fixture.ts';
import { DEATH_LINE, } from './window-trial-probe-draw.test-fixture.ts';
import { scriptedTrialClient, } from '@monochromatic-dev/translation-repair-provider/ts/corpus-run/window-trial-scripted-client.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Logger the pick writes to.
 */
const l = tagged({ tag: 'window-trial-pick-test', },);

/**
 Rosters every case uses.
 */
const MODELS = {
  translatorModelIds: ['hf:cat/Cat-A',].map(function toId(id,) {
    return id as unknown as RosterModelId;
  },),
  judgeModelIds: ['hf:cat/Cat-A',
    'hf:cat/Cat-B',].map(function toId(id,) {
      return id as unknown as RosterModelId;
    },),
};

/**
 Lone slice, which has no neighbour to widen to and so must refuse.
 */
const LONE: readonly ChunkPair[] = [
  slicePairOf({
    sliceIndex: 0,
    source: '猫猫在窗台上打盹。',
    target: 'The cat naps on the windowsill.',
  },),
];

/**
 Client that raises on every exchange.

 @param error - failure every exchange raises

 @returns Client the pick calls through

 @example
 ```ts
 const client = throwingClient({ error: new Error('provider refused', ), },);
 ```
 */
function throwingClient(
  { error, }: { readonly error: Error; },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused',);
    },
    quotas: async () => {
      throw new Error('quotas unused',);
    },
    chatJson: async <ValueT,>(
      _request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      throw error;
    },
  };
}

/**
 Ledger path nested inside a case-owned directory.

 @param dir - case-owned directory the ledger lives under

 @returns Path inside that directory

 @example
 ```ts
 const path = freshLedger({ dir: scratch.path, },);
 ```
 */
function freshLedger({ dir, }: { readonly dir: string; },): string {
  return join(
    dir,
    'trial.jsonl',
  );
}

await describe({
  name: runPick.name,
  concurrency: 1,
  children: [
    it({
      name: 'REPORTS A REFUSAL RATHER THAN RAISING IT, so one unwidenable slice cannot wedge the '
        + 'walk: a refusal writes no ledger row, so an aborting run would redraw the same slice, '
        + 'walk to it and die at it on every restart, never reaching the slices behind it',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-pick-', },);
        const outcome = await runPick({
          client: throwingClient({ error: new Error('unused', ), },),
          slices: LONE,
          pick: {
            entryId: 'Mittens',
            sliceIndex: 0,
            sliceClass: 'relocation',
          },
          entryId: 'Mittens',
          protocol: 'protocol-one',
          ledgerPath: freshLedger({ dir: scratch.path, },),
          done: new Set<string>(),
          models: MODELS,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          sheetFacts: {},
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(outcome.kind,).toBe('refused',);
      },
    },),
    it({
      name: 'WARNS OF A REFUSAL BY THE SENTENCE ITS CLASS DECLARES SAFE, so the operator reads which slice '
        + 'had no window and no message of another class is repeated',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-pick-', },);
        const warnings: string[] = [];
        await runPick({
          client: throwingClient({ error: new Error('unused', ), },),
          slices: LONE,
          pick: {
            entryId: 'Mittens',
            sliceIndex: 0,
            sliceClass: 'relocation',
          },
          entryId: 'Mittens',
          protocol: 'protocol-one',
          ledgerPath: freshLedger({ dir: scratch.path, },),
          done: new Set<string>(),
          models: MODELS,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l: warningRecordingLogger({ base: l, warnings, },),
          sheetFacts: {},
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(warnings,).toEqual([
          'Mittens/0 (relocation): refused, Mittens/0 has no neighbouring section carrying text, so its wide arm '
            + 'would be its narrow arm and the pair would report a false null',
        ],);
      },
    },),
    it({
      name: 'WARNS OF A PROVIDER REFUSAL THAT ENDED THE ARMS by the seat, the status and the provider\'s words, '
        + 'the key its refusal echoed masked, once, then of the slice refused by the caller\'s abort, where the run '
        + 'was stopped while the provider refused',
      fn: async ctx => {
        await using scratch = await scratchDir({ prefix: 'window-pick-', },);
        /**
         Refusal the real client raised over the real transport.
         */
        const failure = await statusFailureOf({
          sinon: ctx.sinon,
          status: 401,
        },);
        /**
         Stop the caller gives while the provider is refusing.
         */
        const stop = new AbortController();
        const warnings: string[] = [];
        const outcome = await runPick({
          client: {
            chatText: async () => {
              throw new Error('chatText unused',);
            },
            quotas: async () => {
              throw new Error('quotas unused',);
            },
            chatJson: async <ValueT,>(
              _request: ChatJsonRequest<ValueT>,
            ): Promise<ChatJsonOutcome<ValueT>> => {
              stop.abort();
              throw failure;
            },
          },
          slices: [
            slicePairOf({
              sliceIndex: 0,
              source: '猫猫在窗台上打盹。',
              target: 'The cat naps on the windowsill.',
            },),
            slicePairOf({
              sliceIndex: 1,
              source: '窗台上有一只鸟。',
              target: 'On the windowsill there is a bird.',
            },),
          ],
          pick: {
            entryId: 'Mittens',
            sliceIndex: 0,
            sliceClass: 'relocation',
          },
          entryId: 'Mittens',
          protocol: 'protocol-one',
          ledgerPath: freshLedger({ dir: scratch.path, },),
          done: new Set<string>(),
          models: MODELS,
          signal: stop.signal,
          perCallTimeoutMs: HANG_STOP_MS,
          l: warningRecordingLogger({ base: l, warnings, },),
          sheetFacts: {},
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(outcome.kind,).toBe('refused',);
        expect(warnings,).toEqual([
          `[produceTranslateSlate] translate hf:cat/Cat-A: ${
            statusFailureLogText({ provider: 'synthetic', status: 401, },)
          }, as the caller stopped the round`,
          'Mittens/0 (relocation): refused, refused by AbortError',
        ],);
      },
    },),
    it({
      name: 'HANDS THE PAGE FACTS ON TO THE ARMS, so a slice drawn from an entry that records a death is judged on sheets '
        + 'carrying the tense candidates of the ballot texts and the identity block naming the date',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-pick-', },);
        const rig = scriptedTrialClient({
          narrowBallot: 0,
          wideBallot: 0,
        },);
        /**
         Block that heads the tense candidates on a sheet.
         */
        const tenseLine = 'These sentences match present-tense shapes in a memorial page about a person who has died.';
        const outcome = await runPick({
          client: rig.client,
          slices: [
            slicePairOf({
              sliceIndex: 0,
              source: '小猫在窗台上睡觉。',
              target: 'The cat sleeps on the windowsill.',
            },),
            slicePairOf({
              sliceIndex: 1,
              source: '猫猫在窗台上打盹，尾巴垂在暖气片旁边。',
              target: 'Mittens naps on the windowsill, tail by the radiator.',
            },),
            slicePairOf({
              sliceIndex: 2,
              source: '傍晚她回到炉火旁。',
              target: 'In the evening she returns to the fire.',
            },),
          ],
          pick: {
            entryId: 'Mittens',
            sliceIndex: 1,
            sliceClass: 'relocation',
          },
          entryId: 'Mittens',
          protocol: 'protocol-one',
          ledgerPath: freshLedger({ dir: scratch.path, },),
          done: new Set<string>(),
          models: MODELS,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          sheetFacts: {
            identityContext: DEATH_LINE,
            lifeEnded: true,
            subjectNames: ['Mittens',],
          },
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(outcome.kind,).toBe('bought',);
        expect(rig.judgeSheets
          .map(function carries(sheet,): readonly boolean[] {
            return [sheet.includes(tenseLine,), sheet.includes(DEATH_LINE,),];
          },),).toEqual(Array.from(
            { length: 3 * MODELS.judgeModelIds.length, },
            function carriesBoth(): readonly boolean[] {
              return [true, true,];
            },
          ),);
      },
    },),
    it({
      name: 'reports a slice the ledger already holds as bought with no rows, so a resumed run '
        + 'walks past it without counting it as a refusal',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-pick-', },);
        const outcome = await runPick({
          client: throwingClient({ error: new Error('unused', ), },),
          slices: LONE,
          pick: {
            entryId: 'Mittens',
            sliceIndex: 0,
            sliceClass: 'relocation',
          },
          entryId: 'Mittens',
          protocol: 'protocol-one',
          ledgerPath: freshLedger({ dir: scratch.path, },),
          done: new Set(['narrow-a',
            'narrow-b',
            'wide',].map(function toKey(arm,) {
              return trialKey({ row: {
                protocol: 'protocol-one',
                entryId: 'Mittens',
                sliceIndex: 0,
                arm,
              }, },);
            },),),
          models: MODELS,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          sheetFacts: {},
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(outcome.kind,).toBe('bought',);
        expect((outcome.kind === 'bought') ? outcome.rows.length : -1,).toBe(0,);
      },
    },),
  ],
},);
