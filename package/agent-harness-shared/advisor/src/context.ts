/**
 Builds serialized conversation context for Advisor calls.
 
 @module
 */

import type {
  Api,
  AssistantMessage,
  Model,
} from '@earendil-works/pi-ai';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  ADVISOR_MESSAGE_TYPE,
  ADVISOR_TOOL_NAME,
  CONTEXT_TRUNCATION_MARKER,
  DEFAULT_CONTEXT_OVERHEAD_TOKENS,
  TOKEN_ESTIMATE_CHARS_PER_TOKEN,
} from './constants.ts';
import {
  latestUserPromptExcerpt,
  NO_USER_PROMPT,
} from './context-user.ts';
import { buildAdvisorUserMessageText, } from './advisor-request.ts';
import type {
  AdvisorAgentMessage,
  AdvisorSessionEntry,
  AdvisorTranscriptSerialization,
} from './ports.ts';
import type {
  AdvisorConfig,
  AdvisorContext,
  AdvisorReadonlyModel,
} from './types.ts';

//region Public API

/**
 Sentinel returned by {@link entryToMessage} / {@link filterMessage} when a session
 entry contributes no message to Advisor context (filtered Advisor artifact,
 current tool call, or empty content). A `unique symbol`; callers narrow with
 `=== MESSAGE_EXCLUDED`.
 */
const MESSAGE_EXCLUDED: unique symbol = Symbol('advisor/current message excluded from context',);

/**
 Options for building Advisor context.
 */
export type BuildAdvisorContextOptions = {
  /**
   Session branch entries from the host transcript port.
   */
  readonly branch: readonly ForeignBorrowed<AdvisorSessionEntry>[];
  /**
   Transcript serialization seam used for LLM conversion and token estimates.
   */
  readonly transcript: AdvisorTranscriptSerialization;
  /**
   Runtime Advisor configuration.
   */
  readonly config: AdvisorConfig;
  /**
   Advisor-model system prompt used for token estimate.
   */
  readonly advisorSystemPrompt: string;
  /**
   Focused question supplied by the primary agent.
   */
  readonly question?: string;
  /**
   Effective serialized-context character budget.
   */
  readonly maxContextChars?: number;
  /**
   Current tool call id to omit from serialized context.
   */
  readonly toolCallId?: string;
};

/**
 Options for deriving context budget from selected Advisor model.
 */
export type MaxContextCharsForAdvisorModelOptions = {
  /**
   Transcript serialization seam used for token reserve estimates.
   */
  readonly transcript: AdvisorTranscriptSerialization;
  /**
   Runtime Advisor configuration.
   */
  readonly config: AdvisorConfig;
  /**
   Selected Advisor model.
   */
  readonly model: AdvisorReadonlyModel;
  /**
   Advisor-model system prompt used for token reserve estimate.
   */
  readonly advisorSystemPrompt: string;
  /**
   Focused question supplied by the primary agent.
   */
  readonly question?: string;
};

/**
 Build deterministic serialized context for an Advisor call.
 
 @param options - branch, config, and current tool call metadata
 
 @returns serialized context and metadata
 
 @example
 ```typescript
 const context = buildAdvisorContext({ branch, config, advisorSystemPrompt });
 ```
 */
