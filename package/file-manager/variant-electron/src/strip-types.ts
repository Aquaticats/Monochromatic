/**
 Core domain types for the pane strip, kept free of DOM and Electron so they
 mirror `package/file-manager/app/src/types.rs`.
 
 Absence is modeled per repo policy: optional properties for absent links
 (`parent`, `active`) and exported `unique symbol` sentinels for lookups
 that can miss, never nullish unions.
 
 @example
 ```ts
 const strip = createStrip();
 ```
 
 @packageDocumentation
 */

/**
 Stable identity for one pane instance: a monotonic counter value that is
 never reused, so it also encodes spawn order.
 
 Deliberately a plain `number` alias rather than a branded intersection:
 primitive intersection as readonly (see the documented precedent in
 `package/jsonc-edit/core-ts/src/edit-state.ts`), and every call site here
 passes ids through named object parameters, which already prevents
 positional mix-ups with rows and columns.
 
 @example
 ```ts
 const id: PaneId = 0;
 ```
 */
export type PaneId = number;

/**
 Sentinel returned when a pane lookup does not match any live pane.
 
 @example
 ```ts
 console.log(typeof PANE_NOT_FOUND);
 ```
 */
export const PANE_NOT_FOUND: unique symbol = Symbol(
  'Requested pane is not live in this strip',
);

/**
 Sentinel returned when no registered pane shows a looked-up location.
 
 @example
 ```ts
 console.log(typeof NO_CANONICAL_PANE);
 ```
 */
export const NO_CANONICAL_PANE: unique symbol = Symbol(
  'No canonical dedup pane is registered for this location',
);

/**
 Classification of what a pane shows; `directory` panes list entries and
 `preview` panes show a single file.
 
 @example
 ```ts
 const kind: PaneLocationKind = 'directory';
 ```
 */
export type PaneLocationKind = 'directory' | 'preview';

/**
 What a pane shows; also the dedup lookup key.
 
 @example
 ```ts
 const location: PaneLocation = { kind: 'directory', path: '/home' };
 ```
 */
export type PaneLocation = {
  readonly kind: PaneLocationKind;
  readonly path: string;
};

/**
 One pane in the strip: identity, location, grid position, and parent link.
 
 @example
 ```ts
 const pane: Pane = {
   column: 0,
   id: 0,
   location: { kind: 'directory', path: '/home' },
   registeredForDedup: true,
   row: 0,
 };
 ```
 */
export type Pane = {
  readonly id: PaneId;
  readonly location: PaneLocation;

  /**
   Zero-based column index (lineage depth); a child sits one column right of
   its parent.
   */
  readonly column: number;

  /**
   Zero-based row index (vertical slot), assigned by the tidy tree layout.
   */
  readonly row: number;

  /**
   Parent pane; absent for a root (or an orphan whose parent was closed).
   */
  readonly parent?: PaneId;

  /**
   Whether this pane is the canonical dedup holder for its location; forced
   duplicates are never registered, so a revisit focuses the canonical pane.
   */
  readonly registeredForDedup: boolean;
};

/**
 The whole strip: every live pane, the focused pane, and the id counter.
 
 @example
 ```ts
 const strip = createStrip();
 console.log(strip.panes.length);
 ```
 */
export type Strip = {
  /**
   Next id to mint; increments so ids are never reused and encode spawn order.
   */
  readonly nextId: number;
  readonly panes: readonly Pane[];

  /**
   The focused pane; absent when none is focused (fresh strip, or the
   focused pane was closed).
   */
  readonly active?: PaneId;
};

/**
 Builds a directory pane location.
 
 @param path - Absolute directory path the pane lists.
 
 @returns Directory location value.
 
 @example
 ```ts
 directoryLocation({ path: '/home' });
 ```
 */
export function directoryLocation({ path, }: { readonly path: string; },): PaneLocation {
  return {
    kind: 'directory',
    path,
  };
}

/**
 Builds a file-preview pane location.
 
 @param path - Absolute file path the pane previews.
 
 @returns Preview location value.
 
 @example
 ```ts
 previewLocation({ path: '/home/photo.png' });
 ```
 */
export function previewLocation({ path, }: { readonly path: string; },): PaneLocation {
  return {
    kind: 'preview',
    path,
  };
}

/**
 Builds an empty strip; a fresh session starts with no panes and no focus.
 
 @returns Strip with no panes.
 
 @example
 ```ts
 const strip = createStrip();
 ```
 */
export function createStrip(): Strip {
  return {
    nextId: 0,
    panes: [],
  };
}
