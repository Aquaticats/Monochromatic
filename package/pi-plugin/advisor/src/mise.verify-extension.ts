/**
 Verifies built Advisor extension registers expected Pi resources.
 
 @module
 */

import type { Provider, } from '@earendil-works/pi-ai';
import {
  createEventBus,
  type ExecResult,
  type ExtensionAPI,
  type ExtensionFactory,
  type MarkdownTransformer,
} from '@earendil-works/pi-coding-agent';
import type { ReadonlyDeep, } from 'type-fest';

//region Constants

/**
 Built extension path consumed by Pi.
 */
const BUILT_EXTENSION_PATH = '../dist/final/node/index.mjs';

/**
 Expected resource registrations from the extension entry point.
 */
const EXPECTED_REGISTRATIONS = [
  'tool:advisor',
  'command:advisor',
  'renderer:pi-advisor.review',
  'event:session_start',
  'event:before_agent_start',
] as const;

//endregion Constants

//region Types

/**
 Built Advisor extension module shape.
 */
type AdvisorExtensionModule = {
  /**
   Pi extension factory.
   */
  readonly default: ExtensionFactory;
};

/**
 Registered tool subset inspected by verification.
 */
type RegisteredTool = {
  /**
   Tool name.
   */
  readonly name: string;
  /**
   Whether tool schema exposes focused-question parameter.
   */
  readonly questionParameterPresent: boolean;
};

//endregion Types

//region Verification

/**
 Verify built extension registers expected Pi resources.
 
 @returns verification result text
 
 @throws when built extension import or registration fails
 */
async function verifyBuiltExtension(): Promise<string> {
  /**
   Built extension module imported through package output.
   */
  const mod: unknown = await import(BUILT_EXTENSION_PATH);
  if (!isAdvisorExtensionModule(mod,)) {
    throw new Error(
      'built Advisor extension does not export a default extension factory',
    );
  }

  /**
   Fake Pi API and its registration call log.
   */
  const fakeApi = fakePiApi();
  await mod.default(fakeApi.api,);

  /**
   Snapshot of recorded registration calls.
   */
  const registrations = fakeApi.registrations();

  /**
   Expected registrations not observed.
   */
  const missing = EXPECTED_REGISTRATIONS.filter(function isMissing(expected,) {
    return !registrations.includes(expected,);
  },);
  if (missing.length
    > 0)
    throw new Error(`missing Advisor registrations: ${missing.join(', ',)}`,);

  assertAdvisorQuestionParameter(fakeApi.tools(),);

  return `Advisor extension verified: ${registrations.join(', ',)}`;
}

/**
 Detect built Advisor extension module shape.
 
 @param value - imported module namespace
 
 @returns whether module exports an extension factory
 */
function isAdvisorExtensionModule(
  value: unknown,
): value is AdvisorExtensionModule {
  if ((value === null) || ((typeof value) !== 'object'))
    return false;
  return ('default' in value) && ((typeof value.default) === 'function');
}

/**
 Return registered Advisor tool from captured registrations.
 
 @param tools - registered tools
 
 @returns registered Advisor tool
 
 @throws when Advisor tool was not captured
 */
function advisorToolOrThrow(
  tools: readonly ReadonlyDeep<RegisteredTool>[],
): ReadonlyDeep<RegisteredTool> {
  for (const tool of tools) {
    if (tool.name
      === 'advisor')
      return tool;
  }
  throw new Error('registered Advisor tool was not captured',);
}

/**
 Assert built Advisor tool exposes focused-question parameter.
 
 @param tools - registered tools
 
 @throws when Advisor question parameter is missing
 */
function assertAdvisorQuestionParameter(
  tools: readonly ReadonlyDeep<RegisteredTool>[],
): void {
  /**
   Registered Advisor tool definition discovered by linear scan.
   */
  const advisorTool = advisorToolOrThrow(tools,);
  if (!advisorTool.questionParameterPresent)
    throw new Error('registered Advisor tool is missing question parameter',);
}

/**
 Build fake Pi API used to verify extension registration.
 
 @returns fake Pi extension API and registrations accessor
 */
