import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

/**
 Logger root for vm-builder; functions derive their own tagged child from it.

 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
export const l: Logger = tagged({ tag: 'vm-builder', },);
