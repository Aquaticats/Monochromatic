/**
 Canonical Advisor tool value and its text-only result projection. @module
 */

import type {
  ContentBlock,
} from '@deepseek-ai/dsh-llm';
import type {
  JsonValue,
} from '@deepseek-ai/dsh-util-values';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  isAdvisorDetails,
  renderAdvisorOperationSummary,
  renderAdvisorSummary,
  type AdvisorDetails,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';
import { dshAdvisorTheme, } from './theme.ts';

//region Types

/**
 Canonical JSON value returned by the Advisor tool's `execute`.

 The review text and `AdvisorDetails` metadata (operation ledger, attempted
 models, aggregate usage) mirror pi's tool result; `progress` folds the final
 metadata-only progress line into the result because DSH tool calls have no
 progress channel.
 */
export type DshAdvisorToolValue = {
  /**
   Full Advisor review text.
   */
  readonly text: string;
  /**
   Structured `AdvisorDetails` metadata when the core produced any.
   */
  readonly details: JsonValue;
  /**
   Latest bounded progress line, empty when no progress update arrived.
   */
  readonly progress: string;
};

//endregion Types

//region Public API

/**
 Project structured Advisor details onto plain JSON for the canonical tool
 value.

 The tool result crosses the JSON boundary regardless, and the shared core's
 readonly arrays miss the mutable `JsonValue` array type, so one serialize
 pass normalizes the shape. The parsed document is held as `unknown` and kept
 only after a JSON-value guard, so no `any` leaves this boundary.

 @param details - structured operation details from the shared core

 @returns JSON view of the same details

 @throws when the details or their normalized document are not JSON data

 @example
 ```ts
 const details = advisorDetailsAsJsonValue(result.details);
 ```
 */
export function advisorDetailsAsJsonValue(
  details: AdvisorDetails,
): JsonValue {
  if (!isJsonValue(details))
    throw new Error(
      `advisor: operation details are not JSON data: ${caughtValueText(details,)}`,
    );
  /**
   Serialized details text at the JSON boundary.
   */
  const serialized = JSON.stringify(details,);
  /**
   Parsed document held as unknown before the JSON-value guard.
   */
  const parsed: unknown = JSON.parse(serialized,);
  /* v8 ignore start -- defensive: a JSON-text parse cannot leave JSON values; kept as the boundary guard */
  if (!isJsonValue(parsed))
    throw new Error(
      `advisor: operation details normalized to non-JSON data: ${caughtValueText(parsed,)}`,
    );
  /* v8 ignore stop */
  return parsed;
}

/**
 Project one validated canonical value onto model-facing text blocks.

 Mirrors pi's result renderer: the operation summary leads when the details
 parse as operation view, the full summary follows for structured details,
 and bare text covers everything else. The folded progress line appends as a
 final paragraph. Markdown rides inside the text content because the DSH
 tool channel is text-only.

 @param value - canonical JSON value, however stale or malformed on replay

 @returns text content blocks for the model-facing tool result

 @example
 ```ts
 const blocks = renderAdvisorToolContent({ text, details, progress });
 ```
 */
export function renderAdvisorToolContent(
  value: JsonValue,
): ContentBlock[] {
  /**
   Normalized view of the canonical value with safe fallbacks.
   */
  const parsed = advisorToolValueFromJson(value,);
  /**
   Operation summary, or the sentinel requesting the structured fallback.
   */
  const operationSummary = renderAdvisorOperationSummary({
    details: parsed.details,
    text: parsed.text,
    expanded: true,
    theme: dshAdvisorTheme,
  },);
  if ((typeof operationSummary)
    === 'string')
    return [{
      type: 'text',
      text: withProgressTail({
        body: operationSummary,
        progress: parsed.progress,
      },),
    },];
  if (!isAdvisorDetails(parsed.details,))
    return [{
      type: 'text',
      text: withProgressTail({
        body: parsed.text,
        progress: parsed.progress,
      },),
    },];
  return [{
    type: 'text',
    text: withProgressTail({
      body: renderAdvisorSummary({
        text: parsed.text,
        details: parsed.details,
        expanded: true,
        theme: dshAdvisorTheme,
      },),
      progress: parsed.progress,
    },),
  },];
}

/**
 Read one canonical value defensively for rendering.

 Logged replays may carry obsolete or partial values, so every field falls
 back to a readable rendering instead of throwing.

 @param value - raw canonical JSON value

 @returns normalized tool value

 @example
 ```ts
 const parsed = advisorToolValueFromJson(value);
 ```
 */
export function advisorToolValueFromJson(
  value: JsonValue,
): DshAdvisorToolValue {
  if ((typeof value)
    === 'string')
    return {
      text: value,
      details: {},
      progress: '',
    };
  if (((typeof value)
    !== 'object')
    || (value === null)
    || Array.isArray(value,))
    return {
      text: JSON.stringify(value,),
      details: {},
      progress: '',
    };
  return {
    text: (typeof value.text)
      === 'string'
      ? value.text
      : JSON.stringify(value,),
    details: (typeof value.details)
      === 'object'
      ? (value.details ?? {})
      : {},
    progress: (typeof value.progress)
      === 'string'
      ? value.progress
      : '',
  };
}

//endregion Public API

//region Helpers

/**
 Whether one parsed document is plain JSON data.

 @param value - parsed document to test

 @returns whether the value is JSON data
 */
function isJsonValue(value: unknown): value is JsonValue {
  if ((value === null)
    || ((typeof value) === 'string')
    || ((typeof value) === 'boolean'))
    return true;
  if ((typeof value) === 'number')
    return Number.isFinite(value,);
  if (Array.isArray(value)) {
    for (const item of value) {
      if (!isJsonValue(item))
        return false;
    }
    return true;
  }
  if ((typeof value) === 'object') {
    for (const entry of Object.values(value,)) {
      if (!isJsonValue(entry))
        return false;
    }
    return true;
  }
  return false;
}

/**
 Append the folded progress paragraph to rendered body text.

 @param body - rendered result body

 @param progress - folded progress line, empty when none was captured

 @returns body with progress tail when present
 */
function withProgressTail(
  {
    body,
    progress,
  }: {
    readonly body: string;
    readonly progress: string;
  },
): string {
  if (progress
    === '')
    return body;
  return `${body}\n\n${progress}`;
}

//endregion Helpers
