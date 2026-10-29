/**
 Browser-side runtime entry point.
 
 Bundled into the output HTML by `../render-html.ts` via rolldown
 (`format: 'iife'`, `minify: true`) and executed once the DOM has
 loaded. Reads the embedded `globalThis.__PROBES__` global injected by the
 HTML composer, instantiates a deck.gl `Deck` with the
 {@link ../deck-config.ts#orbitView OrbitView}, delegates control
 wiring to {@link ./controller-events.ts}, and keeps the URL hash
 synced so the user can copy and share an exact view.
 
 No `let` at module root or function-body root: the controller's
 mutable view of the world lives on a single `const session` object
 that's mutated in place (the binding is `const`; its fields are not).
 
 @example
 ```ts
 // Bundled by rolldown, no manual invocation.
 ```
 */

import type { ReadonlyDeep, } from 'type-fest';
import {
  Deck,
  type OrbitView,
  type PickingInfo,
  type ViewStateChangeParameters,
} from '@deck.gl/core';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  buildLayers,
  computeSceneBounds,
  orbitView,
} from '../deck-config.ts';
import type { PackageProbe, } from '../probe.ts';
import { syncDomFromState, } from './controller-dom.ts';
import {
  wireDimDropdowns,
  wireDisplay,
  wireRanges,
  wireReset,
  wireSearch,
  wireToggles,
} from './controller-events.ts';
import {
  formatTooltipHtml,
  pinTooltip,
  unpinTooltip,
} from './controller-tooltip.ts';
import { computeVisibleIndices, } from './filter.ts';
import {
  recomputeVisibility,
  rerenderLayers,
} from './controller-render.ts';
import type { Session, } from './controller-session-types.ts';
import { detectScheme, } from './scheme.ts';
import {
  type AppState,
  defaultState,
  readStateFromHash,
  writeStateToHash,
} from './state.ts';

//region Globals

declare global {
  /**
   Probe array embedded by `render-html.ts` as a JS literal on the global object.
   */
  var __PROBES__: readonly PackageProbe[];
}

//endregion Globals

//region Helpers

/**
 Reads the embedded probe array from the global injected by
 `render-html.ts`. Throws if absent so a broken bundling step
 surfaces loudly instead of silently rendering an empty scene.
 
 @returns Probe array from `globalThis.__PROBES__`.
 
 @throws When `globalThis.__PROBES__` was not injected.
 */
function getProbes(): readonly PackageProbe[] {
  if (!Object.hasOwn(
    globalThis,
    '__PROBES__',
  ))
    throw new Error('globalThis.__PROBES__ not injected; check render-html.ts',);
  /**
   Probe array injected onto `globalThis` by `render-html.ts`.
   */
  const probes = globalThis.__PROBES__;
  return probes;
}

/**
 Absence marker for {@link pickedProbe} meaning "no pickable probe is under
 the cursor"; never a {@link PackageProbe}.
 */
const NO_PICKED_PROBE: unique symbol = Symbol('deps-cube/no-picked-probe',);

/**
 Extracts the probe payload from a deck.gl picking-info object, or
 returns {@link NO_PICKED_PROBE} when nothing was picked or the picked datum
 lacks the `.probe` field. `info.object` is typed `any` by deck.gl; the cast
 is justified because we own the layer-data contract.
 
 @param info - deck.gl picking info.
 
 @returns Picked probe, or {@link NO_PICKED_PROBE} when no probe is under the cursor.
 */
function pickedProbe(
  info: ForeignBorrowed<PickingInfo>,
): PackageProbe | typeof NO_PICKED_PROBE {
  if ((info.object
    === undefined) || (info.object
      === null))
    return NO_PICKED_PROBE;
  if ((typeof info.object) !== 'object')
    return NO_PICKED_PROBE;
  if (!('probe' in info
    .object))
    return NO_PICKED_PROBE;
  /* oxlint-disable typescript-eslint/no-unsafe-type-assertion, typescript-eslint/no-unsafe-member-access -- ScatterplotLayer is fed ScatterDatum from layer factories; .probe is always a PackageProbe. */
  return info.object
    .probe as PackageProbe;
  /* oxlint-enable typescript-eslint/no-unsafe-type-assertion, typescript-eslint/no-unsafe-member-access */
}

//endregion Helpers

//region Render path

/**
 Serialises the current state into the URL hash via
 `history.replaceState` so the back stack stays clean.
 
 @param session - Source session.
 
 @mutates session - `JSON.stringify` may invoke hooks on state values.
 */
function syncHash(
  { session, }: {
    session: Readonly<{
      state: ReadonlyDeep<AppState>;
    }>;
  },
): void {
  history.replaceState(
    null,
    '',
    writeStateToHash({
      state: session.state,
    },),
  );
}

//endregion Render path

//region Picking

/* oxlint-disable no-restricted-syntax/no-nullish-union -- mirrors deck.gl's `getTooltip` contract: the handler receives the external `PickingInfo` (mutating methods, owned by deck.gl) and must return `{ html: string } | null` per deck.gl's `TooltipContent` type, so neither the param nor the nullish return can be reshaped. */
/**
 Builds the `getTooltip` payload for the deck.gl tooltip widget.
 
 @param info - deck.gl picking info.
 
 @returns `{ html }` for hovered probes, `null` otherwise.
 */
