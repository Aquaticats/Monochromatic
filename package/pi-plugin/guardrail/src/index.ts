/**
 Pi guardrail extension entry point.
 
 Blocks misleading `bun test` Bash invocations and refuses edit/write tool
 calls for configured gitignore-style protected paths.
 
 @module
 */

import type {
  ExtensionAPI,
  ExtensionContext,
  ToolCallEvent,
} from '@earendil-works/pi-coding-agent';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { evaluateBashGuard, } from './bash-guard.ts';
import { loadGuardrailConfig, } from './config.ts';
import {
  createPathGuardMatcher,
  evaluatePathGuard,
} from './path-guard.ts';
import {
  GUARDRAIL_NOT_BLOCKED,
  type GuardrailBlockDecision,
  type GuardrailConfig,
  type GuardrailDecision,
} from './types.ts';

//region Logger

/**
 Logger root for pi guardrail.
 
 @example
 ```typescript
 const child = tagged({ tag: someFunction.name, l });
 ```
 */
const l = tagged({ tag: 'pi-guardrail', },);

//endregion Logger

//region Types

/**
 Dependencies used while registering the extension.
 */
type RegisterGuardrailOptions = {
  /**
   Pi extension API.
   */
  readonly pi: ExtensionAPI;
  /**
   Config loader dependency; defaults to global config file loading.
   */
  readonly loadConfig?: () => Promise<GuardrailConfig>;
};

//endregion Types

//region Extension entry point

/**
 Registers pi guardrail event handlers.
 
 @param pi - pi extension API
 
 @param loadConfig - optional config loader dependency for tests
 
 @example
 ```typescript
 await registerGuardrail({ pi });
 ```
 */
async function registerGuardrail(
  {
    pi,
    loadConfig = loadGuardrailConfig,
  }: RegisterGuardrailOptions,
): Promise<void> {
  /**
   Function-scoped logger for extension registration.
   */
  const innerL = tagged({
    tag: registerGuardrail.name,
    l,
  },);
  /**
   Runtime config loaded once per extension load.
   */
  const config = await loadConfig();
  /**
   Compiled protected-path matcher using gitignore semantics.
   */
  const pathMatcher = createPathGuardMatcher(config.pathRules,);

  /**
   Config source path used in the startup diagnostic.
   */
  const sourcePath = config.source
    .path;
  /**
   Config source presence used in the startup diagnostic.
   */
  const sourceLoaded = config.source
    .loaded;
  /**
   Number of path rules active after defaults and config merge.
   */
  const pathRuleCount = config.pathRules
    .length;
  /**
   Bash guard toggle used in the startup diagnostic.
   */
  const { blockBunTest, } = config;
  innerL.debug([
    `loaded guardrail config from ${sourcePath}`,
    `present=${String(sourceLoaded,)}`,
    `rules=${String(pathRuleCount,)}`,
    `blockBunTest=${String(blockBunTest,)}`,
  ].join('; ',),);

  pi.on(
    'tool_call',
    async function handleToolCall(
      event: ForeignBorrowed<ToolCallEvent>,
      ctx: ExtensionContext,
    ) {
      /**
       Decision from the first guardrail matching this tool call.
       */
      const decision = await evaluateToolCall({
        event,
        ctx,
        config,
        pathMatcher,
      },);
      if (decision === GUARDRAIL_NOT_BLOCKED)
        return undefined;

      innerL.warn(
        `blocked ${event.toolName} tool call: ${decision.reason}`,
      );
      if (ctx.hasUI) {
        ctx.ui
          .notify(
          decision.reason,
          'warning',
          );
      }
      return decision;
    },
  );
}

/**
 Pi guardrail extension.
 
 @param pi - pi extension API
 
 @example
 ```json
 { "packages": ["/var/home/user/Monochromatic/package/pi-plugin/guardrail"] }
 ```
 */
export default async function piGuardrail(pi: ExtensionAPI,): Promise<void> {
  await registerGuardrail({ pi, },);
}

//endregion Extension entry point

//region Tool dispatch

/**
 Evaluates guardrails for a single pi tool call.
 
 The Bash guards answer asynchronously because the real-Git guard resolves the host's real Git executable.
 
 @param event - pi tool-call event
 
 @param ctx - pi extension context
 
 @param config - loaded guardrail config
 
 @param pathMatcher - compiled path matcher
 
 @param realGitBasenames - executable names treated as the real Git binary, defaulting to the resolved install
 
 @returns block decision when a guardrail matches, otherwise {@link GUARDRAIL_NOT_BLOCKED}
 
 @example
 ```typescript
 await evaluateToolCall({ event, ctx, config, pathMatcher });
 ```
 */
async function evaluateToolCall(
  {
    event,
    ctx,
    config,
    pathMatcher,
    realGitBasenames,
  }: {
    readonly event: ToolCallEvent;
    readonly ctx: ExtensionContext;
    readonly config: GuardrailConfig;
    readonly pathMatcher: ReturnType<typeof createPathGuardMatcher>;
    readonly realGitBasenames?: readonly string[];
  },
): Promise<GuardrailDecision> {
  if (event.toolName === 'bash') {
    return await evaluateBashGuard({
      input: event.input,
      blockBunTest: config.blockBunTest,
      ...(realGitBasenames === undefined ? {} : { realGitBasenames, }),
    },);
  }

  if ((event.toolName !== 'edit') && (event.toolName !== 'write'))
    return GUARDRAIL_NOT_BLOCKED;

  return evaluatePathGuard({
    input: event.input,
    cwd: ctx.cwd,
    matcher: pathMatcher,
  },);
}

//endregion Tool dispatch

export {
  evaluateToolCall,
  registerGuardrail,
};
export type { RegisterGuardrailOptions, };

//region Internal exports for built-artifact tests

/**
 Bash guard entry the extension dispatch calls, exported so built-artifact tests
 can drive the same function a Pi tool call reaches.

 @internal
 */
export { evaluateBashGuard, } from './bash-guard.ts';

/**
 Config loading and normalization steps, exported so built-artifact tests can
 drive each stage the extension runs at load.

 @internal
 */
export { loadGuardrailConfig, } from './config.ts';
export { configPathForHome, } from './config-paths.ts';
export { normalizeConfigFile, } from './config-normalize.ts';
export { parseConfigJson, } from './config-file.ts';

/**
 Config file name and the missing-file code it reports, exported so
 built-artifact tests can build a config fixture that matches the loader.

 @internal
 */
export { FILE_NOT_FOUND_CODE, } from './constants.ts';

/**
 Path guard matcher and evaluation, exported so built-artifact tests can check
 the gitignore-style matching the extension installs.

 @internal
 */
export {
  createPathGuardMatcher,
  evaluatePathGuard,
} from './path-guard.ts';

/**
 Tool path extraction and normalization, exported so built-artifact tests can
 check how a tool call's path becomes the value the matcher sees.

 @internal
 */
export {
  extractToolPath,
  normalizeToolPath,
  TOOL_PATH_NOT_FOUND,
  TOOL_PATH_NOT_MATCHABLE,
} from './path-normalize.ts';

/**
 Guardrail decision values and shapes, exported so built-artifact tests can
 compare decisions and build fixtures with the extension's own types.

 @internal
 */
export { GUARDRAIL_NOT_BLOCKED, } from './types.ts';
export type {
  GuardrailConfig,
  GuardrailDecision,
  PathRule,
} from './types.ts';

//endregion Internal exports for built-artifact tests
