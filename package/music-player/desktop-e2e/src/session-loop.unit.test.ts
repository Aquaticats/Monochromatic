/**
 Tests for the session loop of a run: which sessions run, what a session that stops between checks does to the rest of
 the run, and what ends the run at once.
 
 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CheckFailureError,
  SafetyStopError,
} from './errors.ts';
import {
  runChosenSessions,
  type SessionRunner,
  type SessionRunners,
} from './run.ts';
import type { SessionName, } from './selection.ts';

/**
 One recorded failure of the loop.
 */
type Recorded = {
  readonly detail: string;
  readonly id: string;
  readonly title: string;
};

/**
 Builds runners that record the session they ran and then do what each session is told to do.
 
 @param behaviour - what each named session does when it runs; a session that is not named has no runner
 
 @returns the runners and the names of the sessions that ran, in order
 
 @example
 ```ts
 const { runners, ran } = recordingRunners(new Map([['wide', function abort() { throw new CheckFailureError('x',); },],]),);
 ```
 */
function recordingRunners(
  behaviour: ReadonlyMap<SessionName, (context: unknown,) => void>,
): {
  readonly ran: SessionName[];
  readonly runners: SessionRunners<unknown>;
} {
  const ran: SessionName[] = [];
  const runners = new Map<SessionName, SessionRunner<unknown>>();
  for (const [name, action,] of behaviour) {
    runners.set(
      name,
      async function run(context: unknown,): Promise<void> {
        ran.push(name,);
        action(context,);
      },
    );
  }
  return {
    ran,
    runners,
  };
}

await describe({
  name: runChosenSessions.name,
  children: [
    it({
      name: 'records a session that stops between its checks as a failure and runs the next session, so one aborted session hides nothing',
      fn: async ({ expect, },) => {
        const failures: Recorded[] = [];
        const { ran, runners } = recordingRunners(new Map([
          ['wide', function abort() {
            throw new CheckFailureError('timed out waiting for the folder of thirty tracks',);
          },],
          ['narrow', function pass() {
            /* the later session runs */
          },],
        ],),);
        await runChosenSessions({
          context: {},
          names: [
            'wide',
            'narrow',
          ],
          recorder: {
            fail: async function record(failure,) {
              failures.push(failure,);
            },
          },
          runners,
        },);
        expect(ran,).toStrictEqual([
          'wide',
          'narrow',
        ],);
        expect(failures,).toStrictEqual([
          {
            detail: 'timed out waiting for the folder of thirty tracks',
            id: 'session/wide',
            title: 'the wide session ran to its end',
          },
        ],);
      },
    },),
    it({
      name: 'ends the run at once on a safety stop, records nothing and leaves every later session unrun',
      fn: async ({ expect, },) => {
        const failures: Recorded[] = [];
        const { ran, runners } = recordingRunners(new Map([
          ['wide', function unsafe() {
            throw new SafetyStopError('SAFETY STOP: the app stream links elsewhere',);
          },],
          ['narrow', function pass() {
            /* the later session never runs */
          },],
        ],),);
        let caught: unknown;
        try {
          await runChosenSessions({
            context: {},
            names: [
              'wide',
              'narrow',
            ],
            recorder: {
              fail: async function record(failure,) {
                failures.push(failure,);
              },
            },
            runners,
          },);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(SafetyStopError,);
        expect(ran,).toStrictEqual(['wide',],);
        expect(failures,).toStrictEqual([],);
      },
    },),
    it({
      name: 'fails naming the session when the filter selects one this map has no runner for, instead of calling nothing',
      fn: async ({ expect, },) => {
        const { ran, runners } = recordingRunners(new Map([
          ['wide', function pass() {
            /* the only runner this map has */
          },],
        ],),);
        let caught: unknown;
        let recorded = 0;
        try {
          await runChosenSessions({
            context: {},
            names: [
              'wide',
              'states',
            ],
            recorder: {
              fail: async function record() {
                recorded += 1;
              },
            },
            runners,
          },);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(CheckFailureError,);
        expect((caught as Error).message,).toContain('states',);
        expect(ran,).toStrictEqual(['wide',],);
        expect(recorded,).toBe(0,);
      },
    },),
    it({
      name: 'runs only the sessions the filter selected, in the order the filter gives',
      fn: async ({ expect, },) => {
        const { ran, runners } = recordingRunners(new Map([
          ['wide', function pass() {
            /* selected */
          },],
          ['quit', function pass() {
            /* selected */
          },],
        ],),);
        await runChosenSessions({
          context: {},
          names: [
            'quit',
            'wide',
          ],
          recorder: {
            fail: async function record() {
              /* nothing fails */
            },
          },
          runners,
        },);
        expect(ran,).toStrictEqual([
          'quit',
          'wide',
        ],);
      },
    },),
  ],
},);
