/**
 Scripted local provider for real AgentSession interruption verification.
 
 @module
 */

import {
  type Api,
  type AssistantMessage,
  type AssistantMessageEventStream,
  type Context,
  createAssistantMessageEventStream,
  type Model,
  type SimpleStreamOptions,
  type ToolCall,
} from '@earendil-works/pi-ai';
import type { ModelRuntime, } from '@earendil-works/pi-coding-agent';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  interruptionToolCalls,
  type ScriptedProvider,
  type ScriptedToolCall,
} from './pi-runtime-verifier-provider-tools.ts';

/**
 Scripted provider identity.
 */
const PROVIDER_ID = 'pi-goal-runtime-verifier';

/**
 Scripted model identity.
 */
const MODEL_ID = 'interruption-sequence';

/**
 First provider call waits for explicit user abort.
 */
const ABORT_CALL = 1;

/**
 Second provider call executes ordinary tools after abort.
 */
const AFTER_ABORT_TOOL_CALL = ABORT_CALL + 1;

/**
 Third provider call ends in model error.
 */
const ERROR_CALL = AFTER_ABORT_TOOL_CALL + 1;

/**
 Fourth provider call executes ordinary tools after error.
 */
const AFTER_ERROR_TOOL_CALL = ERROR_CALL + 1;

/**
 Fifth provider call waits so verifier can end active goal without continuation.
 */
const FINAL_ABORT_CALL = AFTER_ERROR_TOOL_CALL + 1;

/**
 Sixth provider call executes ordinary tools after clear.
 */
const AFTER_CLEAR_TOOL_CALL = FINAL_ABORT_CALL + 1;

/**
 Seventh provider call waits so verifier can end cleared-goal turn.
 */
const CLEAR_FINAL_ABORT_CALL = AFTER_CLEAR_TOOL_CALL + 1;

/**
 Create empty assistant message for one scripted provider result.
 
 @param model - registered local model
 
 @param stopReason - terminal reason assigned before streaming
 
 @returns mutable provider-owned assistant message
 
 @example
 ```ts
 createAssistantOutput({ model, stopReason: 'toolUse' });
 ```
 */
function createAssistantOutput(
  {
    model,
    stopReason,
  }: {
    readonly model: ForeignBorrowed<Model<Api>>;
    readonly stopReason: AssistantMessage['stopReason'];
  },
): AssistantMessage {
  return {
    role: 'assistant',
    content: [],
    api: model.api,
    provider: model.provider,
    model: model.id,
    usage: {
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
    },
    stopReason,
    timestamp: Date.now(),
  };
}

/**
 Resolve when supplied abort signal fires.
 
 @param signal - AgentSession provider cancellation signal
 
 @throws when AgentSession omits provider signal
 
 @mutates signal - signal.addEventListener registers one abort listener
 
 @example
 ```ts
 await waitForAbort(signal);
 ```
 */
async function waitForAbort(
  { signal, }: { readonly signal?: ForeignBorrowed<AbortSignal>; },
): Promise<void> {
  if (signal === undefined)
    throw new Error('scripted provider received no abort signal',);
  if (signal.aborted)
    return;
  /**
   Resolver completed by one abort event.
   */
  const {
    promise,
    resolve,
  } = Promise.withResolvers<void>();
  signal.addEventListener(
    'abort',
    function resolveAbort(): void {
      resolve();
    },
    { once: true, },
  );
  await promise;
}

/**
 Emit complete scripted tool-use response.
 
 @param model - registered local model
 
 @param calls - ordered tool calls
 
 @param invocation - provider invocation identity
 
 @returns closed assistant event stream
 
 @example
 ```ts
 toolCallStream({ model, calls: interruptionToolCalls('abort'), invocation: 2 });
 ```
 */
