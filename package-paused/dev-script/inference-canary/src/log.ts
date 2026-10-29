/**
 * Root tagged logger for the inference canary.
 * Sub-modules compose deeper tags via `tagged({ tag, l })`.
 *
 * @example
 * ```ts
 * import { l, tagged } from './log.ts';
 *
 * function runProbe(probe: Probe): void {
 *   const rl = tagged({ tag: runProbe.name, l });
 *   rl.info('starting probe execution');
 * }
 * ```
 */
import {
  logger,
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

/**
 * Root tagged logger for all inference canary subsystems.
 * Sub-modules compose deeper tags via `tagged({ tag, l })`.
 *
 * @example
 * ```ts
 * const rl = tagged({ tag: myFn.name, l });
 * rl.info('starting operation');
 * ```
 */
export const l: Logger = tagged({
  tag: 'canary',
  l: logger,
},);

export type { Logger, };
export { tagged, };
