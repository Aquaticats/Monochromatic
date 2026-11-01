/**
 Unit tests for Advisor extension registration and public prompt helpers.
 
 @module
 */

import {
  fauxAssistantMessage,
  fauxProvider,
  getCurrentSystemPrompt,
  type Context,
  type Model,
} from '@earendil-works/pi-ai';
import type {
  ExtensionAPI,
  ExtensionCommandContext,
  ExtensionContext,
  ModelRegistry,
} from '@earendil-works/pi-coding-agent';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import advisor, {
  type AdvisorConfig,
  ADVISOR_MESSAGE_TYPE,
  ADVISOR_TOOL_NAME,
  buildAdvisorStatus,
  buildMainModelGuidance,
  DEFAULT_CONFIG,
} from '../dist/final/node/index.mjs';

//region Fixtures

/** Fixture context budget. */
const CONTEXT_WINDOW = 1_000;

/** Configured Advisor output requirement. */
const ADVISOR_OUTPUT_TOKENS = 32;

/** Fixture max output tokens. */
const MAX_TOKENS = 100;

/** Advertised output capacity below the request budget, so requests clamp. */
const CAPACITY_LIMITED_MAX_TOKENS = ADVISOR_OUTPUT_TOKENS - 1;

/** Fixture model used by registration status helpers. */
const fixtureModel: Model<'faux'> = {
  id: 'reviewer',
  name: 'Reviewer',
  api: 'faux',
  provider: 'faux-provider',
  baseUrl: 'https://example.invalid',
  reasoning: false,
  input: ['text',],
  cost: {
    input: 1,
    output: 2,
    cacheRead: 0,
    cacheWrite: 0,
  },
  contextWindow: CONTEXT_WINDOW,
  maxTokens: MAX_TOKENS,
};

/** Scoped fixture advertising less output capacity than the request budget. */
const capacityLimitedFixtureModel: Model<'faux'> = {
  ...fixtureModel,
  id: 'limited-reviewer',
  name: 'Limited Reviewer',
  provider: 'limited-provider',
  maxTokens: CAPACITY_LIMITED_MAX_TOKENS,
};

/** Expensive current-main fixture used by default-avoidance tests. */
const expensiveFixtureModel: Model<'faux'> = {
  ...fixtureModel,
  name: 'Expensive Reviewer',
  provider: 'expensive-provider',
  cost: {
    input: 10,
    output: 20,
    cacheRead: 0,
    cacheWrite: 0,
  },
};

/** Advisor config fixture. */
const advisorConfig: AdvisorConfig = {
  ...DEFAULT_CONFIG,
  maxAdvisorOutputTokens: ADVISOR_OUTPUT_TOKENS,
  source: {
    globalPath: '/home/test/.pi/agent/extensions/pi-advisor.json',
    projectPath: '/repo/.pi/extensions/pi-advisor.json',
    globalLoaded: false,
    projectLoaded: false,
  },
};

/** Generic recorded extension callback used by registration probes. */
type RecordedHandler = (...args: unknown[]) => unknown;

/** Recorded extension API calls. */
type RecordedPi = {
  /** Registered tool names. */
  tools: string[];
  /** Registered command names. */
  commands: string[];
  /** Registered message renderer types. */
  renderers: string[];
  /** Registered event names. */
  events: string[];
  /** Registered event callbacks keyed by event name. */
  eventHandlers: Map<string, RecordedHandler>;
  /** Registered tool callbacks keyed by tool name. */
  toolHandlers: Map<string, RecordedHandler>;
  /** Registered command callbacks keyed by command name. */
  commandHandlers: Map<string, RecordedHandler>;
  /** Fake extension API. */
  api: ExtensionAPI;
};

/**
 Build a minimal model registry mock.
 
 @param models - available models
 
 @returns model registry mock
 */
function modelRegistryWith(
  models: readonly Model<'faux'>[],
): ModelRegistry {
  return {
    getAvailable() {
      return [...models,];
    },
    getAll() {
      return [...models,];
    },
  } as unknown as ModelRegistry;
}

/**
 Build a minimal extension context mock.
 
 @returns extension context mock
 */
function extensionContext(): ExtensionContext {
  return {
    cwd: '/repo',
    modelRegistry: modelRegistryWith([fixtureModel,],),
    scopedModels: [fixtureModel,],
  } as unknown as ExtensionContext;
}

/**
 Build a minimal extension context with current main model also in scope.
 
 @returns extension context mock
 */
function extensionContextWithCurrentMainModel(): ExtensionContext {
  return {
    cwd: '/repo',
    model: expensiveFixtureModel,
    modelRegistry: modelRegistryWith([
      fixtureModel,
      expensiveFixtureModel,
    ],),
    scopedModels: [
      fixtureModel,
      expensiveFixtureModel,
    ],
  } as unknown as ExtensionContext;
}

/**
 Build a minimal extension context containing one capacity-limited model.

 @returns extension context mock
 */
