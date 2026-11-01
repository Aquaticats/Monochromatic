/**
 Pi extension that tells Linux desktop environments when an agent fully settles.
 
 @module
 */

import type { ExtensionAPI, } from '@earendil-works/pi-coding-agent';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import nanoSpawn from 'nano-spawn';

//region Constants

/**
 Executable implementing the local Freedesktop desktop-notification boundary.
 */
const NOTIFICATION_COMMAND = 'notify-send';

/**
 Limits notification delivery so a broken desktop session cannot hold up Pi settlement indefinitely.
 */
const NOTIFICATION_TIMEOUT_MS = 1_000;

/**
 Immutable desktop-notification payload that does not expose prompt, session, or project information.
 */
const NOTIFICATION_ARGUMENTS = [
  '--app-name=Pi',
  'Pi agent finished',
  'Agent is idle and ready for input.',
] as const;

/**
 Root logger for this extension's notification lifecycle.
 */
const logger = tagged({ tag: 'pi-agent-settled-notification', },);

//endregion Constants

//region Notification boundary

/**
 Command and arguments passed to the desktop-notification executable.
 */
type NotificationInvocation = {
  /**
   Executable resolved through `PATH`.
   */
  readonly command: string;

  /**
   Fixed arguments kept outside shell syntax.
   */
  readonly args: readonly string[];
};

/**
 Isolated child-process request used to verify the notification subprocess boundary.
 */
type NotificationProcessInput = NotificationInvocation & {
  /**
   Discards terminal input because notifications require no interactive data.
   */
  readonly stdin: 'ignore';

  /**
   Prevents the desktop executable from contaminating Pi output.
   */
  readonly stdout: 'ignore';

  /**
   Prevents expected host-capability errors from contaminating Pi output.
   */
  readonly stderr: 'ignore';

  /**
   Bounded subprocess lifetime in milliseconds.
   */
  readonly timeout: number;
};

/**
 Injectable command runner used to verify subprocess options without touching the desktop.
 */
type NotificationProcessRunner = (
  input: NotificationProcessInput,
) => Promise<void>;

/**
 Injectable notification boundary used to verify settled-event delivery.
 */
type NotificationInvoker = (
  invocation: NotificationInvocation,
) => Promise<void>;

/**
 Outcome of an attempted desktop-notification delivery.
 */
type NotificationDeliveryResult =
  | { readonly delivered: true; }
    | {
      readonly delivered: false;
      readonly error: unknown;
    };

/**
 Runs a local desktop-notification process without shell interpretation.
 
 @param command - executable resolved through `PATH`
 
 @param args - fixed values passed outside shell syntax
 
 @param stdin - non-interactive input disposition
 
 @param stdout - output disposition that protects Pi output
 
 @param stderr - error disposition that protects Pi output
 
 @param timeout - bounded subprocess lifetime
 
 @example
 ```ts
 await runNotificationProcess({
   command: 'notify-send',
   args: ['--app-name=Pi', 'Pi agent finished', 'Agent is idle and ready for input.'],
   stdin: 'ignore',
   stdout: 'ignore',
   stderr: 'ignore',
   timeout: 1_000,
 });
 ```
 */
async function runNotificationProcess(
  {
    command,
    args,
    stdin,
    stdout,
    stderr,
    timeout,
  }: NotificationProcessInput,
): Promise<void> {
  await nanoSpawn(
    command,
    [...args,],
    {
      stdin,
      stdout,
      stderr,
      timeout,
    },
  );
}

/**
 Invokes the local desktop-notification executable with bounded, non-interactive process options.
 
 @param invocation - fixed command payload sent to the operating system
 
 @param run - replacement process boundary for tests
 
 @example
 ```ts
 await invokeNotification({
   invocation: {
     command: 'notify-send',
     args: ['--app-name=Pi', 'Pi agent finished', 'Agent is idle and ready for input.'],
   },
 });
 ```
 */
async function invokeNotification(
  {
    invocation,
    run = runNotificationProcess,
  }: {
    readonly invocation: NotificationInvocation;
    readonly run?: NotificationProcessRunner;
  },
): Promise<void> {
  await run({
    ...invocation,
    stdin: 'ignore',
    stdout: 'ignore',
    stderr: 'ignore',
    timeout: NOTIFICATION_TIMEOUT_MS,
  },);
}