function getTooltipForInfo(
  info: ForeignBorrowed<PickingInfo>,
): { html: string; } | null {
  /**
   Probe under the cursor, or {@link NO_PICKED_PROBE} for hover-over-empty-space.
   */
  const probe = pickedProbe(info,);
  if (probe === NO_PICKED_PROBE)
    return null;
  return {
    html: formatTooltipHtml({
      probe,
    },),
  };
}
/* oxlint-enable no-restricted-syntax/no-nullish-union */

/**
 `onClick` handler; pins a tooltip beside the canvas, or unpins it
 when the click misses every glyph.
 
 @param info - deck.gl picking info.
 */
function onCanvasClick(info: ForeignBorrowed<PickingInfo>,): void {
  /**
   Probe under the click, or {@link NO_PICKED_PROBE} for miss-clicks that should unpin instead of pin.
   */
  const probe = pickedProbe(info,);
  if (probe === NO_PICKED_PROBE) {
    unpinTooltip();
    return;
  }
  pinTooltip({
    probe,
  },);
}

//endregion Picking

//region Bootstrap

/**
 Builds the initial {@link Session} from the embedded probes + URL hash.
 
 `onViewStateChange` is wired via `deck.setProps` after the session
 is created so the closure can capture the post-declaration binding
 without a forward reference.
 
 @param probes - Probe array from `globalThis.__PROBES__`.
 
 @returns Hydrated session ready for event wiring.
 */
function createSession(
  { probes, }: { readonly probes: readonly PackageProbe[]; },
): Session {
  /**
   Initial {@link AppState}; uses any bookmarked URL hash, otherwise falls back to the data-driven defaults.
   */
  const initial = readStateFromHash({
    hash: globalThis.location
      .hash,
    fallback: defaultState({
      probes,
    },),
  },);
  /**
   Per-channel data extents used by every layer factory; recomputed only when the dim mapping changes.
   */
  const bounds = computeSceneBounds({
    probes,
    dimMapping: initial.dimMapping,
  },);
  /**
   Set of probe indices that pass the initial filter combination; drives full-vs-faded opacity.
   */
  const visibleIndices = computeVisibleIndices({
    probes,
    toggles: initial.toggles,
    ranges: initial.ranges,
    search: initial.search,
    dimMapping: initial.dimMapping,
  },);
  /**
   Current light/dark scheme snapshot; passed through to layer factories for colour selection.
   */
  const chrome = detectScheme();
  /**
   deck.gl `Deck` instance bound to the `#deck-canvas` element; layers, view state, and tooltips wire through this.
   */
  const deck = new Deck<OrbitView>({
    canvas: 'deck-canvas',
    views: orbitView,
    initialViewState: {
      ...initial.viewState,
      target: [
        initial.viewState
          .target[0],
        initial.viewState
          .target[1],
        initial.viewState
          .target[2],
      ],
    },
    controller: true,
    layers: [
      ...buildLayers({
        probes,
        state: initial,
        visibleIndices,
        bounds,
        chrome,
      },),
    ],
    getTooltip: getTooltipForInfo,
    onClick: onCanvasClick,
  },);
  /**
   Mutable session bundle; every wire handler reads and writes through this single object.
   */
  const session: Session = {
    state: initial,
    bounds,
    visibleIndices,
    chrome,
    deck,
  };
  deck.setProps({
    onViewStateChange: function onViewStateChange(
      params: ForeignBorrowed<ViewStateChangeParameters>,
    ) {
      /**
       Latest view-state delta from deck.gl; copied into the session so hash sync can serialise it.
       */
      const v = params.viewState;
      session.state = {
        ...session.state,
        viewState: {
          target: [
            v.target[0],
            v.target[1],
            v.target[2],
          ],
          zoom: v.zoom,
          rotationOrbit: v.rotationOrbit
            ?? 0,
          rotationX: v.rotationX
            ?? 0,
        },
      };
      syncHash({
        session,
      },);
    },
  },);
  return session;
}

/**
 Entry point; called once at module-load time. Reads probes, builds
 the session, wires every control, paints the initial scene, syncs
 the URL hash, and updates the visibility counter.
 */
function start(): void {
  /**
   Embedded probe payload; throws via {@link getProbes} when the global is missing.
   */
  const probes = getProbes();
  /**
   Hydrated session for this page load.
   */
  const session = createSession({
    probes,
  },);
  syncDomFromState({
    state: session.state,
  },);
  /**
   Closure passed to every wire function; recomputes visibility,
   redraws layers, and syncs the URL hash after a state mutation.
   No args because session + probes are captured from `start`'s
   scope.
   */
  function commit(): void {
    recomputeVisibility({
      session,
      probes,
    },);
    rerenderLayers({
      session,
      probes,
    },);
    syncHash({
      session,
    },);
  }
  wireDimDropdowns({
    session,
    probes,
    commit,
  },);
  wireToggles({
    session,
    commit,
  },);
  wireRanges({
    session,
    commit,
  },);
  wireSearch({
    session,
    commit,
  },);
  wireDisplay({
    session,
    commit,
  },);
  wireReset({
    session,
    probes,
    commit,
  },);
  recomputeVisibility({
    session,
    probes,
  },);
  syncHash({
    session,
  },);
}

start();

//endregion Bootstrap
