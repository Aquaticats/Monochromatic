/**
 URL-hash bookmarking for the deck.gl scene.
 
 Serialises every piece of UI state (camera, dim mapping, filters,
 display toggles, search) into a URL hash so the user can copy and
 share an exact view. Pure functions: no DOM access, no globals
 beyond `encodeURIComponent` / `decodeURIComponent` / `JSON`.
 
 Hash format: `#state=<urlencoded-json>`. URL-encoding is preferred
 over base64 because the JSON payload is short and ASCII-safe in the
 common case, and `encodeURIComponent` handles unicode in the search
 field without the `btoa` "byte string" gotcha.
 
 @example
 ```ts
 const initial = defaultState({ probes });
 const restored = readStateFromHash({ hash: location.hash, fallback: initial });
 // ... user changes filters ...
 location.hash = writeStateToHash({ state: nextState });
 ```
 */

import { caughtValueText as caughtErrorMessage, } from '@monochromatic-dev/caught-value-core/ts';
import type { PackageProbe, } from '../probe.ts';
import {
  type ChannelKey,
  type DataDimKey,
  DIM_UNKNOWN,
  type DimMapping,
  extractDim,
  type RangeState,
  type ToggleKey,
  type ToggleState,
} from './filter.ts';

//region Types

/**
 OrbitView camera state per deck.gl conventions.
 
 - `rotationX`: pitch in degrees (0 = view along the +y axis).
 - `rotationOrbit`: yaw in degrees around the vertical axis.
 - `zoom`: log-like zoom factor; 0 ≈ default.
 - `target`: world-space coordinates of the orbit pivot.
 */
export type ViewState = {
  readonly rotationX: number;
  readonly rotationOrbit: number;
  readonly zoom: number;
  readonly target: readonly [
    number,
    number,
    number,
  ];
};

/**
 Display-only toggles (don't filter data, only chrome).
 */
export type DisplayToggleState = {
  readonly showThresholdPlanes: boolean;
  readonly showWireframe: boolean;
  readonly showAxisLabels: boolean;
  readonly nameLabels: 'none' | 'topN' | 'all';
  readonly showUnknownCluster: boolean;
};

/**
 Full UI state; every value that contributes to a unique view.
 */
export type AppState = {
  readonly viewState: ViewState;
  readonly dimMapping: DimMapping;
  readonly toggles: ToggleState;
  readonly ranges: RangeState;
  readonly search: string;
  readonly displayToggles: DisplayToggleState;
};

//endregion Types

//region Constants

/**
 Default dim mapping per the plan's recommendation.
 */
const DEFAULT_DIM_MAPPING: DimMapping = {
  x: 'logSourceBytes',
  y: 'logDaysStale',
  z: 'logInstallSize',
  color: 'tsRatio',
  shape: 'isLeafNumeric',
  size: 'logDownloads',
};

/**
 Default toggle state; every filter "don't care".
 */
const DEFAULT_TOGGLES: ToggleState = {
  isLeaf: 'any',
  tsMajority: 'any',
  large: 'any',
  recent: 'any',
  permissive: 'any',
  copyleft: 'any',
  hasKnownRepo: 'any',
};

/**
 Default display toggles.
 
 Threshold guide lines default to off because they're an opt-in
 heuristic overlay and clutter the reference-style coordinate-system
 backdrop. Name labels default to `'all'` so every glyph is identifiable
 on first load (the scatter is sparse enough; ~70 catalog entries;
 for the labels to be useful rather than overwhelming). Every other
 chrome element starts on so the scene reads as a proper 3D coordinate
 system on first load.
 */
const DEFAULT_DISPLAY_TOGGLES: DisplayToggleState = {
  showThresholdPlanes: false,
  showWireframe: true,
  showAxisLabels: true,
  nameLabels: 'all',
  showUnknownCluster: true,
};

/**
 Default OrbitView angle; slight tilt + slight orbit so 3 axes are distinguishable.
 */
const DEFAULT_VIEW_STATE: ViewState = {
  rotationX: 30,
  rotationOrbit: -45,
  zoom: 0,
  target: [
    0,
    0,
    0,
  ],
};

/**
 Channel keys, fixed order, used to iterate dim mapping.
 */
const CHANNEL_KEYS: readonly ChannelKey[] = [
  'x',
  'y',
  'z',
  'color',
  'shape',
  'size',
];

//endregion Constants

//region Range computation

/**
 Computes the `[min, max]` extent across all probes for one data dim,
 skipping unknowns (`null`). Falls back to `[0, 0]` when no probe has
 a known value (degenerate, but lets the UI render without NaN).
 
 @param probes - Full probe array.
 
 @param dim - Data dim to scan.
 
 @returns Inclusive `[min, max]` bounds.
 */