export function buildAdvisorContext(
  options: BuildAdvisorContextOptions,
): AdvisorContext {
  /**
   Agent messages included in the secondary Advisor request.
   */
  const messages: ForeignBorrowed<AdvisorAgentMessage>[] = [];
  for (const entry of options.branch) {
    /**
     Advisor-visible message converted from current session entry.
     */
    const message = entryToMessage({
      entry,
      includePriorAdvisorResults: options.config
        .includePriorAdvisorResults,
      ...(options.toolCallId
        === undefined
        ? {}
        : { currentToolCallId: options.toolCallId, }),
    },);
    if (message !== MESSAGE_EXCLUDED)
      messages[messages.length] = message;
  }

  /**
   LLM messages converted from included session messages.
   */
  const llmMessages = options
    .transcript
    .convertToLlm(messages,);
  /**
   Serialized conversation produced by the host transcript seam.
   */
  const serialized = options
    .transcript
    .serializeConversation(llmMessages,);
  /**
   Effective truncation budget supplied by model-aware caller or config cap.
   */
  const maxContextChars = options.maxContextChars
    ?? options
      .config
      .maxContextChars
    ?? Number
      .MAX_SAFE_INTEGER;
  /**
   Truncated serialized conversation and metadata.
   */
  const truncation = truncateContext({
    text: serialized,
    maxChars: maxContextChars,
  },);
  /**
   User-message text sent to Advisor after context truncation.
   */
  const advisorUserMessageText = buildAdvisorUserMessageText({
    contextText: truncation.text,
    ...(options.question === undefined ? {} : { question: options.question, }),
  },);
  /**
   Estimated request input tokens.
   */
  const estimatedInputTokens = options
    .transcript
    .estimateInputTokens({
      systemPrompt: options.advisorSystemPrompt,
      contextText: advisorUserMessageText,
    },);

  /**
   Latest user prompt excerpt, omitted when no user prompt exists.
   */
  const latestExcerpt = latestUserPromptExcerpt(options.branch,);

  return {
    text: truncation.text,
    maxContextChars,
    originalChars: serialized.length,
    finalChars: truncation.text
      .length,
    truncated: truncation.truncated,
    includedMessageCount: messages.length,
    estimatedInputTokens,
    ...(latestExcerpt === NO_USER_PROMPT ? {} : { latestUserPromptExcerpt: latestExcerpt, }),
  };
}

/**
 Derive serialized-context character budget for selected Advisor model.
 
 @param options - config, selected model, and system prompt
 
 @returns effective context character budget after output and overhead reserves
 
 @example
 ```typescript
 maxContextCharsForAdvisorModel({ config, model, advisorSystemPrompt });
 ```
 */
export function maxContextCharsForAdvisorModel(
  options: MaxContextCharsForAdvisorModelOptions,
): number {
  /**
   Non-context user-message text reserved before serialized conversation content.
   */
  const reservedUserMessageText = buildAdvisorUserMessageText({
    contextText: '',
    ...(options.question === undefined ? {} : { question: options.question, }),
  },);
  /**
   Input tokens consumed before serialized conversation content.
   */
  const reservedInputTokens = options
    .transcript
    .estimateInputTokens({
      systemPrompt: options.advisorSystemPrompt,
      contextText: reservedUserMessageText,
    },)
    + DEFAULT_CONTEXT_OVERHEAD_TOKENS;
  /**
   Input tokens left for serialized conversation content.
   */
  const availableContextTokens = Math.max(
    1,
    options.model
      .contextWindow
      - options
        .config
        .maxAdvisorOutputTokens
      - reservedInputTokens,
  );
  /**
   Model-derived serialized conversation character budget.
   */
  const modelContextChars = Math.max(
    1,
    availableContextTokens * TOKEN_ESTIMATE_CHARS_PER_TOKEN,
  );

  return Math.min(
    options.config
      .maxContextChars
      ?? modelContextChars,
    modelContextChars,
  );
}

/**
 Truncate context with stable head and tail preservation.
 
 @param text - text to truncate
 
 @param maxChars - maximum character budget
 
 @returns possibly truncated text and flag
 
 @example
 ```typescript
 truncateContext({ text: 'abcdef', maxChars: 5 });
 ```
 */
export function truncateContext(
  {
    text,
    maxChars,
  }: {
    readonly text: string;
    readonly maxChars: number;
  },
): {
  readonly text: string;
  readonly truncated: boolean;
} {
  if (text.length
    <= maxChars) {
    return {
      text,
      truncated: false,
    };
  }

  /**
   Character budget left after the omission marker.
   */
  const remainingChars = Math.max(
    0,
    maxChars - CONTEXT_TRUNCATION_MARKER
      .length,
  );
  /**
   Head segment length.
   */
  const headChars = Math.ceil(remainingChars / 2,);
  /**
   Tail segment length.
   */
  const tailChars = remainingChars - headChars;

  /**
   Tail text, empty when no tail budget remains.
   */
  const tailText = tailChars === 0 ? '' : text.slice(-tailChars,);

  return {
    text: `${
      text.slice(
        0,
        headChars,
      )
    }${CONTEXT_TRUNCATION_MARKER}${tailText}`,
    truncated: true,
  };
}

