/**
 Current-time context formatter exported from the pi package.
 
 @module
 */

import {
  formatTimeContext as sharedFormatTimeContext,
} from '@monochromatic-dev/agent-harness-shared-current-time-context/ts';

//region Formatting

/**
 Formats a {@link Date} as hidden current-time context using local wall-clock time.
 
 This wrapper keeps the pi package API local while delegating to the shared
 formatter used by the Claude Code prompt-time hook.
 
 {@inheritDoc sharedFormatTimeContext}
 */
function formatTimeContext(now: Date,): string {
  return sharedFormatTimeContext(now,);
}

//endregion Formatting

export { formatTimeContext, };
