/**
 Shared package logger every module tags its own lines from.
 
 @module
 */

import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

//region Logger -- package-tagged parent every module wraps

/**
 Package logger carrying the shared `model-review` tag.
 */
const packageLogger: Logger = tagged({ tag: 'model-review', },);

//endregion Logger -- package-tagged parent every module wraps

export { packageLogger, };