//endregion Public API

//region Entry conversion

/**
 Convert one session entry to an Advisor-visible agent message.
 
 @param entry - session entry to convert
 
 @param includePriorAdvisorResults - whether prior Advisor results should remain visible
 
 @param currentToolCallId - current Advisor tool call id to omit
 
 @returns Advisor-visible agent message, if entry should be included
 */
function entryToMessage(
  {
    entry,
    includePriorAdvisorResults,
    currentToolCallId,
  }: {
    readonly entry: ForeignBorrowed<AdvisorSessionEntry>;
    readonly includePriorAdvisorResults: boolean;
    readonly currentToolCallId?: string;
  },
): ForeignBorrowed<AdvisorAgentMessage> | typeof MESSAGE_EXCLUDED {
  if (entry.type
    === 'message') {
    return filterMessage({
      message: entry.message,
      includePriorAdvisorResults,
      ...(currentToolCallId === undefined ? {} : { currentToolCallId, }),
    },);
  }

  if (entry.type
    === 'compaction') {
    return {
      role: 'compactionSummary',
      summary: entry.summary,
      tokensBefore: entry.tokensBefore,
      timestamp: new Date(entry.timestamp,).getTime(),
    };
  }

  if (entry.type
    === 'branch_summary') {
    return {
      role: 'branchSummary',
      summary: entry.summary,
      fromId: entry.fromId,
      timestamp: new Date(entry.timestamp,).getTime(),
    };
  }

  if (entry.type
    === 'custom_message') {
    if ((!includePriorAdvisorResults) && (entry.customType
      === ADVISOR_MESSAGE_TYPE))
      return MESSAGE_EXCLUDED;
    return {
      role: 'custom',
      customType: entry.customType,
      content: entry.content,
      display: entry.display,
      details: entry.details,
      timestamp: new Date(entry.timestamp,).getTime(),
    };
  }

  return MESSAGE_EXCLUDED;
}

/**
 Filter current or prior Advisor artifacts from one message when needed.
 
 @param message - agent message to filter
 
 @param includePriorAdvisorResults - whether prior Advisor results should remain visible
 
 @param currentToolCallId - current Advisor tool call id to omit
 
 @returns filtered message, if message should be included
 */
function filterMessage(
  {
    message,
    includePriorAdvisorResults,
    currentToolCallId,
  }: ForeignBorrowed<Readonly<{
    message: AdvisorAgentMessage;
    includePriorAdvisorResults: boolean;
    currentToolCallId?: string;
  }>>,
): ForeignBorrowed<AdvisorAgentMessage> | typeof MESSAGE_EXCLUDED {
  if (message.role
    === 'toolResult') {
    if ((currentToolCallId !== undefined) && (message.toolCallId
      === currentToolCallId))
      return MESSAGE_EXCLUDED;
    if ((!includePriorAdvisorResults) && (message.toolName
      === ADVISOR_TOOL_NAME))
      return MESSAGE_EXCLUDED;
    return message;
  }

  if (message.role
    !== 'assistant')
    return message;

  /**
   Assistant content with current Advisor placeholder tool call omitted.
   */
  const content: (typeof message.content)[number][] = [];
  for (const block of message.content) {
    /**
     Whether current block is active Advisor placeholder omitted from context.
     */
    const isCurrentAdvisorCall = (block.type
      === 'toolCall')
      && (block.name
        === ADVISOR_TOOL_NAME)
      && (block.id
        === currentToolCallId);
    if (!isCurrentAdvisorCall)
      content[content.length] = block;
  }
  if (content.length
    === 0)
    return MESSAGE_EXCLUDED;
  return {
    ...message,
    content,
  };
}

//endregion Entry conversion
