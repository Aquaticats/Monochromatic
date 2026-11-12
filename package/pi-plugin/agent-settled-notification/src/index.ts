/**
 Package entry: the agent-settled desktop notification extension and its parts.

 This module only re-exports, so the extension Pi loads, the payload it sends, the
 bounded process boundary it runs and the retry policy it follows are all one
 import away, and no symbol is both imported and re-exported here.
 @module
 */

export {
  default,
  registerAgentSettledNotification,
} from './settled-notification-registration.ts';

export type {
  AgentSettledNotificationRegistration,
  SettledNotificationDeliverer,
  SettledNotificationDeliveryTracker,
} from './settled-notification-registration.ts';

export {
  NOTIFICATION_APP_NAME,
  NOTIFICATION_BODY_WITHOUT_FOLDER,
  NOTIFICATION_COMMAND,
  NOTIFICATION_TITLE,
  notificationArguments,
  notificationBody,
  notificationInvocation,
} from './notification-payload.ts';

export {
  NOTIFICATION_CAPTURE_LIMIT_BYTES,
  NotificationSpawnError,
  runNotificationProcess,
} from './notification-process.ts';

export type {
  NotificationAttemptEnding,
  NotificationAttemptOutcome,
  NotificationInvocation,
  NotificationProcessInput,
  NotificationProcessRunner,
} from './notification-process.ts';

export {
  NOTIFICATION_ATTEMPT_TIMEOUT_MS,
  NOTIFICATION_DELAYS_BEFORE_RETRY_MS,
  NOTIFICATION_TOTAL_BOUND_MS,
  NotificationUnavailableError,
  deliverSettledNotification,
  isAcceptedAttempt,
  notificationFailureReason,
} from './settled-notification-delivery.ts';

export type {
  NotificationClock,
  NotificationDelay,
  SettledNotificationDelivery,
} from './settled-notification-delivery.ts';

export { lodgedImportsIn, } from './bundle-imports.ts';
