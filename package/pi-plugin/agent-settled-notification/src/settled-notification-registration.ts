/**
 Registration of the desktop notification Pi sends when an agent fully settles.

 Delivery is started, not awaited: Pi's settlement hook returns as soon as the
 notification is under way, so a desktop that is slow, refusing or absent can never
 delay an agent. The delivery bounds and retries live in
 `./settled-notification-delivery.ts`, and every outcome is logged here.
 @module
 */

import type {
  AgentSettledEvent,
  ExtensionAPI,
  ExtensionContext,
} from '@earendil-works/pi-coding-agent';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { NotificationInvocation, } from './notification-process.ts';
import { notificationInvocation, } from './notification-payload.ts';
import {
  deliverSettledNotification,
  NotificationUnavailableError,
} from './settled-notification-delivery.ts';

//region Constants

/**
 Root logger for this extension's notification lifecycle.
 */
const logger = tagged({ tag: 'pi-agent-settled-notification', },);

//endregion Constants

//region Types

/**
 Notification boundary the registration starts for one settlement, replaceable for tests.
 */
type SettledNotificationDeliverer = (
  parts: {
    readonly invocation: NotificationInvocation;
  },
) => Promise<void>;

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
  readonly deliver?: SettledNotificationDeliverer;

  /**
   Optional logger receiving handler diagnostics; defaults to the module root logger.
   */
  readonly l?: Logger;
};

/**
 Handle on the deliveries a registration started.

 Pi never receives this: it exists so a verifier can wait for the background work
 the settlement hook deliberately does not wait for.
 */
type SettledNotificationDeliveryTracker = {
  /**
   Waits for every delivery started before this call to finish logging its outcome.
   */
  readonly settled: () => Promise<void>;
};

//endregion Types

//region Failure description

/**
 What a caught failure yielded when this package tried to describe it.
 */
type FailureDescription =
  | {
    /**
     The failure described itself.
     */
    readonly kind: 'described';

    /**
     Text the failure yielded.
     */
    readonly text: string;
  }
    | {
    /**
     Reading the failure's own text threw.
     */
      readonly kind: 'undescribable';

    /**
     Text the reading failure yielded instead.
     */
      readonly text: string;
    };

/**
 Describes a caught failure, keeping the reason it could not be described.

 A failure whose own diagnostic throws must not turn a report into an unhandled
 rejection, so both outcomes travel as values.

 @param error - value a delivery threw

 @returns the failure's text plus whether reading it succeeded

 @example
 ```ts
 describeFailure(new Error('offline')); // { kind: 'described', text: 'offline' }
 ```
 */
function describeFailure(error: unknown,): FailureDescription {
  try {
    return {
      kind: 'described',
      text: caughtValueText(error,),
    };
  }
  catch (refusal) {
    return {
      kind: 'undescribable',
      text: caughtValueText(refusal,),
    };
  }
}

//endregion Failure description

//region Extension registration

/**
 Registers the desktop-notification handler for final agent settlement only.

 @param pi - Pi API receiving the sole lifecycle subscription

 @param deliver - optional notification boundary started by tests

 @param l - optional logger receiving handler diagnostics used by tests

 @returns tracker that waits for the deliveries this registration started

 @mutates pi - `pi.on` stores the `agent_settled` lifecycle registration in the Pi host

 @example
 ```ts
 const registration = registerAgentSettledNotification({ pi, });
 ```
 */