/**
 Delegates fixed notification invocation through the testable subprocess adapter.
 
 @param invocation - executable and fixed arguments sent to the desktop boundary
 
 @example
 ```ts
 await invokeDefaultNotification({
   command: 'notify-send',
   args: ['--app-name=Pi', 'Pi agent finished', 'Agent is idle and ready for input.'],
 });
 ```
 */
async function invokeDefaultNotification(
  invocation: NotificationInvocation,
): Promise<void> {
  await invokeNotification({ invocation, },);
}

/**
 Attempts delivery of the settled-agent notification without propagating a desktop failure to Pi.
 
 @param invoke - replacement desktop-notification boundary for tests
 
 @returns successful delivery state or captured failure for caller-owned logging
 
 @example
 ```ts
 const delivery = await notifyAgentSettled({});
 if (!delivery.delivered)
   console.error(delivery.error);
 ```
 */
async function notifyAgentSettled(
  {
    invoke = invokeDefaultNotification,
  }: {
    readonly invoke?: NotificationInvoker;
  },
): Promise<NotificationDeliveryResult> {
  try {
    await invoke({
      command: NOTIFICATION_COMMAND,
      args: NOTIFICATION_ARGUMENTS,
    },);
    return { delivered: true, };
  }
  catch (error) {
    return {
      delivered: false,
      error,
    };
  }
}

//endregion Notification boundary

//region Extension registration

/**
 Registration inputs for the settled-agent notification event handler.
 */
type AgentSettledNotificationRegistration = {
  /**
   Pi extension API that owns lifecycle-event registration.
   */
  readonly pi: ForeignBorrowed<ExtensionAPI>;

  /**
   Optional injectable notification boundary for deterministic tests.
   */
  readonly invoke?: NotificationInvoker;
};

/**
 Registers the desktop-notification handler for final agent settlement only.
 
 @param pi - Pi API receiving the sole lifecycle subscription
 
 @param invoke - optional desktop-notification boundary used by tests
 
 @mutates pi - `pi.on` stores the `agent_settled` lifecycle registration in the Pi host
 
 @example
 ```ts
 registerAgentSettledNotification({ pi });
 ```
 */
function registerAgentSettledNotification(
  {
    pi,
    invoke,
  }: AgentSettledNotificationRegistration,
): void {
  /**
   Tracks whether the current Pi runtime already surfaced unavailable desktop capability.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-function-root-let -- Session-local failure latch prevents repeated warnings after expected host-capability failures.
  let hasWarnedAboutUnavailableNotification = false;

  pi.on(
    'agent_settled',
    async function handleAgentSettled() {
      /**
       Per-event logger carrying the lifecycle handler boundary.
       */
      const innerLogger = tagged({
        tag: handleAgentSettled.name,
        l: logger,
      },);
      /**
       Result of attempting the static desktop notification.
       */
      const delivery = await notifyAgentSettled((invoke === undefined
        ? {}
        : { invoke, }),);
      if (delivery.delivered) {
        innerLogger.debug('sent settled-agent desktop notification',);
        return;
      }

      /**
       Safe error summary that preserves diagnostics without throwing from Pi's lifecycle hook.
       */
      const failure = caughtValueText(delivery.error,);
      if (!hasWarnedAboutUnavailableNotification) {
        hasWarnedAboutUnavailableNotification = true;
        innerLogger.warn(`settled-agent desktop notification unavailable: ${failure}`,);
        return;
      }
      innerLogger.debug(`settled-agent desktop notification remains unavailable: ${failure}`,);
    },
  );
}

/**
 Registers the agent-settled desktop notification extension.
 
 @param pi - Pi extension API receiving the lifecycle subscription
 
 @mutates pi - `registerAgentSettledNotification` delegates lifecycle registration to `pi.on`
 
 @example
 ```ts
 pi -e ./packages/pi-plugin/agent-settled-notification/src/index.ts
 ```
 */
export default function agentSettledNotification(pi: ForeignBorrowed<ExtensionAPI>,): void {
  registerAgentSettledNotification({ pi, },);
}

//endregion Extension registration

export {
  NOTIFICATION_ARGUMENTS,
  NOTIFICATION_COMMAND,
  NOTIFICATION_TIMEOUT_MS,
  invokeNotification,
  notifyAgentSettled,
  registerAgentSettledNotification,
};

export type {
  NotificationDeliveryResult,
  NotificationInvocation,
  NotificationProcessInput,
  NotificationProcessRunner,
  NotificationInvoker,
};
