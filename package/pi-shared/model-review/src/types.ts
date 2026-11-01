/**
 Public data contracts for shared structured model review.
 
 @module
 */

import type {
  Api,
  AssistantMessageEvent,
  Model,
  ProviderHeaders,
  Tool,
} from '@earendil-works/pi-ai';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

/**
 Provider credentials resolved before review transport.
 
 @example
 ```ts
 const auth: StructuredReviewAuth = { apiKey: 'token' };
 ```
 */
type StructuredReviewAuth = {
  /**
   Provider API key when required.
   */
  readonly apiKey?: string;
  /**
   Provider headers when required.
   */
  readonly headers?: ForeignBorrowed<ProviderHeaders>;
};

/**
 One provider prompt used by structured or direct-JSON transport.
 
 @example
 ```ts
 const prompt: StructuredReviewPrompt = {
   systemPrompt: 'Judge independently.',
   userContent: 'Review this evidence.',
 };
 ```
 */
type StructuredReviewPrompt = {
  /**
   Reviewer system instructions.
   */
  readonly systemPrompt: string;
  /**
   Reviewer user-message body.
   */
  readonly userContent: string;
};

/**
 Initial forced-tool response before caller-specific parsing or retry policy.
 
 @example
 ```ts
 if (result.kind === 'toolCall') parseVerdict(result.arguments);
 ```
 */
type StructuredReviewInitialResult =
  | {
    /**
     Expected tool-call discriminant.
     */
    readonly kind: 'toolCall';
    /**
     Unknown arguments retained for caller-owned strict parsing.
     */
    readonly arguments: unknown;
  }
    | {
    /**
     Missing-tool discriminant.
     */
      readonly kind: 'noToolCall';
    /**
     Finalized text used to construct caller-specific retry instructions.
     */
      readonly textContent: string;
    };

/**
 Isolated model identity recorded at provider request seam.
 
 @example
 ```ts
 const model: StructuredReviewModelSnapshot = {
   api: 'openai-responses',
   id: 'reviewer',
   provider: 'openai',
 };
 ```
 */
type StructuredReviewModelSnapshot = {
  /**
   Selected provider API.
   */
  readonly api: Api;
  /**
   Selected model identifier.
   */
  readonly id: string;
  /**
   Selected provider identifier.
   */
  readonly provider: string;
};

/**
 Isolated message recorded at provider request seam.
 
 @example
 ```ts
 const message: StructuredReviewMessageSnapshot = {
   role: 'user',
   content: 'Evidence',
   timestamp: 1,
 };
 ```
 */
type StructuredReviewMessageSnapshot = {
  /**
   Message role sent to provider.
   */
  readonly role: 'user';
  /**
   Exact text sent to provider.
   */
  readonly content: string;
  /**
   Request message timestamp.
   */
  readonly timestamp: number;
};

/**
 Isolated context recorded immediately before provider dispatch.
 
 @example
 ```ts
 const context: StructuredReviewContextSnapshot = {
   systemPrompt: 'Judge.',
   messages: [],
   toolNames: [],
 };
 ```
 */
type StructuredReviewContextSnapshot = {
  /**
   Exact provider system prompt.
   */
  readonly systemPrompt: string;
  /**
   Exact primitive message projection.
   */
  readonly messages: readonly StructuredReviewMessageSnapshot[];
  /**
   Tool names exposed for this request.
   */
  readonly toolNames: readonly string[];
};

/**
 Isolated stream options recorded immediately before provider dispatch.
 
 @example
 ```ts
 const options: StructuredReviewOptionsSnapshot = { hasSignal: true };
 ```
 */
