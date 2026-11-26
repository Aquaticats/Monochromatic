/**
 Tests for the notify-send sender: a notification the server accepted, one it refused,
 and a notify-send that never ran.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { Notification, } from './alert-message.ts';
import {
  NOTIFY_TIMEOUT_MS,
  notificationSender,
  notifySendArguments,
} from './alert-notify.ts';
import type {
  CommandRequest,
  CommandResult,
  CommandRunner,
} from './run-command.ts';

/**
 The notice every test in this file sends.
 */
const FAILURE_NOTICE: Notification = {
  kind: 'failure',
  summary: 'Garage sync failed',
  body: 'disk full',
};

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
  name: notificationSender.name,
  children: [
    it({
      name: 'reports a sent notification, and asks notify-send for this notice, when it exits 0',
      fn: async function sent() {
        /**
         Requests the sender made.
         */
        const requests: CommandRequest[] = [];
        /**
         The sender under test.
         */
        const sendNotification = notificationSender(fakeRunner({
          stdout: '',
          stderr: '',
          exitCode: 0,
        }, requests,),);
        /**
         What notify-send reported.
         */
        const outcome = await sendNotification(FAILURE_NOTICE,);
        expect(outcome,).toEqual({ sent: true, },);
        expect(requests,).toHaveLength(1,);
        expect(requests[0]?.command,).toBe('notify-send',);
        expect(requests[0]?.args,).toEqual(notifySendArguments(FAILURE_NOTICE,),);
        expect(requests[0]?.timeoutMs,).toBe(NOTIFY_TIMEOUT_MS,);
      },
    },),
    it({
      name: 'reports the refusal with its exit status and stderr when notify-send exits non-zero',
      fn: async function refused() {
        /**
         The sender under test, whose notify-send has no server to talk to.
         */
        const sendNotification = notificationSender(fakeRunner({
          stdout: '',
          stderr: 'no notification server\n',
          exitCode: 1,
        }, [],),);
        /**
         What notify-send reported.
         */
        const outcome = await sendNotification(FAILURE_NOTICE,);
        expect(outcome,).toEqual({
          sent: false,
          problem: 'notify-send exited 1: no notification server',
        },);
      },
    },),
    it({
      name: 'reports the runner problem unchanged when notify-send could not run',
      fn: async function couldNotRun() {
        /**
         The runner's own report, which the sender must pass on.
         */
        const problem = 'notify-send could not be run or was stopped after 30000 ms: Error: spawn notify-send ENOENT';
        /**
         The sender under test.
         */
        const sendNotification = notificationSender(fakeRunner({
          stdout: '',
          stderr: '',
          problem,
        }, [],),);
        /**
         What notify-send reported.
         */
        const outcome = await sendNotification(FAILURE_NOTICE,);
        expect(outcome,).toEqual({
          sent: false,
          problem,
        },);
      },
    },),
  ],
},);
