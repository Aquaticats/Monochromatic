/**
 The line a start that ended without a run leaves in the run's own folder, and the words it puts in the summary and in what the
 launcher prints.

 Every case here is a fact a reader needs and a driver of 2026-11-26 did not write: issue 6556's four starts that ended in 32
 seconds with no summary and no standard error, and its three others, whose summary named an ending but never the wait, its bound,
 how long it had waited, or what to do next.

 @module
 */

import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  START_WITHOUT_A_RUN_FILE_NAME,
  type StartThatEndedWithoutARun,
  lineOfAStartThatEndedWithoutARun,
  linesOfAStartThatEndedWithoutARun,
  sentenceOfAStartThatEndedWithoutARun,
  writeAStartThatEndedWithoutARun,
} from '../dist/final/node/index.mjs';

/**
 A start whose wait had a bound of its own, which is what the four silent starts of 2026-11-26 met: the sops decrypt of the
 checkout's store is given 15 000 ms by the reader this package asks for it (`SOPS_TIMEOUT_MS` in
 `package/env-local-json/core/src/constants.ts`).
 */
const BOUNDED_START: StartThatEndedWithoutARun = {
  waits: [{
    wait: 'the sops decrypt of the checkout\'s store, which the reader this package asks for the store runs',
    bound: '15000 ms',
    waitedMilliseconds: 15_004,
  },],
  what: 'nothing was read from the store, so this machine\'s secret values could not be masked and no Pi was started',
  next: 'start the run again when the host is quieter',
};

/**
 A start whose wait has no bound of its own, because Pi answered and what it answered ended the start.
 */
const UNBOUNDED_START: StartThatEndedWithoutARun = {
  waits: [{
    wait: 'the command list Pi answers with, which says whether the extension masking secret values is loaded',
    waitedMilliseconds: 1_204,
  },],
  what: 'Pi answered with no command that arms the masking',
  next: 'read the run\'s stderr.log for the extension that did not load',
};

/**
 A start two waits ended: Pi left before the prompt, and then the time one start-up may take refused another start.
 */
const TWO_WAITS_START: StartThatEndedWithoutARun = {
  waits: [
    {
      wait: 'the start of Pi, which left before it took the prompt',
      waitedMilliseconds: 168_385,
    },
    {
      wait: 'the time one start-up may take, after which Pi is not started again',
      bound: '120000 ms',
      waitedMilliseconds: 169_654,
    },
  ],
  what: 'Pi exited with status 1 before it took the prompt, and no start followed',
  next: 'read what Pi printed to its standard error, which names why it left',
};

