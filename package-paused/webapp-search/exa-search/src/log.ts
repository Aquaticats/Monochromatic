import {
  logger,
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

/**
 * Root tagged logger for all exa-search subsystems.
 * Tagged with `exa-search` so every log line carries the subsystem prefix.
 * Sub-modules should compose deeper tags via `tagged({ tag, l })`.
 *
 * @see `tagged` for composing nested tags
 */
export const l: Logger = tagged({
  tag: 'exa-search',
  l: logger,
},);
