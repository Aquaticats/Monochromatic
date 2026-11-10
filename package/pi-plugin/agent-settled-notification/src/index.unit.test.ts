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
  createRecordingLogger,
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

        it({
          name: 'logs delivered settlements at debug without warning',
          fn: async () => {
            /** Fake Pi registration harness. */
            const harness = createFakePiHarness();
            /** Recording logger standing in for the module root logger. */
            const recording = createRecordingLogger();

            registerAgentSettledNotification({
              pi: harness.api,
              l: recording.logger,
              invoke: function acceptNotification(): Promise<void> {
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

            expect(recording.calls.warn,).toEqual([],);
            expect(recording.calls.debug,).toHaveLength(1,);
            expect(recording.calls.debug[0],).toContain('sent settled-agent desktop notification',);
          },
        },),

        it({
          name: 'warns once about unavailable delivery and logs repeats at debug',
          fn: async () => {
            /** Fake Pi registration harness. */
            const harness = createFakePiHarness();
            /** Recording logger standing in for the module root logger. */
            const recording = createRecordingLogger();
            /** Delivery failure returned by an unavailable desktop-notification executable. */
            const unavailable = new Error('notify-send unavailable',);

            registerAgentSettledNotification({
              pi: harness.api,
              l: recording.logger,
              invoke: async function rejectNotification(): Promise<void> {
                throw unavailable;
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
              createAgentSettledEvent({ aborted: false, },),
              createExtensionContext(),
            );

            expect(recording.calls.warn,).toHaveLength(1,);
            expect(recording.calls.warn[0],).toContain('settled-agent desktop notification unavailable',);
            expect(recording.calls.warn[0],).toContain(unavailable.message,);
            expect(recording.calls.debug,).toHaveLength(1,);
            expect(recording.calls.debug[0],).toContain('settled-agent desktop notification remains unavailable',);
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
