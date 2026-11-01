/**
 Tests for one slice's three arms.

 WHAT THESE PIN is the property the whole trial rests on: the slate is bought
 ONCE and all three arms judge that same slate. Until the stage was split,
 asking a slice twice resampled the candidates, so two answers differed in the
 slate as well as the evidence and no reading could say which moved the
 verdict. A regression here would not fail loudly; it would produce a
 confident number from a confounded comparison, which is worse.

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
import type { PreparedSheetFacts, } from '@monochromatic-dev/translation-repair-corpus-source';
import {
  makeInsertionChunk,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  armOrderFor,
  completedArms,
  readTrialLedger,
  runSliceArms,
  TRIAL_ARMS,
  trialKey,
  TrialSliceRefusalError,
} from '../../dist/final/node/index.mjs';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';
import { slicePairOf, } from '@monochromatic-dev/translation-repair-slice-reading/ts/corpus-run/slice-pair-of.test-fixture.ts';
import { DEATH_LINE, } from './window-trial-probe-draw.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Words the judges' sheet carries when the slice has no existing translation.
 */
const NOTHING_STANDS = 'there is no existing translation of this passage';

/**
 Logger the arms write to.
 */
const l = tagged({ tag: 'window-trial-slice-test', },);

/**
 Schema name the producing half asks translators for.
 */
const TRANSLATE_SCHEMA = 'translation_report';

/**
 Rosters every case uses.
 */
const MODELS = {
  translatorModelIds: ['hf:cat/Cat-A',
    'hf:cat/Cat-B',].map(function toId(id,) {
      return id as unknown as RosterModelId;
    },),
  judgeModelIds: ['hf:cat/Cat-A',
    'hf:cat/Cat-B',
    'hf:cat/Cat-C',].map(function toId(id,) {
      return id as unknown as RosterModelId;
    },),
};

/**
 Three-slice document, so the middle slice has a neighbour either way.
 */
const SLICES: readonly ChunkPair[] = [
  slicePairOf({
    sliceIndex: 0,
    source: '小猫在窗台上睡觉。',
    target: 'The cat sleeps on the windowsill.',
  },),
  slicePairOf({
    sliceIndex: 1,
    source: '猫猫在窗台上打盹，尾巴垂在暖气片旁边。',
    target: 'The cat is doing the sleeping, with tail by the radiator.',
  },),
  slicePairOf({
    sliceIndex: 2,
    source: '傍晚她回到炉火旁。',
    target: 'In the evening she returns to the fire.',
  },),
];

/**
 Three-slice document whose middle slice is archive English narrating its named subject in the present tense, so
 the detector finds a candidate in it once the entry records a death.
 */
const NARRATING_SLICES: readonly ChunkPair[] = [
  SLICES[0] as ChunkPair,
  slicePairOf({
    sliceIndex: 1,
    source: '猫猫在窗台上打盹，尾巴垂在暖气片旁边。',
    target: 'Mittens naps on the windowsill, tail by the radiator.',
  },),
  SLICES[2] as ChunkPair,
];

/**
 Page facts of an entry that records a death and declares its subject's name.
 */
const DEATH_FACTS: PreparedSheetFacts = {
  identityContext: DEATH_LINE,
  lifeEnded: true,
  subjectNames: ['Mittens',],
};

/**
 First line of the block a sheet carries for tense candidates.
 */
const TENSE_BLOCK_LINE = 'These sentences match present-tense shapes in a memorial page about a person who has died.';

/**
 Client whose translators drift on every call, so a rebought slate would show.

 @returns Client plus the judge sheets and translator call count

 @example
 ```ts
 const rig = driftingClient();
 ```
 */
