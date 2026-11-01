/**
 TUI component renderers for Advisor tool and custom messages.
 
 @module
 */

import type {
  AgentToolResult,
  Theme,
} from '@earendil-works/pi-coding-agent';
import {
  type Component,
  Text,
} from '@earendil-works/pi-tui';
import type { ReadonlyDeep, } from 'type-fest';
import type { ForeignHostCapability, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  fallbackDetails,
  isAdvisorDetails,
  renderAdvisorOperationSummary,
  renderAdvisorSummary,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';
import type {
  AdvisorDetails,
  AdvisorToolParams,
} from './types.ts';

//region Types

/**
 Runtime Advisor result shape accepted from successful or thrown tool execution.
 */
type AdvisorRenderableResult = {
  /**
   Tool content returned or synthesized by Pi.
   */
  readonly content: AgentToolResult<unknown>['content'];
  /**
   Structured success details, absent or malformed after thrown errors.
   */
  readonly details?: unknown;
};

//endregion Types

//region Public renderers

/**
 Render an Advisor tool call row.
 
 @param args - tool parameters
 
 @param theme - current pi theme
 
 @returns TUI component
 
 @mutates theme - theme methods can update Pi host styling caches
 
 @example
 ```typescript
 renderAdvisorCall({ args: {}, theme });
 ```
 */
export function renderAdvisorCall(
  {
    args,
    theme,
  }: {
    readonly args: AdvisorToolParams;
    readonly theme: ForeignHostCapability<Theme>;
  },
): Component {
  /**
   Requested target displayed before execution resolves.
   */
  const target = args.model
    ?? 'default scoped model';
  /**
   Styled tool title.
   */
  const title = theme.fg(
    'toolTitle',
    theme.bold('advisor',),
  );
  /**
   Styled action label.
   */
  const action = theme.fg(
    'accent',
    'Consulting advisor',
  );
  /**
   Styled target model label.
   */
  const targetText = theme.fg(
    'dim',
    target,
  );
  /**
   Styled focused-question indicator.
   */
  const questionText = args.question === undefined
    ? ''
    : theme.fg(
      'dim',
      ' with question',
    );
  return new Text(
    `${title} ${action} ${targetText}${questionText}`,
    0,
    0,
  );
}

/**
 Render an Advisor tool result.
 
 @param result - Advisor tool result
 
 @param expanded - whether result is expanded
 
 @param theme - current pi theme
 
 @returns TUI component
 
 @mutates theme - theme methods can update Pi host styling caches
 
 @example
 ```typescript
 renderAdvisorResult({ result, expanded: false, theme });
 ```
 */
export function renderAdvisorResult(
  {
    result,
    expanded,
    isPartial = false,
    theme,
  }: {
    readonly result: ReadonlyDeep<AdvisorRenderableResult>;
    readonly expanded: boolean;
    readonly isPartial?: boolean;
    readonly theme: ForeignHostCapability<Theme>;
  },
): Component {
  /**
   Text block returned to primary model.
   */
  const text = result.content[0]
    ?.type
    === 'text'
    ? result.content[0]
      .text
    : '(advisor returned no text)';
  if (isPartial)
    return new Text(
      theme.fg(
        'dim',
        text,
      ),
      0,
      0,
    );
  /**
   Operation metadata replaces partial progress with every attempt's terminal state.
   */
  const operationSummary = renderAdvisorOperationSummary({
    details: result.details,
    text,
    expanded,
    theme,
  },);
  if ((typeof operationSummary) === 'string')
    return new Text(
      operationSummary,
      0,
      0,
    );
  if (!isAdvisorDetails(result.details,)) {
    return new Text(
      theme.fg(
        'error',
        text,
      ),
      0,
      0,
    );
  }
  return new Text(
    renderAdvisorSummary({
      text,
      details: result.details,
      expanded,
      theme,
    },),
    0,
    0,
  );
}

/**
 Render a manual `/advisor` custom message.
 
 @param message - custom message payload
 
 @param expanded - whether message is expanded
 
 @param theme - current pi theme
 
 @returns TUI component
 
 @mutates theme - theme methods can update Pi host styling caches
 
 @example
 ```typescript
 renderAdvisorMessage({ message, expanded: true, theme });
 ```
 */
export function renderAdvisorMessage(
  {
    message,
    expanded,
    theme,
  }: {
    readonly message: {
      readonly content: unknown;
      readonly details?: unknown;
    };
    readonly expanded: boolean;
    readonly theme: ForeignHostCapability<Theme>;
  },
): Component {
  /**
   Message text from custom message content.
   */
  const text = (typeof message.content) === 'string'
    ? message.content
    : '(advisor returned no text)';
  /**
   Manual reviews use the same operation summary as tool reviews.
   */
  const operationSummary = renderAdvisorOperationSummary({
    details: message.details,
    text,
    expanded,
    theme,
  },);
  if ((typeof operationSummary) === 'string')
    return new Text(
      operationSummary,
      0,
      0,
    );
  /**
   Structured details when present.
   */
  const details = isAdvisorDetails(message.details,)
    ? message.details
    : fallbackDetails();
  return new Text(
    renderAdvisorSummary({
      text,
      details,
      expanded,
      theme,
    },),
    0,
    0,
  );
}

export {
  firstAdvisoryLine,
  renderAdvisorSummary,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';

//endregion Public renderers
