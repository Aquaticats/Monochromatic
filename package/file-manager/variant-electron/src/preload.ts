/**
 Sandboxed preload script installing the typed file-manager bridge.
 
 Bundled to CommonJS because Electron's sandboxed preload runtime does not
 load ESM; the renderer itself stays a browser ES module and only sees the
 `contextBridge`-exposed surface. IPC results are runtime-validated here so
 the renderer never consumes an unchecked shape.
 
 @example
 ```ts
 // Electron loads this file from dist/app/preload.cjs.
 ```
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  contextBridge,
  ipcRenderer,
} from 'electron';

import type {
  BridgeEntryKind,
  BridgeFileEntry,
  FileManagerBridge,
  ObservedStripState,
} from './bridge-types.js';
import {
  INITIAL_ROOT_CHANNEL,
  LIST_DIRECTORY_CHANNEL,
  REPORT_STATE_CHANNEL,
} from './ipc-channels.js';

/**
 Checks whether a value is one of the three bridge entry kinds.
 
 @param value - Kind candidate from an IPC payload.
 
 @returns Whether the value names a bridge entry kind.
 
 @example
 ```ts
 isBridgeEntryKind('directory');
 ```
 */
function isBridgeEntryKind(value: unknown,): value is BridgeEntryKind {
  return (value === 'directory') || (value === 'file')
    || (value === 'symlink');
}

/**
 Checks whether a value is one listing entry.
 
 @param value - Entry candidate from an IPC payload.
 
 @returns Whether the value is a well-formed bridge entry.
 
 @example
 ```ts
 isBridgeFileEntry({ kind: 'file', name: 'a', path: '/a' });
 ```
 */
function isBridgeFileEntry(value: unknown,): value is BridgeFileEntry {
  return ((typeof value) === 'object')
    && (value !== null)
    && ('kind' in value)
    && isBridgeEntryKind(value.kind,)
    && ('name' in value)
    && (((typeof value.name)) === 'string')
    && ('path' in value)
    && (((typeof value.path)) === 'string');
}

/**
 Parses the main process's directory-listing reply.
 
 @param value - Raw IPC reply.
 
 @returns Validated listing entries.
 
 @throws Error when the reply is not an array of bridge entries.
 
 @example
 ```ts
 parseListingReply({ value: [] });
 ```
 */
function parseListingReply({ value, }: { readonly value: unknown; },): readonly BridgeFileEntry[] {
  if ((!Array.isArray(value,)) || (!value.every(function isEntry(entry,): boolean {
    return isBridgeFileEntry(entry,);
  },)))
    throw new Error('Directory-listing IPC reply was not an array of entries.',);

  return value.filter(isBridgeFileEntry,);
}

/**
 Parses the main process's initial-root reply.
 
 @param value - Raw IPC reply.
 
 @returns Validated root directory path.
 
 @throws Error when the reply is not a string.
 
 @example
 ```ts
 parseRootReply({ value: '/home' });
 ```
 */
function parseRootReply({ value, }: { readonly value: unknown; },): string {
  if ((typeof value) !== 'string')
    throw new Error('Initial-root IPC reply was not a string.',);

  return value;
}

/**
 Reports rendered strip state through Electron IPC.
 
 @param state - Renderer-owned state entering preload bridge.
 
 @mutates state - `ipcRenderer.send` serializes state and may invoke caller-owned property accessors while copying it.
 
 @example
 ```ts
 reportState({
   activePath: '/',
   columnCount: 1,
   overlapCount: 0,
   paneCount: 1,
   ready: true,
   rootPinned: false,
   scrolledDown: false,
   scrollTopPx: 0,
 });
 ```
 */
function reportState(state: ForeignBorrowed<ObservedStripState>,): void {
  ipcRenderer.send(
    REPORT_STATE_CHANNEL,
    state,
  );
}

/**
 Bridge implementation forwarded over IPC to the main process.
 */
const bridge: FileManagerBridge = {
  initialRoot: async function initialRoot(): Promise<string> {
    /**
     Raw reply before shape validation.
     */
    const reply: unknown = await ipcRenderer.invoke(INITIAL_ROOT_CHANNEL,);

    return parseRootReply({ value: reply, },);
  },
  listDirectory: async function listDirectory(path: string,): Promise<readonly BridgeFileEntry[]> {
    /**
     Raw reply before shape validation.
     */
    const reply: unknown = await ipcRenderer.invoke(
      LIST_DIRECTORY_CHANNEL,
      path,
    );

    return parseListingReply({ value: reply, },);
  },
  reportState,
};

contextBridge.exposeInMainWorld(
  'fileManagerBridge',
  bridge,
);