function extensionContextWithCapacityLimitedModel(): ExtensionContext {
  return {
    cwd: '/repo',
    modelRegistry: modelRegistryWith([
      fixtureModel,
      capacityLimitedFixtureModel,
    ],),
    scopedModels: [
      fixtureModel,
      capacityLimitedFixtureModel,
    ],
  } as unknown as ExtensionContext;
}

/**
 Build a minimal command context mock.
 
 @returns command context mock
 */
function commandContext(): ExtensionCommandContext {
  return extensionContext() as unknown as ExtensionCommandContext;
}

/**
 Build a minimal command context with current main model also in scope.
 
 @returns command context mock
 */
function commandContextWithCurrentMainModel(): ExtensionCommandContext {
  return extensionContextWithCurrentMainModel() as unknown as ExtensionCommandContext;
}

/**
 Build a command context containing one capacity-limited model.

 @returns command context mock
 */
function commandContextWithCapacityLimitedModel(): ExtensionCommandContext {
  return extensionContextWithCapacityLimitedModel() as unknown as ExtensionCommandContext;
}

/**
 Build a minimal extension API mock that records registration calls.
 
 @returns recorded API state
 */
function recordedPi(): RecordedPi {
  const recorded: Omit<RecordedPi, 'api'> = {
    tools: [],
    commands: [],
    renderers: [],
    events: [],
    eventHandlers: new Map(),
    toolHandlers: new Map(),
    commandHandlers: new Map(),
  };
  return {
    ...recorded,
    api: {
      registerTool(tool: Record<string, unknown>,) {
        /** Registered tool name from Pi definition. */
        const name = tool.name as string;
        recorded.tools.push(name,);
        recorded.toolHandlers.set(name, tool.execute as RecordedHandler,);
      },
      registerCommand(
        name: string,
        options: Record<string, unknown>,
      ) {
        recorded.commands.push(name,);
        recorded.commandHandlers.set(name, options.handler as RecordedHandler,);
      },
      registerMessageRenderer(customType: string,) {
        recorded.renderers.push(customType,);
      },
      on(
        event: string,
        handler: RecordedHandler,
      ) {
        recorded.events.push(event,);
        recorded.eventHandlers.set(event, handler,);
      },
      getActiveTools() {
        return [];
      },
      setActiveTools() {},
      sendMessage() {},
      appendEntry() {},
    } as unknown as ExtensionAPI,
  };
}

/**
 Get required recorded callback from registration map.
 
 @param handlers - callbacks keyed by registered name
 
 @param name - required registration name
 
 @returns recorded callback
 */
function registeredHandler(
  {
    handlers,
    name,
  }: {
    readonly handlers: ReadonlyMap<string, RecordedHandler>;
    readonly name: string;
  },
): RecordedHandler {
  /** Registered callback for requested name. */
  const handler = handlers.get(name,);
  if (handler === undefined)
    throw new Error(`Missing registered handler: ${name}`,);
  return handler;
}

//endregion Fixtures

