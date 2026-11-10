/**
 Message extraction helpers for child Pi result forwarding.
 
 @module
 */

//region Types

/**
 Minimal text block shape used by Pi assistant messages.
 */
type TextBlockLike = {
  /**
   Content block discriminator.
   */
  readonly type: 'text';
  /**
   Text emitted by assistant.
   */
  readonly text: string;
};

/**
 Minimal assistant message shape needed by spawn-pi.
 */
type AssistantMessageLike = {
  /**
   Message role discriminator.
   */
  readonly role: 'assistant';
  /**
   Assistant content blocks or legacy plain text.
   */
  readonly content: string | readonly unknown[];
};

//endregion Types

//region Type guards

/**
 Detects text content blocks in unknown Pi message content.
 
 @param value - content block candidate.
 
 @returns whether value is text block.
 
 @example
 ```typescript
 isTextBlock({ type: 'text', text: 'done' });
 ```
 */
function isTextBlock(value: unknown,): value is TextBlockLike {
  if ((value === null) || ((typeof value) !== 'object'))
    return false;
  return ('type' in value)
    && (value.type === 'text')
    && ('text' in value)
    && ((typeof value.text) === 'string');
}

/**
 Detects assistant messages in unknown Pi message arrays.
 
 @param value - message candidate.
 
 @returns whether value is assistant message.
 
 @example
 ```typescript
 isAssistantMessage({ role: 'assistant', content: [] });
 ```
 */
function isAssistantMessage(value: unknown,): value is AssistantMessageLike {
  if ((value === null) || ((typeof value) !== 'object'))
    return false;
  return ('role' in value)
    && (value.role === 'assistant')
    && ('content' in value)
    && (((typeof value.content) === 'string') || Array.isArray(value.content,));
}

//endregion Type guards

//region Extraction

/**
 Extracts text from assistant message content.
 
 @param content - {@link AssistantMessageLike} content.
 
 @returns joined text blocks, or content itself for legacy string content.
 
 @example
 ```typescript
 assistantContentText([{ type: 'text', text: 'done' }]);
 ```
 */
function assistantContentText(content: AssistantMessageLike['content'],): string {
  if ((typeof content) === 'string')
    return content;

  return content
    .filter(function keepTextBlock(block,): block is TextBlockLike {
      return isTextBlock(block,);
    },)
    .map(function blockText(block,): string {
      return block.text;
    },)
    .join('\n',);
}

/**
 Extracts last assistant text from Pi agent-end messages, using {@link isAssistantMessage} to
 filter candidates.
 
 @param messages - messages emitted by agent loop.
 
 @returns last assistant text, or empty string when no text exists.
 
 @example
 ```typescript
 extractLastAssistantText([{ role: 'assistant', content: [{ type: 'text', text: 'done' }] }]);
 ```
 */
function extractLastAssistantText(messages: readonly unknown[],): string {
  for (const candidate of messages.toReversed()) {
    if (!isAssistantMessage(candidate,))
      continue;

    /**
     Text content emitted by candidate assistant message.
     */
    const text = assistantContentText(candidate.content,);
    if (text.length > 0)
      return text;
  }

  return '';
}

//endregion Extraction

export {
  assistantContentText,
  extractLastAssistantText,
  isAssistantMessage,
  isTextBlock,
};

export type {
  AssistantMessageLike,
  TextBlockLike,
};