function fakePiApi(): {
  readonly api: ExtensionAPI;
  readonly registrations: () => readonly string[];
  readonly tools: () => readonly ReadonlyDeep<RegisteredTool>[];
} {
  /**
   Locally-owned registration log accessed through closures.
   */
  const registrations: string[] = [];
  /**
   Registered tool definitions accessed through closures.
   */
  const tools: RegisteredTool[] = [];
  /**
   Fake provider registrar accepting both current Pi overloads.
   
   @param providerOrName - complete provider or legacy provider identity
   
   */
  function registerProvider(
    providerOrName: Provider | string,
  ): void {
    /**
     Provider identity normalized from complete-provider or legacy arguments.
     */
    const providerName = (typeof providerOrName)
      === 'string'
      ? providerOrName
      : providerOrName.id;
    registrations.push(`provider:${providerName}`,);
  }
  /**
   Fake extension API that records registration calls into the closure.
   */
  const api: ExtensionAPI = {
    on(event: string,) {
      registrations.push(`event:${event}`,);
      // Pi 0.87 `on` returns an unsubscribe callback.
      return function unsubscribe(): void {
        registrations.push(`unevent:${event}`,);
      };
    },
    registerTool(tool: ReadonlyDeep<{
      readonly name: string;
      readonly parameters?: {
        readonly properties?: Record<string, unknown>;
      };
    }>,) {
      registrations.push(`tool:${tool.name}`,);
      /**
       Captured focused-question parameter schema.
       */
      const questionParameter = tool
        .parameters
        ?.properties
        ?.question;
      /**
       Whether captured schema exposes focused-question parameter.
       */
      const questionParameterPresent = questionParameter !== undefined;
      tools.push({
        name: tool.name,
        questionParameterPresent,
      },);
    },
    registerCommand(name: string,) {
      registrations.push(`command:${name}`,);
    },
    registerShortcut(shortcut: string,) {
      registrations.push(`shortcut:${shortcut}`,);
    },
    registerFlag(name: string,) {
      registrations.push(`flag:${name}`,);
    },
    getFlag() {
      return undefined;
    },
    registerMessageRenderer(customType: string,) {
      registrations.push(`renderer:${customType}`,);
    },
    registerMarkdownTransformer(transformer: MarkdownTransformer,) {
      void transformer;
    },
    registerEntryRenderer(customType: string,) {
      registrations.push(`entry-renderer:${customType}`,);
    },
    registerToolRenderer() {
      registrations.push('tool-renderer:registered',);
    },
    sendMessage(message: { readonly customType: string; },) {
      registrations.push(`message:${message.customType}`,);
    },
    sendUserMessage(content: unknown,) {
      void content;
    },
    appendEntry(customType: string,) {
      registrations.push(`entry:${customType}`,);
    },
    setSessionName(name: string,) {
      void name;
    },
    getSessionName() {
      return undefined;
    },
    setLabel(
      entryId: string,
      label?: string,
    ) {
      void entryId;
      void label;
    },
    exec(
      command: string,
      args: readonly string[],
    ): Promise<ExecResult> {
      void command;
      void args;
      return Promise.resolve({
        stdout: '',
        stderr: '',
        code: 0,
        killed: false,
      },);
    },
    getActiveTools() {
      return [];
    },
    getAllTools() {
      return [];
    },
    getSettings() {
      return {};
    },
    setActiveTools(toolNames: readonly string[],) {
      void toolNames;
    },
    getCommands() {
      return [];
    },
    setModel(model: unknown,) {
      void model;
      return Promise.resolve(false,);
    },
    getThinkingLevel() {
      return 'off';
    },
    setThinkingLevel(level: string,) {
      void level;
    },
    registerProvider,
    unregisterProvider(name: string,) {
      registrations.push(`unprovider:${name}`,);
    },
    registerMcpServer(name: string,) {
      registrations.push(`mcp-server:${name}`,);
    },
    unregisterMcpServer(name: string,) {
      registrations.push(`unmcp-server:${name}`,);
    },
    getMcpServers() {
      return [];
    },
    registerVirtualModel() {
      registrations.push('virtual-model:registered',);
    },
    unregisterVirtualModel(
      provider: string,
      id: string,
    ) {
      registrations.push(`unvirtual-model:${provider}/${id}`,);
    },
    events: createEventBus(),
  };
  return {
    api,
    registrations: function snapshotRegistrations() {
      return [...registrations,];
    },
    tools: function snapshotTools() {
      return [...tools,];
    },
  };
}

//endregion Verification

console.log(await verifyBuiltExtension(),);
