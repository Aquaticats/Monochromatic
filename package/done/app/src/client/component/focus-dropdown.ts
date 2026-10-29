/**
 `<focus-dropdown>`: popover-based dropdown for selecting a focus preset.
 Reads initial value from the `value` attribute and dispatches `change`
 events with `{ value }` when a preset is selected.
 */
import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';
import { FOCUS_DROPDOWN_STYLES, } from './focus-dropdown-styles.ts';

/**
 Available focus preset labels.
 */
const DEFAULT_PRESETS = [
  'Adulting tasks first',
  'Quick wins only',
  'Deep work focus',
];

/**
 `<focus-dropdown>` web component.
 
 Popover-based dropdown that lets users select a focus preset.
 */
class FocusDropdown extends HTMLElement {
  /**
   Shadow root for encapsulated rendering.
   */
  readonly #shadow: ShadowRoot;

  /**
   Currently selected preset value.
   */
  #value: string;

  /**
   Initializes the shadow root with empty value.
   */
  constructor() {
    super();
    this.#shadow = this.attachShadow({ mode: 'open', },);
    this.#value = '';
  }

  /**
   Reads the `value` attribute and renders the dropdown.
   */
  connectedCallback(): void {
    this.#value = this.getAttribute('value',)
      ?? 'Select focus...';
    this.#render();
  }

  /**
   Selects a preset, updates the display, and dispatches a change event.
   
   @param preset - Preset label
   
   @param textSpan - Text element to update
   
   @param menu - Popover menu to hide
   */
  #selectPreset(
    preset: string,
    textSpan: HTMLElement,
    menu: HTMLElement,
  ): void {
    this.#value = preset;
    textSpan.textContent = preset;
    menu.hidePopover();
    this.dispatchEvent(
      new CustomEvent(
        'change',
        {
          bubbles: true,
          detail: { value: preset, },
        },
      ),
    );
  }

  /**
   Renders the trigger button, divider, and popover menu into the shadow root.
   */
  #render(): void {
    /**
     Trigger label captured so the option callback can update it in place.
     */
    const textSpan = h({
      tag: 'span',
      class: 'text',
      text: this.#value,
    },);
    /**
     Pre-bound selector so each option click fires with the correct `this`.
     */
    const selectFn = this.#selectPreset
      .bind(this,);

    /**
     Popover menu captured so the option callback can close it after selection.
     */
    const menu = h({
      tag: 'ul',
      class: 'menu',
      attrs: { popover: 'auto', },
      children: DEFAULT_PRESETS.map(function buildOption(preset,): HTMLElement {
        return h({
          tag: 'li',
          class: 'option',
          text: preset,
          on: {
            click: function handleOptionClick(): void {
              selectFn(
                preset,
                textSpan,
                menu,
              );
            },
          },
        },);
      },),
    },);

    this.#shadow
      .replaceChildren(
      h({
        tag: 'style',
        text: FOCUS_DROPDOWN_STYLES,
      },),
      h({
        tag: 'button',
        class: 'trigger',
        children: [
          textSpan,
          h({
            tag: 'span',
            class: 'divider',
          },),
          h({
            tag: 'span',
            text: '\u25BC',
          },),
        ],
        on: { click: function handleTriggerClick(): void {
          menu.togglePopover();
        }, },
      },),
      menu,
      );
  }
}

customElements.define(
  'focus-dropdown',
  FocusDropdown,
);
