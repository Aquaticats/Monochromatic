/**
 Dim-dropdown and range-slider event wiring for the deck.gl scene controller.
 
 @example
 ```ts
 wireDimDropdowns({ session, probes, commit });
 wireRanges({ session, commit });
 ```
 */

import { computeSceneBounds, } from '../deck-config.ts';
import type { PackageProbe, } from '../probe.ts';
import {
  elInput,
  elSelect,
} from './controller-dom.ts';
import {
  CHANNEL_KEYS,
  DIM_KEYS,
} from './controller-event-constants.ts';
import type {
  Commit,
  Session,
} from './controller-event-types.ts';
import {
  type DataDimKey,
  DIM_UNKNOWN,
  extractDim,
} from './filter.ts';

//region Helpers

/**
 Computes inclusive `[min, max]` extent across probes for one data dim, skipping unknowns.
 
 @param probes - Source probes.
 
 @param dim - Dim whose extent to compute.
 
 @returns Inclusive `[min, max]` bounds, or `[0, 0]` if no probe has known value.
 
 @example
 ```ts
 const extent = computeRangeExtent({ probes, dim: 'logDownloads' });
 ```
 */
function computeRangeExtent(
  {
    probes,
    dim,
  }: {
    readonly probes: readonly PackageProbe[];
    readonly dim: DataDimKey;
  },
): readonly [
  number,
  number,
] {
  /**
   Known non-null values for `dim` across `probes`; basis for `[min, max]` extent.
   */
  const values = probes
    .map(function pluck(probe,) {
      return extractDim({
        probe,
        dim,
      },);
    },)
    .filter(function known(value,): value is number {
      return value !== DIM_UNKNOWN;
    },);
  if (values.length
    === 0) {
    return [
      0,
      0,
    ];
  }
  return [
    Math.min(...values,),
    Math.max(...values,),
  ];
}

//endregion Helpers

//region Wire functions

/**
 Wires the six dim dropdowns.
 
 Changing a dim resets that channel's range slider to the new dim's extent,
 because old bounds are in old dim units and become meaningless.
 
 @param session - Mutable session.
 
 @param probes - Source probes.
 
 @param commit - Callback that re-renders and syncs hash.
 
 @example
 ```ts
 wireDimDropdowns({ session, probes, commit });
 ```
 */
export function wireDimDropdowns(
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
  CHANNEL_KEYS.forEach(function bind(channel,) {
    /**
     Dim-dropdown `<select>` for this channel; source of `change` event below.
     */
    const select = elSelect(`dim-${channel}`,);
    select.addEventListener(
      'change',
      function onChange() {
        /**
         Raw `value` attribute from dropdown; validated against {@link DIM_KEYS}.
         */
        const raw = select.value;
        /**
         Narrowed {@link DataDimKey} matching `raw`; `undefined` rejects stale values.
         */
        const nextDim = DIM_KEYS.find(function match(candidate,) {
          return candidate === raw;
        },);
        if (nextDim === undefined)
          return;
        /**
         Fresh `[min, max]` for newly-selected dim.
         */
        const extent = computeRangeExtent({
          probes,
          dim: nextDim,
        },);
        /**
         Dim mapping with this channel repointed at the newly-selected dim; feeds bounds recompute below.
         */
        const nextDimMapping = {
          ...session.state
            .dimMapping,
          [channel]: nextDim,
        };
        session.state = {
          ...session.state,
          dimMapping: nextDimMapping,
          ranges: {
            ...session.state
              .ranges,
            [channel]: extent,
          },
        };
        /**
         Min-slider for this channel; min/max/value rewritten to new dim's extent below.
         */
        const minSlider = elInput(`range-${channel}-min`,);
        /**
         Max-slider for this channel; min/max/value rewritten to new dim's extent below.
         */
        const maxSlider = elInput(`range-${channel}-max`,);
        minSlider.min = extent[0]
          .toString();
        minSlider.max = extent[1]
          .toString();
        minSlider.value = extent[0]
          .toString();
        maxSlider.min = extent[0]
          .toString();
        maxSlider.max = extent[1]
          .toString();
        maxSlider.value = extent[1]
          .toString();
        session.bounds = computeSceneBounds({
          probes,
          dimMapping: nextDimMapping,
        },);
        commit();
      },
    );
  },);
}

/**
 Wires the six min/max slider pairs.
 
 Live `input` events update the mask immediately for snappy filtering; no
 debounce is used because the mask is O(probes).
 
 @param session - Mutable session.
 
 @param commit - Callback that re-renders and syncs hash.
 
 @example
 ```ts
 wireRanges({ session, commit });
 ```
 */
export function wireRanges(
  {
    session,
    commit,
  }: {
    session: Session;
    commit: Commit;
  },
): void {
  CHANNEL_KEYS.forEach(function bind(channel,) {
    /**
     Min-slider for this channel; closed over by `onInput`.
     */
    const minSlider = elInput(`range-${channel}-min`,);
    /**
     Max-slider for this channel; closed over by `onInput`.
     */
    const maxSlider = elInput(`range-${channel}-max`,);
    /**
     Normalises slider order so crossing handles still yields valid range.
     */
    function onInput(): void {
      /**
       Numeric form of min-slider's current value.
       */
      const minVal = Number(minSlider.value,);
      /**
       Numeric form of max-slider's current value.
       */
      const maxVal = Number(maxSlider.value,);
      session.state = {
        ...session.state,
        ranges: {
          ...session.state
            .ranges,
          [channel]: minVal <= maxVal
            ? [
              minVal,
              maxVal,
            ]
            : [
              maxVal,
              minVal,
            ],
        },
      };
      commit();
    }
    minSlider.addEventListener(
      'input',
      onInput,
    );
    maxSlider.addEventListener(
      'input',
      onInput,
    );
  },);
}

//endregion Wire functions
