/**
 Visible custom-message rendering for Morph Compact restored context.
 
 @module
 */

import type {
  ContextEvent,
  ExtensionAPI,
  MessageRenderer,
  Theme,
} from '@earendil-works/pi-coding-agent';
import {
  Box,
  type Component,
  Text,
} from '@earendil-works/pi-tui';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { ReadonlyDeep, } from 'type-fest';

//region Constants

/**
 Custom message type used for visible Morph Compact context restore notices.
 */
export const MORPH_CONTEXT_MESSAGE_TYPE = 'morph-compact-context';

/**
 Fallback summary when an older or malformed custom message lacks string content.
 */
const FALLBACK_MESSAGE_CONTENT = 'Morph Compact restored compressed context.';

/**
 Fallback expanded details when custom-message metadata is unavailable.
 */
const FALLBACK_CONTEXT_TEXT = '(restored context is unavailable in message details)';

//endregion Constants

//region Types

/**
 Metadata stored on the visible context custom message.
 */
export type MorphContextMessageDetails = {
  /**
   Full compacted context delivered to the agent.
   */
  readonly text: string;
  /**
   Number of JavaScript string code units in {@link text}.
   */
  readonly characterCount: number;
  /**
   Number of newline-delimited lines in {@link text}.
   */
  readonly lineCount: number;
};

/**
 API methods needed to register the visible context renderer.
 */
type VisibleContextRendererApi = Pick<ExtensionAPI, 'registerMessageRenderer'>;

/**
 API methods needed to append the visible context custom message.
 */
type VisibleContextMessageApi = Pick<ExtensionAPI, 'sendMessage'>;

/**
 Custom message payload created for visible restored context.
 */
type VisibleContextMessage = {
  /**
   Custom renderer discriminator.
   */
  readonly customType: typeof MORPH_CONTEXT_MESSAGE_TYPE;
  /**
   Short transcript-visible summary. Full context lives in details.
   */
  readonly content: string;
  /**
   Shows the message in the interactive transcript.
   */
  readonly display: true;
  /**
   Renderer-only metadata with the full compacted context.
   */
  readonly details: MorphContextMessageDetails;
};

/**
 Agent message array shape exposed by pi's `context` event.
 */
type ContextMessages = ContextEvent['messages'];

/**
 Single agent message shape exposed by pi's `context` event.
 */
type ContextMessage = ContextMessages[number];

//endregion Types

//region Message construction

/**
 Count newline-delimited lines in a string.
 
 @param text - text whose lines are counted
 
 @returns line count, or zero for empty text
 
 @example
 ```typescript
 countLines('a\nb'); // 2
 ```
 */
function countLines(text: string,): number {
  if (text === '')
    return 0;
  return text
    .split('\n',)
    .length;
}

/**
 Build details stored on the visible Morph Compact context message.
 
 @param text - full compacted context delivered to the agent
 
 @returns renderer metadata for the visible custom message
 
 @example
 ```typescript
 const details = buildMorphContextMessageDetails({ text: 'context' });
 ```
 */
function buildMorphContextMessageDetails(
  {
    text,
  }: {
    readonly text: string;
  },
): MorphContextMessageDetails {
  return {
    text,
    characterCount: text.length,
    lineCount: countLines(text,),
  };
}

/**
 Build short content displayed in the collapsed transcript message.
 
 @param details - context metadata used for user-facing counts
 
 @returns collapsed custom-message content
 
 @example
 ```typescript
 const content = buildVisibleContextSummary({ details });
 ```
 */
function buildVisibleContextSummary(
  {
    details,
  }: {
    readonly details: MorphContextMessageDetails;
  },
): string {
  /**
   Locale-formatted line count for compact display.
   */
  const lineCount = details.lineCount
    .toLocaleString();
  /**
   Locale-formatted character count for compact display.
   */
  const characterCount = details.characterCount
    .toLocaleString();
  return `Morph Compact restored compressed context (${lineCount} lines, ${characterCount} characters). Expand this message to inspect it.`;
}

/**
 Build visible custom-message payload for restored Morph Compact context.
 
 @param text - full compacted context delivered to the agent
 
 @returns visible custom message with summary content and full details
 
 @example
 ```typescript
 const message = buildVisibleContextMessage({ text: '<morph-compacted-history />' });
 ```
 */
export function buildVisibleContextMessage(
  {
    text,
  }: {
    readonly text: string;
  },
): VisibleContextMessage {
  /**
   Renderer metadata carrying the full context without duplicating it in the
   custom-message content that participates in the LLM transcript.
   */
  const details = buildMorphContextMessageDetails({ text, },);
  return {
    customType: MORPH_CONTEXT_MESSAGE_TYPE,
    content: buildVisibleContextSummary({ details, },),
    display: true,
    details,
  };
}

/**
 Append a visible custom message for restored Morph Compact context.
 
 @param pi - extension API with custom-message support
 
 @param text - full compacted context delivered to the agent
 
 @example
 ```typescript
 sendVisibleCompactContext({ pi, text });
 ```
 */
export function sendVisibleCompactContext(
  {
    pi,
    text,
  }: {
    readonly pi: VisibleContextMessageApi;
    readonly text: string;
  },
): void {
  pi.sendMessage<MorphContextMessageDetails>(
    buildVisibleContextMessage({ text, },),
  );
}

