/**
 Effective scoped-model resolution for pi plugins.
 
 @module
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  NO_ARGV_MODELS,
  parseArgvModelPatterns,
} from './argv-scope.ts';
import { scopedModelFromModel, } from './pattern-match.ts';
import { resolveModelPatterns, } from './scope-patterns.ts';
import { loadSettingsScopePatterns, } from './settings-scope.ts';
import type {
  EffectiveModelScope,
  ReadonlyModel,
  ScopedModel,
  ScopedThinkingLevel,
} from './types.ts';

/**
 Sentinel returned by {@link readLiveScope} when the runtime exposes no
 usable live model scope. A `unique symbol`; narrowed with
 `=== NO_LIVE_SCOPE`.
 */
export const NO_LIVE_SCOPE: unique symbol = Symbol('model-selection/no-live-scope',);

//region Types

/**
 Model registry surface needed for effective scope resolution.
 */
export type ModelScopeRegistry<TModel extends ReadonlyModel = ReadonlyModel,> = {
  /**
   Return models with configured auth.
   */
  getAvailable: () => readonly TModel[];
};

/**
 Narrow context surface needed for effective scope resolution.
 */
export type ResolveEffectiveScopeContext<TModel extends ReadonlyModel = ReadonlyModel,> = {
  /**
   Current working directory.
   */
  readonly cwd: string;
  /**
   Registry exposing available models.
   */
  readonly modelRegistry: ModelScopeRegistry<TModel>;
  /**
   Optional live-scope getter from pi runtime.
   */
  readonly getScopedModels?: () => unknown;
  /**
   Optional live-scope property from pi runtime.
   */
  readonly scopedModels?: unknown;
};

/**
 Options for resolving effective model scope.
 */
export type ResolveEffectiveScopeOptions<TModel extends ReadonlyModel = ReadonlyModel,> = ForeignBorrowed<{
  /**
   Narrow pi extension context.
   */
  readonly ctx: ResolveEffectiveScopeContext<TModel>;
  /**
   Process argv override for tests.
   */
  readonly argv?: readonly string[];
  /**
   Home directory override for tests.
   */
  readonly home?: string;
  /**
   Error prefix used by settings validation.
   */
  readonly errorPrefix?: string;
}>;

/**
 Raw live scope item shapes accepted by the detector.
 */
type RawLiveScopeItem<TModel extends ReadonlyModel = ReadonlyModel,> = TModel | {
  /**
   Pi model object.
   */
  readonly model: TModel;
  /**
   Optional thinking level carried by pi.
   */
  readonly thinkingLevel?: ScopedThinkingLevel;
};

//endregion Types

//region Public API

/**
 Resolve the effective scoped model set.
 
 Resolution order matches pi behavior reconstructed for consumers: live scope,
 argv `--models`, pi settings, then available models.
 
 @param ctx - narrow pi extension context
 
 @param argv - optional argv override for tests
 
 @param home - optional home directory override for tests
 
 @param errorPrefix - optional settings-validation error prefix
 
 @returns scoped models and source metadata
 
 @mutates ctx - invokes optional `getScopedModels` and model-registry `getAvailable` callbacks supplied by context
 
 @example
 ```typescript
 const scope = await resolveEffectiveScope({ ctx });
 ```
 */
export async function resolveEffectiveScope<TModel extends ReadonlyModel,>(
  {
    ctx,
    argv,
    home,
    errorPrefix,
  }: ResolveEffectiveScopeOptions<TModel>,
): Promise<EffectiveModelScope<TModel>> {
  /**
   Live model scope exposed by current or future pi APIs.
   */
  const liveScope = readLiveScope<TModel>(ctx,);
  if (liveScope !== NO_LIVE_SCOPE) {
    return {
      source: 'live',
      entries: liveScope,
    };
  }

  /**
   Patterns from pi's startup `--models` flag.
   */
  const argvPatterns = parseArgvModelPatterns({
    argv: argv
      ?? process
        .argv,
  },);
  if (argvPatterns !== NO_ARGV_MODELS) {
    return {
      source: 'argv',
      entries: resolveModelPatterns({
        patterns: argvPatterns,
        availableModels: ctx
          .modelRegistry
          .getAvailable(),
      },),
      patterns: argvPatterns,
    };
  }

  /**
   Patterns from merged pi settings.
   */
  const settingsScope = await loadSettingsScopePatterns({
    cwd: ctx.cwd,
    ...(home === undefined ? {} : { home, }),
    ...(errorPrefix === undefined ? {} : { errorPrefix, }),
  },);
  if (settingsScope.patterns
    !== undefined) {
    return {
      source: 'settings',
      entries: resolveModelPatterns({
        patterns: settingsScope.patterns,
        availableModels: ctx
          .modelRegistry
          .getAvailable(),
      },),
      patterns: settingsScope.patterns,
    };
  }

  return {
    source: 'available',
    entries: ctx
      .modelRegistry
      .getAvailable()
      .map(function mapAvailableModel(model,) {
        return scopedModelFromModel({ model, },);
      },),
  };
}

