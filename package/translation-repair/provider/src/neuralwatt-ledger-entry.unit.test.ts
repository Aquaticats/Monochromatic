/**
 Tests one line of the NeuralWatt energy ledger and the reading of it: the
 reckonings a line may carry, and the refusal, naming the file and the line,
 for a line that will not read as an energy entry. The ledger writes
 append-only JSON lines (`neuralwatt-ledger.ts`), so this reading decides
 whether a hand-edited or torn file reports energy as unspent.

 REACHED THROUGH THE LEDGER'S `read`, because `neuralWattEntryOf` and
 `isNeuralWattReckoning` are not on the package index, which exports only what
 the siblings and their tests reach, and a test imports the built package and
 not the source. A line that reads shows in the day's reading, by its
 watt-hours and its reckoning; a line that does not read
 shows in the error `read` throws. Every case writes a ledger file in a
 throwaway directory and reads time from a clock the case holds, so no case
 reads the real ledger or the real clock. Fixtures are cat-themed invention.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createNeuralWattLedger,
  NEURALWATT_RECKONINGS,
  NeuralWattLedgerShapeError,
} from '../dist/final/node/index.mjs';
import {
  handClock,
  lineOf,
  MIDNIGHT_DAY,
} from './neuralwatt-ledger.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Fields of a line that reads, to which a case adds or changes one.
 */
const GOOD = {
  at: '2026-10-20T01:00:00.000Z',
  model: 'glm-5.3-flash',
  wh: 1,
  promptTokens: 90,
} as const;

/**
 Reads a ledger file holding the given text.

 @param text - whole file

 @returns Day's reading it gave

 @example
 ```ts
 const energy = await energyOf({ text: lineOf({ ...GOOD, wh: 2, },), },);
 ```
 */
async function energyOf({ text, }: { readonly text: string; },) {
  await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-entry-', },);
  const path = join(scratch.path, 'energy.jsonl',);
  await writeFile(path, text, 'utf8',);
  const { now, } = handClock({ start: '2026-10-20T12:00:00.000Z', },);
  return await createNeuralWattLedger({ path, dailyWh: 550, now, },).read({ day: MIDNIGHT_DAY, },);
}

/**
 What reading a file whose second line is the given one throws; the first line is good.

 @param bad - second line of the file, with its newline where it has one

 @returns Thrown value and path of the file

 @example
 ```ts
 const { refusal, path, } = await refusalFor({ bad: '[1]\n', },);
 ```
 */
async function refusalFor({ bad, }: { readonly bad: string; },) {
  await using scratch = await scratchDir({ prefix: 'whiskers-neuralwatt-entry-', },);
  const path = join(scratch.path, 'energy.jsonl',);
  await writeFile(path, lineOf(GOOD,) + bad, 'utf8',);
  const { now, } = handClock({ start: '2026-10-20T12:00:00.000Z', },);
  return {
    path,
    refusal: await rejectionOf(async function read(): Promise<void> {
      await createNeuralWattLedger({ path, dailyWh: 550, now, },).read({ day: MIDNIGHT_DAY, },);
    },),
  };
}

/**
 Asserts that reading a file with a bad second line throws the shape error naming the file, line 2 and the detail.

 @param bad - second line of the file

 @param detail - phrase the error should carry
 */
async function expectRefused({ bad, detail, }: { readonly bad: string; readonly detail: string; },): Promise<void> {
  const { path, refusal, } = await refusalFor({ bad, },);
  expect(refusal,).toBeInstanceOf(NeuralWattLedgerShapeError,);
  expect(String(refusal,),).toContain(path,);
  expect(String(refusal,),).toContain('line 2',);
  expect(String(refusal,),).toContain(detail,);
}

