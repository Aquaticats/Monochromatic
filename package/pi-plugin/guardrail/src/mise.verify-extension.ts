/**
 Verifies the built pi guardrail extension registers expected Pi events.
 
 @module
 */

import { mkdtemp, } from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';
import {
  createEventBus,
  type ExecResult,
  type ExtensionAPI,
  type MarkdownTransformer,
  type ExtensionFactory,
} from '@earendil-works/pi-coding-agent';

//region Constants

/**
 Built extension path consumed by pi.
 */
const BUILT_EXTENSION_PATH = '../dist/final/node/index.mjs';

/**
 Expected event registrations from the extension entry point.
 */
const EXPECTED_REGISTRATIONS = [
  'event:tool_call',
] as const;

//endregion Constants

//region Types

/**
 Built pi guardrail extension module shape.
 */
type PiGuardrailExtensionModule = {
  /**
   Pi extension factory.
   */
  readonly default: ExtensionFactory;
};

/**
 Disposable environment variable override.
 */
type EnvOverride = {
  /**
   Restores original environment variable value.
   */
  readonly [Symbol.dispose]: () => void;
};

//endregion Types

//region Verification

/**
 Verifies that built extension exports and registers correctly.
 
 @returns verification result text
 
 @throws when built extension import or registration fails
 
 @example
 ```typescript
 console.log(await verifyBuiltExtension());
 ```
 */
async function verifyBuiltExtension(): Promise<string> {
  /**
   Temporary HOME ensuring verification ignores user-global config.
   */
  const home = await mkdtemp(join(
    scratchRoot(),
    'pi-guardrail-home-',
  ),);
  /**
   Environment override forcing config lookup into the temporary HOME.
   */
  using _home = envVar({
    name: 'HOME',
    value: home,
  },);

  /**
   Built extension module imported through package output.
   */
  const mod: unknown = await import(BUILT_EXTENSION_PATH);
  if (!isPiGuardrailExtensionModule(mod,)) {
    throw new Error(
      'built pi guardrail extension does not export a default extension factory',
    );
  }

  /**
   Fake Pi API plus captured registration log.
   */
  const {
    api,
    registrations,
  } = fakePiApi();
  await mod.default(api,);

  /**
   Expected registrations not observed.
   */
  const missing = EXPECTED_REGISTRATIONS.filter(function isMissing(expected,) {
    return !registrations.includes(expected,);
  },);
  if (missing.length > 0) {
    throw new Error(
      `missing pi guardrail registrations: ${missing.join(', ',)}`,
    );
  }

  return `pi guardrail extension verified: ${registrations.join(', ',)}`;
}

/**
 Detects built pi guardrail extension module shape.
 
 @param value - imported module namespace
 
 @returns whether module exports an extension factory
 */
function isPiGuardrailExtensionModule(value: unknown,): value is PiGuardrailExtensionModule {
  if ((value === null) || ((typeof value) !== 'object'))
    return false;
  return ('default' in value) && ((typeof value.default) === 'function');
}

/**
 Builds fake Pi API used to verify extension registration.
 
 @returns fake Pi extension API plus captured registration log
 */
function fakePiApi(): {
  readonly api: ExtensionAPI;
  readonly registrations: readonly string[];
} {
  /**
   Mutable registration call log captured by fake API.
   */
  const registrations: string[] = [];
  /**
   Fake Pi API recording each registration into captured log.
   */
  const api: ExtensionAPI = {
    on(event: string,) {
      registrations.push(`event:${event}`,);
      // Pi 0.87 `on` returns an unsubscribe callback.
      return function unsubscribe(): void {
        registrations.push(`unevent:${event}`,);
      };
    },
    registerTool(tool: { readonly name: string; },) {
      registrations.push(`tool:${tool.name}`,);
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
      return 'high';
    },
    setThinkingLevel(level: string,) {
      void level;
    },
    registerProvider(providerOrName: unknown,) {
      // Pi 0.87 adds a complete-provider overload beside the legacy name-plus-config form.
      /**
       Provider identity recorded for legacy name registrations.
       */
      const providerName = (typeof providerOrName) === 'string'
        ? providerOrName
        : 'complete-provider';
      registrations.push(`provider:${providerName}`,);
    },
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
    registrations,
  };
}

/**
 Temporarily sets an environment variable.
 
 @param name - environment variable name
 
 @param value - replacement value
 
 @returns disposable restorer
 */
function envVar(
  {
    name,
    value,
  }: {
    readonly name: string;
    readonly value: string;
  },
): EnvOverride {
  /**
   Previous value before override.
   */
  const previous = process.env[name];
  process.env[name] = value;
  return {
    [Symbol.dispose]() {
      if (previous === undefined)
        Reflect.deleteProperty(
          process.env,
          name,
        );
      else
        process.env[name] = previous;
    },
  };
}

//endregion Verification

console.log(await verifyBuiltExtension(),);