//endregion Public API

//region Live scope detection

/**
 Read live scoped models from a runtime context when pi exposes them.
 Pi's empty cycle list means no live restriction, including SDK sessions that
 omit `scopedModels`. A nonempty list containing only malformed entries remains
 an authoritative empty result instead of widening selection.
 
 @param getScopedModels - optional live-scope getter
 
 @param scopedModels - optional live-scope property
 
 @returns live scoped models, or {@link NO_LIVE_SCOPE} when unavailable
 
 @mutates getScopedModels - invokes supplied live-scope callback when present

 @example
 ```typescript
 const scope = readLiveScope(ctx);
 ```
 */
export function readLiveScope<TModel extends ReadonlyModel,>(
  {
    getScopedModels,
    scopedModels,
  }: ForeignBorrowed<Pick<ResolveEffectiveScopeContext<TModel>, 'getScopedModels' | 'scopedModels'>>,
): ScopedModel<TModel>[] | typeof NO_LIVE_SCOPE {
  /**
   Raw live scope value from method or property.
   */
  const rawScope = getScopedModels === undefined
    ? scopedModels
    : getScopedModels();
  if ((!Array.isArray(rawScope,)) || (rawScope.length === 0))
    return NO_LIVE_SCOPE;

  return rawScope
    .filter(function keepRawLiveScopeItem(value,): value is RawLiveScopeItem<TModel> {
      return isRawLiveScopeItem(value,);
    },)
    .map(function mapLiveScopeItem(item,) {
      return isModel(item,)
        ? scopedModelFromModel({ model: item, },)
        : scopedModelFromModel({
          model: item.model,
          ...(item.thinkingLevel
            === undefined
            ? {}
            : { thinkingLevel: item.thinkingLevel, }),
        },);
    },);
}

/**
 Detect raw live-scope item shapes.
 
 @param value - value to inspect
 
 @returns whether value is usable as a live scope item
 */
function isRawLiveScopeItem<TModel extends ReadonlyModel,>(
  value: unknown,
): value is RawLiveScopeItem<TModel> {
  if (isModel<TModel>(value,))
    return true;
  if ((value === null) || ((typeof value) !== 'object'))
    return false;
  if (!('model' in value))
    return false;
  return isModel<TModel>(value.model,);
}

/* oxlint-disable typescript/no-unnecessary-type-parameters -- predicate preserves generic caller model shape after structural validation. */
/**
 Detect pi model objects structurally.
 
 @param value - value to inspect
 
 @returns whether value looks like a pi model
 */
function isModel<TModel extends ReadonlyModel,>(
  value: unknown,
): value is TModel {
  if ((value === null) || ((typeof value) !== 'object'))
    return false;
  return ('id' in value)
    && ((typeof value.id) === 'string')
    && ('name' in value)
    && ((typeof value.name) === 'string')
    && ('provider' in value)
    && ((typeof value.provider) === 'string')
    && ('api' in value)
    && ((typeof value.api) === 'string')
    && ('contextWindow' in value)
    && ((typeof value.contextWindow) === 'number')
    && ('maxTokens' in value)
    && ((typeof value.maxTokens) === 'number')
    && ('cost' in value)
    && (value.cost
      !== null)
    && ((typeof value.cost) === 'object');
}
/* oxlint-enable typescript/no-unnecessary-type-parameters */

//endregion Live scope detection
