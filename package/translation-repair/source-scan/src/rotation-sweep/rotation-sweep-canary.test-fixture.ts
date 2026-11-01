import { join, } from 'node:path';

import { isJsonArray, } from '@monochromatic-dev/translation-repair-document-structure/ts';

import { FAMILY_ROOT, } from './rotation-sweep-packages.test-fixture.ts';
import { runSalted, } from './rotation-sweep-run.test-fixture.ts';

//region Rotation sweep canary
// WHAT KEEPS THE SWEEP HONEST (ledger B701). The hook rewrites the function by
// name, so a bundle that stops declaring `benchRotation` the way the hook looks
// for it would leave every salt moving nothing and the sweep passing. Before any
// suite runs, the canary asks the built stage package for its rotation of a few
// prompts over every bench size up to the largest, with no shift and with each
// salt to be run, and stops the sweep unless each answer moved by exactly the
// salt, modulo the bench.

/**
 Largest bench any stage of the family seats.
 */
const LARGEST_BENCH = 9;

/**
 Prompts the canary asks about: short, long with a line break, and empty.
 */
const PROMPTS: readonly string[] = [
  'cat',
  'a longer prompt about whiskers\nand naps',
  '',
];

/**
 Built entry of the stage package, which the canary imports.
 */
const STAGE_ENTRY: string = join(
  FAMILY_ROOT,
  'stage',
  'dist',
  'final',
  'node',
  'index.mjs',
);

/**
 Program the canary runs: the rotation of each prompt over every bench size from one to the largest.
 */
const CANARY_PROGRAM = `
const { benchRotation } = await import(process.argv[1]);
const prompts = ${JSON.stringify(PROMPTS,)};
const answers = [];
for (let size = 1; size <= ${String(LARGEST_BENCH,)}; size += 1) {
  for (const content of prompts) answers.push(benchRotation({ messages: [{ role: 'user', content }], size }));
}
process.stdout.write(JSON.stringify(answers));
`;

/**
 Asks the built stage package for its rotations under a salt.

 @param salt - seats the rotation start is moved by

 @returns The rotations, one per prompt per bench size, sizes ascending

 @throws Error when the program fails or prints anything but a list of numbers

 @example
 ```ts
 const answers = await canaryUnder({ salt: 3, },);
 ```
 */
async function canaryUnder({ salt, }: { readonly salt: number; },): Promise<readonly number[]> {
  /**
   What the program printed.
   */
  const run = await runSalted({
    salt,
    cwd: FAMILY_ROOT,
    args: [
      '--input-type=module',
      '--eval',
      CANARY_PROGRAM,
      STAGE_ENTRY,
    ],
  },);
  if (run.code !== 0)
    throw new Error(`rotation sweep canary failed under salt ${String(salt,)}:\n${run.stdout}${run.stderr}`,);
  /**
   The program's output, parsed.
   */
  const parsed: unknown = JSON.parse(run.stdout,);
  if ((!isJsonArray(parsed,)) || parsed.some(function isNotNumber(answer,): boolean {
    return (typeof answer) !== 'number';
  },))
    throw new Error(`rotation sweep canary printed no list of numbers under salt ${String(salt,)}: ${run.stdout}`,);
  return parsed.filter(function isNumber(answer,): answer is number {
    return (typeof answer) === 'number';
  },);
}

/**
 Stops the sweep unless the hook moves the built stage package's rotation by each salt.

 @param salts - salts the sweep will run, zero among them or not

 @throws Error when a salt moved an answer other than by the salt modulo its bench, or when a salt above zero moved no answer at all

 @example
 ```ts
 await assertSaltMovesRotation({ salts: [0, 3,], },);
 ```
 */
export async function assertSaltMovesRotation({ salts, }: { readonly salts: readonly number[]; },): Promise<void> {
  /**
   Rotations with nothing moved.
   */
  const baseline = await canaryUnder({ salt: 0, },);
  for (const salt of salts.filter(function isShift(candidate,): boolean {
    return candidate > 0;
  },)) {
    /* oxlint-disable no-await-in-loop -- the salts are asked one after another so a failure names the first salt that moved nothing */
    /**
     Rotations under this salt.
     */
    const moved = await canaryUnder({ salt, },);
    /* oxlint-enable no-await-in-loop */
    for (const [at, answer,] of moved.entries()) {
      /**
       Bench size this answer is for: the questions run over the prompts for each size in turn.
       */
      const size = Math.floor(at / PROMPTS.length,) + 1;
      /**
       What the answer must be.
       */
      const expected = ((baseline[at] ?? 0) + salt) % size;
      if (answer !== expected) {
        throw new Error(
          `rotation sweep canary: under salt ${String(salt,)} the stage package answered ${String(answer,)} for question ${String(at,)} (bench of ${String(size,)}), not ${String(expected,)}; the hook does not move the rotation of ${STAGE_ENTRY}, so a sweep would pass with nothing shifted`,
        );
      }
    }
    if (moved.every(function isUnmoved(
      answer,
      at,
    ): boolean {
      return answer === baseline[at];
    },))
      throw new Error(`rotation sweep canary: salt ${String(salt,)} moved no answer`,);
  }
}

//endregion Rotation sweep canary