type StructuredReviewOptionsSnapshot = {
  /**
   Provider API key when present.
   */
  readonly apiKey?: string;
  /**
   Isolated provider headers when present.
   */
  readonly headers?: Readonly<Record<string, unknown>>;
  /**
   Whether cancellation signal reached provider seam.
   */
  readonly hasSignal: boolean;
  /**
   Provider output cap when present.
   */
  readonly maxTokens?: number;
  /**
   Primitive forced-tool selector type when present.
   */
  readonly toolChoiceType?: string;
  /**
   Forced tool name when provider selector carries one.
   */
  readonly toolChoiceName?: string;
};

/**
 Complete isolated provider request snapshot used by deterministic tests.
 
 @example
 ```ts
 testTransport.requests[0]?.context.messages[0]?.content;
 ```
 */
type StructuredReviewRequestSnapshot = {
  /**
   Selected provider identity.
   */
  readonly model: StructuredReviewModelSnapshot;
  /**
   Final provider context.
   */
  readonly context: StructuredReviewContextSnapshot;
  /**
   Final provider stream options.
   */
  readonly options: StructuredReviewOptionsSnapshot;
};

/**
 Data-only deterministic provider seam.
 
 Production callers omit this value. Tests provide ordered streams and inspect
 isolated request snapshots without passing executable callbacks into review code.
 
 @example
 ```ts
 const transport: ScriptedStructuredReviewTransport = {
   nextResponseIndex: 0,
   responses: [stream],
   requests: [],
 };
 ```
 */
type ScriptedStructuredReviewTransport = {
  /**
   Index consumed by next request.
   */
  nextResponseIndex: number;
  /**
   Ordered deterministic response streams.
   */
  readonly responses: readonly ForeignBorrowed<AsyncIterable<AssistantMessageEvent>>[];
  /**
   Isolated requests captured in dispatch order.
   */
  requests: StructuredReviewRequestSnapshot[];
};

/**
 Shared request fields for one candidate attempt.
 
 @example
 ```ts
 const request: StructuredReviewRequest = { model, auth, prompt, signal };
 ```
 */
type StructuredReviewRequest = {
  /**
   Selected reviewer model.
   */
  readonly model: ForeignBorrowed<Model<Api>>;
  /**
   Resolved reviewer credentials.
   */
  readonly auth: ForeignBorrowed<StructuredReviewAuth>;
  /**
   Request-specific reviewer prompt.
   */
  readonly prompt: StructuredReviewPrompt;
  /**
   Cancellation signal shared across initial request and retries.
   */
  readonly signal: AbortSignal;
  /**
   Optional provider output cap.
   */
  readonly maxOutputTokens?: number;
  /**
   Optional data-only deterministic provider seam.
   */
  readonly testTransport?: ForeignBorrowed<ScriptedStructuredReviewTransport>;
};

/**
 Initial forced-tool request fields.
 
 @example
 ```ts
 await runStructuredToolRequest({ ...request, toolName: tool.name, tool });
 ```
 */
type StructuredReviewToolRequest = StructuredReviewRequest & {
  /**
   Exact tool name expected from provider.
   */
  readonly toolName: string;
  /**
   Sole structured verdict tool exposed to provider.
   */
  readonly tool: ForeignBorrowed<Tool>;
};

/**
 Direct-JSON retry request fields.
 
 @example
 ```ts
 await runStructuredJsonRetries({ ...request, expectedToolName: 'submit_review' });
 ```
 */
type StructuredReviewJsonRequest = StructuredReviewRequest & {
  /**
   Expected tool name tolerated if provider emits one during JSON retry.
   */
  readonly expectedToolName: string;
};

export type {
  ScriptedStructuredReviewTransport,
  StructuredReviewAuth,
  StructuredReviewContextSnapshot,
  StructuredReviewInitialResult,
  StructuredReviewJsonRequest,
  StructuredReviewMessageSnapshot,
  StructuredReviewModelSnapshot,
  StructuredReviewOptionsSnapshot,
  StructuredReviewPrompt,
  StructuredReviewRequest,
  StructuredReviewRequestSnapshot,
  StructuredReviewToolRequest,
};
