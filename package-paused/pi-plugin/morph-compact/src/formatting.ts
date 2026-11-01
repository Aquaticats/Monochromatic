/**
 Query extraction and input/output formatting for Morph Compact.
 */

import type { ReadonlyDeep, } from 'type-fest';
import type { SessionEntry, } from '@earendil-works/pi-coding-agent';

//region Text content extraction

/**
 Type guard for text content items in a message content array.
 
 @param item - unknown value to test
 
 @returns whether item is a text content block with type "text" and string text
 
 @example
 ```typescript
 if (isTextContentItem(element)) {
   console.log(element.text); // string
 }
 ```
 */
function isTextContentItem(
  item: unknown,
): item is {
  readonly type: string;
  readonly text: string;
} {
  if (((typeof item) !== 'object') || (item === null))
    return false;
  // Property-existence checks narrow `object` to `object & Record<string, unknown>`
  // without an explicit type assertion
  if ((!('type' in item)) || (!('text' in item)))
    return false;
  /**
   Discriminator value the guard compares to the "text" literal.
   */
  const typeVal = item.type;
  /**
   Captured payload validated to be a string before the guard returns true.
   */
  const textVal = item.text;
  return (
    (typeVal === 'text')
      && ((typeof textVal) === 'string')
  );
}

/**
 Sentinel returned by {@link textFromContent} when content yields no text.
 A unique symbol rather than `undefined`/`''` so the no-empty-sentinel and
 no-nullish-union rules are satisfied while callers can still branch on it.
 */
const NO_TEXT: unique symbol = Symbol('morph compact content has no text',);

/**
 Extract text content from a message content field (string or content array).
 
 @param content - raw message content, either a string or structured array
 
 @returns concatenated text, or {@link NO_TEXT} if no text found
 
 @example
 ```typescript
 textFromContent("hello") // "hello"
 textFromContent([{ type: "text", text: "hi" }]) // "hi"
 ```
 */
export function textFromContent(
  content: unknown,
): string | typeof NO_TEXT {
  if ((typeof content) === 'string') {
    /**
     Cheap pre-check so all-whitespace strings collapse to the sentinel.
     */
    const trimmed = content.trim();
    if (trimmed === '')
      return NO_TEXT;
    return trimmed;
  }
  if (!Array.isArray(content,))
    return NO_TEXT;
  /**
   Joined text harvested from typed content items, or empty for none.
   */
  const result = content
    .filter(function checkItem(item,) {
      return isTextContentItem(item,);
    },)
    .map(function extractTrimmedText(item,) {
      return item.text
        .trim();
    },)
    .filter(function isNonEmpty(trimmed,) {
      return trimmed !== '';
    },)
    .join('\n',);
  if (result === '')
    return NO_TEXT;
  return result;
}

//endregion

//region Query extraction

/**
 Derive a query string for Morph Compact from the conversation.
 Checks custom instructions first, then walks branch entries backwards
 for the last user message or bash execution text.
 
 @param branchEntries - all entries on the current session branch
 
 @param customInstructions - optional user-provided instructions (takes priority)
 
 @returns query string for Morph Compact, or empty string if none found
 
 @example
 ```typescript
 const query = extractLatestQuery({
   branchEntries: entries,
   customInstructions: "focus on auth",
 });
 // Returns "focus on auth" (custom instructions take priority)
 ```
 */
export function extractLatestQuery({
  branchEntries,
  customInstructions,
}: ReadonlyDeep<{
  readonly branchEntries: readonly SessionEntry[];
  readonly customInstructions?: string;
}>,): string {
  /**
   User-supplied instructions short-circuit branch scanning when present.
   */
  const custom = customInstructions?.trim();
  if ((custom !== undefined) && (custom !== ''))
    return custom;

  for (let loopIndex = branchEntries.length
    - 1; loopIndex >= 0; loopIndex -= 1) {
    /**
     Current entry under inspection in the reverse-walk.
     */
    const entry = branchEntries[loopIndex];
    if (entry === undefined)
      continue;
    if ((entry.type
      !== 'message') || (!('message' in entry)))
      continue;
    /**
     Message payload exposed only by structurally verified message entries.
     */
    const { message, } = entry;
    if (message.role
      === 'user') {
      /**
       Concatenated user text; first non-empty result becomes the query.
       */
      const text = textFromContent(message.content,);
      if (text !== NO_TEXT)
        return text;
    }
    if (
      (message.role
        === 'bashExecution')
        && (message.command
          !== undefined)
        && (message.command
          !== '')
    ) {
      return message.command
        .trim();
    }
  }
  return '';
}

//endregion

//region Output wrapping

/**
 Wrap Morph Compact output with explanatory header and XML tags.
 The header helps the LLM understand that the content is a
 verbatim line-deleted transcript, not a structured summary.
 
 @param output - raw Morph Compact output
 
 @returns wrapped output with explanatory header and XML tags
 
 @example
 ```typescript
 const wrapped = wrapMorphOutput("[User]: hello");
 // Contains "<morph-compacted-history>...</morph-compacted-history>"
 ```
 */
export function wrapMorphOutput(output: string,): string {
  return [
    'Morph Compact verbatim transcript of earlier context.',
    'Surviving lines are exact excerpts from the original serialized conversation.',
    'Irrelevant lines were removed.',
    '',
    '<morph-compacted-history>',
    output.trim(),
    '</morph-compacted-history>',
  ]
    .join('\n',);
}

//endregion

//region Input building

/**
 Build the input string for Morph Compact.
 Wraps the previous summary in `<keepContext>` tags so Morph
 preserves it during line deletion, then appends the serialized
 conversation.
 
 @param serializedConversation - full conversation text from serializeConversation
 
 @param previousSummary - optional previous compaction summary to preserve
 
 @returns combined input string for Morph Compact
 
 @example
 ```typescript
 const input = buildMorphInput({
   serializedConversation: conversationText,
   previousSummary,
 });
 ```
 */
export function buildMorphInput({
  serializedConversation,
  previousSummary,
}: {
  readonly serializedConversation: string;
  readonly previousSummary?: string;
},): string {
  /**
   Accumulator for the optional summary block and serialized conversation.
   */
  const parts: string[] = [];
  /**
   Trimmed previous summary; treated as missing when whitespace-only.
   */
  const previous = previousSummary?.trim();
  if ((previous !== undefined) && (previous !== '')) {
    parts.push(
      '<keepContext>',
      '[Previous compacted context]',
      previous,
      '</keepContext>'
    );
  }
  if (serializedConversation.trim()
    !== '')
    parts.push(serializedConversation.trim(),);
  return parts.join('\n\n',);
}

//endregion
