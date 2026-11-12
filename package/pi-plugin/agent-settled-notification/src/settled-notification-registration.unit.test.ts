/**
 Tests for the settled-agent notification registration.

 The delivery boundary is injected everywhere, so no case starts a process or
 reaches a desktop, and every case is about what Pi's settlement hook does and
 does not wait for.
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import agentSettledNotification, {
  NOTIFICATION_COMMAND,
  NOTIFICATION_TITLE,
  NotificationSpawnError,
  NotificationUnavailableError,
  registerAgentSettledNotification,
  type NotificationInvocation,
} from '../dist/final/node/index.mjs';
import {
  createAgentSettledEvent,
  createExtensionContext,
  createFakePiHarness,
  createRecordingLogger,
  getOnlyHandler,
  type AgentSettledHandler,
} from './test-fixtures.ts';

//region Fixtures

/**
 Working folder the fixture context reports for the settled agent.
 */
const AGENT_FOLDER = '/var/mnt/encrypted/repository/example/.claude/worktrees/agent-7';

/**
 Boundary failure whose diagnostic text cannot be read.

 The extension builds its log line from the failure it caught, so a failure that
 refuses to describe itself breaks that construction rather than the delivery,
 which is the state this suite needs to reach.
 */
class UnreadableBoundaryFailureError extends Error {
  /**
   Stable error classification independent of minification.
   */
  override readonly name = 'UnreadableBoundaryFailureError';

  /**
   Refuses the diagnostic text every log line reads when describing a failure.

   @throws Error naming the refusal, which is what reaches the registration
   */
  override get message(): string {
    throw new Error('unreadable boundary failure',);
  }
}

/**
 Registration plus the pieces a case drives it with.
 */
type RegistrationHarness = {
  /**
   Sole settlement handler the registration installed in the fake Pi API.
   */
  readonly handler: AgentSettledHandler;

  /**
   Tracker the registration returned.
   */
  readonly registration: ReturnType<typeof registerAgentSettledNotification>;

  /**
   Logger handed to the registration.
   */
  readonly recording: ReturnType<typeof createRecordingLogger>;
};

/**
 Registers the extension against an injected delivery and captures what it logged.

 @param deliver - delivery boundary the registration starts for one settlement

 @returns installed handler, registration tracker and recorded log calls

 @example
 ```ts
 const harness = registrationWith(async () => {});
 await harness.handler(createAgentSettledEvent({ aborted: false }), createExtensionContext());
 ```
 */
function registrationWith(
  deliver: (parts: { readonly invocation: NotificationInvocation },) => Promise<void>,
): RegistrationHarness {
  /**
   Fake Pi API receiving the lifecycle subscription.
   */
  const pi = createFakePiHarness();
  /**
   Recording logger standing in for the module root logger.
   */
  const recording = createRecordingLogger();
  const registration = registerAgentSettledNotification({
    pi: pi.api,
    l: recording.logger,
    deliver,
  },);
  return {
    recording,
    registration,
    handler: getOnlyHandler({
      handlersByEvent: pi.handlersByEvent,
      event: 'agent_settled',
    },),
  };
}

