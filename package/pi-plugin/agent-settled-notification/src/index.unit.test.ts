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
  NotificationUnavailableError,
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
          name: 'absorbs delivery failure so settlement continues',
          fn: async () => {
            /** Fake Pi registration harness. */
            const harness = createFakePiHarness();
            /** Recording logger standing in for the module root logger. */
            const recording = createRecordingLogger();

            registerAgentSettledNotification({
              pi: harness.api,
              l: recording.logger,
              invoke: async function rejectNotification(): Promise<void> {
                throw new Error('notify-send unavailable',);
              },
            },);

            /** Settled-event callback registered by the extension. */
            const handler = getOnlyHandler({
              handlersByEvent: harness.handlersByEvent,
              event: 'agent_settled',
            },);
            // Awaiting a rejection here fails this test, which is the claim: a
            // desktop failure must not escape Pi's lifecycle hook.
            await handler(
              createAgentSettledEvent({ aborted: false, },),
              createExtensionContext(),
            );
            expect(recording.calls.warn,).toHaveLength(1,);
          },
        },),

        it({
          name: 'surfaces a boundary failure this package cannot describe',
          fn: async () => {
            /** Fake Pi registration harness. */
            const harness = createFakePiHarness();
            /** Recording logger standing in for the module root logger. */
            const recording = createRecordingLogger();
            /**
             Inject boundary failure whose own text conversion throws, so building the
             desktop-refusal error fails instead of the delivery it describes.
             */
            const unreadable = {
              toString: function throwOnText(): string {
                throw new Error('unreadable boundary failure',);
              },
            };

            registerAgentSettledNotification({
              pi: harness.api,
              l: recording.logger,
              invoke: async function rejectNotification(): Promise<void> {
                throw unreadable;
              },
            },);

            /** Settled-event callback registered by the extension. */
            const handler = getOnlyHandler({
              handlersByEvent: harness.handlersByEvent,
              event: 'agent_settled',
            },);
            // A defect this package cannot turn into a desktop refusal must stay
            // visible rather than read as an expected desktop outage.
            await expect(handler(
              createAgentSettledEvent({ aborted: false, },),
              createExtensionContext(),
            ),).rejects.toThrow('unreadable boundary failure',);
            expect(recording.calls.warn,).toEqual([],);
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
            expect(recording.calls.debug[0],).toContain(
              'settled-agent desktop notification remains unavailable',
            );
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
          name: 'resolves after successful delivery',
          fn: async () => {
            await notifyAgentSettled({
              invoke: function acceptNotification(): Promise<void> {
                return Promise.resolve();
              },
            },);
          },
        },),

        it({
          name: 'throws NotificationUnavailableError carrying the desktop failure',
          fn: async () => {
            /** Delivery failure raised by an unavailable desktop-notification executable. */
            const unavailable = new Error('notify-send unavailable',);

            /** Settled-notification attempt refused by the injected desktop boundary. */
            const delivery = notifyAgentSettled({
              invoke: async function rejectNotification(): Promise<void> {
                throw unavailable;
              },
            },);

            await expect(delivery,).rejects.toBeInstanceOf(NotificationUnavailableError,);
            await expect(delivery,).rejects.toHaveProperty('cause', unavailable,);
            await expect(delivery,).rejects.toSatisfy(
              function carriesBoundaryFailure(refusal: unknown,): boolean {
                if (!(refusal instanceof NotificationUnavailableError))
                  return false;
                return refusal.message.includes(unavailable.message,)
                  && refusal.message.includes(NOTIFICATION_COMMAND,);
              },
            );
          },
        },),
      ],
    },),

    //endregion Notification delivery
  ],
},);
