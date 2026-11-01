/**
 LLM wire-format message types used by the vendored pi-coding-agent helpers
 in {@link ./pi-utils.ts}.
 
 Defined locally (not imported from `@earendil-works/pi-ai`) because pi-ai is
 a transitive of `@earendil-works/pi-coding-agent` and is not exposed in
 morph-compact's `node_modules` under pnpm's isolated linker. Every field is
 `readonly` so the conversion helpers satisfy
 message variants are kept index-signature-free so pi's structurally-wider
 `AgentMessage` values (which carry no index signature) assign into them.
 
 @module
 */

/**
 Plain text content block in a message. Matches `TextContent` from `pi-ai`'s
 public types.
 */
export type TextContent = {
  readonly type: 'text';
  readonly text: string;
  readonly textSignature?: string;
};

/**
 Image content block. Matches `pi-ai`'s `ImageContent`. Image blocks are
 dropped from the serialized text; Morph Compact runs on text only.
 */
export type ImageContent = {
  readonly type: 'image';
  readonly data: string;
  readonly mimeType: string;
};

/**
 Thinking content block in an assistant message. Matches `pi-ai`'s
 `ThinkingContent`.
 */
export type ThinkingContent = {
  readonly type: 'thinking';
  readonly thinking: string;
  readonly thinkingSignature?: string;
  readonly redacted?: boolean;
};

/**
 Tool call block in an assistant message. Matches `pi-ai`'s `ToolCall`.
 */
export type ToolCall = {
  readonly type: 'toolCall';
  readonly id: string;
  readonly name: string;
  readonly arguments: Readonly<Record<string, unknown>>;
  readonly thoughtSignature?: string;
};

/**
 User message in LLM-compatible form. Matches `pi-ai`'s `UserMessage`.
 */
export type UserMessage = {
  readonly role: 'user';
  readonly content: string | readonly (TextContent | ImageContent)[];
  readonly timestamp: number;
};

/**
 Assistant message in LLM-compatible form. Loosely matches `pi-ai`'s
 `AssistantMessage`. Fields beyond `role`/`content`/`timestamp` exist at
 runtime (pi's variant is wider) but are unused here, so the local type lists
 only what the serializer reads.
 */
export type AssistantMessage = {
  readonly role: 'assistant';
  readonly content: readonly (TextContent | ThinkingContent | ToolCall)[];
  readonly timestamp: number;
};

/**
 Tool-result message in LLM-compatible form. Matches `pi-ai`'s
 `ToolResultMessage`.
 */
export type ToolResultMessage = {
  readonly role: 'toolResult';
  readonly content: readonly (TextContent | ImageContent)[];
  readonly timestamp: number;
};

/**
 Union of LLM-compatible message roles ({@link UserMessage}, {@link AssistantMessage},
 {@link ToolResultMessage}), equivalent to `pi-ai`'s `Message`.
 */
export type Message = UserMessage | AssistantMessage | ToolResultMessage;

/**
 Discriminated shape of a `bashExecution` `AgentMessage`. pi-coding-agent's
 `core/messages.ts` augments `CustomAgentMessages` with this role, but the
 concrete interface is not re-exported from the package root. Defining it
 locally keeps the runtime erased while letting the switch narrow fields like
 `command` and `output` without `as any`.
 */
export type BashExecutionAgentMessage = {
  readonly role: 'bashExecution';
  readonly command: string;
  readonly output: string;
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors pi-agent-core's BashExecutionMessage.exitCode, a required wire field typed `number | undefined`; under exactOptionalPropertyTypes `exitCode?: number` rejects pi's explicit-undefined value, so the union is required to accept the upstream type verbatim
  readonly exitCode: number | undefined;
  readonly cancelled: boolean;
  readonly truncated: boolean;
  readonly fullOutputPath?: string;
  readonly timestamp: number;
  readonly excludeFromContext?: boolean;
};

/**
 Discriminated shape of a `custom` `AgentMessage` augmenting role in
 pi-coding-agent. Mirrors upstream `CustomMessage`.
 */
export type CustomAgentMessage = {
  readonly role: 'custom';
  readonly customType: string;
  readonly content: string | readonly (TextContent | ImageContent)[];
  readonly display: boolean;
  readonly details?: unknown;
  readonly timestamp: number;
};

/**
 Discriminated shape of a `branchSummary` `AgentMessage` augmenting role.
 */
export type BranchSummaryAgentMessage = {
  readonly role: 'branchSummary';
  readonly summary: string;
  readonly fromId: string;
  readonly timestamp: number;
};

/**
 Discriminated shape of a `compactionSummary` `AgentMessage` augmenting role.
 */
export type CompactionSummaryAgentMessage = {
  readonly role: 'compactionSummary';
  readonly summary: string;
  readonly tokensBefore: number;
  readonly timestamp: number;
};

/**
 Discriminated union of every `AgentMessage` role this module handles. Wider
 than `pi-ai`'s `Message`: covers the custom roles pi-coding-agent layers on
 top via module augmentation ({@link BashExecutionAgentMessage}, {@link CustomAgentMessage},
 {@link BranchSummaryAgentMessage}, {@link CompactionSummaryAgentMessage}).
 */
export type AgentMessage =
  | Message
    | BashExecutionAgentMessage
    | CustomAgentMessage
    | BranchSummaryAgentMessage
    | CompactionSummaryAgentMessage;
