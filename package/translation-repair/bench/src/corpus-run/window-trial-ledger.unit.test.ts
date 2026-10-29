/**
 Tests for the window trial's durable ledger.

 WHAT THESE PIN is survivability of a run that spends roughly 1760 real
 exchanges and has no cache behind it, because the window trial calls the stage directly
 and the slice cache is read by the document driver. Everything here is about
 what happens when the process does NOT reach the end, which is the case the
 ledger exists for and the case that never happens in a passing test unless it
 is written on purpose.

 Fixtures are cat-themed invention mirroring corpus structure only. Each case
 writes into its own throwaway directory.

 @module
 */

import {
  appendFile,
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  accountTrialLedger,
  appendTrialRow,
  completedArms,
  readTrialLedger,
  trialKey,
  type WindowTrialRow,
} from '../../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Protocol digest the ordinary cases buy under.
 */
const PROTOCOL = 'protocol-one';

/**
 Builds one completed arm.

 @param arm - which arm this row is

 @param sliceIndex - slice position

 @param protocol - digest it was bought under

 @returns Row shaped like one a runner appends

 @example
 ```ts
 const row = rowFor({ arm: 'wide', sliceIndex: 3, },);
 ```
 */
function rowFor(
  {
    arm,
    sliceIndex,
    protocol = PROTOCOL,
  }: {
    readonly arm: string;
    readonly sliceIndex: number;
    readonly protocol?: string;
  },
): WindowTrialRow {
  return {
    protocol,
    entryId: 'Mittens',
    sliceIndex,
    arm,
    sliceClass: 'relocation',
    shipped: arm === 'wide',
    decision: 'judged',
    winnerText: 'The cat sleeps on the windowsill.\n',
    judgesHeard: 6,
    judgesSeated: 6,
    position: 0,
  };
}

/**
 Ledger path nested inside a case-owned directory, so no case can read
 another's writes.

 @param dir - case-owned directory the ledger nests under

 @returns Path inside a fresh subdirectory

 @example
 ```ts
 const path = freshLedger({ dir: scratch.path, },);
 ```
 */
function freshLedger({ dir, }: { readonly dir: string; },): string {
  return join(
    dir,
    'nested',
    'trial.jsonl',
  );
}

/**
 Opening of a row as a kill mid-append leaves it: no closing brace and no
 newline.
 */
const TORN_FRAGMENT = '{"protocol":"protocol-one","entr';

/**
 First narrow arm of slice 1, the arm bought before a kill.
 */
const NARROW_A = rowFor({
  arm: 'narrow-a',
  sliceIndex: 1,
},);

/**
 Second narrow arm of slice 1.
 */
const NARROW_B = rowFor({
  arm: 'narrow-b',
  sliceIndex: 1,
},);

/**
 Wide arm of slice 1.
 */
const WIDE = rowFor({
  arm: 'wide',
  sliceIndex: 1,
},);

/**
 One ledger line holding a row, as `appendTrialRow` writes it.

 @param row - arm the line holds

 @returns Line with its newline

 @example
 ```ts
 const line = lineOf({ row: rowFor({ arm: 'wide', sliceIndex: 3, },), },);
 ```
 */
function lineOf({ row, }: { readonly row: WindowTrialRow; },): string {
  return `${JSON.stringify(row,)}\n`;
}

/**
 Writes a ledger by hand, creating its directory, since only
 `appendTrialRow` creates it.

 @param path - ledger file

 @param text - whole content to write

 @example
 ```ts
 await writeLedger({ path, text: TORN_FRAGMENT, },);
 ```
 */
async function writeLedger(
  {
    path,
    text,
  }: {
    readonly path: string;
    readonly text: string;
  },
): Promise<void> {
  await mkdir(
    dirname(path,),
    { recursive: true, },
  );
  await writeFile(
    path,
    text,
  );
}

/**
 Whole message of the refusal a complete line that does not parse raises.

 @param path - ledger file the line sits in

 @param line - one-based number of the line

 @returns Message naming the file and the line, and none of its text

 @example
 ```ts
 const message = unreadableLineMessage({ path, line: 2, },);
 ```
 */
