/**
 Complete {@link ExtensionToolContext} double for adapter tests.

 Pi types `execute` against a whole host context even though adapter execution ignores it.
 Every member is implemented explicitly so adapter tests need neither type assertions nor
 lint suppressions, and the registry is a real `ModelRegistry` over offline in-memory stores
 because pi's class carries private state no plain object can imitate.

 @module
 */

import {
  ModelRegistry,
  ModelRuntime,
  type ExtensionToolContext,
  type ExtensionUIContext,
} from '@earendil-works/pi-coding-agent';
import {
  InMemoryCredentialStore,
  InMemoryModelsStore,
} from '@earendil-works/pi-ai';

//region Public API

/**
 Build a fully typed host context whose members answer inert defaults.

 @returns context accepted by `ToolDefinition.execute`

 @example
 ```ts
 const ctx = await fakeExtensionToolContext();
 await tool.execute('call-1', { query: 'docs' }, undefined, undefined, ctx);
 ```
 */
export async function fakeExtensionToolContext(): Promise<ExtensionToolContext> {
  /**
   Real registry over offline stores, so no home directory state is read.
   */
  const modelRegistry = new ModelRegistry(await ModelRuntime.create({
    modelsPath: null,
    credentials: new InMemoryCredentialStore(),
    modelsStore: new InMemoryModelsStore(),
  },),);

  return {
    ui: fakeUiContext(),
    mode: 'rpc',
    hasUI: false,
    cwd: '/tmp',
    sessionManager: fakeSessionManager(),
    modelRegistry,
    model: undefined,
    scopedModels: [],
    isIdle() {
      return true;
    },
    isProjectTrusted() {
      return true;
    },
    signal: undefined,
    abort() {
      // Fixture stub: these tests drive no aborts.
    },
    hasPendingMessages() {
      return false;
    },
    shutdown() {
      // Fixture stub: these tests own no session.
    },
    getContextUsage() {
      return undefined;
    },
    compact() {
      // Fixture stub: these tests compact nothing.
    },
    getSystemPrompt() {
      return '';
    },
    tools: [],
    executeTool(): Promise<never> {
      throw new Error('fake ExtensionToolContext cannot run nested tools',);
    },
  };
}

//endregion Public API

//region Helpers

/**
 Build the UI half of the double, answering inert defaults.

 @returns UI context whose calls never reach a terminal

 @example
 ```ts
 fakeUiContext();
 ```
 */
function fakeUiContext(): ExtensionUIContext {
  return {
    select() {
      return Promise.resolve(undefined,);
    },
    confirm() {
      return Promise.resolve(false,);
    },
    input() {
      return Promise.resolve(undefined,);
    },
    notify() {
      // Fixture stub: these tests observe no notifications.
    },
    onTerminalInput() {
      return function unsubscribe(): void {
        // Fixture stub: no terminal input is ever delivered.
      };
    },
    setStatus() {
      // Fixture stub: no status line exists.
    },
    setWorkingMessage() {
      // Fixture stub: no working indicator exists.
    },
    setWorkingVisible() {
      // Fixture stub: no working indicator exists.
    },
    setWorkingIndicator() {
      // Fixture stub: no working indicator exists.
    },
    setHiddenThinkingLabel() {
      // Fixture stub: no thinking label exists.
    },
    setWidget() {
      // Fixture stub: no widgets exist.
    },
    setFooter() {
      // Fixture stub: no footer exists.
    },
    setHeader() {
      // Fixture stub: no header exists.
    },
    setTitle() {
      // Fixture stub: no title exists.
    },
    custom(): Promise<never> {
      throw new Error('fake ExtensionToolContext cannot host overlays',);
    },
    pasteToEditor() {
      // Fixture stub: no editor exists.
    },
    setEditorText() {
      // Fixture stub: no editor exists.
    },
    getEditorText() {
      return '';
    },
    editor() {
      return Promise.resolve(undefined,);
    },
    addAutocompleteProvider() {
      // Fixture stub: no autocomplete exists.
    },
    setEditorComponent() {
      // Fixture stub: no editor component exists.
    },
    getEditorComponent() {
      return undefined;
    },
    /**
     Theme slot these tests never render; accessing it names the gap instead of faking one.
     */
    get theme(): never {
      throw new Error('fake ExtensionToolContext cannot render themes; extend the fixture if a test needs one',);
    },
    getAllThemes() {
      return [];
    },
    getTheme() {
      return undefined;
    },
    setTheme() {
      return { success: true, };
    },
    getToolsExpanded() {
      return false;
    },
    setToolsExpanded() {
      // Fixture stub: no tool output state exists.
    },
  };
}

/**
 Build the read-only session half of the double over an empty session.

 @returns session manager reporting one empty session

 @example
 ```ts
 fakeSessionManager();
 ```
 */
function fakeSessionManager(): ExtensionToolContext['sessionManager'] {
  return {
    getCwd() {
      return '/tmp';
    },
    getSessionDir() {
      return '/tmp';
    },
    getSessionId() {
      return 'fake-session';
    },
    getSessionFile() {
      return undefined;
    },
    getSessionName() {
      return undefined;
    },
    getLeafId() {
      return null;
    },
    getLeafEntry() {
      return undefined;
    },
    getEntry() {
      return undefined;
    },
    getLabel() {
      return undefined;
    },
    getBranch() {
      return [];
    },
    buildContextEntries() {
      return [];
    },
    buildSessionProjection() {
      return {
        entries: [],
        messages: [],
        thinkingLevel: 'off',
        model: null,
      };
    },
    getHeader() {
      return null;
    },
    getEntries() {
      return [];
    },
    getTree() {
      return [];
    },
  };
}

//endregion Helpers