/**
 Check whether a context message is Morph Compact's visible transcript marker.
 
 @param message - agent context message to inspect
 
 @returns whether message is a visible Morph Compact marker
 
 @example
 ```typescript
 isVisibleMorphContextMessage(message);
 ```
 */
function isVisibleMorphContextMessage(message: ForeignBorrowed<ContextMessage>,): boolean {
  return (message.role
    === 'custom')
    && (message.customType
      === MORPH_CONTEXT_MESSAGE_TYPE);
}

/**
 Remove visible Morph Compact transcript markers from model context.
 
 The custom message exists only for the interactive transcript. The agent
 receives the real compacted context through the following user message, so
 forwarding this marker would duplicate metadata and leak UI copy to the model.
 
 @param messages - context messages about to be sent to the provider
 
 @returns context messages without visible Morph Compact UI markers
 
 @example
 ```typescript
 const messages = filterVisibleContextMessages({ messages: event.messages });
 ```
 */
export function filterVisibleContextMessages(
  {
    messages,
  }: Readonly<{
    readonly messages: ForeignBorrowed<readonly ContextMessage[]>;
  }>,
): ContextMessages {
  return messages.filter(function keepNonVisibleMorphContextMessage(message,) {
    return !isVisibleMorphContextMessage(message,);
  },);
}

//endregion Message construction

//region Rendering

/**
 Check whether unknown details match {@link MorphContextMessageDetails}.
 
 @param value - details value to validate
 
 @returns whether value contains full visible context metadata
 
 @example
 ```typescript
 if (isMorphContextMessageDetails(message.details)) {
   console.log(message.details.text);
 }
 ```
 */
function isMorphContextMessageDetails(
  value: unknown,
): value is MorphContextMessageDetails {
  if (((typeof value) !== 'object') || (value === null))
    return false;
  if ((!('text' in value))
    || (!('characterCount' in value))
    || (!('lineCount' in value)))
    return false;
  /**
   Candidate fields checked for exact primitive types below.
   */
  const {
    text,
    characterCount,
    lineCount,
  } = value;
  return ((typeof text) === 'string')
    && ((typeof characterCount) === 'number')
    && ((typeof lineCount) === 'number');
}

/**
 Render visible Morph Compact context custom messages.
 
 @param message - custom message to render
 
 @param expanded - whether expanded details are visible
 
 @param theme - current pi theme
 
 @returns TUI component for the transcript
 
 @example
 ```typescript
 renderVisibleContextMessage({ message, expanded: true, theme });
 ```
 */
export function renderVisibleContextMessage(
  {
    message,
    expanded,
    theme,
  }: Readonly<{
    readonly message: Readonly<{
      readonly content: unknown;
      readonly details?: unknown;
    }>;
    readonly expanded: boolean;
    readonly theme: ReadonlyDeep<Theme>;
  }>,
): Component {
  /**
   Summary content rendered in both collapsed and expanded views.
   */
  const content = (typeof message.content) === 'string'
    ? message.content
    : FALLBACK_MESSAGE_CONTENT;
  /**
   Validated full-context details, or undefined for old malformed entries.
   */
  const details = isMorphContextMessageDetails(message.details,)
    ? message.details
    : undefined;
  /**
   Styled message title.
   */
  const title = theme.fg(
    'customMessageLabel',
    theme.bold('[Morph Compact]',),
  );
  /**
   Styled summary line.
   */
  const summary = theme.fg(
    'customMessageText',
    content,
  );
  /**
   Full restored context shown only when expanded.
   */
  const restoredContext = details?.text
    ?? FALLBACK_CONTEXT_TEXT;
  /**
   Styled expanded-view label for the restored context block.
   */
  const expandedDetailsLabel = theme.fg(
    'dim',
    'Restored context delivered to the agent:',
  );
  /**
   Body rendered inside the custom-message box.
   */
  const body = expanded
    ? `${title}\n${summary}\n\n${expandedDetailsLabel}\n${restoredContext}`
    : `${title}\n${summary}`;
  /**
   Box matching pi's default custom-message styling.
   */
  const box = new Box(
    1,
    1,
    function colorCustomMessageBox(text,): string {
      return theme.bg(
        'customMessageBg',
        text,
      );
    },
  );
  box.addChild(new Text(
    body,
    0,
    0,
  ),);
  return box;
}

/**
 Register the renderer for visible Morph Compact context messages.
 
 @param pi - extension API with custom-message renderer registration
 
 @example
 ```typescript
 registerVisibleContextRenderer({ pi });
 ```
 */
export function registerVisibleContextRenderer(
  {
    pi,
  }: {
    readonly pi: VisibleContextRendererApi;
  },
): void {
  pi.registerMessageRenderer<MorphContextMessageDetails>(
    MORPH_CONTEXT_MESSAGE_TYPE,
    function renderMessage(
      message: ForeignBorrowed<Parameters<MessageRenderer<MorphContextMessageDetails>>[0]>,
      options: ForeignBorrowed<Parameters<MessageRenderer<MorphContextMessageDetails>>[1]>,
      theme,
    ) {
      return renderVisibleContextMessage({
        message,
        expanded: options.expanded,
        theme,
      },);
    },
  );
}

//endregion Rendering
