/**
 Browser renderer boot for the sticky-flow file-manager prototype.
 
 Runs as a plain browser ES module inside Electron's sandboxed renderer.
 All layout behavior during scrolling (panes pinning inside their rails,
 non-overlap, clamping) is delegated to CSS normal flow plus
 `position: sticky`; script only mutates the pane model, reconciles the DOM,
 and mirrors observable state across the preload bridge.
 
 @example
 ```ts
 // index.html loads this file as <script type="module">.
 ```
 */

import type { FileManagerBridge, } from './bridge-types.js';
import { createRendererStores, } from './render-dom.js';
import { installKeyboard, } from './renderer-keys.js';
import {
  renderAndReport,
  reportState,
  type RendererSession,
} from './session.js';
import {
  getShellElement,
  showStatus,
} from './shell-dom.js';
import {
  createStrip,
  directoryLocation,
  openRoot,
} from './strip.js';

/**
 Error thrown when the preload bridge is absent or misshapen.
 
 @example
 ```ts
 new MissingBridgeError();
 ```
 */
class MissingBridgeError extends Error {
  /**
   Builds the descriptive bridge-lookup error.
   
   @example
   ```ts
   new MissingBridgeError();
   ```
   */
  public constructor() {
    super('Preload bridge fileManagerBridge is missing; preload.cjs did not run.',);
    this.name = 'MissingBridgeError';
  }
}

/**
 Checks whether a value is the preload bridge, narrowing without an
 unchecked type assertion.
 
 @param value - Global property to check.
 
 @returns Whether the value carries the three bridge functions.
 
 @example
 ```ts
 isFileManagerBridge(Reflect.get(globalThis, 'fileManagerBridge'));
 ```
 */
function isFileManagerBridge(value: unknown,): value is FileManagerBridge {
  return ((typeof value) === 'object')
    && (value !== null)
    && ('initialRoot' in value)
    && (((typeof value.initialRoot)) === 'function')
    && ('listDirectory' in value)
    && (((typeof value.listDirectory)) === 'function')
    && ('reportState' in value)
    && (((typeof value.reportState)) === 'function');
}

/**
 Reads the preload bridge off the global scope.
 
 @returns Verified preload bridge.
 
 @throws MissingBridgeError when the bridge is absent or misshapen.
 
 @example
 ```ts
 const bridge = getBridge();
 ```
 */
function getBridge(): FileManagerBridge {
  /**
   Whatever the preload script exposed on the main world, if anything.
   */
  const candidate: unknown = Reflect.get(
    globalThis,
    'fileManagerBridge',
  );

  if (!isFileManagerBridge(candidate,))
    throw new MissingBridgeError();

  return candidate;
}

/**
 Applies the debug tint when the main process requested it via query string.
 
 @example
 ```ts
 applyDebugTintFromQuery();
 ```
 */
function applyDebugTintFromQuery(): void {
  /**
   Query parameters of the loaded renderer document.
   */
  const query = new URLSearchParams(globalThis.location
    .search,);

  if (query.get('debugTint',) === '1')
    document.body
      .classList
      .add('debug-tint',);
}

/**
 Boots the renderer: opens the root pane over the bridge, installs keyboard
 and scroll wiring, and reports the first observable state.
 
 @example
 ```ts
 await bootFileManager();
 ```
 */
async function bootFileManager(): Promise<void> {
  applyDebugTintFromQuery();

  /**
   Typed bridge installed by the preload script.
   */
  const bridge = getBridge();

  /**
   Directory the first root pane lists.
   */
  const rootPath = await bridge.initialRoot();

  /**
   Root directory listing fetched before the first render.
   */
  const rootListing = await bridge.listDirectory(rootPath,);

  /**
   Model after opening the root pane.
   */
  const opened = openRoot({
    location: directoryLocation({ path: rootPath, },),
    strip: createStrip(),
  },);

  /**
   Mutable renderer session shared by every handler.
   */
  const session: RendererSession = {
    bridge,
    stores: createRendererStores(),
    strip: opened.strip,
    stripElement: getShellElement({ id: 'strip', },),
  };

  session.stores
    .listings
    .set(
    opened.id,
    rootListing,
    );
  installKeyboard({ session, },);
  session.stripElement
    .addEventListener(
    'scroll',
    function reportOnScroll(): void {
      reportState({ session, },);
    },
    { passive: true, },
    );
  renderAndReport({ session, },);
}

try {
  await bootFileManager();
}
catch (error: unknown) {
  showStatus({ text: `Failed to start: ${String(error,)}`, },);
}