//endregion Fixtures

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

            expect([...harness.handlersByEvent.keys(),],).toEqual(['agent_settled',],);
          },
        },),
      ],
    },),

    //endregion Extension registration

    //region Settlement path

    describe({
      name: registerAgentSettledNotification.name,
      children: [
        it({
          name: 'returns from the settlement hook while the delivery is still running',
          fn: async () => {
            /**
             Releases the delivery this case holds open.
             */
            const release = Promise.withResolvers<void>();
            /** Whether the delivery already finished. */
            let delivered = false;
            const harness = registrationWith(async function holdDelivery(): Promise<void> {
              await release.promise;
              delivered = true;
            },);

            /**
             Value the settlement hook returned to Pi.
             */
            const hookResult = harness.handler(
              createAgentSettledEvent({ aborted: false, },),
              createExtensionContext(),
            );

            // A hook that returned a promise would make Pi wait for the desktop;
            // undefined is the whole point of starting the delivery instead.
            expect(hookResult,).toBeUndefined();
            expect(delivered,).toBe(false,);
            release.resolve();
            await harness.registration.settled();
            expect(delivered,).toBe(true,);
          },
        },),

        it({
          name: 'names the settled agent working folder in the notification',
          fn: async () => {
            /** Invocations the injected delivery boundary received. */
            const invocations: NotificationInvocation[] = [];
            const harness = registrationWith(async function captureNotification(
              parts: { readonly invocation: NotificationInvocation },
            ): Promise<void> {
              invocations.push(parts.invocation,);
            },);

            await harness.handler(
              createAgentSettledEvent({ aborted: false, },),
              createExtensionContext({ cwd: AGENT_FOLDER, },),
            );
            await harness.registration.settled();

            expect(invocations,).toHaveLength(1,);
            expect(invocations[0]?.command,).toBe(NOTIFICATION_COMMAND,);
            expect(invocations[0]?.args.at(1,),).toBe(NOTIFICATION_TITLE,);
            expect(invocations[0]?.args.at(-1,),).toContain(AGENT_FOLDER,);
          },
        },),

        it({
          name: 'delivers one notification per settlement whether or not it was aborted',
          fn: async () => {
            /** Invocations the injected delivery boundary received. */
            const invocations: NotificationInvocation[] = [];
            const harness = registrationWith(async function captureNotification(
              parts: { readonly invocation: NotificationInvocation },
            ): Promise<void> {
              invocations.push(parts.invocation,);
            },);

            await harness.handler(
              createAgentSettledEvent({ aborted: false, },),
              createExtensionContext(),
            );
            await harness.handler(
              createAgentSettledEvent({ aborted: true, },),
              createExtensionContext(),
            );
            await harness.registration.settled();

            expect(invocations,).toHaveLength(2,);
            expect(invocations[1]?.args,).toEqual(invocations[0]?.args,);
          },
        },),

        it({
          name: 'logs delivered settlements at debug without warning',
          fn: async () => {
            const harness = registrationWith(async function acceptNotification(): Promise<void> {},);

            await harness.handler(
              createAgentSettledEvent({ aborted: false, },),
              createExtensionContext(),
            );
            await harness.registration.settled();

            expect(harness.recording.calls.warn,).toEqual([],);
            expect(harness.recording.calls.error,).toEqual([],);
            expect(harness.recording.calls.debug,).toHaveLength(1,);
            expect(harness.recording.calls.debug[0],).toContain('sent settled-agent desktop notification',);
          },
        },),

        //endregion Settlement path

        //region Delivery failures

        it({
          name: 'absorbs a delivery failure so settlement continues',
          fn: async () => {
            const harness = registrationWith(async function rejectNotification(): Promise<void> {
              throw new NotificationUnavailableError({
                command: NOTIFICATION_COMMAND,
                attempts: 3,
                reason: 'exited with code 1; stderr: Created too many similar notifications in quick succession',
              },);
            },);

            // Awaiting a rejection here fails this case, which is the claim: a
            // desktop refusal must not escape Pi's settlement hook.
            await harness.handler(
              createAgentSettledEvent({ aborted: false, },),
              createExtensionContext(),
            );
            await harness.registration.settled();

            expect(harness.recording.calls.warn,).toHaveLength(1,);
            expect(harness.recording.calls.warn[0],).toContain(
              'Created too many similar notifications in quick succession',
            );
          },
        },),

        it({
          name: 'logs a start failure whose reason is not a desktop refusal',
          fn: async () => {
            const harness = registrationWith(async function failToStart(): Promise<void> {
              throw new NotificationSpawnError({
                command: NOTIFICATION_COMMAND,
                cause: new Error('spawn notify-send ENOENT',),
              },);
            },);

            await harness.handler(
              createAgentSettledEvent({ aborted: false, },),
              createExtensionContext(),
            );
            await harness.registration.settled();

            expect(harness.recording.calls.warn,).toEqual([],);
            expect(harness.recording.calls.error,).toHaveLength(1,);
            expect(harness.recording.calls.error[0],).toContain(NOTIFICATION_COMMAND,);
          },
        },),

        it({
          name: 'reports a failure this package cannot describe without escaping the hook',
          fn: async () => {
            /**
             Delivery failure whose own diagnostic text throws.
             */
            const unreadable = new UnreadableBoundaryFailureError();
            const harness = registrationWith(async function rejectNotification(): Promise<void> {
              throw unreadable;
            },);

            await harness.handler(
              createAgentSettledEvent({ aborted: false, },),
              createExtensionContext(),
            );
            await harness.registration.settled();

            expect(harness.recording.calls.error,).toHaveLength(1,);
            expect(harness.recording.calls.error[0],).toContain('unreadable boundary failure',);
          },
        },),

        it({
          name: 'warns once about unavailable delivery and logs repeats at debug',
          fn: async () => {
            /** Delivery refusal returned by an unavailable desktop-notification boundary. */
            const unavailable = new NotificationUnavailableError({
              command: NOTIFICATION_COMMAND,
              attempts: 3,
              reason: 'no answer within 6000 ms, so it was killed',
            },);
            const harness = registrationWith(async function rejectNotification(): Promise<void> {
              throw unavailable;
            },);

            await harness.handler(
              createAgentSettledEvent({ aborted: false, },),
              createExtensionContext(),
            );
            await harness.handler(
              createAgentSettledEvent({ aborted: false, },),
              createExtensionContext(),
            );
            await harness.registration.settled();

            expect(harness.recording.calls.warn,).toHaveLength(1,);
            expect(harness.recording.calls.warn[0],).toContain('settled-agent desktop notification unavailable',);
            expect(harness.recording.calls.warn[0],).toContain(unavailable.message,);
            expect(harness.recording.calls.debug,).toHaveLength(1,);
            expect(harness.recording.calls.debug[0],).toContain(
              'settled-agent desktop notification remains unavailable',
            );
          },
        },),

        //endregion Delivery failures
      ],
    },),
  ],
},);
