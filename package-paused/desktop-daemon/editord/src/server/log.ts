/**
 * Root tagged logger for the editord server.
 *
 * All server modules compose deeper tags via `tagged({ tag, l })`.
 */

import {
  logger,
  tagged,
  type Logger as ModuleLogger,
} from '@monochromatic-dev/logger-core/ts';

/**
 * Logger handle passed through editord server modules.
 */
export type Logger = Readonly<ModuleLogger>;

/**
 * Root tagged logger for all editord server subsystems.
 * Sub-modules should compose deeper tags via `tagged({ tag, l })`.
 */
export const l: Logger = tagged({
  tag: 'editord',
  l: logger,
},);

export { tagged, };
