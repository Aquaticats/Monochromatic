/**
 Probe extension for the gated disposable-host verification.

 pi loads this file directly,
 so it stays dependency-free and reports through JSON lines on stderr,
 which RPC mode reserves for the host protocol on stdout.
 It loads before the built extension,
 so its `session_start` handler sees the snapshot catalog the factory
 registers,
 and its `resources_discover` handler sees the catalog after the session-start
 refresh ran.

 @module
 */

//region Constants

/**
 Provider route whose override the assertions inspect.
 */
const TARGET_PROVIDER = 'qwen-token-plan-individual';

//endregion Constants

//region Types

/**
 One chat model as the registry reports it.
 */
type ProbeModel = {
  readonly id: string;
  readonly contextWindow: number;
  readonly maxTokens: number;
  readonly reasoning: boolean;
  readonly thinkingLevelCount: number;
};

/**
 Registry surface the probe reads.
 */
type ProbeRegistry = {
  readonly getAll: () => readonly unknown[];
};

/**
 Event context the probe receives.
 */
type ProbeContext = {
  readonly modelRegistry: ProbeRegistry;
};

/**
 Extension API surface the probe registers against.
 */
type ProbeApi = {
  readonly on: (
    event: string,
    handler: (
      payload: unknown,
      ctx: ProbeContext,
    ) => void,
  ) => void;
};

//endregion Types

//region Reporting

/**
 Write one JSON record to stderr.

 @param record - fields to report

 @example
 ```typescript
 emit({ phase: 'factory' });
 ```
 */
function emit(record: Record<string, unknown>,): void {
  process.stderr
    .write(`${JSON.stringify({
      probe: TARGET_PROVIDER,
      ...record,
    })}\n`,);
}

/**
 Test whether an unknown value is a plain object record.

 @param value - parsed registry entry

 @returns whether the value can be read by key
 */
function isRecord(value: unknown,): value is Record<string, unknown> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Narrow one registry entry to the fields the assertions read.

 @param value - one entry of `registry.getAll()`

 @returns the projection, or `undefined` when the entry carries no model id
 */
function toProbeModel(value: unknown,): ProbeModel | undefined {
  if (!isRecord(value,))
    return undefined;
  if ((typeof value.id) !== 'string')
    return undefined;
  if ((typeof value.contextWindow) !== 'number')
    return undefined;
  if ((typeof value.maxTokens) !== 'number')
    return undefined;
  if ((typeof value.reasoning) !== 'boolean')
    return undefined;
  /**
   Thinking levels the model pins, counted for the assertions.
   */
  const thinkingLevelMap = value.thinkingLevelMap;
  const thinkingLevelCount = ((typeof thinkingLevelMap) === 'object')
    && (thinkingLevelMap !== null)
    ? Object.keys(thinkingLevelMap,)
      .length
    : 0;
  return {
    id: value.id,
    contextWindow: value.contextWindow,
    maxTokens: value.maxTokens,
    reasoning: value.reasoning,
    thinkingLevelCount,
  };
}

/**
 Report one catalog snapshot.

 @param registry - registry the event carried

 @param phase - lifecycle point the snapshot was taken at

 @example
 ```typescript
 snapshot({ registry: ctx.modelRegistry, phase: 'session_start' });
 ```
 */
function snapshot(
  {
    registry,
    phase,
  }: {
    readonly registry: ProbeRegistry;
    readonly phase: string;
  },
): void {
  /**
   Chat models the registry reports at this phase.
   */
  const chat = registry.getAll();
  emit({
    phase,
    chatCount: chat.length,
    routeModels: chat
      .filter(function isRouteModel(entry,) {
        return isRecord(entry,) && (entry.provider === TARGET_PROVIDER);
      },)
      .map(toProbeModel,)
      .filter(function keptModel(model,) {
        return model !== undefined;
      },),
  },);
}

//endregion Reporting

//region Entry point

/**
 Register the two snapshot handlers.

 @param pi - extension API the host passes to this factory

 @example
 ```typescript
 // pi --extension src/verify-probe.ts
 ```
 */
export default function verifyProbe(pi: ProbeApi,): void {
  emit({ phase: 'factory', },);
  pi.on(
    'session_start',
    function onSessionStart(
      _payload: unknown,
      ctx: ProbeContext,
    ): void {
      snapshot({
        registry: ctx.modelRegistry,
        phase: 'session_start',
      },);
    },
  );
  pi.on(
    'resources_discover',
    function onResourcesDiscover(
      _payload: unknown,
      ctx: ProbeContext,
    ): void {
      snapshot({
        registry: ctx.modelRegistry,
        phase: 'resources_discover',
      },);
    },
  );
}

//endregion Entry point
