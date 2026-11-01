/**
 Tagged logger root shared by every dsh-plugin-advisor module. @module
 */

import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

//region Public API

/**
 Logger root for dsh-plugin-advisor modules.

 @example
 ```ts
 const innerL = tagged({ tag: myFn.name, l: dshAdvisorLogger, },);
 ```
 */
export const dshAdvisorLogger: Logger = tagged({
  tag: 'dsh-advisor',
},);

//endregion Public API
