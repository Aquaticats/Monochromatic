/**
 Shared dimension bounds for report creation, initial state, and browser controls.

 Each probe is visited once without building an intermediate values array or
 passing the catalog as call arguments, so stack use stays constant.
 */

import type { PackageProbe, } from '../probe.ts';
import {
  type DataDimKey,
  DIM_UNKNOWN,
  extractDim,
} from './filter.ts';

/**
 Computes inclusive bounds over known readings, preserving numeric min/max semantics.

 @param probes - Catalog whose dimension values determine the bounds

 @param dim - Dimension to extract from each probe

 @param fallback - Bounds used when every reading is unknown or the catalog is empty

 @returns Inclusive minimum and maximum, or the supplied fallback

 @example
 ```ts
 const extent = computeRangeExtent({ probes, dim: 'logDownloads' });
 ```
 */
export function computeRangeExtent(
  {
    probes,
    dim,
    fallback = [
      0,
      0,
    ],
  }: {
    readonly probes: readonly PackageProbe[];
    readonly dim: DataDimKey;
    readonly fallback?: readonly [
      number,
      number,
    ];
  },
): readonly [
  number,
  number,
] {
  /**
   Owned accumulator keeps numeric extrema separate from the absence of readings.
   */
  const extent = {
    min: Number.POSITIVE_INFINITY,
    max: Number.NEGATIVE_INFINITY,
    hasKnownValue: false,
  };
  for (const probe of probes) {
    /**
     Numeric reading or the distinct unknown-dimension marker.
     */
    const value = extractDim({
      probe,
      dim,
    },);
    if (value === DIM_UNKNOWN)
      continue;
    extent.hasKnownValue = true;
    extent.min = Math.min(
      extent.min,
      value,
    );
    extent.max = Math.max(
      extent.max,
      value,
    );
  }
  return extent.hasKnownValue
    ? [
      extent.min,
      extent.max,
    ]
    : fallback;
}
