/**
 `\<focus-dropdown\>`: popover-based dropdown for selecting a focus preset.
 Reads initial value from the `value` attribute and dispatches `change`
 events with `\{ value \}` when a preset is selected.
 */
import { hDom as h, } from '@monochromatic-dev/hyperscript-core/ts';
import { FOCUS_DROPDOWN_STYLES, } from './focus-dropdown-styles.ts';

/**
 Default focus preset options.
 */
const DEFAULT_PRESETS = [
  'Adulting tasks first',
  'Quick wins only',
  'Deep work focus',
];

/**
 `\<focus-dropdown\>` web component for selecting a focus preset.
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
   Reads initial value from attribute and renders the dropdown.
   */
  connectedCallback(): void {
    this.#value = this.getAttribute('value',)
      ?? 'Select focus...';
    this.#render();
  }

  /**
   Renders the trigger button and popover menu with preset options.
   */
  #render(): void {
    /**
     Captured so option-click closures reach this component without `this`-bound functions.
     */
    const self = this;
    /**
     Trigger label span captured so option clicks can update it in place.
     */
    const textSpan = h({
      tag: 'span',
      class: 'text',
      text: this.#value,
    },);
    /**
     Popover menu captured so option clicks can call `hidePopover()` after selection.
     */
    const menu = h({
      tag: 'ul',
      class: 'menu',
      attrs: { popover: 'auto', },
      children: DEFAULT_PRESETS.map(
        function buildOption(preset: string,): HTMLElement {
          return h({
            tag: 'li',
            class: 'option',
            text: preset,
            on: {
              click: function selectPreset(): void {
                self.#value = preset;
                textSpan.textContent = preset;
                menu.hidePopover();
                self.dispatchEvent(
                  new CustomEvent(
                    'change',
                    {
                      bubbles: true,
                      detail: { value: preset, },
                    },
                  ),
                );
              },
            },
          },);
        },
      ),
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
        on: { click: function onTriggerClick(): void {
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
