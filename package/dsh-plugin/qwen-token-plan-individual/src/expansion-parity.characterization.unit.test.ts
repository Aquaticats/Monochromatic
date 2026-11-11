import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  apply,
} from '@deepseek-ai/dsh-llm-pi-ai';

import {
  expandQwenTokenPlanThinkingLevelMap,
  mapQwenTokenPlanIndividualProviders,
  QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE,
  type QwenTokenPlanEffortsDict,
  type QwenTokenPlanModelFacts,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Offered efforts dict covering the three-state vocabulary: a declared `null`
 off (absent from the expanded map, so dispatch sends the parameter's absence),
 declared values whose spellings differ from their level names (so a wire
 assertion cannot pass on level-name fallback), and undeclared levels (pinned
 `null`, hidden).
 */
const OFFERED_EFFORTS: QwenTokenPlanEffortsDict = {
  off: null,
  high: 'turbo',
  max: 'ultra',
};

/**
 Expansion every side must agree on: every level decided, the declared `null`
 off absent from the map entirely.
 */
const EXPECTED_MAP = {
  minimal: null,
  low: null,
  medium: null,
  high: 'turbo',
  xhigh: null,
  max: 'ultra',
};

/**
 Selectable levels every side must offer for `OFFERED_EFFORTS`, in pi-ai
 dispatch order: the declared `null` off still selectable ("supported, send
 nothing"), the declared values selectable, every undeclared level hidden.
 */
const EXPECTED_LEVELS = [
  'off',
  'high',
  'max',
];

/**
 Wire token standing for the one state that sends no `reasoning_effort`
 parameter at all, so absent compares as one visible value.
 */
const ABSENT_WIRE = '<reasoning_effort absent>';

/**
 Model id the characterized route serves: a synthetic id outside pi-ai's
 installed catalog, so no catalog row merges under the declared facts.
 */
const CHARACTERIZED_MODEL_ID = 'parity-probe-31b';

/**
 Facts row the characterized route resolves against: one chat model offering
 `OFFERED_EFFORTS` under the wire-compat pins the facts table carries.
 */
const CHARACTERIZED_FACTS: QwenTokenPlanModelFacts = {
  id: CHARACTERIZED_MODEL_ID,
  name: 'Parity Probe 31B',
  kind: 'chat',
  input: ['text'],
  reasoning: true,
  efforts: OFFERED_EFFORTS,
  compat: {
    thinkingFormat: 'qwen',
    supportsDeveloperRole: false,
    supportsStore: false,
    supportsStrictMode: true,
    supportsReasoningEffort: true,
  },
};

/**
 One captured request body as text: the body string when the transport sent
 one, its JSON form otherwise, so nothing stringifies into
 `[object Object]` and the capture always parses back.

  @param init - request options the dispatch handed the transport

  @returns body text to parse into the request payload

  @example
  ```ts
  const text = bodyTextOf(init);
  ```
  */
function bodyTextOf(init?: RequestInit): string {
  const body: unknown = init?.body;
  if ((typeof body) === 'string') {
    return body;
  }
  return JSON.stringify(body ?? null);
}

/**
 Adapter surface this characterization reaches: model resolution and request
 dispatch, typed structurally so the test needs no harness type imports.
 */
type CharacterizedAdapter = {
  resolveModel: (provider: string, model: string) => Promise<{
    readonly reasoning?: {
      readonly efforts: readonly { readonly id: string }[];
    };
  }>;
  stream: (options: {
    readonly provider: string;
    readonly model: string;
    readonly reasoningEffort?: string;
    readonly messages: readonly unknown[];
  }) => AsyncIterable<unknown>;
};

/**
 Captured registration the fake host records: the routes `apply` registered
 and the adapter that owns them.
 */
type CapturedRegistration = {
  routes?: readonly string[];
  adapter?: CharacterizedAdapter;
};

//endregion Fixtures

//region Fake host

/**
 Minimal Cordis-shaped host for `apply`: delivers one merged child to both
 `inject` callbacks, carries the fiber identity, and records the adapter the
 llm seam registers. Only the members `apply` touches exist.

  @param registration - record the llm registration writes into

  @returns fake context object to hand `apply`

  @example
  ```ts
  const ctx = fakeHostOf({ registration });
  ```
  */
function fakeHostOf({
  registration,
}: {
  readonly registration: CapturedRegistration;
}): object {
  /**
   Merged service child satisfying both `inject` callbacks: the settings
   child's `effect` and `configure`, the authorization child's
   `authorization.registerFlow` and `logger.warn`.
   */
  const child = {
    effect: function effect(): void {},
    settings: {
      configure: function configure(): void {},
    },
    authorization: {
      registerFlow: function registerFlow(): void {},
    },
    logger: {
      warn: function warn(): void {},
    },
  };
  return {
    inject: function inject(
      names: readonly string[],
      run: (child: unknown) => void
    ): void {
      void names;
      run(child);
    },
    fiber: {
      entry: {
        options: {
          id: 'llm-pi-ai',
        },
      },
    },
    on: function on(): void {},
    get: function get(name: string): unknown {
      return name === 'credentials'
        ? {
          resolve: async function resolve(): Promise<{ value: string }> {
            return { value: 'test-key' };
          },
        }
        : undefined;
    },
    waterfall: function waterfall(
      name: string,
      request: unknown,
      headers: unknown,
      next: () => unknown
    ): unknown {
      void name;
      void request;
      void headers;
      return next();
    },
    logger: {
      warn: function warn(): void {},
      error: function error(): void {},
    },
    llm: {
      registerConfigurableProviders: function registerConfigurableProviders() {
        return {
          replace: function replace(): void {},
        };
      },
      registerModelDiscovery: function registerModelDiscovery(): void {},
      registerAdapter: function registerAdapter(
        routes: readonly string[],
        adapter: CharacterizedAdapter
      ): { replace: () => void } {
        registration.routes = routes;
        registration.adapter = adapter;
        return {
          replace: function replace(): void {},
        };
      },
    },
  };
}

/**
 One routed profile whose model declares `OFFERED_EFFORTS`, assembled through
 the same provider mapping the plugin writes.

  @returns config object for `apply`, wrapped as the volatile it expects

  @example
  ```ts
  const config = routedConfigOf();
  ```
  */
function routedConfigOf(): object {
  return {
    providers: {
      get: function get(): Record<string, unknown> {
        return mapQwenTokenPlanIndividualProviders({
          listing: [{ id: CHARACTERIZED_MODEL_ID, },],
          facts: [CHARACTERIZED_FACTS,],
        },);
      },
    },
  };
}

//endregion Fake host

await describe({
  name: 'expansion parity with dsh-llm-pi-ai',
  children: [
    it({
      name: 'expands the three-state efforts dict identically through the adapter and expandQwenTokenPlanThinkingLevelMap',
      fn: async (ctx) => {
        /**
         Registration record the fake host fills in.
         */
        const registration: CapturedRegistration = {};
        apply(
          fakeHostOf({ registration }) as never,
          routedConfigOf() as never
        );
        /**
         Adapter whose catalog the real resolution path materialized from
         `OFFERED_EFFORTS`.
         */
        const { adapter } = registration;
        if (adapter === undefined) {
          throw new Error('expected the llm seam to register an adapter');
        }
        /**
         Shared expansion of the same dict: every level decided, the declared
         `null` off absent.
         */
        const sharedMap = expandQwenTokenPlanThinkingLevelMap({ efforts: OFFERED_EFFORTS });
        expect(Object.hasOwn(sharedMap, 'off')).toBe(false);
        expect(sharedMap).toEqual(EXPECTED_MAP);
        /**
         Levels the adapter's own resolution path offers for the same dict.
         */
        const resolved = await adapter.resolveModel(
          QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE,
          CHARACTERIZED_MODEL_ID
        );
        const offeredLevels = resolved.reasoning?.efforts.map(function byId(effort) {
          return effort.id;
        });
        expect(offeredLevels).toEqual(EXPECTED_LEVELS);
        /**
         Request bodies the stubbed transport captured, one per dispatched
         level, in whatever order the concurrent dispatches reached it.
         */
        const bodies: string[] = [];
        ctx.sinon.stub(globalThis, 'fetch').callsFake(async function capture(url, init) {
          void url;
          bodies.push(bodyTextOf(init));
          return new Response('upstream error', { status: 500 });
        });
        await Promise.all(EXPECTED_LEVELS.map(async function dispatchLevel(level) {
          try {
            for await (const chunk of adapter.stream({
              provider: QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE,
              model: CHARACTERIZED_MODEL_ID,
              reasoningEffort: level,
              messages: [
                {
                  role: 'user',
                  content: [{ type: 'text', text: 'ping' }],
                },
              ],
            })) {
              void chunk;
            }
          } catch (error) {
            // The stubbed 500 ends the dispatch; only the request body matters.
            void error;
          }
        }));
        /**
         Wire token each dispatched body carried, one entry per dispatch: the
         `reasoning_effort` value when the request named one, else the absence
         marker. Sorted so the comparison needs no dispatch order.
         */
        const payloads = bodies.map(function byBody(body) {
          return JSON.parse(body) as Record<string, unknown>;
        });
        const dispatchedWires = payloads.map(function byPayload(payload) {
          return Object.hasOwn(payload, 'reasoning_effort')
            ? String(payload.reasoning_effort)
            : ABSENT_WIRE;
        })
          .toSorted();
        /**
         Wire token every level must dispatch, derived from the shared map: a
         declared spelling becomes the sent value, while the level the map
         leaves absent (the declared `null` off) sends nothing.
         */
        const expectedWires = EXPECTED_LEVELS.map(function byLevel(level) {
          const declaredWire: unknown = Reflect.get(sharedMap, level);
          if ((typeof declaredWire) === 'string') {
            return declaredWire;
          }
          return ABSENT_WIRE;
        })
          .toSorted();
        expect(dispatchedWires).toEqual(expectedWires);
        /**
         Whether each dispatch carried the qwen thinking marker: its presence
         is the positive control that the `thinkingFormat: "qwen"` compat pin
         reached dispatch, since the OpenAI-style default branch sends none.
         */
        const carriedThinkingMarker = payloads.every(function byPayload(payload) {
          return Object.hasOwn(payload, 'enable_thinking');
        });
        expect(carriedThinkingMarker).toBe(true);
      },
    }),
  ],
});