await describe({
  name: advisor.name,
  children: [
    it({
      name: 'registers tool, command, renderer, and lifecycle hooks',
      fn: async () => {
        const recorded = recordedPi();
        await advisor(recorded.api,);
        expect(recorded.tools,).toEqual([ADVISOR_TOOL_NAME,],);
        expect(recorded.commands,).toEqual([ADVISOR_TOOL_NAME,],);
        expect(recorded.renderers,).toEqual([ADVISOR_MESSAGE_TYPE,],);
        expect(recorded.events,).toEqual([
          'tool_result',
          'session_shutdown',
          'session_start',
          'before_agent_start',
        ],);
      },
    },),
    it({
      name: 'forwards prompt snapshot to tool and live prompt options to command',
      fn: async function forwardsProjectContextAcrossRegisteredPaths(): Promise<void> {
        /** Recorded extension registrations under lifecycle integration test. */
        const recorded = recordedPi();
        await advisor(recorded.api,);
        /** Faux provider used at real Advisor provider boundary. */
        const providerFixture = fauxProvider({
          api: 'faux',
          provider: 'faux-provider',
          models: [{
            id: 'reviewer',
            reasoning: false,
            maxTokens: DEFAULT_CONFIG.maxAdvisorOutputTokens,
          },],
        },);
        /** Provider contexts from tool and command calls in order. */
        const providerContexts: Context[] = [];
        providerFixture.setResponses([
          function toolResponse(context,) {
            providerContexts.push(context,);
            return fauxAssistantMessage('tool advisor answer',);
          },
          function commandResponse(context,) {
            providerContexts.push(context,);
            return fauxAssistantMessage('command advisor answer',);
          },
        ],);
        /** Shared extension context with scoped model and provider access. */
        const ctx = {
          cwd: '/repo',
          scopedModels: [providerFixture.getModel(),],
          modelRegistry: {
            getAll() {
              return providerFixture.models;
            },
            getAvailable() {
              return providerFixture.models;
            },
            async getApiKeyAndHeaders() {
              return {
                ok: true,
                apiKey: 'test-key',
              };
            },
            getProvider() {
              return providerFixture.provider;
            },
          },
          sessionManager: {
            buildContextEntries() {
              return [];
            },
          },
        } as unknown as ExtensionContext;
        /** Prompt hook capturing tool-run project context. */
        const beforeAgentStart = registeredHandler({
          handlers: recorded.eventHandlers,
          name: 'before_agent_start',
        },);
        /** Registered Advisor tool execution boundary. */
        const executeTool = registeredHandler({
          handlers: recorded.toolHandlers,
          name: ADVISOR_TOOL_NAME,
        },);
        /** Session boundary that clears tool-run snapshot. */
        const sessionStart = registeredHandler({
          handlers: recorded.eventHandlers,
          name: 'session_start',
        },);
        /** Registered manual Advisor command boundary. */
        const executeCommand = registeredHandler({
          handlers: recorded.commandHandlers,
          name: ADVISOR_TOOL_NAME,
        },);

        await beforeAgentStart(
          {
            type: 'before_agent_start',
            prompt: 'Review current work.',
            systemPrompt: 'Main prompt.',
            systemPromptOptions: {
              cwd: '/repo',
              contextFiles: [{
                path: '/repo/AGENTS.md',
                content: 'Tool snapshot guidance.',
              },],
            },
          },
          ctx,
        );
        await executeTool(
          'advisor-context-tool',
          { model: 'faux-provider/reviewer', },
          undefined,
          undefined,
          ctx,
        );
        await sessionStart({}, ctx,);
        /** Command context exposing current prompt options after idle. */
        const commandCtx = {
          ...ctx,
          async waitForIdle() {},
          getSystemPromptOptions() {
            return {
              cwd: '/repo',
              contextFiles: [{
                path: '/repo/AGENTS.md',
                content: 'Command live guidance.',
              },],
            };
          },
          ui: {
            notify() {},
          },
        } as unknown as ExtensionCommandContext;
        await executeCommand('faux-provider/reviewer', commandCtx,);

        expect(providerContexts,).toHaveLength(2,);
        expect(getCurrentSystemPrompt(providerContexts[0]?.messages ?? [],),).toContain('Tool snapshot guidance.',);
        expect(getCurrentSystemPrompt(providerContexts[1]?.messages ?? [],),).toContain('Command live guidance.',);
        expect(getCurrentSystemPrompt(providerContexts[1]?.messages ?? [],),).not.toContain('Tool snapshot guidance.',);
      },
    },),
  ],
},);

await describe({
  name: buildMainModelGuidance.name,
  children: [
    it({
      name: 'lists scoped slugs and default model',
      fn: async () => {
        const guidance = await buildMainModelGuidance({
          ctx: extensionContext(),
          config: advisorConfig,
        },);
        expect(guidance,).toContain('faux-provider/reviewer',);
        expect(guidance,).toContain('advisor({}) default model: faux-provider/reviewer',);
      },
    },),
    it({
      name: 'lists non-current default model when alternate remains',
      fn: async () => {
        const guidance = await buildMainModelGuidance({
          ctx: extensionContextWithCurrentMainModel(),
          config: advisorConfig,
        },);
        expect(guidance,).toContain('expensive-provider/reviewer',);
        expect(guidance,).toContain('advisor({}) default model: faux-provider/reviewer',);
      },
    },),
    it({
      name: 'lists every scoped model in allowed slugs',
      fn: async () => {
        const guidance = await buildMainModelGuidance({
          ctx: extensionContextWithCapacityLimitedModel(),
          config: advisorConfig,
        },);
        expect(guidance,).toContain(
          'Allowed Advisor model slugs: faux-provider/reviewer, limited-provider/limited-reviewer',
        );
      },
    },),
  ],
},);

await describe({
  name: buildAdvisorStatus.name,
  children: [
    it({
      name: 'reports enablement and scoped default',
      fn: async () => {
        const status = await buildAdvisorStatus({
          ctx: commandContext(),
          config: advisorConfig,
          enabled: true,
        },);
        expect(status,).toContain('Advisor: on',);
        expect(status,).toContain('Default model: faux-provider/reviewer',);
      },
    },),
    it({
      name: 'reports non-current default model when alternate remains',
      fn: async () => {
        const status = await buildAdvisorStatus({
          ctx: commandContextWithCurrentMainModel(),
          config: advisorConfig,
          enabled: true,
        },);
        expect(status,).toContain('Scoped models: 2',);
        expect(status,).not.toContain('faux-provider/reviewer, expensive-provider/reviewer',);
        expect(status,).toContain('Default model: faux-provider/reviewer',);
      },
    },),
    it({
      name: 'reports request budget and scoped count',
      fn: async () => {
        const status = await buildAdvisorStatus({
          ctx: commandContextWithCapacityLimitedModel(),
          config: advisorConfig,
          enabled: true,
        },);
        expect(status,).toContain(
          `Advisor request budget: ${String(ADVISOR_OUTPUT_TOKENS,)} output tokens, clamped to each route's advertised maximum`,
        );
        expect(status,).toContain('Scoped models: 2',);
        expect(status,).not.toContain('faux-provider/reviewer, limited-provider/limited-reviewer',);
      },
    },),
  ],
},);
