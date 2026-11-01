/**
 * Root tagged logger for the editord client.
 *
 * All client modules compose deeper tags via `tagged({ tag, l })`.
 */

import {
  logger,
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

/**
 * Root tagged logger for all editord client subsystems.
 * Sub-modules should compose deeper tags via `tagged({ tag, l })`.
 */
export const l: Logger = tagged({
  tag: 'editord',
  l: logger,
},);

export type { Logger, };
export { tagged, };
