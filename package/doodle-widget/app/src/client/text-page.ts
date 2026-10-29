/**
 Text entry serialization and DOM construction for multi-page state.
 
 Provides type-safe serialization, restoration, and shared factory
 for text input elements. Serialization captures position, style,
 and content; the factory creates consistently styled inputs.
 */

/**
 Serialized representation of a text input for page state persistence.
 
 @example
 ```ts
 const entries = serializeTextEntries(layer);
 restoreTextEntries({ entries, layer: otherLayer });
 ```
 */
export type TextEntryData = {
  /**
   Text content of the input
   */
  readonly value: string;
  /**
   CSS percentage string for horizontal position
   */
  readonly insetInlineStart: string;
  /**
   CSS percentage string for vertical position
   */
  readonly insetBlockStart: string;
  /**
   CSS color string captured at creation
   */
  readonly color: string;
  /**
   Font size in pixels as a numeric string
   */
  readonly fontSize: string;
};

/**
 Creates a styled text input element with position, color, and font size.
 
 Sets `type`, `className`, `autocomplete`, inline position styles,
 text color, font size, and data attributes. Does not set `value`
 or `readOnly`: callers handle those based on context (new input
 vs. restored entry).
 
 @param insetInlineStart - CSS percentage string for horizontal position
 
 @param insetBlockStart - CSS percentage string for vertical position
 
 @param color - CSS color string
 
 @param fontSize - font size in pixels as a numeric string
 
 @returns configured input element
 
 @example
 ```ts
 const input = createTextInput({
   insetInlineStart: '50%', insetBlockStart: '30%',
   color: '#c24e2e', fontSize: '20',
 });
 ```
 */
export function createTextInput({
  insetInlineStart,
  insetBlockStart,
  color,
  fontSize,
}: {
  readonly insetInlineStart: string;
  readonly insetBlockStart: string;
  readonly color: string;
  readonly fontSize: string;
},): HTMLInputElement {
  /**
   Configured before insertion so the caller can finalize any extra styling.
   */
  const input = document.createElement('input',);
  input.type = 'text';
  input.className = 'text-input';
  input.autocomplete = 'off';
  input.style
    .insetInlineStart = insetInlineStart;
  input.style
    .insetBlockStart = insetBlockStart;
  input.style
    .color = color;
  input.style
    .fontSize = `${fontSize}px`;
  input.dataset
    .color = color;
  input.dataset
    .fontSize = fontSize;
  return input;
}

/**
 Serializes all text entries in a text layer element.
 
 Caller must finalize any active input before calling this function
 to ensure all entries are readonly and complete.
 
 @param layer - text layer div containing `.text-input` elements
 
 @returns array of serialized text entry data
 
 @example
 ```ts
 const data = serializeTextEntries(textLayer);
 ```
 */
export function serializeTextEntries(layer: HTMLDivElement,): TextEntryData[] {
  /**
   Live `NodeList` captured here so {@link Array.from} can map each entry through {@link serializeInput}.
   */
  const inputs = layer.querySelectorAll<HTMLInputElement>('.text-input',);
  return Array.from(
    inputs,
    function serializeInput(input,): TextEntryData {
      return {
        value: input.value,
        insetInlineStart: input.style
          .insetInlineStart,
        insetBlockStart: input.style
          .insetBlockStart,
        color: input.dataset
          .color
          ?? '',
        fontSize: input.dataset
          .fontSize
          ?? '',
      };
    },
  );
}

/**
 Recreates text inputs from serialized data.
 
 All restored inputs are set to readonly since they were finalized
 during serialization.
 
 @param entries - serialized text entry data to restore
 
 @param layer - text layer element to append inputs to
 
 @example
 ```ts
 restoreTextEntries({ entries: savedData, layer: textLayer });
 ```
 */
export function restoreTextEntries({
  entries,
  layer,
}: {
  readonly entries: readonly TextEntryData[];
  readonly layer: HTMLDivElement;
},): void {
  for (const entry of entries) {
    /**
     Detached input built first so its value and readonly flag are set before insertion.
     */
    const input = createTextInput({
      insetInlineStart: entry.insetInlineStart,
      insetBlockStart: entry.insetBlockStart,
      color: entry.color,
      fontSize: entry.fontSize,
    },);
    input.value = entry.value;
    input.readOnly = true;
    layer.append(input,);
  }
}

/**
 Clears all text entries and restores from serialized data.
 
 Combines clearing and restoring into a single operation used
 by undo/redo and page switching.
 
 @param entries - serialized text entry data to restore
 
 @param layer - text layer element to clear and repopulate
 
 @param clearFn - function that clears existing text entries
 
 @example
 ```ts
 replaceTextEntries({
   entries: snapshot.textEntries,
   layer: textLayer,
   clearFn: clearTextEntries,
 });
 ```
 */
export function replaceTextEntries({
  entries,
  layer,
  clearFn,
}: {
  readonly entries: readonly TextEntryData[];
  readonly layer: HTMLDivElement;
  readonly clearFn: () => void;
},): void {
  clearFn();
  restoreTextEntries({
    entries,
    layer,
  },);
}
