/**
 Shared per-dim metadata: display names, kind classification (continuous /
 binary / categorical), per-channel acceptance allowlist, and toggle
 labels.
 
 Owned here rather than in `./deck-labels.ts` so both the axis-label
 `TextLayer` builder (used at runtime by deck.gl) and the control-panel
 dropdown emitter (used at HTML-generation time by Node) can import the
 same constants without crossing the Node/browser boundary or duplicating
 the table.
 
 @example
 ```ts
 import { acceptsDim, DIM_DISPLAY_NAMES } from './dim-meta.ts';
 if (!acceptsDim({ channel: 'shape', dim: 'tsRatio' })) {
   // shape only accepts binary/categorical; render this option disabled
 }
 ```
 */

import type {
  ChannelKey,
  DataDimKey,
  ToggleKey,
} from './script/filter.ts';

//region Types

/**
 Coarse type classification used by the channel-acceptance allowlist.
 */
export type DimKind = 'continuous' | 'binary' | 'categorical';

//endregion Types

//region Display names

/**
 Per-dim display string for axis labels, dropdowns, and tooltips.
 */
export const DIM_DISPLAY_NAMES: Record<DataDimKey, string> = {
  logSourceBytes: 'log10(source bytes)',
  logDaysStale: 'log10(days stale)',
  logInstallSize: 'log10(install bytes)',
  logDownloads: 'log10(weekly downloads)',
  tsRatio: 'TypeScript ratio',
  runtimeDepCount: 'runtime dep count',
  transitiveDepCount: 'transitive dep count',
  logPackageAge: 'log10(age, days)',
  isLeafNumeric: 'is leaf (0/1)',
  licenseClassNumeric: 'license (0=permissive…3=unknown)',
};

//endregion Display names

//region Dim kinds

/**
 Kind classification per dim. Drives {@link CHANNEL_ACCEPTED_KINDS}.
 
 Continuous = anything ordinal with meaningful interpolation; binary =
 two discrete values; categorical = small set of unordered codes.
 */
export const DIM_KINDS: Record<DataDimKey, DimKind> = {
  logSourceBytes: 'continuous',
  logDaysStale: 'continuous',
  logInstallSize: 'continuous',
  logDownloads: 'continuous',
  tsRatio: 'continuous',
  runtimeDepCount: 'continuous',
  transitiveDepCount: 'continuous',
  logPackageAge: 'continuous',
  isLeafNumeric: 'binary',
  licenseClassNumeric: 'categorical',
};

//endregion Dim kinds

//region Channel-acceptance allowlist

/**
 Which kinds each visual channel meaningfully represents.
 
 Spatial axes (`x`, `y`, `z`) and `color` accept every kind; the
 renderer's accessors normalise any numeric output gracefully, so binary
 on `x` clusters probes onto two lines and categorical on `color` gives
 banded hues rather than crashes. Aesthetically coarser, but useful for
 audit queries like "stack on x by leaf-ness".
 
 `shape` accepts only binary/categorical; the filled-vs-stroked split
 is fundamentally a 2-state encoding; continuous values get thresholded
 at 0.5 which discards most of the information.
 
 `size` accepts only continuous; radius is a magnitude. Binary on
 size yields two radii and conveys nothing the shape channel doesn't.
 */
export const CHANNEL_ACCEPTED_KINDS: Record<ChannelKey, readonly DimKind[]> = {
  x: [
    'continuous',
    'binary',
    'categorical',
  ],
  y: [
    'continuous',
    'binary',
    'categorical',
  ],
  z: [
    'continuous',
    'binary',
    'categorical',
  ],
  color: [
    'continuous',
    'binary',
    'categorical',
  ],
  shape: [
    'binary',
    'categorical',
  ],
  size: ['continuous',],
};

//endregion Channel-acceptance allowlist

//region Toggle labels

/**
 Human-readable legend per 3-state boolean filter toggle.
 */
export const TOGGLE_LABELS: Record<ToggleKey, string> = {
  isLeaf: 'Is leaf (no runtime deps)',
  tsMajority: 'TS-majority (≥ 95%)',
  large: 'Large (≥ 10 KB source)',
  recent: 'Recent (< 1 year)',
  permissive: 'Permissive license',
  copyleft: 'Copyleft license',
  hasKnownRepo: 'Has known GH repo',
};

//endregion Toggle labels

//region Helpers

/**
 Tests whether a visual channel will represent a given dim.
 
 @param channel - Channel key being assigned.
 
 @param dim - Candidate dim.
 
 @returns `true` if the channel's accepted-kinds list contains the dim's kind.
 
 @example
 ```ts
 acceptsDim({ channel: 'size', dim: 'isLeafNumeric' }); // false
 acceptsDim({ channel: 'x', dim: 'isLeafNumeric' });    // true
 ```
 */
export function acceptsDim(
  {
    channel,
    dim,
  }: {
    readonly channel: ChannelKey;
    readonly dim: DataDimKey;
  },
): boolean {
  return CHANNEL_ACCEPTED_KINDS[channel]
    .includes(DIM_KINDS[dim],);
}

//endregion Helpers
