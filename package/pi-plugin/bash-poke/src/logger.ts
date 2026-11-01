/**
 Bash-poke logger root.

 @module
 */

import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

import { LOGGER_TAG, } from './constants.ts';

//region Logger

/**
 Root logger every subsystem wraps so records carry a bash-poke tag chain.
 
 @example
 ```ts
 tagged({ tag: startJob.name, l: bashPokeLogger, },);
 ```
 */
const bashPokeLogger: Logger = tagged({ tag: LOGGER_TAG, },);

//endregion Logger

export { bashPokeLogger, };
