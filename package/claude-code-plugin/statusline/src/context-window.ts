/**
 Claude context-window counter formatting.
 
 @module
 */

import {
  MAGENTA,
  WHITE,
  YELLOW,
  color,
} from './ansi.ts';
import type { StatuslineInput, } from './types.ts';

//region Context window constants

/**
 Token count at or above which the used segment renders white.
 */
const CONTEXT_THRESHOLD_WHITE = 900_000;

/**
 Token count at or above which the used segment renders magenta.
 */
const CONTEXT_THRESHOLD_MAGENTA = 200_000;

/**
 Token count at or above which the used segment renders yellow.
 */
const CONTEXT_THRESHOLD_YELLOW = 100_000;

/**
 Base for thousands grouping.
 */
const THOUSANDS = 1_000;

/**
 Width of the thousands group before the comma.
 */
const THOUSANDS_GROUP_WIDTH = 3;

/**
 Width of the full used-token field.
 */
const USED_TOKEN_FIELD_WIDTH = 7;

//endregion Context window constants

//region Usage math

/**
 Computes total used tokens from Claude's usage payload.
 
 @param usage - current usage payload from Claude Code
 
 @returns total used tokens across input, output, and cache buckets
 
 @example
 ```ts
 usedTokens({ input_tokens: 1, output_tokens: 2 });
 ```
 */
function usedTokens(
  usage: NonNullable<NonNullable<StatuslineInput['context_window']>['current_usage']>,
): number {
  return (usage.input_tokens ?? 0)
    + (usage.cache_creation_input_tokens ?? 0)
    + (usage.cache_read_input_tokens ?? 0)
    + (usage.output_tokens ?? 0);
}

//endregion Usage math

//region Formatting

/**
 Formats used-token count with a fixed-width thousands layout.
 
 @param used - used token count
 
 @returns fixed-width used-token text
 
 @example
 ```ts
 formatUsedTokens(51045);
 ```
 */
function formatUsedTokens(used: number,): string {
  if (used >= THOUSANDS) {
    /**
     Thousands group before the comma.
     */
    const thousandsGroup = String(Math.floor(used / THOUSANDS,),)
      .padStart(THOUSANDS_GROUP_WIDTH,);
    /**
     Ones group after the comma.
     */
    const onesGroup = String(used % THOUSANDS,)
      .padStart(
      THOUSANDS_GROUP_WIDTH,
      '0',
      );
    return `${thousandsGroup},${onesGroup}`;
  }

  return String(used,)
    .padStart(USED_TOKEN_FIELD_WIDTH,);
}

/**
 Selects ANSI color for context-window usage.
 
 @param used - used token count
 
 @returns ANSI color code, or empty string when usage is below color thresholds
 
 @example
 ```ts
 contextWindowColor(200_000);
 ```
 */
function contextWindowColor(used: number,): string {
  if (used >= CONTEXT_THRESHOLD_WHITE)
    return WHITE;
  if (used >= CONTEXT_THRESHOLD_MAGENTA)
    return MAGENTA;
  if (used >= CONTEXT_THRESHOLD_YELLOW)
    return YELLOW;
  return '';
}

/**
 Formats used/total token counter with color based on usage level.
 
 @param used - used token count
 
 @param total - context-window token capacity
 
 @returns formatted context-window segment
 
 @example
 ```ts
 formatContextWindow({ used: 51_045, total: 1_000_000 });
 ```
 */
function formatContextWindow({
  used,
  total,
}: Readonly<{
  used: number;
  total: number;
}>,): string {
  /**
   Used-token count rendered in fixed-width form.
   */
  const usedText = formatUsedTokens(used,);
  /**
   Total token count rendered with locale-aware separators.
   */
  const totalText = total.toLocaleString('en-US',);
  /**
   Colour code picked from context usage bands.
   */
  const code = contextWindowColor(used,);

  return code.length > 0
    ? `${color({
      code,
      value: usedText,
    },)}/${totalText}`
    : `${usedText}/${totalText}`;
}

/**
 Formats context window segment from Claude statusline input.
 
 @param input - statusline input payload
 
 @returns context-window segment, or empty string when usage is unavailable
 
 @example
 ```ts
 formatContextWindowSegment(input);
 ```
 */
function formatContextWindowSegment(input: StatuslineInput,): string {
  /**
   Context-window payload from the input.
   */
  const contextWindow = input.context_window;
  if (contextWindow === undefined)
    return '';

  /**
   Current-usage subtree from the input.
   */
  const usage = contextWindow.current_usage;
  /**
   Total context-window size.
   */
  const total = contextWindow.context_window_size ?? 0;

  if (usage === undefined)
    return '';

  /**
   Sum of every input/output/cache token bucket.
   */
  const used = usedTokens(usage,);
  if ((used <= 0) || (total <= 0))
    return '';

  return formatContextWindow({
    used,
    total,
  },);
}

//endregion Formatting

export {
  contextWindowColor,
  formatContextWindow,
  formatContextWindowSegment,
  formatUsedTokens,
  usedTokens,
};
