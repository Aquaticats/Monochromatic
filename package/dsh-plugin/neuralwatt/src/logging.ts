/**
 Tagged logger root shared by every dsh-plugin-neuralwatt module. @module
 */

import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

//region Public API

/**
 Logger root for dsh-plugin-neuralwatt modules.

 @example
 ```ts
 const innerL = tagged({ tag: myFn.name, l: neuralWattLogger, },);
 ```
 */
export const neuralWattLogger: Logger = tagged({
  tag: 'dsh-neuralwatt',
},);

//endregion Public API
