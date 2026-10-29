import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

/**
 Logger root for file-enforcer after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
export const l: Logger = tagged({ tag: 'file-enforcer', },);
