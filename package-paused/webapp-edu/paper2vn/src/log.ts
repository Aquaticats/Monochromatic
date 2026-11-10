/**
 Root logger for paper2vn.
 
 Every module tags its own lines below this one, so a line names the package
 first and the module second. The browser bundle and the Node build script
 share it: `\@monochromatic-dev/logger-core` picks its sinks from the runtime.
 
 @module
 */

import {
  logger,
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

/**
 Root logger, tagged with the package name so no module repeats it by hand.
 */
const l: Logger = tagged({
  tag: 'paper2vn',
  l: logger,
},);

export { l, };
