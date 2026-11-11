/**
 Verifies the built Qwen Token Plan Individual extension registers the provider and lifecycle
 handlers Pi expects.

 The fake Pi API follows the built-module verification pattern in
 `package/pi-plugin/thinking-default/src/mise.verify-extension.ts`.

 @module
 */

import {
  createEventBus,
  type ExecResult,
  type ExtensionAPI,
  type ExtensionFactory,
  type MarkdownTransformer,
} from '@earendil-works/pi-coding-agent';
import {
  QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE,
} from '@monochromatic-dev/agent-harness-shared-qwen-token-plan-individual-listing/ts';

//region Constants

/**
 Built extension path consumed by Pi.
 */
const BUILT_EXTENSION_PATH = '../dist/final/node/index.mjs';

/**
 Expected registrations from the extension entry point.
 */
const EXPECTED_REGISTRATIONS = [
  'provider:qwen-token-plan-individual',
  'event:session_start',
  'event:session_shutdown',
] as const;

//endregion Constants

//region Types

/**
 Built Qwen Token Plan Individual extension module shape.
 */
type QwenTokenPlanIndividualExtensionModule = {
  /**
   Pi extension factory.
   */
  readonly default: ExtensionFactory;
};

//endregion Types

//region Verification

/**
 Type guard for the built extension module shape.

 @param value - imported module namespace

 @returns whether the module carries a default extension factory
 */
function isQwenTokenPlanIndividualExtensionModule(
  value: unknown,
): value is QwenTokenPlanIndividualExtensionModule {
  return ((typeof value) === 'object')
    && (value !== null)
    && ('default' in value)
    && ((typeof value.default) === 'function');
}

/**
 Build a fake Pi API recording registrations.

 @returns the fake API and its registration log
 */
function fakePiApi(): {
  readonly api: ExtensionAPI;
  readonly registrations: string[];
} {
  /**
   Registrations the extension made, in order.
   */
  const registrations: string[] = [];
  /**
   Fake Pi API implementation recording every registration it receives.
   */
  const api: ExtensionAPI = {
    on(event: string,) {
      registrations.push(`event:${event}`,);
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
        truncated: false,
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
      /**
       Provider identity recorded for both registration forms.
       */
      const providerId = ((typeof providerOrName) === 'string')
        ? providerOrName
        : (((typeof providerOrName) === 'object')
          && (providerOrName !== null)
          && ('id' in providerOrName)
          && ((typeof providerOrName.id) === 'string')
          ? providerOrName.id
          : 'complete-provider');
      registrations.push(`provider:${providerId}`,);
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
 Assert the built extension reports no missing registration.

 @param registrations - registration log the fake API recorded

 @throws when an expected registration is absent
 */
function assertRegistrations(
  registrations: readonly string[],
): void {
  /**
   Expected registrations not observed.
   */
  const missing = EXPECTED_REGISTRATIONS.filter(function isMissing(expected,) {
    return !registrations.includes(expected,);
  },);
  if (missing.length > 0)
    throw new Error(
      `built qwen-token-plan-individual extension is missing registrations: ${missing.join(', ')}`,
    );
}

/**
 Verify the built extension exports, registers, and names the route.

 @returns verification result text

 @throws when the built extension import or registration fails

 @example
 ```typescript
 console.log(await verifyBuiltExtension());
 ```
 */
async function verifyBuiltExtension(): Promise<string> {
  /**
   Built extension module imported through package output.
   */
  const mod: unknown = await import(BUILT_EXTENSION_PATH);
  if (!isQwenTokenPlanIndividualExtensionModule(mod,)) {
    throw new Error(
      'built qwen-token-plan-individual extension does not export a default extension factory',
    );
  }

  /**
   Fake Pi API plus its captured registration log.
   */
  const {
    api,
    registrations,
  } = fakePiApi();
  await mod.default(api,);
  assertRegistrations(registrations,);
  return `PASS qwen-token-plan-individual extension: ${registrations.join(', ')} (route ${QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE})`;
}

//endregion Verification

/**
 Run the verification and print its result.
 */
const result = await verifyBuiltExtension();
console.log(result);
