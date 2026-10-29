/**
 Logger root for rust-catalog-update.

 @module
 */

import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

/**
 Root logger; functions re-tag it with their own name.

 @example
 ```ts
 const log = tagged({ tag: someFunction.name, l, },);
 ```
 */
export const l: Logger = tagged({ tag: 'rust-catalog-update', },);
