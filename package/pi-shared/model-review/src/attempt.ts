/**
 Callback-free structured reviewer request transport.
 
 @module
 */

import type {
  Context,
  Tool,
} from '@earendil-works/pi-ai';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type {
  ForeignBorrowed,
  ForeignHostCapability,
} from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ReviewSimpleStreamOptions, } from './provider-data.ts';
import { streamStructuredReview, } from './provider-streams.ts';
import {
  collectDirectJson,
  collectStructuredStream,
  EmptyStructuredReviewTextError,
} from './stream-collection.ts';
import { toolChoiceForApi, } from './tool-choice.ts';
import type {
  StructuredReviewAuth,
  StructuredReviewInitialResult,
  StructuredReviewJsonRequest,
  StructuredReviewPrompt,
  StructuredReviewRequest,
  StructuredReviewToolRequest,
} from './types.ts';

/**
 Shared package logger.
 */
const parentLogger = tagged({ tag: 'model-review', },);

/**
 Attempt transport logger.
 */
const l = tagged({
  tag: 'attempt',
  l: parentLogger,
},);

/**
 Build cancellation signal covering caller abort and complete attempt timeout.
 
 Caller creates this once and shares it across forced-tool and JSON requests,
 preserving one deadline across complete candidate attempt.
 
 @param signal - optional caller cancellation signal
 
 @param timeoutMs - complete attempt timeout
 
 @returns composed abort signal
 
 @mutates signal - DOM dependent-signal relations can retain caller signal
 
 @example
 ```ts
 structuredReviewSignal({ timeoutMs: 10_000 });
 ```
 */
function structuredReviewSignal(
  {
    signal,
    timeoutMs,
  }: {
    readonly signal?: ForeignHostCapability<AbortSignal>;
    readonly timeoutMs: number;
  },
): AbortSignal {
  /**
   Timeout-only abort signal.
   */
  const timeoutSignal = AbortSignal.timeout(timeoutMs,);
  return signal === undefined
    ? timeoutSignal
    : AbortSignal.any([
      signal,
      timeoutSignal,
    ],);
}

/**
 Build Pi AI stream options without widening absent fields.
 
 @param auth - resolved provider credentials
 
 @param signal - complete attempt cancellation signal
 
 @param toolChoice - optional provider-specific forced selector
 
 @param maxOutputTokens - optional output cap
 
 @returns final provider stream options
 
 @example
 ```ts
 buildAttemptStreamOptions({ auth: {}, signal: AbortSignal.timeout(1000) });
 ```
 */
function buildAttemptStreamOptions(
  {
    auth,
    signal,
    toolChoice,
    maxOutputTokens,
  }: {
    readonly auth: ForeignBorrowed<StructuredReviewAuth>;
    readonly signal: AbortSignal;
    readonly toolChoice?: unknown;
    readonly maxOutputTokens?: number;
  },
): ReviewSimpleStreamOptions {
  return {
    signal,
    ...(auth.apiKey === undefined ? {} : { apiKey: auth.apiKey, }),
    ...(auth.headers === undefined ? {} : { headers: { ...auth.headers, }, }),
    ...(toolChoice === undefined ? {} : { toolChoice, }),
    ...(maxOutputTokens === undefined ? {} : { maxTokens: maxOutputTokens, }),
  };
}

/**
 Build one-message provider context from caller prompt.
 
 @param prompt - reviewer system and user content
 
 @param tool - optional structured tool for initial request
 
 @returns Pi AI provider context
 
 @example
 ```ts
 contextForPrompt({ prompt: { systemPrompt: 'Judge.', userContent: 'Evidence' } });
 ```
 */
function contextForPrompt(
  {
    prompt,
    tool,
  }: ForeignBorrowed<Readonly<{
    readonly prompt: StructuredReviewPrompt;
    readonly tool?: Readonly<Tool>;
  }>>,
): Context {
  return {
    systemPrompt: prompt.systemPrompt,
    messages: [{
      role: 'user',
      content: prompt.userContent,
      timestamp: Date.now(),
    },],
    ...(tool === undefined ? {} : { tools: [tool,], }),
  };
}