await describe({
  name: 'NeuralWatt ledger entry',
  concurrency: 1,
  children: [
    describe({
      name: 'a line that reads',
      children: [
        it({
          name: 'COUNTS A MEASURED LINE AT ITS WATT-HOURS with no reckoning, and a line of zero watt-hours, '
            + 'since a call may use nothing',
          fn: async () => {
            const energy = await energyOf({
              text: lineOf(GOOD,) + lineOf({ ...GOOD, wh: 0, promptTokens: -1, },)
                + lineOf({ ...GOOD, wh: 0.5, cat: 'tabby', },),
            },);
            expect(energy,).toMatchObject({
              spentWh: 1.5,
              reckonedWh: 0,
              calls: 3,
            },);
          },
        },),

        it({
          name: 'KNOWS THE TWO REASONS A LINE\'S WATT-HOURS ARE AN ESTIMATE, the stream that carried no energy comment '
            + 'and the attempt that never finished, and counts each toward the reckoned share',
          fn: async () => {
            expect(NEURALWATT_RECKONINGS,).toEqual(['unreported-estimate', 'abandoned-estimate',],);
            const energy = await energyOf({
              text: lineOf({ ...GOOD, wh: 0.25, estimated: 'unreported-estimate', },)
                + lineOf({ ...GOOD, wh: 0.5, estimated: 'abandoned-estimate', },)
                + lineOf({ ...GOOD, wh: 2, },),
            },);
            expect(energy,).toMatchObject({ spentWh: 2.75, reckonedWh: 0.75, calls: 3, },);
          },
        },),

        it({
          name: 'READS A LINE CARRYING THE OLDER billedUsd FIELD AS AN ORDINARY ENERGY LINE, alone or beside a '
            + 'reckoning: it counts at its watt-hours, adds to the reckoned share only through its reckoning, and '
            + 'marks nothing about the day',
          fn: async () => {
            /**
             Readings of the two files.
             */
            const energies = await Promise.all([
              lineOf({ ...GOOD, billedUsd: 0.00001, },),
              lineOf({ ...GOOD, billedUsd: 0.2, estimated: 'abandoned-estimate', },),
            ].map(async function read(text,) {
              return await energyOf({ text, },);
            },),);
            expect(energies.map(function shape(energy,) {
              return { spentWh: energy.spentWh, reckonedWh: energy.reckonedWh, calls: energy.calls, billedKey: 'billedToday' in energy, };
            },),).toEqual([
              { spentWh: 1, reckonedWh: 0, calls: 1, billedKey: false, },
              { spentWh: 1, reckonedWh: 1, calls: 1, billedKey: false, },
            ],);
          },
        },),
      ],
    },),

    describe({
      name: 'a line that will not read',
      children: [
        it({
          name: 'IS A THROWN ERROR NAMING THE FILE AND THE LINE, and the kind of parse failure for a line that is not '
            + 'JSON, never quoting the line',
          fn: async () => {
            const { path, refusal, } = await refusalFor({ bad: '{"at": "tabby-secret\n', },);
            expect(refusal,).toBeInstanceOf(NeuralWattLedgerShapeError,);
            expect(String(refusal,),).toContain(`ledger ${path} line 2 violated expectations: not valid JSON (SyntaxError)`,);
            expect(String(refusal,),).not.toContain('tabby-secret',);
          },
        },),

        it({
          name: 'REFUSES JSON THAT IS NOT AN OBJECT, an array, a number, a string and null',
          fn: async () => {
            await Promise.all(['[1]\n', '5\n', '"at"\n', 'null\n',].map(async function check(bad,): Promise<void> {
              await expectRefused({ bad, detail: 'not a JSON object', },);
            },),);
          },
        },),

        it({
          name: 'REFUSES AN INSTANT THAT IS NOT SPELLED THE WAY toISOString WRITES IT, a missing field, a number, a '
            + 'word, an offset that names the right instant and a stamp with no zone that would read as local time',
          fn: async () => {
            await Promise.all([
              lineOf({ model: GOOD.model, wh: 1, promptTokens: 1, },),
              lineOf({ ...GOOD, at: 5, },),
              lineOf({ ...GOOD, at: 'yesterday', },),
              lineOf({ ...GOOD, at: '2026-10-20T08:00:00.000-04:00', },),
              lineOf({ ...GOOD, at: '2026-10-20T12:00:00.000', },),
            ].map(async function check(bad,): Promise<void> {
              await expectRefused({ bad, detail: 'at is not an instant spelled as toISOString writes it', },);
            },),);
          },
        },),

        it({
          name: 'REFUSES A MODEL THAT IS NOT A STRING, absent included',
          fn: async () => {
            await Promise.all([
              lineOf({ ...GOOD, model: 5, },),
              lineOf({ ...GOOD, model: null, },),
              lineOf({ at: GOOD.at, wh: 1, promptTokens: 1, },),
            ].map(async function check(bad,): Promise<void> {
              await expectRefused({ bad, detail: 'model is not a string', },);
            },),);
          },
        },),

        it({
          name: 'REFUSES WATT-HOURS THAT ARE NOT A FINITE NON-NEGATIVE NUMBER, a string, a negative, one too large to '
            + 'be finite (1e999 parses to infinity) and a missing figure',
          fn: async () => {
            await Promise.all([
              lineOf({ ...GOOD, wh: '1', },),
              lineOf({ ...GOOD, wh: -0.001, },),
              '{"at":"2026-10-20T01:00:00.000Z","model":"glm-5.3-flash","wh":1e999,"promptTokens":90}\n',
              lineOf({ at: GOOD.at, model: GOOD.model, promptTokens: 90, },),
            ].map(async function check(bad,): Promise<void> {
              await expectRefused({ bad, detail: 'wh is not a non-negative number', },);
            },),);
          },
        },),

        it({
          name: 'REFUSES A PROMPT-TOKEN COUNT THAT IS NOT A FINITE NUMBER, a string, a missing one and one too large to '
            + 'be finite',
          fn: async () => {
            await Promise.all([
              lineOf({ ...GOOD, promptTokens: '90', },),
              lineOf({ at: GOOD.at, model: GOOD.model, wh: 1, },),
              '{"at":"2026-10-20T01:00:00.000Z","model":"glm-5.3-flash","wh":1,"promptTokens":1e999}\n',
            ].map(async function check(bad,): Promise<void> {
              await expectRefused({ bad, detail: 'promptTokens is not a number', },);
            },),);
          },
        },),

        it({
          name: 'REFUSES A RECKONING THIS PACKAGE DOES NOT WRITE, including one that is not a string',
          fn: async () => {
            await Promise.all(['guess', 'abandoned', 5, null,].map(async function check(estimated,): Promise<void> {
              await expectRefused({ bad: lineOf({ ...GOOD, estimated, },), detail: 'estimated is not a reckoning this package writes', },);
            },),);
          },
        },),
      ],
    },),
  ],
},);
