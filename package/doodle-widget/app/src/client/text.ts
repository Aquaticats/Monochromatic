/**
 Text placement for the doodle widget.
 
 Creates text inputs as absolutely-positioned DOM elements inside a
 text layer. Inputs become readonly once the user finishes typing;
 no conversion to a different element type is needed.
 */

import {
  getStrokeColor,
  getStrokeWidth,
} from './drawing-config.ts';
import type { NormalizedPoint, } from './drawing.ts';
import { createTextInput, } from './text-page.ts';

/**
 Multiplier converting a 0..1 normalized coordinate to a percentage
 */
const PERCENT_SCALE = 100;

/**
 Multiplier converting stroke width to text font size in pixels
 */
const TEXT_SIZE_FACTOR = 2;

//region State

/**
 Active text-input editing state.
 
 `idle` when no input is focused; `editing` while one is open, carrying the
 element so finalize and discard act on the same node.
 */
type ActiveInput =
  | { readonly kind: 'idle'; }
    | {
      readonly kind: 'editing';
      readonly input: HTMLInputElement;
    };

/**
 Text editing state container.
 
 Stored as object properties so module-root state stays in a `const`
 container (`no-module-root-let` would otherwise reject top-level `let`).
 */
const textState: {
  /**
   Text layer container, set once via {@link setTextLayer}; absent until then
   */
  layerElement?: HTMLDivElement;
  /**
   Active text input, or `idle` discriminant when none is focused
   */
  activeInput: ActiveInput;
} = {
  activeInput: { kind: 'idle', },
};

//endregion State

/**
 Custom event type dispatched on the text layer when a text input
 is finalized with content. Subscribers (e.g. undo system) listen
 on the text layer element directly.
 */
const TEXT_FINALIZED_EVENT = 'textfinalized';

/**
 Sets the text layer container element.
 
 @param layer - div element that holds text overlays
 
 @example
 ```ts
 setTextLayer(document.querySelector('.text-layer'));
 ```
 */
export function setTextLayer(layer: HTMLDivElement,): void {
  textState.layerElement = layer;
}

/**
 Finalizes the active input: makes it readonly if it has content,
 removes it if empty. Dispatches a `textfinalized` custom event
 on the text layer when content is kept.
 
 @example
 ```ts
 finalizeActiveInput();
 ```
 */
export function finalizeActiveInput(): void {
  /**
   Captured before narrowing so the discriminant check and field access read one value.
   */
  const active = textState.activeInput;
  if (active.kind === 'idle')
    return;

  /**
   Whether the input has non-empty content worth keeping
   */
  const hasContent = active.input
    .value
    .trim()
    !== '';
  if (hasContent)
    active.input
      .readOnly = true;
  else
    active.input
      .remove();

  textState.activeInput = { kind: 'idle', };

  /**
   Captured so the presence check and dispatch read the same layer reference.
   */
  const layer = textState.layerElement;
  if (hasContent && (layer !== undefined))
    layer
      .dispatchEvent(new CustomEvent(TEXT_FINALIZED_EVENT,),);
}

/**
 Creates a text input at the given normalized position and appends
 it to the text layer.
 
 Finalizes any previously active input first.
 
 @param position - normalized [0..1] coordinate for placement
 
 @example
 ```ts
 placeTextInput([0.5, 0.5]);
 ```
 */
export function placeTextInput(position: NormalizedPoint,): void {
  finalizeActiveInput();

  /**
   Captured so the guard and append below read one layer reference.
   */
  const layer = textState.layerElement;
  if (layer === undefined)
    return;

  /**
   Active color captured at text input creation
   */
  const color = getStrokeColor();
  /**
   Text font size derived from active stroke width
   */
  const textSizePx = getStrokeWidth()
    * TEXT_SIZE_FACTOR;

  /**
   Built before listener wiring so the keydown closure can capture the same node.
   */
  const input = createTextInput({
    insetInlineStart: `${String(position[0]
      * PERCENT_SCALE,)}%`,
    insetBlockStart: `${String(position[1]
      * PERCENT_SCALE,)}%`,
    color,
    fontSize: String(textSizePx,),
  },);

  input.addEventListener(
    'keydown',
    function handleKeydown(event: KeyboardEvent,): void {
      if (event.key
        === 'Enter') {
        event.preventDefault();
        finalizeActiveInput();
      }
      else if (event.key
        === 'Escape') {
        discardActiveInput();
      }
    },
  );

  input.addEventListener(
    'blur',
    finalizeActiveInput,
  );

  textState.activeInput = {
    kind: 'editing',
    input,
  };
  layer
    .append(input,);
  /**
   preventScroll stops the browser from scrolling the overflow-hidden
   canvas container to reveal the input when it extends past the edge.
   */
  input.focus({ preventScroll: true, },);
}

/**
 Discards the active input without keeping it.
 
 @example
 ```ts
 discardActiveInput();
 ```
 */
export function discardActiveInput(): void {
  /**
   Captured so the discriminant check and removal act on one input.
   */
  const active = textState.activeInput;
  if (active.kind === 'editing') {
    active.input
      .remove();
    textState.activeInput = { kind: 'idle', };
  }
}

/**
 Removes all text entries and discards any active input.
 
 @example
 ```ts
 clearTextEntries();
 ```
 */
export function clearTextEntries(): void {
  discardActiveInput();
  /**
   Captured so the presence check and clear read one layer reference.
   */
  const layer = textState.layerElement;
  if (layer !== undefined)
    layer
      .replaceChildren();
}
