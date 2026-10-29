/**
 Plain-text rendering helpers for Advisor output.
 
 @module
 */

import type { AdvisorTheme, } from './ports.ts';
import type { ForeignHostCapability, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { ADVISOR_TOOL_NAME, } from './constants.ts';
import type { AdvisorDetails, } from './types.ts';

/**
 Milliseconds in one second for duration formatting.
 */
const MILLISECONDS_PER_SECOND = 1_000;

/**
 Markdown heading marker character.
 */
const MARKDOWN_HEADING_MARKER = '#';

/**
 Character separating Markdown heading marker from heading text.
 */
const MARKDOWN_HEADING_SEPARATOR = ' ';

//region Public helpers

/**
 Render Advisor result into a plain string for TUI Text components and tests.
 
 @param text - full Advisor text
 
 @param details - structured Advisor details
 
 @param expanded - whether full text should be shown
 
 @param theme - current pi theme
 
 @returns rendered text
 
 @mutates theme - theme methods can update Pi host styling caches
 
 @example
 ```typescript
 renderAdvisorSummary({ text, details, expanded: false, theme });
 ```
 */
export function renderAdvisorSummary(
  {
    text,
    details,
    expanded,
    theme,
  }: {
    readonly text: string;
    readonly details: AdvisorDetails;
    readonly expanded: boolean;
    readonly theme: ForeignHostCapability<AdvisorTheme>;
  },
): string {
  /**
   Header line with model and duration metadata.
   */
  const header = formatHeader({
    details,
    theme,
  },);
  if (expanded)
    return `${header}\n\n${text}`;

  /**
   First advisory line for collapsed rendering.
   */
  const firstLine = firstAdvisoryLine(text,);
  /**
   Styled first advisory line.
   */
  const styledFirstLine = theme.fg(
    'toolOutput',
    firstLine,
  );
  return `${header}\n${styledFirstLine}`;
}

/**
 Extract first non-empty Advisor text line.
 
 @param text - full Advisor text
 
 @returns first non-empty line or fallback
 
 @example
 ```typescript
 firstAdvisoryLine('\nLooks good');
 ```
 */
export function firstAdvisoryLine(
  text: string,
): string {
  /**
   Non-empty response lines after whitespace normalization.
   */
  const nonEmptyLines = text
    .split('\n',)
    .map(function trimLine(line,) {
      return line.trim();
    },)
    .filter(function keepLine(line,) {
      return line !== '';
    },);
  return nonEmptyLines
    .find(function keepBodyLine(line,) {
      return !isMarkdownHeadingLine(line,);
    },)
    ?? nonEmptyLines.at(0,)
    ?? '(advisor returned no text)';
}

/**
 Runtime guard for Advisor details.
 
 @param value - value to inspect
 
 @returns whether value is Advisor details
 
 @example
 ```typescript
 isAdvisorDetails({ selectedSlug: 'p/m' });
 ```
 */
export function isAdvisorDetails(
  value: unknown,
): value is AdvisorDetails {
  if ((value === null) || ((typeof value) !== 'object'))
    return false;
  return ('selectedSlug' in value)
    && ((typeof value.selectedSlug) === 'string')
    && ('provider' in value)
    && ((typeof value.provider) === 'string')
    && ('scopeSource' in value)
    && ((typeof value.scopeSource) === 'string')
    && ('scopedSlugs' in value)
    && Array
      .isArray(value.scopedSlugs,)
    && ('durationMs' in value)
    && ((typeof value.durationMs) === 'number')
    && ('contextBudgetChars' in value)
    && ((typeof value.contextBudgetChars) === 'number')
    && ('contextChars' in value)
    && ((typeof value.contextChars) === 'number')
    && ('estimatedInputTokens' in value)
    && ((typeof value.estimatedInputTokens) === 'number')
    && ('truncated' in value)
    && ((typeof value.truncated) === 'boolean')
    && ('stopReason' in value)
    && ((typeof value.stopReason) === 'string');
}

/**
 Build fallback details for malformed custom messages.
 
 @returns fallback Advisor details
 
 @example
 ```typescript
 fallbackDetails();
 ```
 */
export function fallbackDetails(): AdvisorDetails {
  return {
    selectedSlug: 'unknown/unknown',
    provider: 'unknown',
    scopeSource: 'available',
    scopedSlugs: [],
    durationMs: 0,
    contextBudgetChars: 0,
    contextChars: 0,
    estimatedInputTokens: 0,
    truncated: false,
    stopReason: 'stop',
  };
}

//endregion Public helpers

//region Internal helpers

/**
 Detect Markdown ATX heading lines.
 
 @param line - trimmed Advisor response line
 
 @returns whether line starts with one or more hash markers followed by a space
 */
function isMarkdownHeadingLine(
  line: string,
): boolean {
  /**
   Line without leading indentation.
   */
  const trimmedLine = line.trimStart();
  if (!trimmedLine.startsWith(MARKDOWN_HEADING_MARKER,))
    return false;

  /**
   Index of first non-heading-marker character.
   */
  const firstNonMarkerIndex = firstNonHeadingMarkerIndex(trimmedLine,);
  return (firstNonMarkerIndex > 0)
    && (trimmedLine.at(firstNonMarkerIndex,) === MARKDOWN_HEADING_SEPARATOR);
}

/**
 Find first character index after leading Markdown heading markers.
 
 @param line - line starting with at least one heading marker
 
 @returns index after leading heading markers
 */
function firstNonHeadingMarkerIndex(
  line: string,
): number {
  /**
   Cursor pointing at first non-heading-marker character.
   */
  let cursor = 0;
  while (line.at(cursor,) === MARKDOWN_HEADING_MARKER) {
    cursor += 1;
  }
  return cursor;
}

/**
 Format Advisor result header.
 
 @param details - Advisor result details
 
 @param theme - current pi theme
 
 @returns styled header text
 
 @mutates theme - theme methods can update Pi host styling caches
 */
function formatHeader(
  {
    details,
    theme,
  }: {
    readonly details: AdvisorDetails;
    readonly theme: ForeignHostCapability<AdvisorTheme>;
  },
): string {
  /**
   Styled tool name.
   */
  const title = theme.fg(
    'toolTitle',
    theme.bold(ADVISOR_TOOL_NAME,),
  );
  /**
   Styled selected model.
   */
  const model = theme.fg(
    'accent',
    details.selectedSlug,
  );
  /**
   Styled metadata.
   */
  const metadata = theme.fg(
    'dim',
    `${formatDuration(details.durationMs,)} ${formatContext(details,)}`,
  );
  return `${title} ${model} ${metadata}`;
}

/**
 Cutoff below which a duration renders with one decimal place. Matches
 `@monochromatic-dev/numeric-format-core` (Q5 default-to-richer pick);
 above it, decimals add noise without adding signal.
 */
const DECIMAL_BELOW_MS = 10;

/**
 Format elapsed milliseconds.
 
 @param durationMs - elapsed milliseconds
 
 @returns human-readable duration with unit suffix
 */
function formatDuration(
  durationMs: number,
): string {
  if (durationMs < DECIMAL_BELOW_MS)
    return `${durationMs.toFixed(1,)}ms`;
  return durationMs < MILLISECONDS_PER_SECOND
    ? `${durationMs.toFixed(0,)}ms`
    : `${(durationMs / MILLISECONDS_PER_SECOND).toFixed(1,)}s`;
}

/**
 Format context metadata.
 
 @param details - Advisor result details
 
 @returns context metadata summary
 */
function formatContext(
  details: AdvisorDetails,
): string {
  /**
   Truncation marker for concise output.
   */
  const truncated = details.truncated ? 'truncated' : 'full';
  return [
    `${details.provider} ${details.contextChars}/${details.contextBudgetChars} chars`,
    `${details.estimatedInputTokens} tokens`,
    truncated,
  ]
    .join(' ',);
}

//endregion Internal helpers