function toolCallStream(
  {
    model,
    calls,
    invocation,
  }: {
    readonly model: Model<Api>;
    readonly calls: readonly ScriptedToolCall[];
    readonly invocation: number;
  },
): AssistantMessageEventStream {
  /**
   Provider event stream consumed by real AgentSession loop.
   */
  const stream = createAssistantMessageEventStream();
  /**
   Provider-owned result populated before terminal event.
   */
  const output = createAssistantOutput({
    model,
    stopReason: 'toolUse',
  },);
  stream.push({
    type: 'start',
    partial: output,
  },);
  for (const [index, call,] of calls.entries()) {
    /**
     Complete tool call inserted into assistant message.
     */
    const toolCall: ToolCall = {
      type: 'toolCall',
      id: `runtime-${invocation}-${index}`,
      name: call.name,
      arguments: call.arguments,
    };
    output.content
      .push(toolCall,);
    stream.push({
      type: 'toolcall_start',
      contentIndex: index,
      partial: output,
    },);
    stream.push({
      type: 'toolcall_end',
      contentIndex: index,
      toolCall,
      partial: output,
    },);
  }
  stream.push({
    type: 'done',
    reason: 'toolUse',
    message: output,
  },);
  stream.end();
  return stream;
}

/**
 Emit immediate scripted model error.
 
 @param model - registered local model
 
 @returns closed error event stream
 
 @example
 ```ts
 errorStream(model);
 ```
 */
function errorStream(model: Model<Api>,): AssistantMessageEventStream {
  /**
   Provider event stream consumed by real AgentSession loop.
   */
  const stream = createAssistantMessageEventStream();
  /**
   Error result driving goal error-settlement continuation.
   */
  const output = createAssistantOutput({
    model,
    stopReason: 'error',
  },);
  output.errorMessage = 'scripted runtime verifier error';
  stream.push({
    type: 'start',
    partial: output,
  },);
  stream.push({
    type: 'error',
    reason: 'error',
    error: output,
  },);
  stream.end();
  return stream;
}

/**
 Emit provider turn that remains active until AgentSession aborts it.
 
 @param model - registered local model
 
 @param signal - AgentSession provider cancellation signal
 
 @param started - stage resolver called after stream start
 
 @returns event stream closed after abort
 
 @mutates signal - waitForAbort registers one abort listener
 
 @example
 ```ts
 abortStream({ model, signal, started: resolve });
 ```
 */
function abortStream(
  {
    model,
    signal,
    started,
  }: {
    readonly model: ForeignBorrowed<Model<Api>>;
    readonly signal?: ForeignBorrowed<AbortSignal>;
    readonly started: () => void;
  },
): AssistantMessageEventStream {
  /**
   Provider event stream consumed by real AgentSession loop.
   */
  const stream = createAssistantMessageEventStream();
  /**
   Pending assistant result finalized only after cancellation.
   */
  const output = createAssistantOutput({
    model,
    stopReason: 'aborted',
  },);
  stream.push({
    type: 'start',
    partial: output,
  },);
  started();
  /**
   Finish provider stream after real AgentSession cancellation.
   
   @example
   ```ts
   await finishAfterAbort();
   ```
   */
  async function finishAfterAbort(): Promise<void> {
    await waitForAbort(signal === undefined ? {} : { signal, },);
    output.errorMessage = 'scripted turn aborted';
    stream.push({
      type: 'error',
      reason: 'aborted',
      error: output,
    },);
    stream.end();
  }
  void finishAfterAbort();
  return stream;
}

/**
 Register scripted local provider and return its model plus stage latches.
 
 @param modelRuntime - disposable model registry
 
 @returns scripted model and observable stage boundaries
 
 @mutates modelRuntime - modelRuntime.registerProvider installs provider and modelRuntime.getModel reads its mutable registry
 
 @example
 ```ts
 const provider = registerInterruptionProvider(modelRuntime);
 ```
 */
