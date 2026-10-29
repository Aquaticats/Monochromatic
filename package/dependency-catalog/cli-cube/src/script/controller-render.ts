/**
 Visibility and live-layer updates for the browser controller.
 
 @module
 */

import { buildLayers, } from '../deck-config.ts';
import type { PackageProbe, } from '../probe.ts';
import type { Session, } from './controller-session-types.ts';
import { computeVisibleIndices, } from './filter.ts';

/**
 Recomputes visible indices and updates the visibility counter.
 
 @param session - Mutable session receiving computed visibility.
 
 @param probes - Source probes.
 
 @example
 ```ts
 recomputeVisibility({ session, probes, });
 ```
 */
export function recomputeVisibility({
  session,
  probes,
}: {
  session: Session;
  probes: readonly PackageProbe[];
}): void {
  session.visibleIndices = computeVisibleIndices({
    probes,
    toggles: session.state
      .toggles,
    ranges: session.state
      .ranges,
    search: session.state
      .search,
    dimMapping: session.state
      .dimMapping,
  },);
  /**
   Counter element absent from tests and partial pages.
   */
  const counter = document.querySelector<HTMLElement>('#visibility-counter',);
  if (counter !== null) {
    counter.textContent = `${session.visibleIndices
      .size
      .toString()} of ${probes.length
        .toString()} visible`;
  }
}

/**
 Pushes freshly built layers into the live Deck renderer.
 
 @param session - Mutable live renderer session.
 
 @param probes - Source probes.
 
 @mutates session through session.deck.setProps renderer capability
 
 @example
 ```ts
 rerenderLayers({ session, probes, });
 ```
 */
export function rerenderLayers({
  session,
  probes,
}: {
  session: Session;
  probes: readonly PackageProbe[];
}): void {
  /**
   Layers rebuilt from current session inputs.
   */
  const layers = buildLayers({
    probes,
    state: session.state,
    visibleIndices: session.visibleIndices,
    bounds: session.bounds,
    chrome: session.chrome,
  },);
  session.deck
    .setProps({ layers: [...layers,], },);
}
