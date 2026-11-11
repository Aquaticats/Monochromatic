/**
 Tagged logger root shared by every dsh-plugin-qwen-token-plan-individual
 module. @module
 */

import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

//region Public API

/**
 Logger root for dsh-plugin-qwen-token-plan-individual modules.

 @example
 ```ts
 const innerL = tagged({ tag: myFn.name, l: qwenTokenPlanIndividualLogger, },);
 ```
 */
export const qwenTokenPlanIndividualLogger: Logger = tagged({
  tag: 'dsh-qwen-token-plan-individual',
},);

//endregion Public API
