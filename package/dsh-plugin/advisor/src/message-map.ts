/**
 DSH message to pi-ai-shaped message conversion for the shared Advisor
 transcript port. @module
 */

import type {
  AssistantMessage,
  JsonObject,
  Message as PiMessage,
  TextContent,
  ThinkingContent,
  ToolCall,
  Usage,
} from '@earendil-works/pi-ai';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import type {
  ContentBlock,
  Message as DshMessage,
} from '@deepseek-ai/dsh-llm';
import type {
  AdvisorAgentMessage,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';
import type {
  ForeignBorrowed,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  DSH_MODEL_API,
  DSH_UNKNOWN_TOOL_NAME,
} from './constants.ts';

//region Content conversion

/**
 Image attachment facts read from one display block.
 */
type AdvisorImageAttachmentFacts = {
  /**
   Display name of the attachment, when the store recorded one.
   */
  readonly name?: string;
  /**
   Image width in pixels.
   */
  readonly width: number;
  /**
   Image height in pixels.
   */
  readonly height: number;
  /**
   Media type of the stored image.
   */
  readonly mediaType: string;
};

/**
 File attachment facts read from one display block.
 */
type AdvisorFileAttachmentFacts = {
  /**
   Display name of the stored file.
   */
  readonly name: string;
  /**
   Stored file size in bytes.
   */
  readonly bytes: number;
};

/**
 Whether one unknown attachment value carries image facts.

 @param value - raw attachment value from the display block

 @returns whether the value carries image facts
 */
function isAdvisorImageAttachmentFacts(
  value: unknown,
): value is AdvisorImageAttachmentFacts {
  return ((typeof value) === 'object')
    && (value !== null)
    && ('width' in value)
    && ('height' in value)
    && ('mediaType' in value);
}

/**
 Whether one unknown attachment value carries file facts.

 @param value - raw attachment value from the display block

 @returns whether the value carries file facts
 */
function isAdvisorFileAttachmentFacts(
  value: unknown,
): value is AdvisorFileAttachmentFacts {
  return ((typeof value) === 'object')
    && (value !== null)
    && ('name' in value)
    && ('bytes' in value);
}

/**
 Parse model-emitted tool arguments text into a JSON object.

 @param raw - raw JSON string produced by the model

 @returns parsed object with a parse diagnostic attached on failure

 @example
 ```ts
 const parsed = parseAdvisorToolArguments(raw);
 ```
 */
export function parseAdvisorToolArguments(
  raw: string,
): {
  readonly arguments: JsonObject;
  readonly argumentsError?: string;
} {
  try {
    /* oxlint-disable typescript/no-unsafe-type-assertion -- JSON.parse output for a JSON-object document is the pi-ai ToolCall.arguments type; the catch branch below covers non-object documents, so the assertion only widens JsonValue to JsonObject at the exact JSON boundary. */
    /**
     Parsed JSON document at the arguments boundary.
     */
    const parsed = JSON.parse(raw,) as JsonObject;
    /* oxlint-enable typescript/no-unsafe-type-assertion */
    if (((typeof parsed) === 'object') && (parsed !== null)
      && (!Array.isArray(parsed,)))
      return { arguments: parsed, };
    return {
      arguments: {},
      argumentsError: `arguments were not a JSON object: ${raw}`,
    };
  }
  catch (error) {
    return {
      arguments: {},
      argumentsError: `arguments were not valid JSON: ${caughtValueText(error,)}; raw: ${raw}`,
    };
  }
}

/**
 Project image or file attachment blocks onto stable placeholder text.

 DSH routes send placeholder text naming attachments instead of bytes, and
 pi-ai image content requires inline bytes the derived history does not
 carry, so attachment references stay text.

 @param block - image or file content block

 @returns placeholder text naming the attachment
 */
function attachmentPlaceholderText(
  block: ContentBlock,
): string {
  if (block.type === 'image') {
    /**
     Raw attachment value held as unknown before structural narrowing.
     */
    const rawAttachment: unknown = block.attachment;
    if (!isAdvisorImageAttachmentFacts(rawAttachment))
      throw new Error(
        `advisor: image display block carried unreadable attachment facts: ${caughtValueText(rawAttachment,)}`,
      );
    /**
     Display name of the attached image, when the attachment carries one.
     */
    const name = rawAttachment.name ?? 'attached image';
    return `[image: ${name} (${rawAttachment.width}x${rawAttachment.height}, ${rawAttachment.mediaType})]`;
  }
  if (block.type === 'file') {
    /**
     Raw attachment value held as unknown before structural narrowing.
     */
    const rawAttachment: unknown = block.attachment;
    if (!isAdvisorFileAttachmentFacts(rawAttachment))
      throw new Error(
        `advisor: file display block carried unreadable attachment facts: ${caughtValueText(rawAttachment,)}`,
      );
    return `[file: ${rawAttachment.name} (${String(rawAttachment.bytes,)} bytes)]`;
  }
  throw new Error(
    `advisor: display mapping received a non-attachment block: ${JSON.stringify(block,)}`,
  );
}

/**
 Map DSH content blocks to visible pi-ai text content.

 Attachment, tool-call, and reasoning blocks project to placeholder text so
 review context keeps a truthful account of what the model saw; pi-ai image
 content requires inline bytes the derived history does not carry.

 @param blocks - DSH message content blocks

 @returns pi-ai text content blocks

 @example
 ```ts
 const visible = mapDshDisplayContent(message.content);
 ```
 */
export function mapDshDisplayContent(
  blocks: readonly ForeignBorrowed<ContentBlock>[],
): TextContent[] {
  /**
   Visible content collected in block order.
   */
  const content: TextContent[] = [];
  for (const block of blocks) {
    if (block.type === 'text')
      content.push({
        type: 'text',
        text: block.text,
      },);
    else if ((block.type === 'image') || (block.type === 'file'))
      content.push({
        type: 'text',
        text: attachmentPlaceholderText(block,),
      },);
    else if (block.type === 'tool-call')
      content.push({
        type: 'text',
        text: `[tool call: ${block.name}(${block.arguments})]`,
      },);
  }
  return content;
}

/**
 Map DSH content blocks to pi-ai assistant content.

 `text`, `reasoning`, and `tool-call` map structurally; `tool-addition` and
 `tool-removal` have no pi-ai counterpart and are dropped.

 @param blocks - DSH assistant message content blocks

 @returns pi-ai assistant content blocks

 @example
 ```ts
 const content = mapDshAssistantContent(message.content);
 ```
 */
export function mapDshAssistantContent(
  blocks: readonly ForeignBorrowed<ContentBlock>[],
): (ForeignBorrowed<TextContent> | ForeignBorrowed<ThinkingContent> | ForeignBorrowed<ToolCall>)[] {
  /**
   Assistant content collected in block order.
   */
  const content: (ForeignBorrowed<TextContent> | ForeignBorrowed<ThinkingContent> | ForeignBorrowed<ToolCall>)[] = [];
  for (const block of blocks) {
    if (block.type === 'text')
      content.push({
        type: 'text',
        text: block.text,
      },);
    else if (block.type === 'reasoning')
      content.push({
        type: 'thinking',
        thinking: block.text,
      },);
    else if (block.type === 'tool-call') {
      /**
       Parsed tool arguments with parse diagnostic, when malformed.
       */
      const parsed = parseAdvisorToolArguments(block.arguments,);
      content.push({
        type: 'toolCall',
        id: block.id,
        name: block.name,
        arguments: parsed.arguments,
        ...(parsed.argumentsError === undefined
          ? {}
          : { argumentsError: parsed.argumentsError, }),
      },);
    }
  }
  return content;
}

//endregion Content conversion

//region Message conversion

/**
 Usage stand-in for messages the derived history records without provider
 accounting.

 DSH derived messages carry no usage; the shared core reads usage only from
 live completion results, so history placeholders stay zero-valued.

 @returns zero-valued pi-ai usage record

 @example
 ```ts
 const usage = emptyAdvisorUsage();
 ```
 */
export function emptyAdvisorUsage(): Usage {
  return {
    input: 0,
    output: 0,
    cacheRead: 0,
    cacheWrite: 0,
    totalTokens: 0,
    cost: {
      input: 0,
      output: 0,
      cacheRead: 0,
      cacheWrite: 0,
      total: 0,
    },
  };
}

/**
 Convert one derived DSH message to a pi-ai-shaped Advisor agent message.

 Tool results resolve their tool name from the call-id map built over earlier
 assistant messages, so prior Advisor result filtering keeps parity.

 @param message - derived DSH session message to convert

 @param toolNamesByCallId - tool names indexed by provider-issued call id

 @param timestampMs - snapshot timestamp recorded on the converted message

 @returns pi-ai-shaped Advisor agent message

 @throws when the derived history carries a role the pinned vocabulary lacks

 @example
 ```ts
 const message = mapDshMessageToAdvisorMessage({
   message,
   toolNamesByCallId,
   timestampMs,
 });
 ```
 */
export function mapDshMessageToAdvisorMessage(
  {
    message,
    toolNamesByCallId,
    timestampMs,
  }: {
    readonly message: DshMessage;
    readonly toolNamesByCallId: ReadonlyMap<string, string>;
    readonly timestampMs: number;
  },
): AdvisorAgentMessage {
  if (message.role === 'system') {
    return {
      role: 'system',
      content: mapDshDisplayContent(message.content,),
      timestamp: timestampMs,
    };
  }
  if (message.role === 'developer') {
    return {
      role: 'system',
      content: mapDshDisplayContent(message.content,),
      timestamp: timestampMs,
    };
  }
  if (message.role === 'user') {
    return {
      role: 'user',
      content: mapDshDisplayContent(message.content,),
      timestamp: timestampMs,
    };
  }
  if (message.role === 'assistant') {
    /**
     pi-ai-shaped assistant content for this response.
     */
    const content = mapDshAssistantContent(message.content,);
    return {
      role: 'assistant',
      content,
      api: DSH_MODEL_API,
      provider: message.source
        .provider,
      model: message.source
        .model,
      usage: emptyAdvisorUsage(),
      stopReason: content.some(function hasToolCall(block,) {
        return block.type === 'toolCall';
      },)
        ? 'toolUse'
        : 'stop',
      timestamp: timestampMs,
    } satisfies AssistantMessage;
  }
  if (message.role === 'tool') {
    return {
      role: 'toolResult',
      toolCallId: message.toolCallId,
      toolName: toolNamesByCallId.get(message.toolCallId,) ?? DSH_UNKNOWN_TOOL_NAME,
      content: mapDshDisplayContent(message.content,),
      isError: message.isError ?? false,
      timestamp: timestampMs,
    };
  }
  /**
   Unreachable under the pinned dsh-llm message vocabulary; a new role needs a
   deliberate mapping here rather than a silent drop.
   */
  throw new Error(
    `advisor: derived session message carries unmapped role: ${JSON.stringify(message,)}`,
  );
}

/**
 Collect tool-call names by call id over the derived history.

 @param messages - derived DSH session messages

 @returns call-id to tool-name map for tool-result name resolution

 @example
 ```ts
 const names = collectAdvisorToolNames(messages);
 ```
 */
export function collectAdvisorToolNames(
  messages: readonly ForeignBorrowed<DshMessage>[],
): Map<string, string> {
  /**
   Tool names indexed by provider-issued call id.
   */
  const names = new Map<string, string>();
  for (const message of messages) {
    if (message.role !== 'assistant')
      continue;
    for (const block of message.content) {
      if (block.type === 'tool-call')
        names.set(
          block.id,
          block.name,
        );
    }
  }
  return names;
}

/**
 Type guard over the pi-ai message role union used by transcript filtering.

 @param message - Advisor agent message to test

 @returns whether the message is a pi-ai message role

 @example
 ```ts
 const kept = messages.filter((message) => isPiAgentMessage(message));
 ```
 */
export function isPiAgentMessage(
  message: AdvisorAgentMessage,
): message is PiMessage {
  return (message.role === 'system')
    || (message.role === 'user')
    || (message.role === 'assistant')
    || (message.role === 'toolResult');
}

//endregion Message conversion