function driftingClient({ ballot = 0, }: {
  readonly ballot?: number;
} = {},): {
  readonly client: ModelClient;
  readonly judgeSheets: string[];
  readonly translateSheets: string[];
  readonly served: { count: number; };
} {
  /**
   Translator calls served, which drives the drift and is itself the evidence
   that the slate was bought once.
   */
  const served = { count: 0, };

  /**
   Sheets the judges received.
   */
  const judgeSheets: string[] = [];

  /**
   Sheets the translators received, where the incumbent rides.
   */
  const translateSheets: string[] = [];

  return {
    served,
    judgeSheets,
    translateSheets,
    client: {
      chatText: async () => {
        throw new Error('chatText unused',);
      },
      quotas: async () => {
        throw new Error('quotas unused',);
      },
      chatJson: async <ValueT,>(
        request: ChatJsonRequest<ValueT>,
      ): Promise<ChatJsonOutcome<ValueT>> => {
        if (request.responseFormat
          ?.json_schema
          .name === TRANSLATE_SCHEMA) {
          served.count += 1;
          translateSheets.push(request.messages
            .map(function toContent(message,) {
              return messageText({ message, },);
            },)
            .join('\n',),);

          /**
           Rendering that differs on every call.
           */
          const value: unknown = {
            translation: `A fresh rendering number ${String(served.count,)}.`,
          };
          if (!request.validate(value,)) {
            return {
              kind: 'schema-mismatch',
              rawText: JSON.stringify(value,),
              detail: 'reply failed the wire guard',
            };
          }
          return {
            kind: 'ok',
            value,
            rawText: JSON.stringify(value,),
          };
        }

        judgeSheets.push(request.messages
          .map(function toContent(message,) {
            return messageText({ message, },);
          },)
          .join('\n',),);

        /**
         Ballot this rig casts on every slate, 0 declining everything so the
         archive stands and no case here depends on which candidate wins.
         */
        const ballotReply: unknown = {
          best: ballot,
          assessment: 'fixture',
        };
        if (!request.validate(ballotReply,)) {
          return {
            kind: 'schema-mismatch',
            rawText: JSON.stringify(ballotReply,),
            detail: 'reply failed the wire guard',
          };
        }
        return {
          kind: 'ok',
          value: ballotReply as ValueT,
          rawText: JSON.stringify(ballotReply,),
        };
      },
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

/**
 Arms already bought, keyed the way the ledger keys them.

 THROUGH `trialKey` RATHER THAN A LITERAL, because a test that spells the key
 itself agrees with nothing: the runner and the ledger once disagreed on the
 separator and every hand-written fixture passed anyway.

 @param arms - arms to mark bought for the slice every case uses

 @returns Key set shaped like `completedArms` returns

 @example
 ```ts
 const done = doneFor({ arms: [TRIAL_ARMS.wide,], },);
 ```
 */
function doneFor(
  { arms, }: { readonly arms: readonly string[]; },
): ReadonlySet<string> {
  return new Set(arms.map(function toKey(arm,) {
    return trialKey({ row: {
      protocol: 'protocol-one',
      entryId: 'Mittens',
      sliceIndex: 1,
      arm,
    }, },);
  },),);
}

await describe({
  name: runSliceArms.name,
  children: [
    it({
      name: 'BUYS THE SLATE ONCE AND JUDGES IT THREE TIMES, which is the property the whole trial '
        + 'rests on. The translators here drift on every call, so a rebought slate would show as a '
        + 'second round of translator calls and as different candidate text between arms',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-slice-', },);
        const rig = driftingClient();
        const rows = await runSliceArms({
          client: rig.client,
          slices: SLICES,
          sliceIndex: 1,
          sliceClass: 'relocation',
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

        expect(rows.length,).toBe(3,);
        // Two translators, called once each. A second production would double it.
        expect(rig.served
          .count,).toBe(MODELS.translatorModelIds.length,);
        // Three arms times three judges.
        expect(rig.judgeSheets
          .length,).toBe(3 * MODELS.judgeModelIds.length,);
      },
    },),
    it({
      name: 'shows the surrounding original to the WIDE arm only, so the three arms differ in '
        + 'exactly the evidence under test and in nothing else',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-slice-', },);
        const rig = driftingClient();
        await runSliceArms({
          client: rig.client,
          slices: SLICES,
          sliceIndex: 1,
          sliceClass: 'relocation',
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

        /**
         Sheets carrying the wider window.
         */
        const wide = rig.judgeSheets
          .filter(function carries(sheet,) {
            return sheet.includes('SURROUNDING ORIGINAL',);
          },);
        expect(wide.length,).toBe(MODELS.judgeModelIds.length,);
      },
    },),
    it({
      name: 'APPENDS EACH ARM AS IT COMPLETES rather than at the end, so a kill costs one arm and '
        + 'not the slice: every row is on disk before the next arm is bought',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-slice-', },);
        const rig = driftingClient();
        const ledgerPath = freshLedger({ dir: scratch.path, },);
        await runSliceArms({
          client: rig.client,
          slices: SLICES,
          sliceIndex: 1,
          sliceClass: 'relocation',
          entryId: 'Mittens',
          protocol: 'protocol-one',
          ledgerPath,
          done: new Set<string>(),
          models: MODELS,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          sheetFacts: {},
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect((await readTrialLedger({ path: ledgerPath, },)).map(function toArm(row,) {
          return row.arm;
        },),).toEqual([...armOrderFor({
          protocol: 'protocol-one',
          entryId: 'Mittens',
          sliceIndex: 1,
        },),],);
      },
    },),
    it({
      name: 'BUYS NOTHING AT ALL for a slice whose three arms the ledger already holds, not even '
        + 'the slate: producing is the expensive half, and paying for it to throw it away would '
        + 'cost most of what resumption is meant to save',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-slice-', },);
        const rig = driftingClient();
        const rows = await runSliceArms({
          client: rig.client,
          slices: SLICES,
          sliceIndex: 1,
          sliceClass: 'relocation',
          entryId: 'Mittens',
          protocol: 'protocol-one',
          ledgerPath: freshLedger({ dir: scratch.path, },),
          done: doneFor({
            arms: [TRIAL_ARMS.narrowFirst,
              TRIAL_ARMS.narrowSecond,
              TRIAL_ARMS.wide,],
          },),
          models: MODELS,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          sheetFacts: {},
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(rows.length,).toBe(0,);
        expect(rig.served
          .count,).toBe(0,);
        expect(rig.judgeSheets
          .length,).toBe(0,);
      },
    },),
    it({
      name: 'BUYS NOTHING FOR A PARTLY BOUGHT SLICE, which is the resumption path that would '
        + 'otherwise re-introduce the confound the stage was split to remove: the slate cannot be '
        + 'reproduced, so finishing the remaining arms here would judge different candidates from '
        + 'the arms already on disk while the ledger showed a complete triple',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-slice-', },);
        const rig = driftingClient();
        const rows = await runSliceArms({
          client: rig.client,
          slices: SLICES,
          sliceIndex: 1,
          sliceClass: 'relocation',
          entryId: 'Mittens',
          protocol: 'protocol-one',
          ledgerPath: freshLedger({ dir: scratch.path, },),
          // A kill after the first two arms.
          done: doneFor({
            arms: [TRIAL_ARMS.narrowFirst,
              TRIAL_ARMS.narrowSecond,],
          },),
          models: MODELS,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          sheetFacts: {},
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(rows.length,).toBe(0,);
        expect(rig.served
          .count,).toBe(0,);
        expect(rig.judgeSheets
          .length,).toBe(0,);
      },
    },),
    it({
      name: 'RESUMES OFF ITS OWN LEDGER, buying nothing the first run already wrote. This goes '
        + 'through the real file rather than a hand-written key set, which is the only way to '
        + 'catch the two key builders disagreeing: they did, on a NUL separator against a space, '
        + 'and every fixture that spelled the key itself passed while no live run ever resumed',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-slice-', },);
        const ledgerPath = freshLedger({ dir: scratch.path, },);

        /**
         First run, which buys all three arms and writes them.
         */
        const first = driftingClient();
        await runSliceArms({
          client: first.client,
          slices: SLICES,
          sliceIndex: 1,
          sliceClass: 'relocation',
          entryId: 'Mittens',
          protocol: 'protocol-one',
          ledgerPath,
          done: new Set<string>(),
          models: MODELS,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          sheetFacts: {},
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        /**
         Second run, resuming off exactly what the first wrote.
         */
        const second = driftingClient();
        const rows = await runSliceArms({
          client: second.client,
          slices: SLICES,
          sliceIndex: 1,
          sliceClass: 'relocation',
          entryId: 'Mittens',
          protocol: 'protocol-one',
          ledgerPath,
          done: completedArms({
            rows: await readTrialLedger({ path: ledgerPath, },),
            protocol: 'protocol-one',
          },),
          models: MODELS,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          sheetFacts: {},
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(rows.length,).toBe(0,);
        expect(second.served
          .count,).toBe(0,);
        expect(second.judgeSheets
          .length,).toBe(0,);
      },
    },),
    it({
      name: 'records the PANEL EACH ARM DECIDED ON, since the fan-out proceeds once half the '
        + 'roster answers and an arm that lost judges is otherwise written as an ordinary keep',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-slice-', },);
        const rig = driftingClient();
        const rows = await runSliceArms({
          client: rig.client,
          slices: SLICES,
          sliceIndex: 1,
          sliceClass: 'relocation',
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

        for (const row of rows) {
          expect(row.judgesSeated,).toBe(MODELS.judgeModelIds.length,);
          expect(row.judgesHeard,).toBe(MODELS.judgeModelIds.length,);
        }
      },
    },),
    it({
      name: 'REFUSES a slice with no neighbouring section before spending anything, since its wide '
        + 'arm would be its narrow arm and the pair would report a false null',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-slice-', },);
        const rig = driftingClient();

        /**
         Attempt on a lone slice, which has no window to widen to.
         */
        const attempt = runSliceArms({
          client: rig.client,
          slices: [SLICES[1] as ChunkPair,],
          sliceIndex: 0,
          sliceClass: 'relocation',
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
        /**
         What the attempt refused with.
         */
        const refusal = await rejectionOf(async function refused(): Promise<unknown> {
          return await attempt;
        },);
        expect(refusal,).toBeInstanceOf(TrialSliceRefusalError,);
        expect(String(refusal,),).toBe(
          'TrialSliceRefusalError: Mittens/0 has no neighbouring section carrying text, so its wide arm would be '
            + 'its narrow arm and the pair would report a false null',
        );
        // And it refused before buying anything.
        expect(rig.served
          .count,).toBe(0,);
      },
    },),
    it({
      name: 'SHOWS EVERY JUDGE THE TENSE CANDIDATES of the ballot texts on all three arms, and every voice the identity '
        + 'block naming the death, when the entry records one, so the instrument measures the sheet the pass asks',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-slice-', },);
        const rig = driftingClient();
        await runSliceArms({
          client: rig.client,
          slices: NARRATING_SLICES,
          sliceIndex: 1,
          sliceClass: 'relocation',
          entryId: 'Mittens',
          protocol: 'protocol-one',
          ledgerPath: freshLedger({ dir: scratch.path, },),
          done: new Set<string>(),
          models: MODELS,
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l,
          sheetFacts: DEATH_FACTS,
          stageSettings: SCRIPTED_STAGE_SETTINGS,
        },);

        expect(rig.judgeSheets
          .map(function carries(sheet,): readonly boolean[] {
            return [sheet.includes(TENSE_BLOCK_LINE,), sheet.includes(DEATH_LINE,),];
          },),).toEqual(Array.from(
            { length: 3 * MODELS.judgeModelIds.length, },
            function carriesBoth(): readonly boolean[] {
              return [true, true,];
            },
          ),);
        expect(rig.translateSheets
          .map(function carries(sheet,): boolean {
            return sheet.includes(DEATH_LINE,);
          },),).toEqual(Array.from(
            { length: MODELS.translatorModelIds.length, },
            function carriesIdentity(): boolean {
              return true;
            },
          ),);
      },
    },),
    it({
      name: 'SHOWS NO TENSE BLOCK AND NO DEATH when the entry records none, on the slice that shows both when it does, '
        + 'which separates a block the page facts bring from one every sheet carries',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-slice-', },);
        const rig = driftingClient();
        await runSliceArms({
          client: rig.client,
          slices: NARRATING_SLICES,
          sliceIndex: 1,
          sliceClass: 'relocation',
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

        expect(rig.judgeSheets
          .length,).toBe(3 * MODELS.judgeModelIds.length,);
        expect(rig.judgeSheets
          .some(function carries(sheet,): boolean {
            return sheet.includes(TENSE_BLOCK_LINE,) || sheet.includes(DEATH_LINE,);
          },),).toBe(false,);
      },
    },),
    it({
      name: 'THROWS the draw-and-preparation disagreement for a slice index the slices do not carry, '
        + 'since the two were made from different text',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-slice-', },);
        const rig = driftingClient();
        /**
         What a run on a slice index off the prepared list threw.
         */
        const refusal = await rejectionOf(async function runsOffTheList(): Promise<unknown> {
          return await runSliceArms({
            client: rig.client,
            slices: SLICES,
            sliceIndex: 99,
            sliceClass: 'relocation',
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
        },);
        expect(refusal,).toBeInstanceOf(TrialSliceRefusalError,);
        expect(String(refusal,),).toBe(
          'TrialSliceRefusalError: Mittens has no slice 99; the draw and the preparation disagree, which means they were '
            + 'made from different text',
        );
      },
    },),
    it({
      name: 'TELLS THE JUDGES NOTHING STANDS BEHIND A DECLINE for a slice whose target is an insertion, '
        + 'where the same source with a rendering keeps its fallback',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-slice-', },);
        /**
         Source both trials share.
         */
        const source = '猫猫在窗台上打盹。';
        /**
         Rendering the archive carries for it.
         */
        const incumbent = 'The cat dozes on the sill.';
        /**
         Trial over the slice the archive rendered.
         */
        const present = driftingClient({ ballot: 1, },);
        await runSliceArms({
          client: present.client,
          slices: [slicePairOf({
            sliceIndex: 0,
            source,
            target: incumbent,
          },), SLICES[1] as ChunkPair, SLICES[2] as ChunkPair,],
          sliceIndex: 0,
          sliceClass: 'relocation',
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
        /**
         Trial over the same source with the target side an insertion.
         */
        const absent = driftingClient({ ballot: 1, },);
        await runSliceArms({
          client: absent.client,
          slices: [{
            source: slicePairOf({
              sliceIndex: 0,
              source,
              target: incumbent,
            },).source,
            target: makeInsertionChunk({
              sliceIndex: 0,
              offset: 0,
            },),
          }, SLICES[1] as ChunkPair, SLICES[2] as ChunkPair,],
          sliceIndex: 0,
          sliceClass: 'relocation',
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
        expect(present.translateSheets
          .join('\n',)
          .includes(incumbent,),).toBe(true,);
        expect(absent.judgeSheets
          .every(function saysNothingStands(sheet,): boolean {
            return sheet.includes(NOTHING_STANDS,);
          },),).toBe(true,);
        expect(present.judgeSheets
          .some(function saysNothingStands(sheet,): boolean {
            return sheet.includes(NOTHING_STANDS,);
          },),).toBe(false,);
      },
    },),
  ],
},);
