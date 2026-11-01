/**
 Conversation text serialization mirroring the pi transcript format the
 shared Advisor context builder expects. @module
 */

import type {
  ImageContent,
  Message as PiMessage,
  TextContent,
  ThinkingContent,
  ToolCall,
} from '@earendil-works/pi-ai';
import type {
  ForeignBorrowed,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

//region Constants

/**
 Tool-result characters kept in serialized conversation text.

 Mirrors the pi compaction serializer so both variants hand reviewers the
 same bounded tool results.
 */
export const ADVISOR_TOOL_RESULT_MAX_CHARS = 2_000;

//endregion Constants

//region Text extraction

/**
 Content block kinds carrying extractable text for serialization.
 */
type SerializableContent = TextContent | ImageContent | ThinkingContent | ToolCall;

/**
 Extract and join text blocks from message content.

 Mirrors pi-ai `contentText`: non-text blocks contribute nothing, and string
 content passes through unchanged.

 @param content - message content to extract from

 @param separator - join separator between text blocks

 @returns joined text blocks

 @example
 ```ts
 const text = advisorContentText({ content: message.content });
 ```
 */
export function advisorContentText(
  {
    content,
    separator = '\n',
  }: {
    readonly content: string | readonly ForeignBorrowed<SerializableContent>[];
    readonly separator?: string;
  },
): string {
  if ((typeof content) === 'string')
    return content;
  return content
    .filter(function isTextBlock(block,) {
      return block.type === 'text';
    },)
    .map(function readText(
      block: ForeignBorrowed<TextContent>,
    ) {
      return block.text;
    },)
    .join(separator,);
}

/**
 Truncate serialized text with a stable character count note.

 @param text - text to bound

 @param maxChars - character budget

 @returns original text under budget, else a head slice plus truncation note

 @example
 ```ts
 const bounded = truncateAdvisorSummaryText({ text, maxChars: 2_000 });
 ```
 */
export function truncateAdvisorSummaryText(
  {
    text,
    maxChars,
  }: {
    readonly text: string;
    readonly maxChars: number;
  },
): string {
  if (text.length
    <= maxChars)
    return text;
  /**
   Characters dropped from the tail.
   */
  const truncatedChars = text.length
    - maxChars;
  return `${text.slice(
    0,
    maxChars,
  )}\n\n[... ${String(truncatedChars,)} more characters truncated]`;
}

//endregion Text extraction

//region Conversation serialization

/**
 Render one assistant tool call as `name(k=v, ...)` for conversation text.

 @param block - pi-ai tool-call content block

 @returns single tool-call rendering
 */
function toolCallText(
  block: ToolCall,
): string {
  /**
   Rendered `key=value` pairs for the call arguments.
   */
  const argsText = Object
    .entries(block.arguments,)
    .map(function renderArgument([
      key,
      value,
    ],) {
      return `${key}=${JSON.stringify(value,)}`;
    },)
    .join(', ',);
  return `${block.name}(${argsText})`;
}

/**
 Serialize pi-ai messages into the labeled conversation text reviewers read.

 Mirrors the pi compaction serializer format: `[User]`, `[Assistant]`,
 `[Assistant tool calls]`, and `[Tool result]` paragraphs joined by blank
 lines, with system messages excluded and tool results bounded.

 @param messages - pi-ai messages converted from the session transcript

 @returns serialized conversation text

 @example
 ```ts
 const contextText = serializeAdvisorConversationText(messages);
 ```
 */
export function serializeAdvisorConversationText(
  messages: readonly ForeignBorrowed<PiMessage>[],
): string {
  /**
   Serialized paragraphs in message order.
   */
  const parts: string[] = [];
  for (const msg of messages) {
    if (msg.role
      === 'user') {
      /**
       Visible user text for this message.
       */
      const userText = advisorContentText({
        content: msg.content,
        separator: '',
      },);
      if (userText
        !== '')
        parts.push(`[User]: ${userText}`,);
      continue;
    }
    if (msg.role
      === 'assistant') {
      /**
       Assistant content blocks read through the borrowed element projection.
       */
      const assistantContent: readonly (ForeignBorrowed<TextContent> | ForeignBorrowed<ThinkingContent> | ForeignBorrowed<ToolCall>)[] = msg.content;
      /**
       Rendered tool calls requested by this response.
       */
      const toolCalls: string[] = [];
      for (const block of assistantContent) {
        if (block.type
          === 'toolCall')
          toolCalls.push(toolCallText(block,),);
      }
      if (assistantContent.some(function hasText(block,) {
        return block.type
          === 'text';
      },))
        parts.push(`[Assistant]: ${advisorContentText({ content: assistantContent, })}`,);
      if (toolCalls.length
        > 0)
        parts.push(`[Assistant tool calls]: ${toolCalls.join('; ',)}`,);
      continue;
    }
    if (msg.role
      === 'toolResult') {
      /**
       Visible tool-result text for this message.
       */
      const resultText = advisorContentText({
        content: msg.content,
        separator: '',
      },);
      if (resultText
        !== '')
        parts.push(
          `[Tool result]: ${truncateAdvisorSummaryText({
            text: resultText,
            maxChars: ADVISOR_TOOL_RESULT_MAX_CHARS,
          },)}`,
        );
    }
  }
  return parts.join('\n\n',);
}

//endregion Conversation serialization
