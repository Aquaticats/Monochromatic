/**
 Types for the sandboxed preload bridge between renderer and main process.
 
 The renderer stays a plain browser module (`sandbox`, `contextIsolation`,
 and `nodeIntegration: false` all stay enabled), so every filesystem read and
 every observed-state write crosses this typed bridge.
 
 @example
 ```ts
 const bridge: FileManagerBridge = globalThis.window.fileManagerBridge;
 ```
 
 @packageDocumentation
 */

/**
 Directory/file/symlink classification driving click behavior and sort, as
 reported by the main process without following symlinks.
 
 @example
 ```ts
 const kind: BridgeEntryKind = 'directory';
 ```
 */
export type BridgeEntryKind = 'directory' | 'file' | 'symlink';

/**
 One row of a directory listing crossing the bridge.
 
 @example
 ```ts
 const entry: BridgeFileEntry = { kind: 'directory', name: 'docs', path: '/home/docs' };
 ```
 */
export type BridgeFileEntry = {
  readonly kind: BridgeEntryKind;

  /**
   Final path segment shown to the user.
   */
  readonly name: string;

  /**
   Absolute path used to open or spawn from this entry.
   */
  readonly path: string;
};

/**
 Shallow scalar snapshot mirrored into the boundary-test state file; keys are
 compared by strict equality, so geometry facts are reported as booleans and
 counts, never raw floats.
 
 @example
 ```ts
 const state: ObservedStripState = {
   activePath: '/home',
   columnCount: 1,
   overlapCount: 0,
   paneCount: 1,
   ready: true,
   rootPinned: false,
   scrolledDown: false,
   scrollTopPx: 0,
 };
 ```
 */
export type ObservedStripState = {
  /**
   Location path of the focused pane, or empty when none is focused.
   */
  readonly activePath: string;
  readonly columnCount: number;

  /**
   Pane pairs whose rendered boxes intersect; sticky flow must keep this 0.
   */
  readonly overlapCount: number;
  readonly paneCount: number;

  /**
   Whether the renderer booted and completed at least one render pass.
   */
  readonly ready: boolean;

  /**
   Whether the first root pane is pinned to the scroller's top edge while the
   strip is scrolled down: the observable fact of sticky behavior.
   */
  readonly rootPinned: boolean;

  /**
   Whether the strip's vertical scroll offset is greater than zero.
   */
  readonly scrolledDown: boolean;

  /**
   Raw vertical scroll offset in whole pixels, for log forensics.
   */
  readonly scrollTopPx: number;
};

/**
 Renderer-facing API installed by the preload script.
 
 @example
 ```ts
 const entries = await globalThis.window.fileManagerBridge.listDirectory({ path: '/home' });
 ```
 */
export type FileManagerBridge = {
  /**
   Absolute path of the directory the first root pane lists.
   
   @returns Root directory path chosen by the main process.
   */
  readonly initialRoot: () => Promise<string>;

  /**
   Lists a directory, sorted directories-first then case-insensitively.
   
   @param path - Absolute directory path to list.
   
   @returns Sorted entries of the directory.
   */
  readonly listDirectory: (path: string,) => Promise<readonly BridgeFileEntry[]>;

  /**
   Mirrors the renderer's observed state to the main process, which persists
   it for boundary tests when a state path is configured.
   
   @param state - Snapshot to persist.
   */
  readonly reportState: (state: ObservedStripState,) => void;
};

/**
 Window shape after the preload script installed the bridge.
 
 @example
 ```ts
 const bridged = globalThis.window as BridgedWindow;
 ```
 */
export type BridgedWindow = Window & {
  readonly fileManagerBridge: FileManagerBridge;
};