await describe({
  name: '',
  children: [
    describe({
      name: lineOfAStartThatEndedWithoutARun.name,
      children: [
        it({
          name: 'names the wait, its bound, how long it had waited, and what to do next',
          fn: async () => {
            const line = lineOfAStartThatEndedWithoutARun({
              runFolder: '/runs/a-1',
              start: BOUNDED_START,
            },);
            // One line, because a launcher and a batch print lines: a break here would read as two records.
            expect(line,).not.toContain('\n',);
            // The folder, so a reader of several lines knows which run this is.
            expect(line,).toContain('/runs/a-1',);
            // The wait, in words a reader can follow to the call that ran it.
            expect(line,).toContain('sops decrypt',);
            // Its bound and how long it had waited, both in milliseconds.
            expect(line,).toContain('15000 ms',);
            expect(line,).toContain('15004 ms',);
            // Why no run happened, and what to do next.
            expect(line,).toContain('could not be masked',);
            expect(line,).toContain('start the run again when the host is quieter',);
          },
        },),
        it({
          name: 'says a wait that has no bound of its own has none, rather than leaving it out',
          fn: async () => {
            const line = lineOfAStartThatEndedWithoutARun({
              runFolder: '/runs/a-1',
              start: UNBOUNDED_START,
            },);
            expect(line,).toContain('no bound of its own',);
            expect(line,).toContain('1204 ms',);
          },
        },),
        it({
          name: 'names both waits in the order they ended, for a start two of them ended',
          fn: async () => {
            const line = lineOfAStartThatEndedWithoutARun({
              runFolder: '/runs/a-1',
              start: TWO_WAITS_START,
            },);
            const promptWait = line.indexOf('left before it took the prompt',);
            const sequenceWait = line.indexOf('time one start-up may take',);
            expect(promptWait,).toBeGreaterThan(-1,);
            expect(sequenceWait,).toBeGreaterThan(promptWait,);
            expect(line,).toContain('120000 ms',);
            expect(line,).toContain('169654 ms',);
          },
        },),
      ],
    },),

    describe({
      name: sentenceOfAStartThatEndedWithoutARun.name,
      children: [
        it({
          name: 'drops the folder and the ending, so a line that already says both can carry it',
          fn: async () => {
            const sentence = sentenceOfAStartThatEndedWithoutARun({ start: BOUNDED_START, },);
            expect(sentence,).not.toContain('/runs/a-1',);
            expect(sentence,).not.toContain('ended without a run',);
            expect(sentence,).toContain('15000 ms',);
            expect(sentence,).toContain('start the run again when the host is quieter',);
          },
        },),
      ],
    },),

    describe({
      name: writeAStartThatEndedWithoutARun.name,
      children: [
        it({
          name: 'writes the line under its own name in the run folder, and answers the same line',
          fn: async () => {
            /**
             A run folder of this test's own, which no other test reads.
             */
            const folder = await mkdtemp(join(scratchRoot(), 'start-without-a-run-',),);
            try {
              await mkdir(folder, { recursive: true, },);
              const written = await writeAStartThatEndedWithoutARun({
                runFolder: folder,
                start: BOUNDED_START,
              },);
              const kept = await readFile(
                join(
                  folder,
                  START_WITHOUT_A_RUN_FILE_NAME,
                ),
                'utf8',
              );
              expect(kept,).toBe(`${written}\n`,);
              expect(written,).toContain('sops decrypt',);
            }
            finally {
              await rm(
                folder,
                { recursive: true, force: true, },
              );
            }
          },
        },),
        it({
          name: 'writes nothing over a line an earlier write of the same folder left',
          fn: async () => {
            const folder = await mkdtemp(join(scratchRoot(), 'start-without-a-run-',),);
            try {
              const file = join(
                folder,
                START_WITHOUT_A_RUN_FILE_NAME,
              );
              await writeFile(file, 'a line an earlier attempt left\n',);
              /**
               What the second write answered.
               */
              const written = await writeAStartThatEndedWithoutARun({
                runFolder: folder,
                start: UNBOUNDED_START,
              },);
              // The file the earlier attempt left is not replaced, and the answer says so rather than claiming a write.
              expect(await readFile(file, 'utf8',),).toBe('a line an earlier attempt left\n',);
              expect(written,).toContain('not written',);
            }
            finally {
              await rm(
                folder,
                { recursive: true, force: true, },
              );
            }
          },
        },),
      ],
    },),

    describe({
      name: linesOfAStartThatEndedWithoutARun.name,
      children: [
        it({
          name: 'answers the line a start left, for the launcher to print',
          fn: async () => {
            const folder = await mkdtemp(join(scratchRoot(), 'start-without-a-run-',),);
            try {
              const written = await writeAStartThatEndedWithoutARun({
                runFolder: folder,
                start: BOUNDED_START,
              },);
              expect(await linesOfAStartThatEndedWithoutARun({ runFolder: folder, },),).toEqual([written,],);
            }
            finally {
              await rm(
                folder,
                { recursive: true, force: true, },
              );
            }
          },
        },),
        it({
          name: 'answers nothing for a folder a start that ended with a run left alone',
          fn: async () => {
            const folder = await mkdtemp(join(scratchRoot(), 'start-without-a-run-',),);
            try {
              expect(await linesOfAStartThatEndedWithoutARun({ runFolder: folder, },),).toEqual([],);
            }
            finally {
              await rm(
                folder,
                { recursive: true, force: true, },
              );
            }
          },
        },),
      ],
    },),
  ],
},);
