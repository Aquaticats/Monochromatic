/**
 Claude statusline composition.
 
 @module
 */

import {
  YELLOW,
  color,
} from './ansi.ts';
import { readRememberedActivityWord, } from './activity-memory.ts';
import { formatContextWindowSegment, } from './context-window.ts';
import { readEffortIndicator, } from './effort.ts';
import { formatModelDisplay, } from './model.ts';
import { formatRateLimits, } from './rate-limit.ts';
import type { StatuslineInput, } from './types.ts';

//region Constants

/**
 Separator inserted between rendered segments.
 */
const STATUSLINE_SEPARATOR = '    ';

//endregion Constants

//region Segment formatting

/**
 Formats model segment from statusline input and effort indicator.
 
 @param input - statusline input payload
 
 @param effortIndicator - rendered effort indicator symbol
 
 @returns model segment, or empty string when display name is absent
 
 @example
 ```ts
 formatModelSegment({ input, effortIndicator: '○' });
 ```
 */
function formatModelSegment({
  input,
  effortIndicator,
}: Readonly<{
  input: StatuslineInput;
  effortIndicator: string;
}>,): string {
  /**
   Model metadata pulled from the input.
   */
  const modelMetadata = input.model;
  if (modelMetadata === undefined)
    return '';

  /**
   Model display name pulled from the input.
   */
  const displayName = modelMetadata.display_name;
  if (displayName === undefined)
    return '';
  if (displayName.length === 0)
    return '';

  /**
   Trimmed display form of the model name.
   */
  const model = formatModelDisplay(displayName,);
  if (effortIndicator.length === 0)
    return model;

  return `${model} ${color({
    code: YELLOW,
    value: effortIndicator,
  },)}`;
}

/**
 Reads activity word, remembered across renders, when transcript path is available.
 
 @param input - statusline input payload
 
 @returns activity word, or empty string when unavailable
 
 @example
 ```ts
 await activitySegment(input);
 ```
 */
async function activitySegment(input: StatuslineInput,): Promise<string> {
  /**
   Transcript path from Claude statusline input.
   */
  const transcriptPath = input.transcript_path;
  if (transcriptPath === undefined)
    return '';
  if (transcriptPath.length === 0)
    return '';

  return await readRememberedActivityWord({ transcriptPath, },);
}

/**
 Joins non-empty statusline segments.
 
 @param segments - segment candidates in display order
 
 @returns joined statusline
 
 @example
 ```ts
 joinStatuslineSegments(['Opus', '', 'Testing']);
 ```
 */
function joinStatuslineSegments(segments: readonly string[],): string {
  return segments
    .filter(function isNonEmpty(segment,): boolean {
      return segment.length > 0;
    },)
    .join(STATUSLINE_SEPARATOR,);
}

//endregion Segment formatting

//region Public render

/**
 Renders complete Claude statusline text.
 
 @param input - statusline input payload
 
 @param renderedAtMs - render timestamp in epoch milliseconds
 
 @returns statusline text, or empty string when every segment is absent
 
 @example
 ```ts
 await renderStatusline({ input, renderedAtMs: Date.now() });
 ```
 */
async function renderStatusline({
  input,
  renderedAtMs,
}: Readonly<{
  input: StatuslineInput;
  renderedAtMs: number;
}>,): Promise<string> {
  /**
   Effort-level indicator read from `~/.claude/settings.json`.
   */
  const effortIndicator = await readEffortIndicator();
  /**
   Composed model segment.
   */
  const modelSegment = formatModelSegment({
    input,
    effortIndicator,
  },);
  /**
   Rendered context-window segment.
   */
  const contextSegment = formatContextWindowSegment(input,);
  /**
   Rendered shared rate-limit segment.
   */
  const rateSegment = formatRateLimits({
    ...(input.rate_limits === undefined ? {} : { rateLimits: input.rate_limits, }),
    sampledAtMs: renderedAtMs,
    renderedAtMs,
  },);
  /**
   Context-aware activity word extracted from transcript tail.
   */
  const activityWord = await activitySegment(input,);

  return joinStatuslineSegments([
    modelSegment,
    contextSegment,
    rateSegment,
    activityWord,
  ],);
}

//endregion Public render

export {
  STATUSLINE_SEPARATOR,
  activitySegment,
  formatModelSegment,
  joinStatuslineSegments,
  renderStatusline,
};
