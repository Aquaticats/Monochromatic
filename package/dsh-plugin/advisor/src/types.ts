/**
 DSH-adapter-side types: catalog facts, transcript sources, and plugin row
 config. @module
 */

import type {
  Api,
  Model,
} from '@earendil-works/pi-ai';
import type {
  Message as DshMessage,
  LlmRuntime,
  ReasoningEffortId,
} from '@deepseek-ai/dsh-llm';
import type {
  SessionId,
} from '@deepseek-ai/dsh-session/types';
import type {
  AssembleContext,
} from '@deepseek-ai/dsh-system-prompt';
import type {
  AdvisorModelResolution,
  AdvisorOperationSnapshot,
} from '@monochromatic-dev/agent-harness-shared-advisor/ts';

/**
 Read-only `ctx.llm` surface the plugin consumes.

 `LlmRuntime` is a nominal class (private fields), so tests substitute this
 structural pick while the live instance satisfies it unchanged.

 @example
 ```ts
 const catalog = await buildDshAdvisorCatalog({ llm: ctx.llm, sessionId: agent.id, maxAdvisorOutputTokens });
 ```
 */
export type DshAdvisorLlmSurface = Pick<
  LlmRuntime,
  'listProviders'
    | 'listModels'
    | 'resolveModelInfo'
    | 'stream'
  >;

/**
 One catalog route projected into a pi-ai-shaped model record for the shared
 core, plus the DSH-only facts status and request mapping read.
 */
export type DshAdvisorModelFacts = {
  /**
   pi-ai-shaped model record handed to shared selection and budget code.
   */
  readonly model: Model<Api>;
  /**
   Whether `LlmResolvedModelInfo.defaultMaxTokens` was absent for this route.
   */
  readonly outputCapacityUnknown: boolean;
  /**
   Whether `LlmResolvedModelInfo.context` was absent for this route.
   */
  readonly contextWindowUnknown: boolean;
  /**
   Reasoning effort selected by the DSH rule (greatest effort below `max`),
   when the route advertises any efforts.
   */
  readonly reasoningEffort?: ReasoningEffortId;
  /**
   Canonical `provider/model` slug used for lookups and diagnostics.
   */
  readonly slug: string;
};

/**
 Model-resolution port backed by one `ctx.llm` catalog snapshot.

 The snapshot is built asynchronously per operation because
 `LlmRuntime.resolveModelInfo` is async while the shared port reads models
 synchronously.

 @example
 ```ts
 const catalog = await buildDshAdvisorCatalog({ llm, sessionId, maxAdvisorOutputTokens });
 ```
 */
export type DshAdvisorCatalog = AdvisorModelResolution & {
  /**
   Catalog routes in discovery order, one per provider/model pair.
   */
  readonly entries: readonly DshAdvisorModelFacts[];
  /**
   Facts indexed by canonical `provider/model` slug.
   */
  readonly factsBySlug: ReadonlyMap<string, DshAdvisorModelFacts>;
};

/**
 Minimal session surface the transcript port reads: the derived durable
 message list of one agent session.
 */
export type DshAdvisorMessageSource = {
  /**
   Derive the compaction-aware message history for this session.
   */
  deriveMessages: () => readonly DshMessage[];
};

/**
 Structural agent surface the review pipeline reads.

 The live `Agent` interface carries the cordis `Context` and other nominal
 members, so tool and command adapters project it onto this surface once and
 tests substitute it directly.

 @example
 ```ts
 const agent = toDshAdvisorAgentSurface(exec.agent, workingDirectory);
 ```
 */
export type DshAdvisorAgentSurface = {
  /**
   Session identity stamped onto Advisor requests for provider routing.
   */
  readonly sessionId: SessionId;
  /**
   Derived-history source for the review context.
   */
  readonly session: DshAdvisorMessageSource;
  /**
   Session working directory for project-scope resolution.
   */
  readonly cwd: string;
  /**
   Agent route identity used as the current main model to avoid.
   */
  readonly options: {
    /**
     Provider route of the agent's conversation model, when configured.
     */
    readonly provider?: string;
    /**
     Model id of the agent's conversation model, when configured.
     */
    readonly model?: string;
  };
  /**
   Resolve after the agent's current activity settles.
   */
  readonly whenIdle: () => Promise<void>;
  /**
   Persist one Advisor operation ledger to the durable session log.
   */
  readonly appendOperationRecord: (operation: AdvisorOperationSnapshot) => void;
  /**
   Build the system-prompt assembly context carrying this agent's identity.

   The agent-loop's `model` and `provider` prompt variables read
   `context.agent.options`, so an assembly built without this context fails to
   render any section referencing them.
   */
  readonly assembleContext: (signal?: AbortSignal) => AssembleContext;
};
