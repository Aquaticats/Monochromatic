/**
 Host ports the neutral Advisor core reads: model resolution, transcript, and theme.

 The core never imports a harness host. Harness adapters implement these
 three ports from their own runtime objects (pi passes `ModelRegistry`,
 `SessionManager`, and `Theme`; a DSH adapter passes its equivalents), and
 the shared pipeline below behaves identically for every implementation.

 @module
 */

import type {
  Api,
  ImageContent,
  Message,
  Model,
  Provider,
  ProviderHeaders,
  TextContent,
} from '@earendil-works/pi-ai';
import type {
  ForeignBorrowed,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

//region Model-resolution port

/**
 Request authentication resolved for one provider call.
 Mirrors the host registry's resolution result shape.
 */
export type AdvisorRequestAuth = {
  /**
   Whether authentication resolved.
   */
  readonly ok: true;
  /**
   API key to send, when the host resolves one.
   */
  readonly apiKey?: string;
  /**
   Additional provider headers to send, when the host resolves any.
   */
  readonly headers?: ProviderHeaders;
} | {
  /**
   Whether authentication resolved.
   */
  readonly ok: false;
  /**
   Host authentication diagnostic surfaced in the failure record.
   */
  readonly error: string;
};

/**
 Model-resolution port: the host model-registry slice the Advisor core reads.

 Covers scope reads (`getAvailable`), explicit-slug lookup (`getAll`),
 provider dispatch (`getProvider`), and request authentication
 (`getApiKeyAndHeaders`). A host `ModelRegistry` satisfies this
 structurally, so adapters pass their registry object unchanged.

 @example
 ```typescript
 const modelResolution = piExtensionContext.modelRegistry;
 ```
 */
export type AdvisorModelResolution = {
  /**
   Models with configured auth, used for effective scope resolution.
   */
  getAvailable: () => readonly Model<Api>[];
  /**
   Every registry model, used to classify out-of-scope explicit slugs.
   */
  getAll: () => readonly Model<Api>[];
  /* oxlint-disable no-restricted-syntax/no-nullish-union -- mirrors pi-coding-agent ModelRegistry.getProvider (`dist/core/model-registry.d.ts`), which returns undefined for an unregistered provider id; the core branches on that absence, so the mirror cannot restate it */
  /**
   Registered provider stream implementation for a provider id.
   */
  getProvider: (provider: string) => Provider | undefined;
  /* oxlint-enable no-restricted-syntax/no-nullish-union */
  /**
   Resolve request authentication for one model.
   */
  getApiKeyAndHeaders: (model: Model<Api>) => Promise<AdvisorRequestAuth>;
};

//endregion Model-resolution port

//region Transcript port

/**
 Session entry with a user or assistant message.
 Carries the transcript discriminator `type: 'message'`.
 */
export type AdvisorMessageSessionEntry = {
  /**
   Transcript discriminator.
   */
  readonly type: 'message';
  /**
   Agent message recorded for this entry.
   */
  readonly message: AdvisorAgentMessage;
  /**
   Host entry timestamp.
   */
  readonly timestamp: string;
};

/**
 Session entry carrying a compaction summary.
 */
export type AdvisorCompactionSessionEntry = {
  /**
   Transcript discriminator.
   */
  readonly type: 'compaction';
  /**
   Compaction summary text.
   */
  readonly summary: string;
  /**
   Token count before compaction.
   */
  readonly tokensBefore: number;
  /**
   Host entry timestamp.
   */
  readonly timestamp: string;
};

/**
 Session entry carrying a branch summary.
 */
export type AdvisorBranchSummarySessionEntry = {
  /**
   Transcript discriminator.
   */
  readonly type: 'branch_summary';
  /**
   Branch summary text.
   */
  readonly summary: string;
  /* oxlint-disable no-restricted-syntax/no-nullish-union -- mirrors pi-coding-agent BranchSummaryMessage.fromId (`dist/core/messages.d.ts`) and the session entry it wraps, which record null when unrecorded; the core forwards the value unchanged */
  /**
   Branched-from entry id, or null when unrecorded.
   */
  readonly fromId: string | null;
  /* oxlint-enable no-restricted-syntax/no-nullish-union */
  /**
   Host entry timestamp.
   */
  readonly timestamp: string;
};

/**
 Session entry carrying an extension-injected custom message.
 */
export type AdvisorCustomMessageSessionEntry = {
  /**
   Transcript discriminator.
   */
  readonly type: 'custom_message';
  /**
   Extension identity for the injected message.
   */
  readonly customType: string;
  /**
   Injected content.
   */
  readonly content: string | (TextContent | ImageContent)[];
  /**
   Whether the host renders this message.
   */
  readonly display: boolean;
  /**
   Extension metadata excluded from model context.
   */
  readonly details?: unknown;
  /**
   Host entry timestamp.
   */
  readonly timestamp: string;
};

/**
 Session entries the Advisor context filter passes over untouched.
 The discriminants enumerate host entry kinds that contribute no message.
 */
export type AdvisorOpaqueSessionEntry = {
  /**
   Transcript discriminator of an entry kind the filter ignores.
   */
  readonly type:
    | 'thinking_level_change'
      | 'model_change'
      | 'usage'
      | 'custom'
      | 'context_edit'
      | 'label'
      | 'session_info';
};

/**
 Transcript port session entry: the host session-entry union as the
 Advisor core reads it. A host `SessionEntry` union satisfies this
 structurally, so adapters pass their entries unchanged.

 @example
 ```typescript
 const branch: readonly ForeignBorrowed<AdvisorSessionEntry>[] = sessionManager.buildContextEntries();
 ```
 */
export type AdvisorSessionEntry =
  | AdvisorMessageSessionEntry
    | AdvisorCompactionSessionEntry
    | AdvisorBranchSummarySessionEntry
    | AdvisorCustomMessageSessionEntry
    | AdvisorOpaqueSessionEntry;

/**
 Extension-injected custom message the filter forwards to the LLM seam.
 */
export type AdvisorCustomAgentMessage = {
  /**
   Message role discriminator.
   */
  readonly role: 'custom';
  /**
   Extension identity for the injected message.
   */
  readonly customType: string;
  /**
   Injected content.

   The inner array stays mutable to mirror pi-coding-agent
   `CustomMessage.content` in both directions under strict function-type
   variance.
   */
  readonly content: string | (TextContent | ImageContent)[];
  /**
   Whether the host renders this message.
   */
  readonly display: boolean;
  /**
   Extension metadata excluded from model context.
   */
  readonly details?: unknown;
  /**
   Host message timestamp in milliseconds.
   */
  readonly timestamp: number;
};

/**
 Branch-summary message the filter forwards to the LLM seam.
 */
export type AdvisorBranchSummaryAgentMessage = {
  /**
   Message role discriminator.
   */
  readonly role: 'branchSummary';
  /**
   Branch summary text.
   */
  readonly summary: string;
  /* oxlint-disable no-restricted-syntax/no-nullish-union -- mirrors pi-coding-agent BranchSummaryMessage.fromId (`dist/core/messages.d.ts`), which records null when unrecorded; the core forwards the value unchanged */
  /**
   Branched-from entry id, or null when unrecorded.
   */
  readonly fromId: string | null;
  /* oxlint-enable no-restricted-syntax/no-nullish-union */
  /**
   Host message timestamp in milliseconds.
   */
  readonly timestamp: number;
};

/**
 Compaction-summary message the filter forwards to the LLM seam.
 */
export type AdvisorCompactionSummaryAgentMessage = {
  /**
   Message role discriminator.
   */
  readonly role: 'compactionSummary';
  /**
   Compaction summary text.
   */
  readonly summary: string;
  /**
   Token count before compaction.
   */
  readonly tokensBefore: number;
  /**
   Host message timestamp in milliseconds.
   */
  readonly timestamp: number;
};

/**
 Agent message of a host role the filter forwards untouched.

 Mirrors pi-coding-agent `BashExecutionMessage` (`dist/core/messages.d.ts`)
 field for field so host unions satisfy the port in both directions.
 */
export type AdvisorOpaqueAgentMessage = {
  /**
   Message role discriminator of a role the filter does not inspect.
   */
  readonly role: 'bashExecution';
  /**
   Shell command text.
   */
  readonly command: string;
  /**
   Captured command output.
   */
  readonly output: string;
  /* oxlint-disable no-restricted-syntax/no-nullish-union -- mirrors pi-coding-agent BashExecutionMessage.exitCode (`dist/core/messages.d.ts`), which is absent-valued on cancellation; the filter forwards the value unchanged */
  /**
   Process exit code, or undefined when the run was cancelled.
   */
  readonly exitCode: number | undefined;
  /* oxlint-enable no-restricted-syntax/no-nullish-union */
  /**
   Whether the run was cancelled.
   */
  readonly cancelled: boolean;
  /**
   Whether captured output was truncated.
   */
  readonly truncated: boolean;
  /**
   Host path holding the full output when truncation applied.
   */
  readonly fullOutputPath?: string;
  /**
   Host message timestamp in milliseconds.
   */
  readonly timestamp: number;
};

/**
 Transcript-port agent message: the host agent-message union as the
 Advisor core reads it. A host `AgentMessage` union (pi-ai messages plus
 host custom roles) satisfies this structurally.
 */
export type AdvisorAgentMessage =
  | Message
    | AdvisorCustomAgentMessage
    | AdvisorBranchSummaryAgentMessage
    | AdvisorCompactionSummaryAgentMessage
    | AdvisorOpaqueAgentMessage;

/**
 Transcript serialization seam: host LLM conversion, serialization, and
 token estimation. These three functions are the only transcript behavior
 the core delegates; filtering and budgeting stay in the core.
 */
export type AdvisorTranscriptSerialization = {
  /**
   Convert host agent messages to LLM-compatible messages.

   Parameters and returns stay mutable to match the host conversion
   functions (pi-coding-agent `convertToLlm`) under strict function-type
   variance, so hosts pass their converter unchanged.
   */
  convertToLlm: (messages: AdvisorAgentMessage[]) => Message[];
  /**
   Serialize LLM messages into the request conversation text.

   The parameter stays mutable to match the host serializer
   (pi-coding-agent `serializeConversation`) under strict function-type
   variance, so hosts pass their serializer unchanged.
   */
  serializeConversation: (messages: Message[]) => string;
  /**
   Estimate request input tokens for a system prompt and serialized context.
   */
  estimateInputTokens: (options: {
    /**
     Advisor-model system prompt.
     */
    readonly systemPrompt: string;
    /**
     Serialized conversation context.
     */
    readonly contextText: string;
  }) => number;
};

/**
 Transcript port: session snapshot reads plus the serialization seam.

 `buildContextEntries` returns the frozen session snapshot the operation
 builds once per run; the core never re-reads the session mid-operation.

 @example
 ```typescript
 const branch = transcript.buildContextEntries();
 ```
 */
export type AdvisorTranscript = AdvisorTranscriptSerialization & {
  /**
   Current session entries included in model context.
   */
  buildContextEntries: () => readonly ForeignBorrowed<AdvisorSessionEntry>[];
};

//endregion Transcript port

//region Theme port

/**
 Semantic foreground tokens the core's plain-text renderers request.

 Each theme adapter maps the tokens to its own colors; the set is the
 closed vocabulary the core may ask for. Every member is a foreground
 token of pi-coding-agent `ThemeColor`, so host themes satisfy the port
 under strict function-type variance.
 */
export type AdvisorThemeColor =
  | 'accent'
    | 'dim'
    | 'error'
    | 'success'
    | 'toolOutput'
    | 'toolTitle'
    | 'warning';

/**
 Theme port: the styling surface plain-text renderers call.

 A host `Theme` satisfies this structurally: its color method takes the
 host token union, a supertype of {@link AdvisorThemeColor}. A headless
 adapter may implement `fg` and `bold` as identity functions.

 @example
 ```typescript
 theme.fg('accent', selectedSlug);
 ```
 */
export type AdvisorTheme = {
  /**
   Wrap text in a semantic foreground token.
   */
  fg: (
    color: AdvisorThemeColor,
    text: string,
  ) => string;
  /**
   Wrap text in bold emphasis.
   */
  bold: (text: string) => string;
};

//endregion Theme port
