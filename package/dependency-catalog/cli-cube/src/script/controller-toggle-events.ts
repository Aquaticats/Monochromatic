/**
 Toggle and search event wiring for the deck.gl scene controller.
 
 @example
 ```ts
 wireToggles({ session, commit });
 wireSearch({ session, commit });
 ```
 */

import { elInput, } from './controller-dom.ts';
import { TOGGLE_VALUES, } from './controller-event-constants.ts';
import type {
  Commit,
  Session,
} from './controller-event-types.ts';
import type { ToggleKey, } from './filter.ts';
import { TOGGLE_KEYS, } from './state.ts';

//region Wire functions

/**
 Wires the seven 3-state toggle radio groups.
 
 Radios share a `name="toggle-<key>"` so the browser handles mutual exclusion;
 event handling is delegated from each fieldset.
 
 @param session - Mutable session.
 
 @param commit - Callback that re-renders and syncs hash.
 
 @example
 ```ts
 wireToggles({ session, commit });
 ```
 */
export function wireToggles(
  {
    session,
    commit,
  }: {
    session: Session;
    commit: Commit;
  },
): void {
  TOGGLE_KEYS.forEach(function bind(key: ToggleKey,) {
    /**
     Fieldset wrapping three radios for this toggle; event delegation hangs off it.
     */
    const fieldset = document.querySelector<HTMLFieldSetElement>(
      `[data-toggle="${key}"]`,
    );
    if (fieldset === null)
      return;
    fieldset.addEventListener(
      'change',
      function onChange(event,) {
        /**
         Delegated target; narrowed to `HTMLInputElement` before reading `.value`.
         */
        const input = event.target;
        if (!(input instanceof HTMLInputElement))
          return;
        /**
         Narrowed toggle value matching input value; `undefined` rejects unexpected values.
         */
        const next = TOGGLE_VALUES.find(function match(candidate,) {
          return candidate === input
            .value;
        },);
        if (next === undefined)
          return;
        session.state = {
          ...session.state,
          toggles: {
            ...session.state
              .toggles,
            [key]: next,
          },
        };
        commit();
      },
    );
  },);
}

/**
 Wires the name-search input.
 
 Live `input` events update the mask on every keystroke; invalid regexes are
 absorbed by `searchMatches` and yield zero matches rather than crashing.
 
 @param session - Mutable session.
 
 @param commit - Callback that re-renders and syncs hash.
 
 @example
 ```ts
 wireSearch({ session, commit });
 ```
 */
export function wireSearch(
  {
    session,
    commit,
  }: {
    session: Session;
    commit: Commit;
  },
): void {
  /**
   Search-box `<input>`; mirrored to `state.search` on every keystroke.
   */
  const input = elInput('search',);
  input.addEventListener(
    'input',
    function onInput() {
      session.state = {
        ...session.state,
        search: input.value,
      };
      commit();
    },
  );
}

//endregion Wire functions
