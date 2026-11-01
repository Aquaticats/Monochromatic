/**
 Verifies built Pi Search Fetch extension registers expected Pi resources.
 
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
 Built extension path consumed by Pi.
 */
const BUILT_EXTENSION_PATH = '../dist/final/node/index.mjs';

/**
 Expected resource registrations from the extension entry point.
 */
const EXPECTED_REGISTRATIONS = [
  'tool:web_search',
  'tool:web_fetch',
] as const;

/**
 Temp home prefix for isolated config loading.
 */
const TEMP_HOME_PREFIX = 'pi-search-fetch-verify-';

//endregion Constants

//region Types

/**
 Built Pi Search Fetch extension module shape.
 */
type LinkupExtensionModule = {
  /**
   Pi extension factory.
   */
  readonly default: ExtensionFactory;
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
   Temp home directory avoiding real user config during verification.
   */
  const tempHome = await mkdtemp(join(
    scratchRoot(),
    TEMP_HOME_PREFIX,
  ),);
  process.env
    .HOME = tempHome;
  process.chdir(tempHome,);

  /**
   Built extension module imported through package output.
   */
  const mod: unknown = await import(BUILT_EXTENSION_PATH);
  if (!isLinkupExtensionModule(mod,))
    throw new Error('built Pi Search Fetch extension does not export a default extension factory');

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
  if (missing.length > 0)
    throw new Error(`missing Pi Search Fetch registrations: ${missing.join(', ',)}`,);

  /**
   Unexpected registrations observed.
   */
  const unexpected = registrations.filter(function isUnexpected(registration,) {
    return !isExpectedRegistration(registration,);
  },);
  if (unexpected.length > 0)
    throw new Error(`unexpected Pi Search Fetch registrations: ${unexpected.join(', ',)}`,);

  return `Pi Search Fetch extension verified: ${registrations.join(', ',)}`;
}

/**
 Detect built Pi Search Fetch extension module shape.
 
 @param value - imported module namespace
 
 @returns whether module exports an extension factory
 */
function isLinkupExtensionModule(value: unknown,): value is LinkupExtensionModule {
  if ((value === null) || ((typeof value) !== 'object'))
    return false;
  return ('default' in value)
    && ((typeof value.default) === 'function');
}

/**
 Return whether a registration key belongs to this extension.
 
 @param registration - registration key recorded from fake Pi API
 
 @returns whether registration is expected
 */
function isExpectedRegistration(registration: string,): boolean {
  return EXPECTED_REGISTRATIONS.some(function isExpected(expected,) {
    return expected === registration;
  },);
}

/**
 Build fake Pi API used to verify extension registration.
 
 @returns fake Pi extension API and registrations accessor
 */
function fakePiApi(): {
  readonly api: ExtensionAPI;
  readonly registrations: () => readonly string[];
} {
  /**
   Locally owned registration log accessed through closures.
   */
  const registrations: string[] = [];
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
      return 'off';
    },
    setThinkingLevel(level: string,) {
      void level;
    },
    registerProvider(
      providerOrName: unknown,
      config?: unknown,
    ) {
      void config;
      registrations.push(`provider:${String(providerOrName,)}`,);
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
    registrations: function snapshot() {
      return [...registrations,];
    },
  };
}

//endregion Verification

console.log(await verifyBuiltExtension(),);