/**
 Dispatch one direct-JSON request and parse its text object.
 
 @param request - callback-free request data
 
 @returns unknown JSON object for caller-owned strict parsing
 
 @mutates request - provider consumes model, auth, signal, and test transport capabilities
 
 @throws {@link EmptyStructuredReviewTextError} when provider emits no text
 
 @example
 ```ts
 await runDirectJsonRequest(request);
 ```
 */
function runDirectJsonRequest(
  request: ForeignBorrowed<StructuredReviewJsonRequest>,
): Promise<unknown> {
  /**
   Final direct-JSON provider options.
   */
  const options = buildAttemptStreamOptions({
    auth: request.auth,
    signal: request.signal,
    ...(request.maxOutputTokens === undefined
      ? {}
      : { maxOutputTokens: request.maxOutputTokens, }),
  },);
  /**
   Direct-JSON provider stream.
   */
  const stream = streamStructuredReview({
    model: request.model,
    context: contextForPrompt({ prompt: request.prompt, },),
    options,
    ...(request.testTransport === undefined
      ? {}
      : { testTransport: request.testTransport, }),
  },);
  return collectDirectJson({
    stream,
    expectedToolName: request.expectedToolName,
  },);
}

/**
 Run initial forced-tool request without caller-specific parsing or retry logic.
 
 @param request - model, auth, prompt, tool, signal, and optional test script data
 
 @returns tool arguments or omitted-tool text
 
 @mutates request - provider consumes model, auth, signal, and test transport capabilities
 
 @example
 ```ts
 const result = await runStructuredToolRequest(request);
 ```
 */
async function runStructuredToolRequest(
  request: ForeignBorrowed<StructuredReviewToolRequest>,
): Promise<StructuredReviewInitialResult> {
  /**
   Per-call logger carrying selected reviewer identity.
   */
  const innerL = tagged({
    tag: runStructuredToolRequest.name,
    l,
  },);
  innerL.debug(
    `starting structured request with ${request.model
      .provider}/${request.model
        .id} using ${request.toolName}`,
  );
  /**
   Initial forced-tool provider options.
   */
  const options = buildAttemptStreamOptions({
    auth: request.auth,
    signal: request.signal,
    toolChoice: toolChoiceForApi({
      api: request.model
        .api,
      toolName: request.toolName,
    },),
    ...(request.maxOutputTokens === undefined
      ? {}
      : { maxOutputTokens: request.maxOutputTokens, }),
  },);
  /**
   Initial forced-tool event stream.
   */
  const stream = streamStructuredReview({
    model: request.model,
    context: contextForPrompt({
      prompt: request.prompt,
      tool: request.tool,
    },),
    options,
    ...(request.testTransport === undefined
      ? {}
      : { testTransport: request.testTransport, }),
  },);
  /**
   Unparsed initial provider result.
   */
  const result = await collectStructuredStream({
    stream,
    expectedToolName: request.toolName,
  },);
  if (result.kind === 'noToolCall')
    innerL.warn(`reviewer omitted ${request.toolName}; caller must retry with direct JSON`,);
  return result;
}

/**
 Run one direct-JSON request, retrying once only when first response is empty.
 
 Caller builds prompt from initial omitted-tool text before calling this function.
 
 @param request - model, auth, complete retry prompt, signal, and optional script data
 
 @returns unknown JSON object for caller-owned strict parsing
 
 @mutates request - provider consumes model, auth, signal, and test transport capabilities
 
 @example
 ```ts
 const value = await runStructuredJsonRetries(request);
 ```
 */
async function runStructuredJsonRetries(
  request: ForeignBorrowed<StructuredReviewJsonRequest>,
): Promise<unknown> {
  try {
    return await runDirectJsonRequest(request,);
  }
  catch (error) {
    if (!(error instanceof EmptyStructuredReviewTextError))
      throw error;
    /**
     Per-call logger for bounded empty-output retry.
     */
    const innerL = tagged({
      tag: runStructuredJsonRetries.name,
      l,
    },);
    innerL.warn('first direct JSON retry was empty; retrying direct JSON once more',);
    return runDirectJsonRequest(request,);
  }
}

export {
  runStructuredJsonRetries,
  runStructuredToolRequest,
  structuredReviewSignal,
};
