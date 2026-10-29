/**
 Electron main process for the no-Vite ESM TypeScript counter app.
 
 Linux startup deliberately forces Chromium's Ozone Wayland backend before
 Electron's `ready` event. The package's boundary test also unsets `DISPLAY`,
 so XWayland cannot hide a regression.
 
 @example
 ```ts
 // Electron loads this file from dist/app/main.mjs.
 ```
 */

import {
  rename,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  app,
  BrowserWindow,
} from 'electron';
import { caughtValueStack, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core';

import {
  APP_TITLE_PREFIX,
  parseDocumentTitle,
  UNPARSEABLE_DOCUMENT_TITLE,
} from './counter.js';

/**
 Width of the demo window in density-independent pixels.
 
 @example
 ```ts
 console.log(mainWindowWidth);
 ```
 */
const mainWindowWidth = 800;

/**
 Height of the demo window in density-independent pixels.
 
 @example
 ```ts
 console.log(mainWindowHeight);
 ```
 */
const mainWindowHeight = 600;

/**
 Environment variable pointing at a JSON file where tests observe renderer state.
 
 @example
 ```ts
 console.log(statePathEnvironmentVariable);
 ```
 */
const statePathEnvironmentVariable = 'MONOCHROMATIC_ELECTRON_COUNTER_STATE_PATH';

/**
 Main-process logger tagged at module boundary.
 
 @example
 ```ts
 mainLogger.info('starting');
 ```
 */
const mainLogger = tagged({ tag: 'electron-counter-main', },);

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
 Writes test-observable renderer state when the boundary-test state path is set.
 
 @param count - Counter value observed from renderer title.
 
 @example
 ```ts
 await writeObservedCounterState({ count: 1 });
 ```
 */
async function writeObservedCounterState({ count, }: { readonly count: number; },): Promise<void> {
  /**
   Optional state path used only by automated boundary tests.
   */
  const statePath = process.env[statePathEnvironmentVariable];

  if (statePath === undefined)
    return;

  /**
   Unique temp path used so state readers never observe a truncated JSON file.
   */
  const tempStatePath = `${statePath}.${process.pid}.${Date.now()}.tmp`;

  await writeFile(
    tempStatePath,
    `${JSON.stringify(
      { count, },
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
 Observes a renderer title update and mirrors the count to the optional test state file.
 
 @param title - Title reported by Electron's `page-title-updated` event.
 
 @example
 ```ts
 await observeCounterTitle({ title: 'Monochromatic ESM TS Counter :: count=1' });
 ```
 */
async function observeCounterTitle({ title, }: { readonly title: string; },): Promise<void> {
  /**
   Parsed count from this app's title convention.
   */
  const count = parseDocumentTitle({ title, },);

  if (count === UNPARSEABLE_DOCUMENT_TITLE)
    return;

  mainLogger.info(`Observed renderer counter value ${count}.`,);
  await writeObservedCounterState({ count, },);
}

/**
 Runs an async title observation and logs any failure, because Electron event
 emitters do not await async listeners.
 
 @param title - Title reported by the renderer.
 
 @example
 ```ts
 observeCounterTitleFromEvent({ title: 'Monochromatic ESM TS Counter :: count=0' });
 ```
 */
function observeCounterTitleFromEvent({ title, }: { readonly title: string; },): void {
  void (async function observeCounterTitleTask(): Promise<void> {
    try {
      await observeCounterTitle({ title, },);
    }
    catch (error: unknown) {
      mainLogger.error(
        `Failed to observe renderer title: ${caughtValueStack(error,)}`,
      );
    }
  })();
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
    height: mainWindowHeight,
    title: APP_TITLE_PREFIX,
    webPreferences: {
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
    },
    width: mainWindowWidth,
  },);

  mainWindow.webContents
    .on(
    'page-title-updated',
    function handlePageTitleUpdated(
      _event: unknown,
      title: string,
    ): void {
      observeCounterTitleFromEvent({ title, },);
    },
    );

  mainWindow.webContents
    .on(
    'did-finish-load',
    function handleDidFinishLoad(): void {
      observeCounterTitleFromEvent({
        title: mainWindow.webContents
          .getTitle(),
      },);
    },
    );

  await mainWindow.loadFile(rendererHtmlPath,);
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
  mainLogger.error(
    `Failed to create activated main window: ${caughtValueStack(error,)}`,
  );
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
  mainLogger.error(
    `Failed to start Electron counter app: ${caughtValueStack(error,)}`,
  );
}

/**
 Starts Electron asynchronously without blocking ESM module evaluation.
 
 Electron emits `ready` after the main module finishes evaluating, so a
 top-level `await app.whenReady()` deadlocks startup under Electron's ESM
 loader.
 
 @example
 ```ts
 startElectronCounterApp();
 ```
 */
function startElectronCounterApp(): void {
  void (async function startElectronCounterAppTask(): Promise<void> {
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
startElectronCounterApp();
