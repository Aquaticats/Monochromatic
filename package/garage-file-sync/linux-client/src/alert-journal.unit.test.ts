/**
 Tests for the run's journal read: the lines a readable journal gives, and the two ways
 the read reports why it could not use a journal.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  JOURNAL_TIMEOUT_MS,
  journalReader,
  readRunJournal,
} from './alert-journal.ts';
import type {
  CommandRequest,
  CommandResult,
  CommandRunner,
} from './run-command.ts';
import { SYNC_UNIT, } from './units.ts';

/**
 Escape character that starts a terminal colour sequence.
 */
const ESCAPE = '\u001B';

/**
 Invocation id systemd reports for the run an alert describes.
 */
const INVOCATION_ID = 'abcdef0123456789';

/**
 Journal text as journalctl prints a coloured run, with padding around one line.

 The read must hand every line back trimmed and without the colour sequences.
 */
const COLOURED_JOURNAL = `${ESCAPE}[0;32mBisync successful${ESCAPE}[0m\n  ERROR : disk full  \n`;

/**
 Builds a command runner that reports one result and records what it was asked to run.

 @param result - what the command reports

 @param requests - list the requests are appended to

 @returns the runner
 */
function fakeRunner(result: CommandResult, requests: CommandRequest[],): CommandRunner {
  return async function report(command,) {
    requests.push(command,);
    return result;
  };
}

await describe({
  name: readRunJournal.name,
  children: [
    it({
      name: 'returns the lines of a readable journal, each trimmed and without colour',
      fn: async function readableJournal() {
        /**
         The reader under test, over a journalctl that printed a coloured run.
         */
        const readThroughRunner = journalReader(fakeRunner({
          stdout: COLOURED_JOURNAL,
          stderr: '',
          exitCode: 0,
        }, [],),);
        /**
         What the read produced.
         */
        const journal = await readThroughRunner({
          unit: SYNC_UNIT,
          invocationId: INVOCATION_ID,
        },);
        expect(journal,).toEqual({
          lines: [
            'Bisync successful',
            'ERROR : disk full',
            '',
          ],
        },);
      },
    },),
    it({
      name: 'asks journalctl for the invocation the request names, of the unit it names',
      fn: async function readsTheNamedInvocation() {
        /**
         Requests the reader made.
         */
        const requests: CommandRequest[] = [];
        /**
         The reader under test.
         */
        const readThroughRunner = journalReader(fakeRunner({
          stdout: '',
          stderr: '',
          exitCode: 0,
        }, requests,),);
        await readThroughRunner({
          unit: SYNC_UNIT,
          invocationId: INVOCATION_ID,
        },);
        expect(requests,).toHaveLength(1,);
        expect(requests[0]?.command,).toBe('journalctl',);
        expect(requests[0]?.args,).toContain(`--invocation=${INVOCATION_ID}`,);
        expect(requests[0]?.args,).toContain(`--unit=${SYNC_UNIT}`,);
        expect(requests[0]?.timeoutMs,).toBe(JOURNAL_TIMEOUT_MS,);
      },
    },),
    it({
      name: 'reports the exit status and stderr of a journalctl that refused the read, and no lines',
      fn: async function refusedRead() {
        /**
         The reader under test, over a journalctl that has no such invocation.
         */
        const readThroughRunner = journalReader(fakeRunner({
          stdout: '',
          stderr: 'Failed to find invocation\n',
          exitCode: 1,
        }, [],),);
        /**
         What the read produced.
         */
        const journal = await readThroughRunner({
          unit: SYNC_UNIT,
          invocationId: INVOCATION_ID,
        },);
        expect(journal,).toEqual({
          lines: [],
          problem: 'journalctl exited 1: Failed to find invocation',
        },);
      },
    },),
    it({
      name: 'reports the runner problem and no lines when journalctl could not run',
      fn: async function couldNotRun() {
        /**
         The runner's own report, which the read must pass on.
         */
        const problem = 'journalctl could not be run or was stopped after 30000 ms: Error: spawn journalctl ENOENT';
        /**
         The reader under test.
         */
        const readThroughRunner = journalReader(fakeRunner({
          stdout: '',
          stderr: '',
          problem,
        }, [],),);
        /**
         What the read produced.
         */
        const journal = await readThroughRunner({
          unit: SYNC_UNIT,
          invocationId: INVOCATION_ID,
        },);
        expect(journal,).toEqual({
          lines: [],
          problem,
        },);
      },
    },),
  ],
},);