function computeExtent(
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
   Known dim readings (unknowns stripped) so `Math.min`/`Math.max` see only real numbers.
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

/**
 Builds a `RangeState` with every channel set to its data extent.
 
 @param probes - Full probe array.
 
 @param dimMapping - Current channel → dim mapping.
 
 @returns Range state spanning the full data extent on every channel.
 */
function computeFullRanges(
  {
    probes,
    dimMapping,
  }: {
    readonly probes: readonly PackageProbe[];
    readonly dimMapping: DimMapping;
  },
): RangeState {
  /**
   Per-channel `[channel, extent]` tuples; feeds `Object.fromEntries` to build the record.
   */
  const entries = CHANNEL_KEYS.map(function rangeForChannel(channel,) {
    return [
      channel,
      computeExtent({
        probes,
        dim: dimMapping[channel],
      },),
    ] as const;
  },);
  /**
   Channel-to-extent record before the unsafe cast back to `RangeState`.
   */
  const record = Object.fromEntries(entries,);
  // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- Object.fromEntries() returns Record<string, V>; the entries above exhaust ChannelKey.
  return record as RangeState;
}

//endregion Range computation

//region Defaults

/**
 Initial UI state given a freshly-loaded probe array.
 
 Dim mapping and toggles use the plan's defaults; ranges are the full
 data extent so every probe passes the range filter; search is empty;
 display toggles all on; camera at default tilt + orbit.
 
 Every nested object is deep-cloned from the module-level `DEFAULT_*`
 constants via `structuredClone` so the controller's mutate-in-place
 wire handlers (`session.state.toggles[key] = next`, etc.) cannot
 corrupt the shared defaults. Without this, the first user toggle
 would silently rewrite {@link DEFAULT_TOGGLES}, and subsequent
 `defaultState()` calls (e.g. from the reset button) would return the
 already-corrupted constants.
 
 @param probes - Full probe array.
 
 @returns Default {@link AppState} for first render.
 
 @example
 ```ts
 const initial = defaultState({ probes });
 ```
 */
export function defaultState(
  { probes, }: { readonly probes: readonly PackageProbe[]; },
): AppState {
  return {
    viewState: structuredClone(DEFAULT_VIEW_STATE,),
    dimMapping: structuredClone(DEFAULT_DIM_MAPPING,),
    toggles: structuredClone(DEFAULT_TOGGLES,),
    ranges: computeFullRanges({
      probes,
      dimMapping: DEFAULT_DIM_MAPPING,
    },),
    search: '',
    displayToggles: structuredClone(DEFAULT_DISPLAY_TOGGLES,),
  };
}

//endregion Defaults

//region Encoding

/**
 Serialises an {@link AppState} to a URL-safe string.
 
 Returns the encoded payload only; callers prepend `state=` to use it
 inside a hash. Use {@link writeStateToHash} for the full `#state=…`
 convenience.
 
 @param state - The state to encode.
 
 @returns URL-safe encoded string.
 
 @mutates state - `JSON.stringify` may invoke `toJSON`, getters, or proxy traps.
 
 @example
 ```ts
 const encoded = encodeState({ state });
 location.hash = `state=${encoded}`;
 ```
 */
export function encodeState(
  { state, }: { state: AppState; },
): string {
  return encodeURIComponent(JSON.stringify(state,),);
}

/**
 Absence marker meaning "this payload could not be restored to an
 {@link AppState}"; shared by {@link validateAppState} and
 {@link decodeState} because the latter propagates the former's absence
 directly. Never a restored state.
 
 @example
 ```ts
 const restored = decodeState({ encoded, },);
 if (restored === STATE_INVALID)
   return fallback;
 ```
 */
export const STATE_INVALID: unique symbol = Symbol('deps-cube encoded state cannot be restored',);

/**
 Shallow shape check; every top-level field present and of the right
 primitive kind. Doesn't deep-validate `dimMapping` values or range
 tuple shapes; if those are wrong the renderer will surface NaN /
 undefined and the user can reset via the URL.
 
 @param value - Parsed JSON value, untrusted.
 
 @returns Value typed as {@link AppState}, or {@link STATE_INVALID} when malformed.
 */
function validateAppState(value: unknown,): AppState | typeof STATE_INVALID {
  if (((typeof value) !== 'object') || (value === null))
    return STATE_INVALID;
  /**
   Top-level {@link AppState} fields that must all be present for the shape check to pass.
   */
  const required: readonly (keyof AppState)[] = [
    'viewState',
    'dimMapping',
    'toggles',
    'ranges',
    'search',
    'displayToggles',
  ];
  /**
   `true` only when every required field is present on the parsed object.
   */
  const has = required.every(function present(key,) {
    return key in value;
  },);
  if (!has)
    return STATE_INVALID;
  // oxlint-disable-next-line typescript-eslint/no-unsafe-type-assertion -- shape verified above; deep-validation is intentionally cheap.
  return value as AppState;
}

/**
 Decodes a URL-safe encoded string back into an {@link AppState}.
 
 Returns {@link STATE_INVALID} for any failure (URI malformed, JSON parse
 error, missing required fields, each logged via {@link caughtErrorMessage})
 so the caller can fall back to the default state without crashing.
 
 @param encoded - The URL-safe encoded string (no leading `state=`).
 
 @returns Restored state, or {@link STATE_INVALID} if the input is malformed.
 
 @example
 ```ts
 const restored = decodeState({ encoded: match[1] });
 if (restored === STATE_INVALID) {
   // fall back to defaults
 }
 ```
 */
export function decodeState(
  { encoded, }: { readonly encoded: string; },
): AppState | typeof STATE_INVALID {
  try {
    /**
     URI-decoded JSON payload extracted from the hash.
     */
    const json = decodeURIComponent(encoded,);
    /**
     Untrusted parsed JSON; downgraded to `unknown` so `validateAppState` is the only narrowing path.
     */
    const parsed = JSON.parse(json,) as unknown;
    return validateAppState(parsed,);
  }
  catch (error) {
    console.warn(
      `[deps-cube] encoded state decode failed: ${caughtErrorMessage(error,)}`,
    );
    return STATE_INVALID;
  }
}

//endregion Encoding

//region Hash helpers

/**
 Literal token preceding the URL-encoded payload in the location hash.
 */
const STATE_PARAM = 'state=';

/**
 Extracts the URL-encoded `state` parameter from a hash-stripped query
 string. Returns `''` when the parameter is absent. Mirrors
 `/(?:^|&)state=([^&]+)/` with a linear `indexOf` walk: locate either
 a leading `state=` or a preceding `&state=`, then read until the next
 `&` (or end of string).
 
 @param s - hash string with the leading `#` already removed
 
 @returns captured payload, or `''` when no `state=` parameter exists
 */
function extractStateParam(s: string,): string {
  /**
   Cursor at which the payload starts, or `-1` when no parameter is present.
   */
  const payloadStart = s.startsWith(STATE_PARAM,)
    ? STATE_PARAM.length
    : (function findAfterAmp(): number {
      /**
       Position of `&state=`; `-1` ends the search.
       */
      const ampIdx = s.indexOf(`&${STATE_PARAM}`,);
      return ampIdx === (-1) ? (-1) : (ampIdx + 1
        + STATE_PARAM
          .length);
    })();
  if (payloadStart === (-1))
    return '';
  /**
   Exclusive end of the payload at the next `&` (or string end).
   */
  const ampEnd = s.indexOf(
    '&',
    payloadStart,
  );
  return s.slice(
    payloadStart,
    ampEnd === (-1) ? s.length : ampEnd,
  );
}

/**
 Extracts the `state=` payload from a hash string (with or without
 leading `#`) and decodes it, falling back to `fallback` on any
 failure.
 
 @param hash - The hash string (typically `globalThis.location.hash`).
 
 @param fallback - State to return when the hash has no valid payload.
 
 @returns Decoded state, or `fallback` if absent/malformed.
 
 @example
 ```ts
 const state = readStateFromHash({
   hash: location.hash,
   fallback: defaultState({ probes }),
 });
 ```
 */
export function readStateFromHash(
  {
    hash,
    fallback,
  }: {
    readonly hash: string;
    readonly fallback: AppState;
  },
): AppState {
  /**
   Hash with the leading `#` removed so `state=` always appears at offset 0 or after `&`.
   */
  const stripped = hash.startsWith('#',) ? hash.slice(1,) : hash;
  /**
   URL-encoded payload from the `state=` parameter; `''` when absent.
   */
  const encoded = extractStateParam(stripped,);
  if (encoded === '')
    return fallback;
  /**
   Round-tripped state, or {@link STATE_INVALID} when the payload fails to parse or validate.
   */
  const decoded = decodeState({
    encoded,
  },);
  return decoded === STATE_INVALID ? fallback : decoded;
}

/**
 Builds a hash string (`#state=…`) for a given state.
 
 @param state - State to serialise.
 
 @returns Hash including the leading `#`.
 
 @mutates state - `JSON.stringify` may invoke hooks on state values.
 
 @example
 ```ts
 location.hash = writeStateToHash({ state });
 ```
 */
export function writeStateToHash(
  { state, }: { state: AppState; },
): string {
  return `#state=${
    encodeState({
      state,
    },)
  }`;
}

/**
 Provided for tests and other call sites that need the keys.
 */
export const TOGGLE_KEYS: readonly ToggleKey[] = [
  'isLeaf',
  'tsMajority',
  'large',
  'recent',
  'permissive',
  'copyleft',
  'hasKnownRepo',
];

//endregion Hash helpers
