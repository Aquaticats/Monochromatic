/**
 Tests for the thinking-default extension entry point.
 
 @module
 */

import type {
  ExtensionAPI,
  ExtensionContext,
  SessionStartEvent,
} from '@earendil-works/pi-coding-agent';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { ThinkingDefaultLevel, ThinkingLevelMapFragment, } from './model-policy.ts';

//region Mock infrastructure

/** Minimal handler signature matching the pi events tested here. */
type HandlerFn = (event: unknown, ctx: ExtensionContext,) => unknown;

/** Shape of the mock registration map. */
type RegistrationMap = Map<string, HandlerFn[]>;

/** Minimal model shape used by entry-point tests. */
type TestModel = {
  /** Model identifier. */
  id: string;
  /** Whether the model emits reasoning. */
  reasoning?: boolean;
  /** Provider thinking-level map fragment, when the model declares one. */
  thinkingLevelMap?: ThinkingLevelMapFragment;
};

/** Minimal model-selection event shape used by entry-point tests. */
type ModelSelectEventLike = {
  /** Event type discriminant. */
  type: 'model_select';
  /** Newly selected model. */
  model: TestModel;
  /** Previous model, absent for first selection. */
  previousModel?: TestModel;
  /** Source of the model selection. */
  source: 'set' | 'cycle' | 'restore';
};

/** Mock API plus captured side effects. */
type MockApiHarness = {
  /** Mock pi extension API. */
  api: ExtensionAPI;
  /** Registered event handlers keyed by event name. */
  registrations: RegistrationMap;
  /** Ordered levels passed to `pi.setThinkingLevel`. */
  setCalls: ThinkingDefaultLevel[];
};

/**
 Creates a mock ExtensionAPI that records event registrations and thinking writes.
 
 @param currentLevel - value returned by `getThinkingLevel`
 
 @returns mock API with captured registrations and set calls
 
 @example
 ```typescript
 const harness = createMockApi({ currentLevel: 'high' });
 ```
 */
function createMockApi(
  {
    currentLevel,
  }: {
    currentLevel: string;
  },
): MockApiHarness {
  /** Registered event handlers. */
  const registrations: RegistrationMap = new Map();
  /** Recorded thinking-level writes. */
  const setCalls: ThinkingDefaultLevel[] = [];

  const api = {
    on(
      event: string,
      handler: HandlerFn,
    ) {
      const existing = registrations.get(event,) ?? [];
      existing.push(handler,);
      registrations.set(event, existing,);
    },
    getThinkingLevel() {
      return currentLevel;
    },
    setThinkingLevel(level: ThinkingDefaultLevel,) {
      setCalls.push(level,);
    },
  } as unknown as ExtensionAPI;

  return {
    api,
    registrations,
    setCalls,
  };
}

/**
 Retrieves the single registered handler for an event.
 
 @param registrations - event registration map
 
 @param event - event name to look up
 
 @returns registered handler
 
 @throws when event has no registered handler
 
 @example
 ```typescript
 const handler = getHandler({ registrations, event: 'model_select' });
 ```
 */
function getHandler(
  {
    registrations,
    event,
  }: {
    registrations: RegistrationMap;
    event: string;
  },
): HandlerFn {
  /** Handlers registered for the requested event. */
  const handlers = registrations.get(event,);
  if ((handlers === undefined) || (handlers.length === 0))
    throw new Error(`No handler registered for event: ${event}`,);
  /** First registered handler for the requested event. */
  const [handler,] = handlers;
  if (handler === undefined)
    throw new Error(`No handler registered for event: ${event}`,);
  return handler;
}

/**
 Creates a minimal extension context carrying a current model.
 
 @param model - context model to expose
 
 @returns extension context for handler invocation
 
 @example
 ```typescript
 const ctx = createContext({ model: { id: 'gpt-5.5' } });
 ```
 */
function createContext(
  {
    model,
  }: {
    model?: TestModel;
  },
): ExtensionContext {
  return { model, } as unknown as ExtensionContext;
}

//endregion Mock infrastructure

/** Extension entry point imported dynamically to match sibling Pi package tests. */
const {
  default: thinkingDefaults,
  registerThinkingDefaults,
} = await import('./index.ts');