function unreadableLineMessage(
  {
    path,
    line,
  }: {
    readonly path: string;
    readonly line: number;
  },
): string {
  return `window trial ledger ${path}: line ${String(line,)} ends in a newline and does not parse as JSON `
    + '(SyntaxError). A kill mid-append leaves only an unterminated last line, which the next append removes, '
    + 'so this line was written by something else: a build that appended a row onto such a fragment, or two '
    + 'runs appending at once. What it bought cannot be read from here. Remove that line by hand to buy its '
    + 'arms again, or move the ledger aside to start a fresh one.';
}

await describe({
  name: 'window trial ledger',
  children: [
    it({
      name: 'reads back every arm it appended, in order, and CREATES THE DIRECTORY on the way, so '
        + 'a runner pointed at a fresh output path does not lose its first arm to a missing parent',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-trial-', },);
        const path = freshLedger({ dir: scratch.path, },);
        for (const arm of ['narrow-a',
          'narrow-b',
          'wide',]) {
          // SEQUENTIAL ON PURPOSE, which is what this case asserts: the ledger
          // is append-ordered and a runner appends one arm at a time as it
          // completes. Racing these would test a shape no runner produces.
          /* oxlint-disable-next-line no-await-in-loop -- append order is the assertion */
          await appendTrialRow({
            path,
            row: rowFor({
              arm,
              sliceIndex: 3,
            },),
          },);
        }

        const rows = await readTrialLedger({ path, },);
        expect(rows.length,).toBe(3,);
        expect(rows.map(function toArm(row,) {
          return row.arm;
        },),).toEqual(['narrow-a',
          'narrow-b',
          'wide',],);
      },
    },),
    it({
      name: 'reports an ABSENT ledger as empty rather than throwing, since that is the ordinary '
        + 'state before the first arm is bought and a runner should not need to pre-create it',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-trial-', },);
        expect((await readTrialLedger({ path: freshLedger({ dir: scratch.path, },), },)).length,).toBe(0,);
      },
    },),
    it({
      name: 'DROPS A TORN FINAL LINE and keeps everything before it, which is the whole case this '
        + 'exists for: a process killed mid-append leaves a fragment, and losing the run rather '
        + 'than one arm would defeat the point of appending as it goes',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-trial-', },);
        const path = freshLedger({ dir: scratch.path, },);
        await appendTrialRow({
          path,
          row: rowFor({
            arm: 'narrow-a',
            sliceIndex: 1,
          },),
        },);
        // A kill in the middle of the second append.
        await appendTrialRow({
          path,
          row: rowFor({
            arm: 'narrow-b',
            sliceIndex: 1,
          },),
        },);
        const whole = await readTrialLedger({ path, },);
        expect(whole.length,).toBe(2,);

        /**
         The same file with its last line truncated mid-JSON.
         */
        const torn = `${JSON.stringify(rowFor({
          arm: 'narrow-a',
          sliceIndex: 1,
        },),)}\n{"protocol":"protocol-one","entr`;
        await writeFile(
          path,
          torn,
        );

        const rows = await readTrialLedger({ path, },);
        expect(rows.length,).toBe(1,);
        expect(rows[0]?.arm,).toBe('narrow-a',);
      },
    },),
    it({
      name: 'LEAVES OUT a whole line that is no trial row of this shape, here a row an older build wrote '
        + 'under another field name, counts it, and keeps the rows on both sides of it',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-trial-', },);
        const path = freshLedger({ dir: scratch.path, },);

        /**
         Row before the older one.
         */
        const first = rowFor({
          arm: 'narrow-a',
          sliceIndex: 1,
        },);

        /**
         Row after it.
         */
        const last = rowFor({
          arm: 'wide',
          sliceIndex: 1,
        },);

        /**
         The same arm as builds before the index was renamed wrote it: its
         slice under `chunkIndex`, and no `sliceIndex` at all.
         */
        const { sliceIndex: chunkIndex, ...older } = rowFor({
          arm: 'narrow-b',
          sliceIndex: 1,
        },);
        await appendTrialRow({
          path,
          row: first,
        },);
        await appendFile(
          path,
          `${JSON.stringify({ ...older, chunkIndex, },)}\n`,
        );
        await appendTrialRow({
          path,
          row: last,
        },);
        expect(await readTrialLedger({ path, },),).toEqual([
          first,
          last,
        ],);
        expect(await accountTrialLedger({ path, },),).toEqual({
          rows: [
            first,
            last,
          ],
          leftOut: 1,
          tornTail: false,
        },);
      },
    },),
    it({
      name: 'ACCOUNTS FOR A TORN LAST LINE: the rows before it kept, nothing left out as no row, and the '
        + 'tear said, the file itself untouched by the read',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-trial-', },);
        const path = freshLedger({ dir: scratch.path, },);

        /**
         The one arm bought before the kill.
         */
        const first = rowFor({
          arm: 'narrow-a',
          sliceIndex: 1,
        },);

        /**
         Ledger as the kill left it.
         */
        const text = `${lineOf({ row: first, },)}${TORN_FRAGMENT}`;
        await writeLedger({
          path,
          text,
        },);
        expect(await accountTrialLedger({ path, },),).toEqual({
          rows: [first,],
          leftOut: 0,
          tornTail: true,
        },);
        expect(await readFile(
          path,
          'utf8',
        ),).toBe(text,);
      },
    },),
    it({
      name: 'READS BACK THREE ROWS after a torn last line and two appended rows, the first append having '
        + 'removed the fragment rather than joined a row onto it',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-trial-', },);
        const path = freshLedger({ dir: scratch.path, },);

        // The one arm bought before the kill, then the two a resumed run buys.
        await writeLedger({
          path,
          text: `${lineOf({ row: NARROW_A, },)}${TORN_FRAGMENT}`,
        },);
        await appendTrialRow({
          path,
          row: NARROW_B,
        },);
        await appendTrialRow({
          path,
          row: WIDE,
        },);
        expect(await readTrialLedger({ path, },),).toEqual([
          NARROW_A,
          NARROW_B,
          WIDE,
        ],);
        expect(await readFile(
          path,
          'utf8',
        ),).toBe([NARROW_A, NARROW_B, WIDE,].map(function toLine(row,) {
          return lineOf({ row, },);
        },)
          .join('',),);
      },
    },),
    it({
      name: 'READS BACK TWO ROWS after a torn last line and one appended row, so the arm a resumed run '
        + 'buys first is not lost and bought again',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-trial-', },);
        const path = freshLedger({ dir: scratch.path, },);

        // The one arm bought before the kill, then the one a resumed run buys.
        await writeLedger({
          path,
          text: `${lineOf({ row: NARROW_A, },)}${TORN_FRAGMENT}`,
        },);
        await appendTrialRow({
          path,
          row: NARROW_B,
        },);
        expect(await readTrialLedger({ path, },),).toEqual([
          NARROW_A,
          NARROW_B,
        ],);
      },
    },),
    it({
      name: 'KEEPS a whole last row that lost only its newline, ending its line before the row appended '
        + 'after it',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-trial-', },);
        const path = freshLedger({ dir: scratch.path, },);

        // A row whose line ends, a row whose newline the kill cut off, and the
        // row a resumed run appends.
        await writeLedger({
          path,
          text: `${lineOf({ row: NARROW_A, },)}${JSON.stringify(NARROW_B,)}`,
        },);
        await appendTrialRow({
          path,
          row: WIDE,
        },);
        expect(await readTrialLedger({ path, },),).toEqual([
          NARROW_A,
          NARROW_B,
          WIDE,
        ],);
      },
    },),
    it({
      name: 'APPENDS TO A LEDGER FILE THAT EXISTS AND IS EMPTY as to one never written, the row alone in it',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-trial-', },);
        const path = freshLedger({ dir: scratch.path, },);

        await writeLedger({
          path,
          text: '',
        },);
        await appendTrialRow({
          path,
          row: WIDE,
        },);
        expect(await readFile(
          path,
          'utf8',
        ),).toBe(lineOf({ row: WIDE, },),);
      },
    },),
    it({
      name: 'LEAVES A LEDGER THAT ENDS IN BLANK SPACE AFTER ITS LAST NEWLINE AS IT IS, appending the row after '
        + 'that space and reading both rows back',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-trial-', },);
        const path = freshLedger({ dir: scratch.path, },);

        await writeLedger({
          path,
          text: `${lineOf({ row: NARROW_A, },)}  `,
        },);
        await appendTrialRow({
          path,
          row: WIDE,
        },);
        expect(await readFile(
          path,
          'utf8',
        ),).toBe(`${lineOf({ row: NARROW_A, },)}  ${lineOf({ row: WIDE, },)}`,);
        expect(await readTrialLedger({ path, },),).toEqual([
          NARROW_A,
          WIDE,
        ],);
      },
    },),
    it({
      name: 'REFUSES a line that ends in a newline and does not parse, wherever it sits, naming the file '
        + 'and the line number and none of the line\'s text, since a kill mid-append leaves only an '
        + 'unterminated last line',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-trial-', },);

        /**
         Ledger whose first line is the fragment, a whole row after it.
         */
        const leading = freshLedger({ dir: scratch.path, },);
        await writeLedger({
          path: leading,
          text: `${TORN_FRAGMENT}\n${
            lineOf({
              row: rowFor({
                arm: 'wide',
                sliceIndex: 1,
              },),
            },)
          }`,
        },);

        /**
         Read that must not succeed.
         */
        const read = readTrialLedger({ path: leading, },);
        await expect(read,).rejects
          .toHaveProperty(
          'name',
          'TrialLedgerLineError',
          );
        await expect(read,).rejects
          .toHaveProperty(
          'message',
          unreadableLineMessage({
            path: leading,
            line: 1,
          },),
          );
        await expect(read,).rejects
          .toHaveProperty(
          'messageNamesOnly',
          true,
          );

        /**
         Ledger whose last line is the fragment with a newline after it, which
         no single cut-off append leaves.
         */
        const trailing = join(
          scratch.path,
          'nested',
          'ended.jsonl',
        );
        await writeLedger({
          path: trailing,
          text: `${
            lineOf({
              row: rowFor({
                arm: 'wide',
                sliceIndex: 1,
              },),
            },)
          }${TORN_FRAGMENT}\n`,
        },);
        await expect(readTrialLedger({ path: trailing, },),).rejects
          .toHaveProperty(
          'message',
          unreadableLineMessage({
            path: trailing,
            line: 2,
          },),
          );
      },
    },),
    it({
      name: 'skips only arms bought under THIS protocol, so a trial re-run after the rosters or '
        + 'the corpus pin moved buys fresh rather than mixing two experiments into one tally',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'window-trial-', },);
        const path = freshLedger({ dir: scratch.path, },);
        await appendTrialRow({
          path,
          row: rowFor({
            arm: 'wide',
            sliceIndex: 5,
          },),
        },);
        await appendTrialRow({
          path,
          row: rowFor({
            arm: 'wide',
            sliceIndex: 6,
            protocol: 'protocol-two',
          },),
        },);

        /**
         Arms the current run may skip.
         */
        const done = completedArms({
          rows: await readTrialLedger({ path, },),
          protocol: PROTOCOL,
        },);
        expect(done.size,).toBe(1,);

        /**
         Key of the arm this run bought.
         */
        const ownKey = trialKey({
          row: rowFor({
            arm: 'wide',
            sliceIndex: 5,
          },),
        },);

        /**
         Key of the arm an earlier protocol bought.
         */
        const otherKey = trialKey({
          row: rowFor({
            arm: 'wide',
            sliceIndex: 6,
            protocol: 'protocol-two',
          },),
        },);
        expect(done.has(ownKey,),).toBe(true,);
        // The other protocol's arm is not skippable, and was not deleted either.
        expect(done.has(otherKey,),).toBe(false,);
        expect((await readTrialLedger({ path, },)).length,).toBe(2,);
      },
    },),
    it({
      name: 'keys an arm by protocol, entry, slice AND arm together, so the two narrow runs of one '
        + 'slice are distinguishable: pooling them would erase the run-to-run band the whole '
        + 'comparison is read against',
      fn: async () => {
        expect(trialKey({
          row: rowFor({
            arm: 'narrow-a',
            sliceIndex: 2,
          },),
        },),).not
          .toBe(trialKey({
            row: rowFor({
              arm: 'narrow-b',
              sliceIndex: 2,
            },),
          },),);
      },
    },),
  ],
},);
