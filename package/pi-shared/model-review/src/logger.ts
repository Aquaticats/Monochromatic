/**
 Shared package logger every module tags its own lines from.
 
 @module
 */

import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

/**
 Package logger carrying the shared `model-review` tag.
 */
const packageLogger: Logger = tagged({ tag: 'model-review', },);

export { packageLogger, };
