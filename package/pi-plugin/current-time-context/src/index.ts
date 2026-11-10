/**
 Pi extension that injects hidden current local wall-clock time context.
 
 @module
 */

import type {
  BeforeAgentStartEventResult,
  ExtensionAPI,
} from '@earendil-works/pi-coding-agent';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { formatTimeContext, } from './format-time-context.ts';

//region Constants

/**
 Custom message type used for hidden current-time context entries.
 */
const CURRENT_TIME_CONTEXT_TYPE = 'current-time-context';

//endregion Constants

//region Extension entry point

/**
 Current Time Context pi extension.
 
 Subscribes to `before_agent_start` and returns one hidden custom message
 containing local 24-hour wall-clock time at minute precision.
 
 @param pi - pi extension API
 
 @mutates pi - `pi.on` stores the `before_agent_start` event registration in the Pi host
 
 @example
 ```typescript
 // In ~/.pi/agent/settings.json:
 { "packages": ["./packages/pi-plugin/current-time-context"] }
 ```
 */
export default function currentTimeContext(pi: ForeignBorrowed<ExtensionAPI>,): void {
  pi.on(
    'before_agent_start',
    function handleBeforeAgentStart(): BeforeAgentStartEventResult {
      return {
        message: {
          customType: CURRENT_TIME_CONTEXT_TYPE,
          content: formatTimeContext(new Date(),),
          display: false,
        },
      };
    },
  );
}

//endregion Extension entry point

export {
  CURRENT_TIME_CONTEXT_TYPE,
};
export { formatTimeContext, } from './format-time-context.ts';

/**
 Whether a custom message holds this extension's time-context content.

The shape is package behavior: it is what the hidden message must look like for
a consumer to read it back. Exported for the built-artifact tests, as
`test-import/require-eventual-artifact` asks of a symbol no consumer needs on the
public surface.

@internal
 */
export { isTimeContextContent, } from './time-context-shape.ts';
