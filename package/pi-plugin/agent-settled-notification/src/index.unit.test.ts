/**
 Tests for the built agent-settled notification extension.
 
 @module
 */

import type { ExtensionAPI, } from '@earendil-works/pi-coding-agent';
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

//region Test harness

/** Minimal event handler shape needed to drive the registered Pi lifecycle callback. */
type EventHandler = (
  event: unknown,
  context: unknown,
) => unknown;

/** Fake Pi API and captured lifecycle registrations. */
type FakePiHarness = {
  /** API supplied to the extension under test. */
  readonly api: ExtensionAPI;

  /** Handlers grouped by lifecycle event name. */
  readonly handlersByEvent: ReadonlyMap<string, readonly EventHandler[]>;
};

/**
 Creates a fake Pi extension API that records lifecycle subscriptions.
 
 @returns fake API and captured event handlers
 
 @example
 ```ts
 const harness = createFakePiHarness();
 ```
 */
function createFakePiHarness(): FakePiHarness {
  /** Mutable registrations populated through the fake `on()` method. */
  const handlersByEvent = new Map<string, EventHandler[]>();

  /** Fake Pi API implementing only the event-registration surface under test. */
  const api = {
    on(
      event: string,
      handler: EventHandler,
    ): void {
      /** Existing handlers registered for this lifecycle event. */
      const existingHandlers = handlersByEvent.get(event,)
        ?? [];
      handlersByEvent.set(
        event,
        [
          ...existingHandlers,
          handler,
        ],
      );
    },
  } as unknown as ExtensionAPI;

  return {
    api,
    handlersByEvent,
  };
}

/**
 Retrieves the only handler registered for an event.
 
 @param handlersByEvent - captured Pi registrations
 
 @param event - lifecycle event whose callback is needed
 
 @returns registered lifecycle callback
 
 @throws when registration is absent or duplicated
 
 @example
 ```ts
 getOnlyHandler({ handlersByEvent: new Map(), event: 'agent_settled' });
 ```
 */
function getOnlyHandler(
  {
    handlersByEvent,
    event,
  }: {
    readonly handlersByEvent: ReadonlyMap<string, readonly EventHandler[]>;
    readonly event: string;
  },
): EventHandler {
  /** Captured handlers for the requested event. */
  const handlers = handlersByEvent.get(event,);
  if (handlers === undefined)
    throw new Error(`Missing handler registration: ${event}`,);
  if (handlers.length !== 1)
    throw new Error(`Expected one handler for ${event}, received ${handlers.length}`,);

  /** Sole handler registered for the requested lifecycle event. */
  const [handler,] = handlers;
  if (handler === undefined)
    throw new Error(`Missing handler registration: ${event}`,);
  return handler;
}

//endregion Test harness

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
          name: 'delivers one static notification for each settled event',
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
            await handler({}, {},);
            await handler({}, {},);

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
