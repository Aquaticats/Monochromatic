/**
 Disposable native Codex values shared only by tests.
 
 @module
 */
import {
  createAssistantMessageEventStream,
  hasApi,
  type Api,
  type AssistantMessage,
  type AssistantMessageEventStream,
  type Model,
  type TranscriptContext,
} from '@earendil-works/pi-ai';
import { normalizeContext, } from '@earendil-works/pi-ai/utils/transcript';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

//region Native values: preserve original model identities in completed history.

/**
 Recursively marks a fixture override deeply readonly, so no nested native object stays writable.

 @typeParam T - override value whose reachable objects this fixture only copies
 */
type DeepReadonly<T,> = T extends readonly (infer TItem)[] ? readonly DeepReadonly<TItem>[]
  : T extends object ? { readonly [TKey in keyof T]: DeepReadonly<T[TKey]> }
    : T;

/**
 Explicit capability overrides exercised by host scenarios, not arbitrary optional model fields.
 */
export type FixtureModelOverrides = {
  readonly id?: string;
  readonly name?: string;
  readonly contextWindow?: number;
  readonly maxTokens?: number;
  readonly reasoning?: boolean;
  readonly input?: readonly ('text' | 'image')[];
  readonly headers?: Readonly<NonNullable<Model<'openai-codex-responses'>['headers']>>;
  readonly thinkingLevelMap?: Readonly<NonNullable<Model<'openai-codex-responses'>['thinkingLevelMap']>>;
  readonly inputLimits?: DeepReadonly<NonNullable<Model<'openai-codex-responses'>['inputLimits']>>;
};

/**
 Build independent native metadata without an upstream compatibility allowlist.
 
 @param overrides - capabilities needed to distinguish tested catalog changes
 
 @returns native Codex model owned by its test
 
 @example
 ```ts
 const custom = fixtureModel({ id: 'future-native', });
 ```
 */
export function fixtureModel(overrides: FixtureModelOverrides = {},): Model<'openai-codex-responses'> {
  /**
   Owned readonly input list is copied into the native model's mutable array boundary.
   */
  const {
    input,
    ...metadata
  } = overrides;
  return {
    id: 'gpt-host-fixture',
    name: 'Host fixture',
    api: 'openai-codex-responses',
    provider: 'openai-codex',
    baseUrl: 'https://host-fixture.invalid/backend-api',
    reasoning: true,
    input: [
      'text',
      'image',
    ],
    thinkingLevelMap: {
      high: 'high',
      xhigh: 'xhigh',
    },
    cost: {
      input: 1,
      output: 2,
      cacheRead: 0.5,
      cacheWrite: 0,
    },
    contextWindow: 200_000,
    maxTokens: 8_192,
    ...metadata,
    ...(input === undefined ? {} : { input: [...input,], }),
  };
}

/**
 Prepare normalized synthetic transcript using installed native API.
 
 @returns independently normalized test request
 
 @example
 ```ts
 const context = fixtureContext();
 ```
 */
export function fixtureContext(): TranscriptContext {
  return normalizeContext({
    systemPrompt: 'Synthetic test instructions.',
    messages: [{
      role: 'user',
      content: 'Synthetic test input.',
      timestamp: 0,
    },],
  },);
}

/**
 Explicit response variations used to exercise host failure and overflow handling.
 */
export type FixtureAssistantOverrides = {
  readonly usage?: Readonly<Omit<AssistantMessage['usage'], 'cost'>> & {
    readonly cost: Readonly<AssistantMessage['usage']['cost']>;
  };
  readonly stopReason?: AssistantMessage['stopReason'];
  readonly errorMessage?: string;
};

/**
 Create canonical assistant history for finite offline provider calls.
 
 @param model - original native identity preserved in session history
 
 @param overrides - response variations for failure and overflow controls
 
 @returns canonical fixture response
 
 @example
 ```ts
 const message = fixtureAssistant({ model: fixtureModel() });
 ```
 */
export function fixtureAssistant({
  model,
  overrides = {},
}: {
  readonly model: ForeignBorrowed<Model<Api>>;
  readonly overrides?: FixtureAssistantOverrides;
},): AssistantMessage {
  return {
    role: 'assistant',
    content: [{
      type: 'text',
      text: 'Fixture answer.',
    },],
    api: model.api,
    provider: model.provider,
    model: model.id,
    timestamp: 0,
    stopReason: 'stop',
    usage: {
      input: 10,
      output: 2,
      cacheRead: 0,
      cacheWrite: 0,
      totalTokens: 12,
      cost: {
        input: 0,
        output: 0,
        cacheRead: 0,
        cacheWrite: 0,
        total: 0,
      },
    },
    ...overrides,
  };
}

/**
 Produce terminal native stream including actual error event protocol.
 
 @param message - terminal response supplied by test provider
 
 @returns settled stream consumable by real pi host
 
 @throws Error when response has not reached a terminal state
 
 @example
 ```ts
 const events = fixtureStream(fixtureAssistant({ model: fixtureModel() }));
 ```
 */
export function fixtureStream(message: ForeignBorrowed<AssistantMessage>,): AssistantMessageEventStream {
  if (message.stopReason === 'pending')
    throw new Error('Fixture terminal stream cannot emit a pending response.',);
  /**
   Native event queue belongs only to this finite fixture invocation.
   */
  const events = createAssistantMessageEventStream();
  events.push({
    type: 'start',
    partial: message,
  },);
  if ((message.stopReason === 'error') || (message.stopReason === 'aborted'))
    events.push({
      type: 'error',
      reason: message.stopReason,
      error: message,
    },);
  else
    events.push({
      type: 'done',
      reason: message.stopReason,
      message,
    },);
  events.end(message,);
  return events;
}

/**
 Reject missing or differently configured model instead of substituting it.
 
 @param model - actual model obtained from native registry
 
 @returns checked native Codex model
 
 @throws Error when native Codex fixture is absent
 
 @example
 ```ts
 const native = requireCodexModel(fixtureModel());
 ```
 */
export function requireCodexModel(model?: ForeignBorrowed<Model<Api>>,): Model<'openai-codex-responses'> {
  if ((model === undefined) || (!hasApi(
    model,
    'openai-codex-responses',
  )))
    throw new Error('Expected native Codex fixture model.',);
  return model;
}

/**
 Capture rejection while distinguishing unexpected successful completion.
 
 @param fn - failing operation whose actual rejection is asserted
 
 @returns caught rejection value
 
 @throws Error when operation unexpectedly succeeds
 
 @example
 ```ts
 const error = await caughtFailure(function rejected() {
   throw new Error('fixture');
 });
 ```
 */
export async function caughtFailure(fn: () => unknown,): Promise<unknown> {
  try {
    await fn();
  }
  catch (error) {
    return error;
  }
  throw new Error('Expected fixture operation to fail.',);
}

//endregion Native values
