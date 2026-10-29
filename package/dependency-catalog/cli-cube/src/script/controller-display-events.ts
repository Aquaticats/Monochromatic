/**
 Display-option and reset event wiring for the deck.gl scene controller.
 
 @example
 ```ts
 wireDisplay({ session, commit });
 wireReset({ session, probes, commit });
 ```
 */

import { computeSceneBounds, } from '../deck-config.ts';
import type { PackageProbe, } from '../probe.ts';
import {
  el,
  elInput,
  elSelect,
  syncDomFromState,
} from './controller-dom.ts';
import type {
  Commit,
  Session,
} from './controller-event-types.ts';
import { unpinTooltip, } from './controller-tooltip.ts';
import { defaultState, } from './state.ts';

//region Types

/**
 Display-toggle boolean keys controlled by checkboxes.
 */
type CheckboxDisplayKey = 'showWireframe' | 'showThresholdPlanes' | 'showAxisLabels'
  | 'showUnknownCluster';

//endregion Types

//region Helpers

/**
 Binds one checkbox to its display-toggles key.
 
 @param id - Element id of checkbox.
 
 @param key - Display-toggles boolean controlled by checkbox.
 
 @param session - Mutable session.
 
 @param commit - Callback that re-renders and syncs hash.
 
 @example
 ```ts
 bindCheckbox({ id: 'display-wireframe', key: 'showWireframe', session, commit });
 ```
 */
function bindCheckbox(
  {
    id,
    key,
    session,
    commit,
  }: {
    id: string;
    key: CheckboxDisplayKey;
    session: Session;
    commit: Commit;
  },
): void {
  /**
   Checkbox `<input>` resolved by id; `change` event drives bound display toggle.
   */
  const input = elInput(id,);
  input.addEventListener(
    'change',
    function onChange() {
      session.state = {
        ...session.state,
        displayToggles: {
          ...session.state
            .displayToggles,
          [key]: input.checked,
        },
      };
      commit();
    },
  );
}

//endregion Helpers

//region Wire functions


/**
 Wires four display checkboxes plus `name-labels` select.
 
 Each change triggers a full layer rebuild via `commit`.
 
 @param session - Mutable session.
 
 @param commit - Callback that re-renders and syncs hash.
 
 @example
 ```ts
 wireDisplay({ session, commit });
 ```
 */
export function wireDisplay(
  {
    session,
    commit,
  }: {
    session: Session;
    commit: Commit;
  },
): void {
  bindCheckbox({
    id: 'display-wireframe',
    key: 'showWireframe',
    session,
    commit,
  },);
  bindCheckbox({
    id: 'display-planes',
    key: 'showThresholdPlanes',
    session,
    commit,
  },);
  bindCheckbox({
    id: 'display-axis-labels',
    key: 'showAxisLabels',
    session,
    commit,
  },);
  bindCheckbox({
    id: 'display-unknown',
    key: 'showUnknownCluster',
    session,
    commit,
  },);
  /**
   Name-labels `<select>`; controls how many probe names are rendered as text.
   */
  const nameLabels = elSelect('name-labels',);
  nameLabels.addEventListener(
    'change',
    function onChange() {
      /**
       Raw `value` from select; narrowed against allowed name-label modes below.
       */
      const raw = nameLabels.value;
      if ((raw !== 'none') && (raw !== 'topN')
        && (raw !== 'all'))
        return;
      session.state = {
        ...session.state,
        displayToggles: {
          ...session.state
            .displayToggles,
          nameLabels: raw,
        },
      };
      commit();
    },
  );
}

/**
 Wires reset button to restore `defaultState({ probes })`.
 
 After state swap, every control's DOM value is re-synced.
 
 @param session - Mutable session.
 
 @param probes - Source probes.
 
 @param commit - Callback that re-renders and syncs hash.
 
 @example
 ```ts
 wireReset({ session, probes, commit });
 ```
 */
export function wireReset(
  {
    session,
    probes,
    commit,
  }: {
    session: Session;
    probes: readonly PackageProbe[];
    commit: Commit;
  },
): void {
  /**
   Reset button; click handler swaps state back to {@link defaultState}.
   */
  const button = el('reset',);
  button.addEventListener(
    'click',
    function onClick() {
      /**
       Pristine state derived from source probes; replaces entire session state below.
       */
      const next = defaultState({
        probes,
      },);
      session.state = next;
      session.bounds = computeSceneBounds({
        probes,
        dimMapping: next.dimMapping,
      },);
      syncDomFromState({
        state: next,
      },);
      unpinTooltip();
      commit();
    },
  );
}

//endregion Wire functions
