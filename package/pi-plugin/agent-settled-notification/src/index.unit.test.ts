/**
 Tests for the built agent-settled notification extension.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import agentSettledNotification, {
  NOTIFICATION_ARGUMENTS,
  NOTIFICATION_COMMAND,
  NOTIFICATION_TIMEOUT_MS,
  invokeNotification,
  notifyAgentSettled,
  registerAgentSettledNotification,
  type NotificationInvocation,
  type NotificationProcessInput,
} from '../dist/final/node/index.mjs';
import {
  createAgentSettledEvent,
  createExtensionContext,
  createFakePiHarness,
  getOnlyHandler,
} from './test-fixtures.ts';

await describe({
  name: '',
  children: [
    //region Extension registration

    describe({
      name: agentSettledNotification.name,
      children: [
        it({
          name: 'registers only the final agent-settled event',
          fn: async () => {
            /** Fake Pi registration harness. */
            const harness = createFakePiHarness();

            agentSettledNotification(harness.api,);

            expect([...harness.handlersByEvent.keys()],).toEqual(['agent_settled',],);
          },
        },),
      ],
    },),

    //endregion Extension registration

    //region Notification delivery

    describe({
      name: registerAgentSettledNotification.name,
      children: [
        it({
          name: 'delivers one static notification for completed and aborted settlements',
          fn: async () => {
            /** Fake Pi registration harness. */
            const harness = createFakePiHarness();
            /** Captured operating-system notification requests. */
            const invocations: NotificationInvocation[] = [];

            registerAgentSettledNotification({
              pi: harness.api,
              invoke: function captureNotification(
                invocation: NotificationInvocation,
              ): Promise<void> {
                invocations.push(invocation,);
                return Promise.resolve();
              },
            },);

            /** Settled-event callback registered by the extension. */
            const handler = getOnlyHandler({
              handlersByEvent: harness.handlersByEvent,
              event: 'agent_settled',
            },);
            await handler(
              createAgentSettledEvent({ aborted: false, },),
              createExtensionContext(),
            );
            await handler(
              createAgentSettledEvent({ aborted: true, },),
              createExtensionContext(),
            );

            expect(invocations,).toEqual([
              {
                command: NOTIFICATION_COMMAND,
                args: NOTIFICATION_ARGUMENTS,
              },
              {
                command: NOTIFICATION_COMMAND,
                args: NOTIFICATION_ARGUMENTS,
              },
            ],);
          },
        },),
      ],
    },),

    describe({
      name: invokeNotification.name,
      children: [
        it({
          name: 'uses bounded non-interactive subprocess options',
          fn: async () => {
            /** Captured child-process request. */
            const requests: NotificationProcessInput[] = [];

            await invokeNotification({
              invocation: {
                command: NOTIFICATION_COMMAND,
                args: NOTIFICATION_ARGUMENTS,
              },
              run: function captureProcess(
                input: NotificationProcessInput,
              ): Promise<void> {
                requests.push(input,);
                return Promise.resolve();
              },
            },);

            expect(requests,).toEqual([
              {
                command: NOTIFICATION_COMMAND,
                args: NOTIFICATION_ARGUMENTS,
                stdin: 'ignore',
                stdout: 'ignore',
                stderr: 'ignore',
                timeout: NOTIFICATION_TIMEOUT_MS,
              },
            ],);
          },
        },),
      ],
    },),

    describe({
      name: notifyAgentSettled.name,
      children: [
        it({
          name: 'returns delivery failure without throwing',
          fn: async () => {
            /** Delivery failure returned by an unavailable desktop-notification executable. */
            const unavailable = new Error('notify-send unavailable',);

            /** Settled-notification result after injected delivery failure. */
            const delivery = await notifyAgentSettled({
              invoke: async function rejectNotification(): Promise<void> {
                throw unavailable;
              },
            },);

            expect(delivery,).toEqual({
              delivered: false,
              error: unavailable,
            },);
          },
        },),
      ],
    },),

    //endregion Notification delivery
  ],
},);
