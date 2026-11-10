/**
 Electron main process for the sticky-flow file-manager prototype.
 
 Linux startup forces Chromium's Ozone Wayland backend before Electron's
 `ready` event, matching the electron-counter package, so the nested-Wayland
 boundary test exercises the same pure-Wayland path a user gets.
 
 @example
 ```ts
 // Electron loads this file from dist/app/main.mjs.
 ```
 */

import {
  rename,
  writeFile,
} from 'node:fs/promises';
import { homedir, } from 'node:os';
import {
  join,
  resolve,
} from 'node:path';

import {
  app,
  BrowserWindow,
  ipcMain,
} from 'electron';
import { caughtValueStack, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core';

import type { BridgeFileEntry, } from './bridge-types.js';
import {
  APP_TITLE,
  DEBUG_TINT_ENVIRONMENT_VARIABLE,
  DEFAULT_WINDOW_HEIGHT,
  DEFAULT_WINDOW_WIDTH,
  ROOT_DIRECTORY_ENVIRONMENT_VARIABLE,
  STATE_PATH_ENVIRONMENT_VARIABLE,
} from './constants.js';
import {
  INITIAL_ROOT_CHANNEL,
  LIST_DIRECTORY_CHANNEL,
  REPORT_STATE_CHANNEL,
} from './ipc-channels.js';
import { ListingOutsideRootError, } from './confinement.js';
import { listDirectory, } from './listing.js';

/**
 Main-process logger tagged at module boundary.
 
 @example
 ```ts
 mainLogger.info('starting');
 ```
 */
const mainLogger = tagged({ tag: 'file-manager-electron-main', },);

/**
 Absolute path to the renderer HTML file in the staged app directory.
 
 @example
 ```ts
 console.log(rendererHtmlPath);
 ```
 */
const rendererHtmlPath = join(
  import.meta.dirname,
  'index.html',
);

/**
 Absolute path to the CommonJS preload bundle in the staged app directory.
 
 @example
 ```ts
 console.log(preloadPath);
 ```
 */
const preloadPath = join(
  import.meta.dirname,
  'preload.cjs',
);

/**
 Directory the first root pane lists; every listing request must stay under
 it so the renderer cannot walk arbitrary host paths.
 
 @example
 ```ts
 console.log(rootDirectory);
 ```
 */
const rootDirectory = resolve(
  process.env[ROOT_DIRECTORY_ENVIRONMENT_VARIABLE] ?? homedir(),
);

/**
 Adds Chromium switches required for pure Wayland operation on Linux.
 
 @example
 ```ts
 configureLinuxWayland();
 ```
 */
function configureLinuxWayland(): void {
  if (process.platform !== 'linux') {
    mainLogger.info(`Wayland switch configuration skipped for ${process.platform}.`,);
    return;
  }

  app.commandLine
    .appendSwitch(
    'ozone-platform',
    'wayland',
    );
  app.commandLine
    .appendSwitch(
    'enable-features',
    'WaylandWindowDecorations',
    );
  process.env
    .XDG_SESSION_TYPE = 'wayland';
  mainLogger.info('Forced Chromium Ozone platform to Wayland for Linux.',);
}

/**
 Checks whether a value is a shallow record of JSON scalars, the only shape
 the boundary-test state file accepts.
 
 @param value - Renderer-supplied state payload.
 
 @returns Whether the payload is a shallow scalar record.
 
 @example
 ```ts
 isShallowScalarRecord({ count: 1 });
 ```
 */
function isShallowScalarRecord(
  value: unknown,
): value is Readonly<Record<string, string | number | boolean>> {
  if (((typeof value) !== 'object') || (value === null)
    || Array.isArray(value,))
    return false;

  return Object.values(value,)
    .every(function isScalar(entry,): boolean {
      return ((typeof entry) === 'string')
        || ((typeof entry) === 'number')
        || ((typeof entry) === 'boolean');
    },);
}

/**
 Persists renderer state atomically when the boundary-test state path is set.
 
 @param state - Shallow scalar state snapshot.
 
 @mutates state - `JSON.stringify` may invoke record accessors or proxy traps.
 
 @example
 ```ts
 await writeObservedState({ state: { ready: true } });
 ```
 */
async function writeObservedState(
  { state, }: { state: Readonly<Record<string, string | number | boolean>>; },
): Promise<void> {
  /**
   Optional state path used only by automated boundary tests.
   */
  const statePath = process.env[STATE_PATH_ENVIRONMENT_VARIABLE];

  if (statePath === undefined)
    return;

  /**
   Unique temp path so state readers never observe a truncated JSON file.
   */
  const tempStatePath = `${statePath}.${process.pid}.${Date.now()}.tmp`;

  await writeFile(
    tempStatePath,
    `${JSON.stringify(
      state,
      null,
      2,
    )}\n`,
    'utf8',
  );
  await rename(
    tempStatePath,
    statePath,
  );
}

/**
 Handles one renderer state report from an Electron event without awaiting
 inside the emitter, because Electron does not await async listeners.
 
 @param payload - Raw renderer-supplied state payload.
 
 @mutates payload - `JSON.stringify` may invoke record accessors or proxy traps after shape validation.
 
 @example
 ```ts
 observeStateFromEvent({ payload: { ready: true } });
 ```
 */
function observeStateFromEvent({ payload, }: { readonly payload: unknown; },): void {
  void (async function observeStateTask(): Promise<void> {
    try {
      if (!isShallowScalarRecord(payload,))
        throw new Error('Renderer state report was not a shallow scalar record.',);

      await writeObservedState({ state: payload, },);
    }
    catch (error: unknown) {
      mainLogger.error(`Failed to persist renderer state: ${caughtValueStack(error,)}`,);
    }
  })();
}

/**
 Registers the IPC surface consumed by the preload bridge.
 
 @example
 ```ts
 registerIpcHandlers();
 ```
 */
function registerIpcHandlers(): void {
  ipcMain.handle(
    INITIAL_ROOT_CHANNEL,
    function handleInitialRoot(): string {
      return rootDirectory;
    },
  );
  ipcMain.handle(
    LIST_DIRECTORY_CHANNEL,
    async function handleListDirectory(
      _event: unknown,
      path: unknown,
    ): Promise<readonly BridgeFileEntry[]> {
      if ((typeof path) !== 'string')
        throw new Error('Listing request path must be a string.',);

      return await listDirectory({
        path,
        rootDirectory,
      },);
    },
  );
  ipcMain.on(
    REPORT_STATE_CHANNEL,
    /**
     Receives renderer state for asynchronous persistence.
     
     @param _event - Electron event unused by state persistence.
     
     @param payload - Renderer payload that may expose serialization hooks.
     
     @mutates payload - `JSON.stringify` may invoke record accessors or proxy traps after shape validation.
     */
    function handleReportState(
      _event: unknown,
      payload: unknown,
    ): void {
      observeStateFromEvent({ payload, },);
    },
  );
}

/**
 Creates the main BrowserWindow and loads the renderer HTML entry.
 
 @returns Created BrowserWindow.
 
 @example
 ```ts
 await createMainWindow();
 ```
 */
async function createMainWindow(): Promise<BrowserWindow> {
  mainLogger.info('Creating main window.',);
  /**
   BrowserWindow hosting the sandboxed renderer.
   */
  const mainWindow = new BrowserWindow({
    height: DEFAULT_WINDOW_HEIGHT,
    title: APP_TITLE,
    webPreferences: {
      contextIsolation: true,
      nodeIntegration: false,
      preload: preloadPath,
      sandbox: true,
    },
    width: DEFAULT_WINDOW_WIDTH,
  },);

  /**
   Renderer query switching on the visible rail debug tint.
   */
  const query = (process.env[DEBUG_TINT_ENVIRONMENT_VARIABLE] === undefined)
    ? {}
    : { debugTint: '1', };

  await mainWindow.loadFile(
    rendererHtmlPath,
    { query, },
  );
  mainLogger.info('Renderer loaded.',);
  return mainWindow;
}

/**
 Logs a failure from an app activation path that Electron does not await.
 
 @param error - Caught activation error.
 
 @example
 ```ts
 logActivationCreateError({ error: new Error('boom') });
 ```
 */
function logActivationCreateError({ error, }: { readonly error: unknown; },): void {
  mainLogger.error(`Failed to create activated main window: ${caughtValueStack(error,)}`,);
}

/**
 Creates a replacement main window when the app is activated with none open.
 
 @example
 ```ts
 createMainWindowForActivation();
 ```
 */
function createMainWindowForActivation(): void {
  void (async function createActivatedMainWindow(): Promise<void> {
    try {
      await createMainWindow();
    }
    catch (error: unknown) {
      logActivationCreateError({ error, },);
    }
  })();
}

/**
 Installs lifecycle hooks that are independent of a specific window instance.
 
 @example
 ```ts
 installAppLifecycleHandlers();
 ```
 */
function installAppLifecycleHandlers(): void {
  app.on(
    'window-all-closed',
    function handleWindowAllClosed(): void {
      mainLogger.info('All windows closed.',);
      if (process.platform !== 'darwin')
        app.quit();
    },
  );

  app.on(
    'activate',
    function handleActivate(): void {
      mainLogger.info('App activated.',);
      if (BrowserWindow.getAllWindows()
        .length
        === 0)
        createMainWindowForActivation();
    },
  );
}

/**
 Logs a failure from initial app startup.
 
 @param error - Caught startup error.
 
 @example
 ```ts
 logStartupError({ error: new Error('boom') });
 ```
 */
function logStartupError({ error, }: { readonly error: unknown; },): void {
  mainLogger.error(`Failed to start file-manager app: ${caughtValueStack(error,)}`,);
}

/**
 Starts Electron asynchronously without blocking ESM module evaluation.
 
 Electron emits `ready` after the main module finishes evaluating, so a
 top-level `await app.whenReady()` deadlocks startup under Electron's ESM
 loader (see doc/troubleshooting on the Electron ESM ready deadlock).
 
 @example
 ```ts
 startFileManagerApp();
 ```
 */
function startFileManagerApp(): void {
  void (async function startFileManagerAppTask(): Promise<void> {
    try {
      mainLogger.info('Waiting for Electron app readiness.',);
      await app.whenReady();
      mainLogger.info('Electron app is ready.',);
      await createMainWindow();
    }
    catch (error: unknown) {
      logStartupError({ error, },);
      app.quit();
    }
  })();
}

configureLinuxWayland();
app.enableSandbox();
installAppLifecycleHandlers();
registerIpcHandlers();
startFileManagerApp();
