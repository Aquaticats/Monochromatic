/**
 What one answer of the run's own `process` tool says it holds: the processes it names, one or many, and nothing for an answer that
 names none a reader can follow.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  PROCESSES_RECORD_KIND,
  processesInToolAnswer,
  processesNamedIn,
} from '../dist/final/node/index.mjs';

/**
 One process, as the tool reports it.
 */
const A_PROCESS = {
  id: 'proc_7adb',
  name: 'native-build',
  pid: 486_667,
  command: 'cargo build',
  cwd: '/repo/.claude/worktrees/pi-a',
  status: 'running',
} as const;

/**
 Another process, as a list answer reports it.
 */
const ANOTHER_PROCESS = {
  id: 'proc_8bce',
  name: 'clippy',
  pid: 486_668,
  command: 'cargo clippy',
  cwd: '/repo/.claude/worktrees/pi-a',
  status: 'exited',
} as const;

await describe({
  name: '',
  children: [
    describe({
      name: processesInToolAnswer.name,
      children: [
        it({
          name: 'reads the one process a start or an update answers with, under the tool answer details',
          fn: async () => {
            expect(processesInToolAnswer({ answer: { content: [], details: { action: 'start', process: A_PROCESS, }, }, },),)
              .toEqual([A_PROCESS,],);
          },
        },),
        it({
          name: 'reads every process a list answers with, and leaves out an entry the tool named no process in',
          fn: async () => {
            expect(
              processesInToolAnswer({
                answer: {
                  details: {
                    action: 'list',
                    processes: [A_PROCESS, ANOTHER_PROCESS, { name: 'no id at all', },],
                  },
                },
              },),
            ).toEqual([A_PROCESS, ANOTHER_PROCESS,],);
          },
        },),
        it({
          name: 'reads no process from an answer that names none, or that is not an answer at all',
          fn: async () => {
            expect(processesInToolAnswer({ answer: { content: [{ type: 'text', text: 'no process here', },], }, },),).toEqual([],);
            expect(processesInToolAnswer({ answer: 'a string, not an answer', },),).toEqual([],);
            expect(processesInToolAnswer({ answer: undefined, },),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: processesNamedIn.name,
      children: [
        it({
          name: 'reads the shape the feed keeps a whole tool answer in, which is the list of processes itself',
          fn: async () => {
            const record = {
              kind: PROCESSES_RECORD_KIND,
              processes: [A_PROCESS, ANOTHER_PROCESS,],
            };
            expect(processesNamedIn({ value: record, },),).toEqual([A_PROCESS, ANOTHER_PROCESS,],);
          },
        },),
        it({
          name: 'reads no process from a value that is not an object, and fills in what a process record left out',
          fn: async () => {
            expect(processesNamedIn({ value: undefined, },),).toEqual([],);
            expect(processesNamedIn({ value: [A_PROCESS,], },),).toEqual([],);
            // A tool answer that names a process and nothing about it: the name is the tool's own, and the rest is empty rather
            // than guessed at.
            expect(processesNamedIn({ value: { process: { id: 'proc_1', }, }, },),).toEqual([{
              id: 'proc_1',
              name: 'proc_1',
              command: '',
              cwd: '',
            },],);
          },
        },),
      ],
    },),
  ],
},);
