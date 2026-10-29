/**
 Shared constants for browser-side controller event wiring modules.
 
 @example
 ```ts
 import { CHANNEL_KEYS } from './controller-event-constants.ts';
 ```
 */

import type {
  ChannelKey,
  DataDimKey,
  ToggleValue,
} from './filter.ts';

//region Constants

/**
 Channel keys, fixed order.
 */
export const CHANNEL_KEYS: readonly ChannelKey[] = [
  'x',
  'y',
  'z',
  'color',
  'shape',
  'size',
];

/**
 Valid data-dim keys for dropdown-value validation.
 */
export const DIM_KEYS: readonly DataDimKey[] = [
  'logSourceBytes',
  'logDaysStale',
  'logInstallSize',
  'logDownloads',
  'tsRatio',
  'runtimeDepCount',
  'transitiveDepCount',
  'logPackageAge',
  'isLeafNumeric',
  'licenseClassNumeric',
];

/**
 Valid toggle values for radio-input validation.
 */
export const TOGGLE_VALUES: readonly ToggleValue[] = [
  'any',
  'yes',
  'no',
];

//endregion Constants
