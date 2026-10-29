/**
 Shared Advisor run-context assembly for DSH operations. @module
 */

import type {
  AdvisorRunContext,
  AdvisorTranscript,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';
import type {
  DshAdvisorCatalog,
} from './types.ts';

//region Types

/**
 Inputs for assembling one shared Advisor run context.
 */
export type CreateDshAdvisorRunContextOptions = {
  /**
   Catalog snapshot serving the model-resolution port.
   */
  readonly catalog: DshAdvisorCatalog;
  /**
   Transcript port over the captured session snapshot.
   */
  readonly transcript: AdvisorTranscript;
  /**
   Working directory for project-scope resolution.
   */
  readonly cwd: string;
  /**
   Agent route identity used as the current main model to avoid, when both
   provider and model are configured.
   */
  readonly mainProvider?: string;
  /**
   Agent model id paired with `mainProvider`.
   */
  readonly mainModel?: string;
  /**
   Display name of the agent model, letting default avoidance recognize the
   same model served on another provider route.
   */
  readonly mainModelName?: string;
};

//endregion Types

//region Public API

/**
 Assemble the shared run context for one Advisor operation.

 The current main model is supplied only when the agent carries a complete
 `provider/model` route, so default selection can exclude it exactly as pi
 excludes its active primary model.

 The live scoped-model getter exposes every catalog route the DSH runtime
 resolved with configured auth.
 DSH has no pi-style scoped-model cycle list,
 so without this getter scope resolution falls back to pi-only sources
 and reports an empty scope.

 @param options - catalog, transcript, working directory, and agent route

 @returns shared run context

 @example
 ```ts
 const ctx = createDshAdvisorRunContext({ catalog, transcript, cwd: exec.cwd });
 ```
 */
export function createDshAdvisorRunContext(
  options: CreateDshAdvisorRunContextOptions,
): AdvisorRunContext {
  return {
    modelRegistry: options.catalog,
    getScopedModels: function getScopedModels(): unknown {
      return options.catalog
        .getAvailable();
    },
    transcript: options.transcript,
    cwd: options.cwd,
    ...((options.mainProvider === undefined) || (options.mainModel === undefined)
      ? {}
      : {
        model: {
          provider: options.mainProvider,
          id: options.mainModel,
          ...(options.mainModelName === undefined
            ? {}
            : { name: options.mainModelName, }),
        },
      }),
  };
}

//endregion Public API