function registerAgentSettledNotification(
  {
    pi,
    deliver = deliverSettledNotification,
    l = logger,
  }: AgentSettledNotificationRegistration,
): SettledNotificationDeliveryTracker {
  /* oxlint-disable no-restricted-syntax/no-function-root-let -- Session-local failure latch prevents repeated warnings after expected host-capability failures. */
  /**
   Tracks whether the current Pi runtime already surfaced unavailable desktop capability.
   */
  let hasWarnedAboutUnavailableNotification = false;
  /* oxlint-enable no-restricted-syntax/no-function-root-let */

  /**
   Deliveries this registration started and has not finished reporting.
   */
  const unfinishedDeliveries = new Set<Promise<void>>();

  /**
   Per-registration logger carrying the lifecycle handler boundary.
   */
  const innerLogger = tagged({
    tag: 'handleAgentSettled',
    l,
  },);

  /**
   Reports that the desktop accepted the settled-agent notification.
   */
  function reportDelivered(): void {
    innerLogger.debug('sent settled-agent desktop notification',);
  }

  /**
   Reports a delivery that did not reach the desktop, keeping the executable's own words.

   A refusal is warned about once per Pi runtime and then logged at debug, because a
   desktop that stays unavailable would otherwise warn on every settlement. Anything
   that is not a desktop refusal is a defect in this package and stays at error level.

   @param error - value the delivery threw
   */
  function reportRefused(error: unknown,): void {
    /**
     The failure's own text, plus whether it could be read at all.
     */
    const described = describeFailure(error,);

    if (described.kind === 'undescribable') {
      innerLogger.error(
        `settled-agent desktop notification failed and could not be described: ${described.text}`,
      );
      return;
    }

    // Nothing awaits this delivery, so a defect is reported at error level instead
    // of escaping as an unhandled rejection Pi could not attribute.
    if (!(error instanceof NotificationUnavailableError)) {
      innerLogger.error(`settled-agent desktop notification failed: ${described.text}`,);
      return;
    }

    if (!hasWarnedAboutUnavailableNotification) {
      hasWarnedAboutUnavailableNotification = true;
      innerLogger.warn(`settled-agent desktop notification unavailable: ${described.text}`,);
      return;
    }
    innerLogger.debug(`settled-agent desktop notification remains unavailable: ${described.text}`,);
  }

  /**
   Delivers one settlement and reports the outcome, never throwing.

   @param invocation - executable and payload the notification carries
   */
  async function deliverAndReport(
    {
      invocation,
    }: {
      readonly invocation: NotificationInvocation;
    },
  ): Promise<void> {
    try {
      await deliver({ invocation, },);
    }
    catch (error) {
      reportRefused(error,);
      return;
    }
    reportDelivered();
  }

  /**
   Removes one delivery from the tracked set once it has finished reporting.

   @param delivery - delivery whose completion is awaited
   */
  async function forgetDelivery(delivery: Promise<void>,): Promise<void> {
    await delivery;
    unfinishedDeliveries.delete(delivery,);
  }

  /**
   Starts delivering one settlement without making Pi wait for it.

   @param invocation - executable and payload the notification carries
   */
  function startDelivery(
    {
      invocation,
    }: {
      readonly invocation: NotificationInvocation;
    },
  ): void {
    /**
     Delivery already running in the background, tracked until it finishes reporting.
     */
    const delivery = deliverAndReport({ invocation, },);
    unfinishedDeliveries.add(delivery,);
    void forgetDelivery(delivery,);
  }

  pi.on(
    'agent_settled',
    function handleAgentSettled(
      _event: ForeignBorrowed<AgentSettledEvent>,
      context: ForeignBorrowed<ExtensionContext>,
    ): void {
      startDelivery({
        invocation: notificationInvocation({ folder: context.cwd, },),
      },);
    },
  );

  return {
    settled: async function settled(): Promise<void> {
      await Promise.all(unfinishedDeliveries,);
    },
  };
}

/**
 Registers the agent-settled desktop notification extension.

 @param pi - Pi extension API receiving the lifecycle subscription

 @mutates pi - `registerAgentSettledNotification` delegates lifecycle registration to `pi.on`

 @example
 ```ts
 pi -e ./package/pi-plugin/agent-settled-notification/src/index.ts
 ```
 */
export default function agentSettledNotification(pi: ForeignBorrowed<ExtensionAPI>,): void {
  registerAgentSettledNotification({ pi, },);
}

//endregion Extension registration

export { registerAgentSettledNotification, };

export type {
  AgentSettledNotificationRegistration,
  SettledNotificationDeliverer,
  SettledNotificationDeliveryTracker,
};
