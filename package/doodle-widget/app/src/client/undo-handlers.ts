/**
 Undo/redo handler setup for the doodle widget.
 
 Wires up undo/redo buttons, Ctrl+Z / Ctrl+Shift+Z keyboard
 shortcuts, and provides a `pushSnapshot` function for other
 modules to record state after completing actions.
 */

import {
  getStrokes,
  redraw,
  setStrokes,
} from './drawing.ts';
import { getCurrentPageIndex, } from './pages.ts';
import {
  replaceTextEntries,
  serializeTextEntries,
} from './text-page.ts';
import { clearTextEntries, } from './text.ts';
import {
  canRedo,
  canUndo,
  NO_SNAPSHOT,
  pushSnapshot as pushHistorySnapshot,
  redo,
  type Snapshot,
  undo,
} from './undo-history.ts';

/**
 Dependencies for undo/redo handler setup.
 */
export type UndoHandlerDeps = {
  /**
   Undo button
   */
  readonly undoBtn: HTMLButtonElement;
  /**
   Redo button
   */
  readonly redoBtn: HTMLButtonElement;
  /**
   Canvas 2D rendering context for redraw after restore
   */
  readonly ctx: CanvasRenderingContext2D;
  /**
   Returns current canvas dimensions
   */
  readonly getCanvasSize: () => {
    cw: number;
    ch: number;
  };
  /**
   Text layer element for serializing and restoring text entries
   */
  readonly textLayer: HTMLDivElement;
};

/**
 Sets up undo/redo button handlers, keyboard shortcuts, and returns
 functions for other modules to push snapshots and update button state.
 
 @param deps - DOM elements and shared state accessors, see {@link UndoHandlerDeps}
 
 @mutates deps - `undoBtn.addEventListener` and `redoBtn.addEventListener` change event targets and retain handlers.
 
 @returns {@link pushSnapshot} to call after state-changing actions,
   {@link updateUndoButtons} to refresh button disabled state
 
 @example
 ```ts
 const { pushSnapshot, updateUndoButtons } = setupUndoHandlers(deps);
 ```
 */
export function setupUndoHandlers(deps: UndoHandlerDeps,): {
  pushSnapshot: () => void;
  updateUndoButtons: () => void;
} {
  /**
   Destructured up front so every closure inside this factory captures the same handles.
   */
  const {
    undoBtn,
    redoBtn,
    ctx,
    getCanvasSize,
    textLayer,
  } = deps;

  /**
   Refreshes the disabled state of undo/redo buttons based on
   history availability for the current page.
   */
  function updateUndoButtons(): void {
    /**
     Page index read once so both button states reflect the same page.
     */
    const pageIndex = getCurrentPageIndex();
    undoBtn.disabled = !canUndo(pageIndex,);
    redoBtn.disabled = !canRedo(pageIndex,);
  }

  /**
   Captures the current state and pushes it to the undo history.
   
   Call after any state-changing action completes (stroke, erase,
   text finalization, clear).
   */
  function pushSnapshot(): void {
    pushHistorySnapshot({
      pageIndex: getCurrentPageIndex(),
      snapshot: {
        strokes: [...getStrokes(),],
        textEntries: serializeTextEntries(textLayer,),
      },
    },);
    updateUndoButtons();
  }

  /**
   Restores a snapshot from the undo history.
   
   @param snapshot - {@link Snapshot} state to restore
   */
  function restoreSnapshot(snapshot: Snapshot,): void {
    setStrokes([...snapshot.strokes,],);
    replaceTextEntries({
      entries: snapshot.textEntries,
      layer: textLayer,
      clearFn: clearTextEntries,
    },);
    /**
     Canvas dimensions resolved here so the redraw uses the current layout.
     */
    const {
      cw,
      ch,
    } = getCanvasSize();
    redraw({
      ctx,
      cw,
      ch,
    },);
    updateUndoButtons();
  }

  undoBtn.addEventListener(
    'click',
    function handleUndo(): void {
      /**
       Absent when the page's undo stack is empty, in which case the click is ignored.
       */
      const snapshot = undo(getCurrentPageIndex(),);
      if (snapshot !== NO_SNAPSHOT)
        restoreSnapshot(snapshot,);
    },
  );

  redoBtn.addEventListener(
    'click',
    function handleRedo(): void {
      /**
       Absent when the page's redo stack is empty, in which case the click is ignored.
       */
      const snapshot = redo(getCurrentPageIndex(),);
      if (snapshot !== NO_SNAPSHOT)
        restoreSnapshot(snapshot,);
    },
  );

  document.addEventListener(
    'keydown',
    function handleUndoRedoKey(event: KeyboardEvent,): void {
      /**
       Skip when focus is inside a text input to preserve native text undo
       */
      if (event.target
        instanceof HTMLInputElement)
        return;

      /**
       Either control or meta gates the shortcut so it does not fire on bare keys.
       */
      const hasModifier = event.ctrlKey
        || event
          .metaKey;
      if (!hasModifier)
        return;

      /**
       Lower-cased so the comparison matches regardless of caps-lock or shift-induced casing.
       */
      const key = event.key
        .toLowerCase();
      if (key === 'z') {
        event.preventDefault();
        /**
         Shift inverts the direction so Ctrl+Shift+Z redoes instead of undoing.
         */
        const snapshot = event.shiftKey
          ? redo(getCurrentPageIndex(),)
          : undo(getCurrentPageIndex(),);
        if (snapshot !== NO_SNAPSHOT)
          restoreSnapshot(snapshot,);
      }
      else if (key === 'y') {
        event.preventDefault();
        /**
         Absent when the redo stack is empty, in which case the keystroke is consumed harmlessly.
         */
        const snapshot = redo(getCurrentPageIndex(),);
        if (snapshot !== NO_SNAPSHOT)
          restoreSnapshot(snapshot,);
      }
    },
  );

  updateUndoButtons();

  return {
    pushSnapshot,
    updateUndoButtons,
  };
}
