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
 One catalog model as the registry reports it.
 */
type ProbeModel = {
  readonly id: string;
  readonly contextWindow: number;
  readonly maxTokens: number;
  readonly reasoning: boolean;
  readonly thinkingLevelCount: number;
};

/**
 One registry entry fields the assertions read,
 before the thinking-level count is derived.
 */
type RegistryModel = {
  readonly id: string;
  readonly contextWindow: number;
  readonly maxTokens: number;
  readonly reasoning: boolean;
  readonly thinkingLevelMap?: Readonly<Record<string, unknown>>;
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
 Test whether an unknown registry entry carries the fields the assertions
 read.

 @param value - one entry of `registry.getAll()`

 @returns whether the entry reads as a model
 */
function isRegistryModel(value: unknown,): value is RegistryModel {
  return isRecord(value,)
    && ((typeof value.id) === 'string')
    && ((typeof value.contextWindow) === 'number')
    && ((typeof value.maxTokens) === 'number')
    && ((typeof value.reasoning) === 'boolean');
}

/**
 Narrow one registry entry to the fields the assertions read.

 @param model - registry entry already narrowed by `isRegistryModel`

 @returns the report projection of the entry
 */
function toProbeModel(model: RegistryModel,): ProbeModel {
  /**
   Count of thinking levels the model pins,
   zero when it pins none.
   */
  const pinnedThinkingLevels = model.thinkingLevelMap === undefined
    ? 0
    : Object.keys(model.thinkingLevelMap,)
      .length;
  return {
    id: model.id,
    contextWindow: model.contextWindow,
    maxTokens: model.maxTokens,
    reasoning: model.reasoning,
    thinkingLevelCount: pinnedThinkingLevels,
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
      .filter(isRegistryModel,)
      .map(toProbeModel,),
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
