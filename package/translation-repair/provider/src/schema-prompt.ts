import type { ChatMessage, } from '@monochromatic-dev/llm-llm-type/ts';

import {
  type JsonSchemaResponseFormat,
  messageText,
  type VisionMessage,
} from './chat-contract.ts';

//region Schema prompt
// The response schema, restated inside the system prompt.
//
// THE OWNER ASKED FOR THIS DIRECTLY: "Some model/provider pairs can behave
// badly w/o a detailed system prompt, including but not limited to giving wrong
// tool call formats", and "please make sure to put even the full tool schema
// into system prompts". Before that, not one of the seventeen modules that
// build a `role: 'system'` message mentioned the shape it expects back. The
// schema travelled as the API-level `response_format` field and nowhere else,
// so a model that does not honour that field had nothing to go on.
//
// DERIVED FROM THE VALUE ON THE WIRE, never hand-copied. A second spelling of
// the schema would drift from the first the day either changed, and a prompt
// that describes a shape the request no longer asks for is worse than a prompt
// that describes none.
//
// APPLIED AT THE `chatJson` SEAM rather than in each prompt builder. That seam
// is where the messages and the response format are both in hand, and it is the
// only place where "every call carrying a schema states it" can be true by
// construction. Seventeen edits would have left the seventeenth to be
// remembered, and the eighteenth prompt to be written without it.
//
// IDEMPOTENT, so a call that passes through more than one seam, or a re-route
// that rebuilds a request, states the schema once.

/**
 Line opening the block, which is also the marker that makes appending it
 idempotent.

 SEARCHED FOR RATHER THAN COUNTED: the transform asks whether a system message
 already carries this exact line, so the whole block is added once however many
 seams a request crosses.
 */
export const SCHEMA_BLOCK_HEADING = 'RESPONSE SCHEMA, which your reply must satisfy exactly:';

/**
 Indent width for the rendered schema, wide enough to read nesting.
 */
const SCHEMA_INDENT = 2;

/**
 What `findIndex` reports when a conversation carries no system message.
 */
const NO_SYSTEM_MESSAGE = -1;

/**
 Rules stated beside the schema, in the terms the observed failures broke.

 THE SECOND RULE IS NOT GENERIC ADVICE. One measured failure returned
 `{"checks": "\n[{\"region\": ...` , a JSON-STRINGIFIED ARRAY where the schema
 declares an array of objects: valid JSON of the wrong shape, which is exactly
 what a schema in the prompt is supposed to prevent.
 */
const SCHEMA_RULES: readonly string[] = [
  'FORMAT RULES. Each of these names a mistake models make here:',
  '',
  '- Reply with one JSON object and nothing else. No prose before or after it,',
  '  and no fenced code block around it.',
  '- Pass the object itself. Do not pass a string that contains JSON, and do',
  '  not escape its braces.',
  '- Every value carries the type the schema gives it, spelled as JSON. An',
  '  array is a JSON array, never a string containing one, and a number is a',
  '  number, never a string containing one.',
  '- Use exactly the property names the schema lists, spelled exactly. Do not',
  '  rename one, and do not add a property the schema does not list.',
  '- Every property the schema requires must be present, including where the',
  '  honest value is an empty string or an empty array. Omitting it is not the',
  '  same as saying it is empty.',
];

/**
 Turns the response format a call already sends into prompt text.

 @param format - response format going on the wire for this same call

 @returns Block to place inside a system prompt

 @example
 ```ts
 const block = renderSchemaForPrompt({ format: CRITIC_RESPONSE_FORMAT, },);
 ```
 */
export function renderSchemaForPrompt(
  { format, }: { readonly format: JsonSchemaResponseFormat; },
): string {
  /**
   Schema envelope, whose name a model can be asked for by name.
   */
  const { json_schema: envelope, } = format;

  return [
    SCHEMA_BLOCK_HEADING,
    '',
    `Schema name: ${envelope.name}`,
    '',
    ...SCHEMA_RULES,
    '',
    '```json',
    JSON.stringify(
      envelope.schema,
      null,
      SCHEMA_INDENT,
    ),
    '```',
  ].join('\n',);
}

/**
 Whether any system message already states a schema.

 @param messages - conversation as the caller built it

 @returns Whether the block is already present

 @example
 ```ts
 if (schemaAlreadyStated({ messages, },)) return messages;
 ```
 */
function schemaAlreadyStated(
  { messages, }: { readonly messages: readonly (ChatMessage | VisionMessage)[]; },
): boolean {
  return messages.some(function statesIt(message,): boolean {
    if (message.role !== 'system')
      return false;

    /**
     Prompt text of this system message.
     */
    const text = messageText({ message, },);

    return text.includes(SCHEMA_BLOCK_HEADING,);
  },);
}

/**
 Returns one message with the block added to whatever it already carries.

 TAKES BOTH CONTENT SHAPES, because a system message may carry a plain string
 or an array of parts, and a transform that handled only the string would drop
 the schema on exactly the calls that also send a picture.

 @param message - system message to extend

 @param block - rendered schema block

 @returns Message carrying its original content and then the block

 @example
 ```ts
 const amended = withBlockAppended({ message, block, },);
 ```
 */
function withBlockAppended(
  {
    message,
    block,
  }: {
    readonly message: ChatMessage | VisionMessage;
    readonly block: string;
  },
): ChatMessage | VisionMessage {
  /**
   Content as this message carries it, string or parts.
   */
  const { content, } = message;

  if ((typeof content) === 'string')
    return {
      ...message,
      content: `${content}\n\n${block}`,
    };

  return {
    ...message,
    content: [
      ...content,
      {
        type: 'text',
        text: block,
      },
    ],
  };
}

/**
 States the call's own response schema inside its system prompt.

 NO SYSTEM MESSAGE MEANS ONE IS ADDED rather than the schema being dropped.
 A call that asks for a schema and says nothing about it is the case this
 exists to remove, and a caller that deliberately sends no system prompt is
 still asking for a shape it never states.

 RETURNS THE SAME ARRAY when there is nothing to add, so a caller can compare
 by identity and skip rebuilding a request.

 @param messages - conversation as the caller built it

 @param responseFormat - schema this call sends, absent for free text

 @returns Conversation whose system prompt states the schema

 @example
 ```ts
 const messages = withSchemaInSystemPrompt({ messages: asked, responseFormat, },);
 ```
 */
export function withSchemaInSystemPrompt(
  {
    messages,
    responseFormat,
  }: {
    readonly messages: readonly (ChatMessage | VisionMessage)[];
    readonly responseFormat?: JsonSchemaResponseFormat;
  },
): readonly (ChatMessage | VisionMessage)[] {
  if (responseFormat === undefined)
    return messages;

  if (schemaAlreadyStated({ messages, },))
    return messages;

  /**
   Schema restated as prompt text.
   */
  const block = renderSchemaForPrompt({ format: responseFormat, },);

  /**
   Where the system message sits, or {@link NO_SYSTEM_MESSAGE}.
   */
  const at = messages.findIndex(function isSystem(message,): boolean {
    return message.role === 'system';
  },);

  if (at === NO_SYSTEM_MESSAGE)
    return [
      {
        role: 'system',
        content: block,
      },
      ...messages,
    ];

  return messages.map(function amend(
    message,
    index,
  ): ChatMessage | VisionMessage {
    if (index !== at)
      return message;
    return withBlockAppended({
      message,
      block,
    },);
  },);
}

//endregion Schema prompt
