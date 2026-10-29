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

import type { Dirent, } from 'node:fs';
import {
  readdir,
  rename,
  writeFile,
} from 'node:fs/promises';
import { homedir, } from 'node:os';
import {
  join,
  resolve,
  sep,
} from 'node:path';

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  app,
  BrowserWindow,
  ipcMain,
} from 'electron';
import { caughtValueStack, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core';

import type {
  BridgeEntryKind,
  BridgeFileEntry,
} from './bridge-types.js';
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
import { sortBridgeEntries, } from './listing-sort.js';

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
 Error thrown when a listing request escapes the configured root directory.
 
 @example
 ```ts
 new ListingOutsideRootError({ path: '/etc' });
 ```
 */
class ListingOutsideRootError extends Error {
  /**
   Builds a descriptive confinement error.
   
   @param path - Rejected request path.
   
   @example
   ```ts
   new ListingOutsideRootError({ path: '/etc' });
   ```
   */
  public constructor({ path, }: { readonly path: string; },) {
    super(`Refusing to list ${path}: outside the configured root ${rootDirectory}`,);
    this.name = 'ListingOutsideRootError';
  }
}

/**
 Resolves and confines a renderer-supplied path under the root directory.
 
 @param path - Renderer-supplied directory path.
 
 @returns Resolved absolute path safe to read.
 
 @throws ListingOutsideRootError when the path escapes the root.
 
 @example
 ```ts
 confinePath({ path: '/home' });
 ```
 */
function confinePath({ path, }: { readonly path: string; },): string {
  /**
   Fully resolved request path with `..` segments folded away.
   */
  const resolved = resolve(path,);

  if ((resolved !== rootDirectory) && (!resolved.startsWith(`${rootDirectory}${sep}`,)))
    throw new ListingOutsideRootError({ path, },);

  return resolved;
}

/**
 Classifies one `readdir` dirent probe into a bridge entry kind, checking
 symlinks first because a symlink also answers the directory probe on some
 platforms.
 
 @param directory - Whether the dirent answered the directory probe.
 
 @param symlink - Whether the dirent answered the symlink probe.
 
 @returns Bridge entry kind.
 
 @example
 ```ts
 kindOfDirent({ directory: true, symlink: false });
 ```
 */
function kindOfDirent(
  {
    directory,
    symlink,
  }: {
    readonly directory: boolean;
    readonly symlink: boolean;
  },
): BridgeEntryKind {
  if (symlink)
    return 'symlink';

  if (directory)
    return 'directory';

  return 'file';
}

/**
 Lists one directory into sorted bridge entries.
 
 @param path - Renderer-supplied directory path.
 
 @returns Sorted entries of the directory.
 
 @throws ListingOutsideRootError when the path escapes the root.
 
 @example
 ```ts
 await listDirectory({ path: '/home' });
 ```
 */
async function listDirectory({ path, }: { readonly path: string; },): Promise<readonly BridgeFileEntry[]> {
  /**
   Confined absolute directory path.
   */
  const confined = confinePath({ path, },);

  /**
   Raw directory entries with type information, without following symlinks.
   */
  const dirents = await readdir(
    confined,
    { withFileTypes: true, },
  );

  return sortBridgeEntries({
    entries: dirents.map(function toBridgeEntry(
      dirent: ForeignBorrowed<Dirent>,
    ): BridgeFileEntry {
      return {
        kind: kindOfDirent({
          directory: dirent.isDirectory(),
          symlink: dirent.isSymbolicLink(),
        },),
        name: dirent.name,
        path: join(
          confined,
          dirent.name,
        ),
      };
    },),
  },);
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

      return await listDirectory({ path, },);
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