function registerInterruptionProvider(
  modelRuntime: ForeignBorrowed<ModelRuntime>,
): ScriptedProvider {
  /**
   Call count retained for exact sequence validation.
   */
  const invocations: true[] = [];
  /**
   First call latch controlled by scripted stream.
   */
  const firstStage = Promise.withResolvers<void>();
  /**
   Error-call latch controlled by scripted stream.
   */
  const errorStage = Promise.withResolvers<void>();
  /**
   Final call latch controlled by scripted stream.
   */
  const finalStage = Promise.withResolvers<void>();
  /**
   Post-clear final call latch controlled by scripted stream.
   */
  const clearFinalStage = Promise.withResolvers<void>();
  /**
   Produce next deterministic provider response.
   
   @param model - selected scripted model
   
   @param options - stream cancellation options
   
   @returns response for current script position
   
   @example
   ```ts
   scriptedResponse({ model, options });
   ```
   */
  function scriptedResponse(
    {
      model,
      options,
    }: {
      readonly model: ForeignBorrowed<Model<Api>>;
      readonly options?: ForeignBorrowed<SimpleStreamOptions>;
    },
  ): AssistantMessageEventStream {
    invocations.push(true,);
    /**
     Current provider call identity after recording invocation.
     */
    const invocation = invocations.length;
    if (invocation === ABORT_CALL) {
      return abortStream({
        model,
        ...(options?.signal === undefined ? {} : { signal: options.signal, }),
        started: firstStage.resolve,
      },);
    }
    if (invocation === AFTER_ABORT_TOOL_CALL) {
      return toolCallStream({
        model,
        calls: interruptionToolCalls('abort',),
        invocation,
      },);
    }
    if (invocation === ERROR_CALL) {
      errorStage.resolve();
      return errorStream(model,);
    }
    if (invocation === AFTER_ERROR_TOOL_CALL) {
      return toolCallStream({
        model,
        calls: interruptionToolCalls('error',),
        invocation,
      },);
    }
    if (invocation === FINAL_ABORT_CALL) {
      return abortStream({
        model,
        ...(options?.signal === undefined ? {} : { signal: options.signal, }),
        started: finalStage.resolve,
      },);
    }
    if (invocation === AFTER_CLEAR_TOOL_CALL) {
      return toolCallStream({
        model,
        calls: interruptionToolCalls('clear',),
        invocation,
      },);
    }
    if (invocation === CLEAR_FINAL_ABORT_CALL) {
      return abortStream({
        model,
        ...(options?.signal === undefined ? {} : { signal: options.signal, }),
        started: clearFinalStage.resolve,
      },);
    }
    return errorStream(model,);
  }
  modelRuntime.registerProvider(
    PROVIDER_ID,
    {
      name: 'Pi goal runtime verifier',
      baseUrl: 'https://pi-goal-verifier.invalid',
      apiKey: 'disposable-verifier-key',
      api: PROVIDER_ID,
      models: [{
        id: MODEL_ID,
        name: 'Interruption sequence',
        reasoning: false,
        input: ['text',],
        cost: {
          input: 0,
          output: 0,
          cacheRead: 0,
          cacheWrite: 0,
        },
        contextWindow: 16_384,
        maxTokens: 4_096,
      },],
      streamSimple(
        model: ForeignBorrowed<Model<Api>>,
        _context: ForeignBorrowed<Context>,
        options?: ForeignBorrowed<SimpleStreamOptions>,
      ): AssistantMessageEventStream {
        return scriptedResponse({
          model,
          ...(options === undefined ? {} : { options, }),
        },);
      },
    },
  );
  /**
   Registered scripted model selected without credential discovery.
   */
  const model = modelRuntime.getModel(
    PROVIDER_ID,
    MODEL_ID,
  );
  if (model === undefined)
    throw new Error('scripted interruption model registration failed',);
  return {
    model,
    firstTurnStarted: firstStage.promise,
    errorTurnStarted: errorStage.promise,
    finalTurnStarted: finalStage.promise,
    clearFinalTurnStarted: clearFinalStage.promise,
    invocationCount() {
      return invocations.length;
    },
  };
}

export { registerInterruptionProvider, };