await describe({
  name: thinkingDefaults.name,
  children: [
    //region Registration

    it({
      name: 'registers session_start and model_select handlers',
      fn: async function testRegistrations() {
        const { api, registrations, } = createMockApi({ currentLevel: 'high', },);
        registerThinkingDefaults({ pi: api, },);

        expect(registrations.get('session_start',),).toHaveLength(1,);
        expect(registrations.get('model_select',),).toHaveLength(1,);
      },
    },),

    //endregion Registration

    //region Handler behavior

    it({
      name: 'session_start applies policy from context model',
      fn: async function testSessionStartUsesContextModel() {
        const { api, registrations, setCalls, } = createMockApi({
          currentLevel: 'high',
        },);
        /** Settings restoration calls after applying target defaults. */
        const restoreCalls: string[] = [];
        registerThinkingDefaults({
          pi: api,
          restoreDefaultThinkingLevel: async function restoreDefaultThinkingLevel(): Promise<boolean> {
            restoreCalls.push('restore',);
            return true;
          },
        },);
        const handler = getHandler({ registrations, event: 'session_start', },);

        await handler(
          { type: 'session_start', reason: 'startup', } satisfies SessionStartEvent,
          createContext({ model: { id: 'gpt-5.5', }, },),
        );

        expect(setCalls,).toEqual(['xhigh',],);
        expect(restoreCalls,).toEqual(['restore',],);
      },
    },),
    it({
      name: 'session_start applies xhigh for non-GPT model that supports xhigh',
      fn: async function testSessionStartAppliesXhighForCapableNonGpt() {
        const { api, registrations, setCalls, } = createMockApi({
          currentLevel: 'high',
        },);
        /** Settings restoration calls after applying target defaults. */
        const restoreCalls: string[] = [];
        registerThinkingDefaults({
          pi: api,
          restoreDefaultThinkingLevel: async function restoreDefaultThinkingLevel(): Promise<boolean> {
            restoreCalls.push('restore',);
            return true;
          },
        },);
        const handler = getHandler({ registrations, event: 'session_start', },);

        await handler(
          { type: 'session_start', reason: 'startup', } satisfies SessionStartEvent,
          createContext({
            model: {
              id: 'synthetic/hf:zai-org/GLM-5.2',
              reasoning: true,
              thinkingLevelMap: { xhigh: 'max', },
            },
          },),
        );

        expect(setCalls,).toEqual(['xhigh',],);
        expect(restoreCalls,).toEqual(['restore',],);
      },
    },),
    it({
      name: 'session_start does nothing when context has no model',
      fn: async function testSessionStartWithoutModel() {
        const { api, registrations, setCalls, } = createMockApi({
          currentLevel: 'high',
        },);
        /** Settings restoration calls after applying target defaults. */
        const restoreCalls: string[] = [];
        registerThinkingDefaults({
          pi: api,
          restoreDefaultThinkingLevel: async function restoreDefaultThinkingLevel(): Promise<boolean> {
            restoreCalls.push('restore',);
            return true;
          },
        },);
        const handler = getHandler({ registrations, event: 'session_start', },);

        await handler(
          { type: 'session_start', reason: 'startup', } satisfies SessionStartEvent,
          createContext({},),
        );

        expect(setCalls,).toHaveLength(0,);
        expect(restoreCalls,).toHaveLength(0,);
      },
    },),
    it({
      name: 'model_select applies policy from selected event model',
      fn: async function testModelSelectUsesEventModel() {
        const { api, registrations, setCalls, } = createMockApi({
          currentLevel: 'xhigh',
        },);
        /** Settings restoration calls after applying target defaults. */
        const restoreCalls: string[] = [];
        registerThinkingDefaults({
          pi: api,
          restoreDefaultThinkingLevel: async function restoreDefaultThinkingLevel(): Promise<boolean> {
            restoreCalls.push('restore',);
            return true;
          },
        },);
        const handler = getHandler({ registrations, event: 'model_select', },);

        await handler(
          {
            type: 'model_select',
            model: { id: 'synthetic/hf:moonshotai/Kimi-K2.6', },
            source: 'set',
          } satisfies ModelSelectEventLike,
          createContext({ model: { id: 'gpt-5.5', }, },),
        );

        expect(setCalls,).toEqual(['high',],);
        expect(restoreCalls,).toEqual(['restore',],);
      },
    },),
    //endregion Handler behavior
  ],
},);
