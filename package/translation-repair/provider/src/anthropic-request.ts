import type { ChatMessage, } from '@monochromatic-dev/llm-llm-type/ts';

import type { VisionMessage, } from './chat-contract.ts';

import {
  type AnthropicContentBlock,
  contentBlocksFor,
} from './anthropic-content.ts';
import {
  answerToolDefinition,
  type AnthropicToolDefinition,
  type ReadableResponseFormat,
  renderToolSystemPrompt,
} from './anthropic-tool.ts';
import {
  answerCeilingFor,
  type HyperServedId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Anthropic request
// THE BODY ONE CALL SENDS, assembled from what the pipeline already builds.
//
// Three differences from the OpenAI-compatible request, and they are the whole
// of this file:
//
//   The system prompt is a TOP-LEVEL FIELD, not a message. Every wire in this
//   pipeline opens its message array with a system message, so those are lifted
//   out here rather than rewritten upstream.
//
//   `max_tokens` is REQUIRED. The OpenAI side leaves it off by default, on the
//   measured grounds that thinking tokens count against it and a tight cap
//   truncates mid-thinking. There is no leaving it off here, so it defaults to
//   the per-model ceiling the owner decided on: the lower of the measured
//   answer bound and the model's own cap.
//
//   Structured output is a TOOL rather than a `response_format`, which
//   `anthropic-tool.ts` carries.
//
// STREAMING IS NOT A PARAMETER. The owner's instruction is that streaming is
// always on, and every stream guard reads a live stream,
// so a non-streaming body would be a call none of them cover.

/**
 Roles the Messages API accepts inside `messages`.
 */
type SpeakingRole = 'user' | 'assistant';

/**
 Refusal raised when the messages cannot form a Messages API conversation.

 @example
 ```ts
 throw new EmptyConversationError({ detail: 'no message outside the system prompt', },);
 ```
 */
export class EmptyConversationError extends Error {
  /**
   Declares this message safe to forward: it names which structural rule the request broke, never a message's content.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds failure naming what the conversation was missing.

   @param detail - which requirement the message array failed

   @example
   ```ts
   new EmptyConversationError({ detail: 'opens on an assistant turn', },);
   ```
   */
  public constructor(
    { detail, }: { readonly detail: string; },
  ) {
    super(`Messages cannot be sent to the Messages API: ${detail}`,);
    this.name = 'EmptyConversationError';
  }
}

/**
 How the model is told to choose among the tools offered.

 @example
 ```ts
 const choice: AnthropicToolChoice = { type: 'tool', name: 'repair', };
 ```
 */
export type AnthropicToolChoice =
  | {
    /**
     Discriminator marking a named tool the model must call.
     */
    readonly type: 'tool';

    /**
     Tool it is required to call.
     */
    readonly name: string;
  }
    | {
    /**
     Discriminator leaving the choice to the model.
     */
      readonly type: 'auto';
    };

/**
 One turn as the Messages API takes it.

 @example
 ```ts
 const turn: AnthropicMessage = { role: 'user', content: [{ type: 'text', text, },], };
 ```
 */
export type AnthropicMessage = {
  /**
   Who is speaking, which excludes the system role by construction.
   */
  readonly role: SpeakingRole;

  /**
   What they said, in the order the model reads it.
   */
  readonly content: readonly AnthropicContentBlock[];
};

/**
 Whole body of one Messages API call.

 @example
 ```ts
 const body: AnthropicRequestBody = buildAnthropicBody({ modelId, messages, },);
 ```
 */
export type AnthropicRequestBody = {
  /**
   Model serving this call, as this provider spells it.
   */
  readonly model: HyperServedId;

  /**
   Token ceiling, required by this protocol and never above either bound.
   */
  readonly max_tokens: number;

  /**
   Always on, so every stream guard covers this transport too.
   */
  readonly stream: true;

  /**
   Instruction and, where a schema was stated, the whole answer protocol.
   */
  readonly system: string;

  /**
   Conversation, opening on a user turn and alternating from there.
   */
  readonly messages: readonly AnthropicMessage[];

  /**
   Answer tool, absent where the caller stated no schema.
   */
  readonly tools?: readonly AnthropicToolDefinition[];

  /**
   How to choose among them, absent for the same reason.
   */
  readonly tool_choice?: AnthropicToolChoice;
};

/**
 Joins every system message into the one instruction this protocol takes.

 @param messages - conversation as the caller built it

 @returns Instruction text, empty where the caller sent no system message

 @throws {@link EmptyConversationError} where a system message carries
 pictures, which the `system` field has no shape for

 @example
 ```ts
 const instruction = systemTextOf({ messages, },);
 ```
 */
export function systemTextOf(
  { messages, }: { readonly messages: readonly (ChatMessage | VisionMessage)[]; },
): string {
  return messages
    .filter(function isSystem(message,): boolean {
      return message.role === 'system';
    },)
    .map(function toText(message,): string {
      if ((typeof message.content) !== 'string')
        throw new EmptyConversationError({
          detail: 'a system message carries content parts, which the system field cannot express',
        },);
      return message.content;
    },)
    .join('\n\n',);
}

/**
 Folds one turn into the turns before it, merging a repeated role.

 MERGING RATHER THAN REFUSING, because the OpenAI-compatible provider accepts
 consecutive same-role messages and a caller routed to either provider must be
 asked the same question by both.

 @param merged - turns folded so far

 @param turn - turn to fold in

 @returns Turns with this one appended or merged into the last

 @example
 ```ts
 const turns = converted.reduce(function fold(merged, turn,) {
   return foldTurn({ merged, turn, },);
 }, [],);
 ```
 */
function foldTurn(
  {
    merged,
    turn,
  }: {
    readonly merged: readonly AnthropicMessage[];
    readonly turn: AnthropicMessage;
  },
): readonly AnthropicMessage[] {
  /**
   Turn this one may belong to.
   */
  const previous = merged.at(-1,);

  if (previous?.role !== turn.role)
    return [
      ...merged,
      turn,
    ];

  return [
    ...merged.slice(
      0,
      -1,
    ),
    {
      role: turn.role,
      content: [
        ...previous.content,
        ...turn.content,
      ],
    },
  ];
}

/**
 Conversation the Messages API takes, system messages already lifted out.

 @param messages - conversation as the caller built it

 @returns Turns, opening on a user turn and alternating from there

 @throws {@link EmptyConversationError} where nothing is left outside the
 system prompt, or the conversation opens on an assistant turn

 @example
 ```ts
 const turns = speakingTurns({ messages, },);
 ```
 */
export function speakingTurns(
  { messages, }: { readonly messages: readonly (ChatMessage | VisionMessage)[]; },
): readonly AnthropicMessage[] {
  /**
   Every turn that is not the system prompt, converted and merged.
   */
  const turns = messages
    // FLAT-MAPPED RATHER THAN FILTERED so the surviving role narrows to a
    // speaking one by itself. A filter leaves the union intact and the role
    // would have to be asserted, which is a claim about what the filter did.
    .flatMap(function toTurn(message,): readonly AnthropicMessage[] {
      if (message.role === 'system')
        return [];

      return [
        {
          role: message.role,
          content: contentBlocksFor({ message, },),
        },
      ];
    },)
    .reduce<readonly AnthropicMessage[]>(
    function fold(
      merged: readonly AnthropicMessage[],
      turn: AnthropicMessage,
    ): readonly AnthropicMessage[] {
      return foldTurn({
        merged,
        turn,
      },);
    },
    [],
    );

  /**
   Turn the conversation opens on, which this protocol requires to be a user.
   */
  const [opening,] = turns;

  if (opening === undefined)
    throw new EmptyConversationError({
      detail: 'no message outside the system prompt, and this protocol requires at least one',
    },);

  if (opening.role !== 'user')
    throw new EmptyConversationError({
      detail: 'conversation opens on an assistant turn, and this protocol requires a user turn first',
    },);

  return turns;
}

/**
 Forces current model to call answer tool.

 @param name - answer tool model must call

 @returns Forced tool choice

 @example
 ```ts
 const choice = toolChoiceFor({ name, },);
 ```
 */
function toolChoiceFor(
  { name, }: { readonly name: string; },
): AnthropicToolChoice {
  return {
    type: 'tool',
    name,
  };
}

/**
 Tool fields of the body, for a call that stated a schema.

 BUILDS THE TOOL ONCE and names it in both fields, so `tool_choice` cannot
 force a tool spelled differently from the one offered.

 TAKES A SCHEMA RATHER THAN AN ABSENCE. Whether a call has one is decided by
 the caller that also decides the system field, and threading the absence
 through here would put that decision in two places.

 @param responseFormat - structured-output constraint caller stated

 @returns Both tool fields

 @example
 ```ts
 const fields = toolFieldsFor({ responseFormat, },);
 ```
 */
function toolFieldsFor(
  { responseFormat, }: { readonly responseFormat: ReadableResponseFormat; },
): {
  readonly tools: readonly AnthropicToolDefinition[];
  readonly tool_choice: AnthropicToolChoice;
} {
  /**
   Answer tool, whose name both fields carry.
   */
  const tool = answerToolDefinition({ responseFormat, },);

  return {
    tools: [tool,],
    tool_choice: toolChoiceFor({ name: tool.name, },),
  };
}

/**
 Assembles the whole body of one Messages API call.

 @param modelId - model serving this call

 @param messages - conversation as the caller built it, system message included

 @param responseFormat - structured-output constraint, omitted for free text

 @param maxTokens - caller's own ceiling, which only ever lowers the ask

 @returns Body to serialise

 @throws {@link EmptyConversationError} where the messages cannot form a
 conversation

 @example
 ```ts
 const body = buildAnthropicBody({ modelId, messages, responseFormat, },);
 ```

 @remarks
 NO THINKING PARAMETER AND NO TOKEN BUDGET: THE WIRE IS ANTHROPIC'S BUT THE
 PROVIDER IS CHARM HYPER. The owner's rulings of 2026-10-22: "No, I'm only
 against thinking params on 3rd tier providers." "Anthropic counts as first
 rate, neuralwatt counts as second rate." "Xiaomi and deepseek count as first
 rate." "But we should never disable thinking on anything." They narrowed the
 standing instruction of 2026-08-25 ("Please don't set any thinking parameter
 or budget tokens... These providers and models have known issues with
 non-default thinking or budget tokens and we'd rather not step on the mines"),
 which now holds for third tier alone. Anthropic itself is first rate, but
 this builder serves Hyper's route, the rulings name no tier for Hyper, and so
 it is treated as third tier and not cleared. A value that disables thinking
 is never sent, here or anywhere.

 That covers `reasoning_effort` on the OpenAI-shaped side, which is the same
 lever under the name that provider documents it by.

 Recorded at the build site rather than only in a document, because this is
 where someone would add one. `doc/audit/where-a-round-spends-its-wall-clock.md`
 carries the measurements that led here and the levers that stay open.
 */
export function buildAnthropicBody(
  {
    modelId,
    messages,
    responseFormat,
    maxTokens,
  }: {
    readonly modelId: HyperServedId;
    readonly messages: readonly (ChatMessage | VisionMessage)[];
    readonly responseFormat?: ReadableResponseFormat;
    readonly maxTokens?: number;
  },
): AnthropicRequestBody {
  /**
   Per-model ceiling the owner decided on, lowered by the caller's own.
   */
  const ceiling = answerCeilingFor({ modelId, },);

  /**
   Every system message, joined into the one field this protocol takes.
   */
  const instruction = systemTextOf({ messages, },);

  return {
    model: modelId,
    max_tokens: Math.min(
      ceiling,
      maxTokens ?? ceiling,
    ),
    stream: true,
    system: (responseFormat === undefined)
      ? instruction
      : renderToolSystemPrompt({
        instruction,
        responseFormat,
      },),
    messages: speakingTurns({ messages, },),
    // Conditional spread keeps both tool fields absent for a free-text call.
    ...((responseFormat === undefined)
      ? {}
      : toolFieldsFor({ responseFormat, },)),
  };
}

//endregion Anthropic request
