/**
 Window sizes, focus orders and pointer targets that the checks rely on.
 
 @example
 ```ts
 import { WIDE_WINDOW, } from './layout.ts';
 ```
 
 @packageDocumentation
 */

/**
 The wide window, in which the folder list and the track list sit side by side.
 */
export const WIDE_WINDOW = {
  width: 1_000,
  height: 800,
} as const;

/**
 The narrow window, in which the layout folds and the folder list becomes a picker overlay.
 */
export const NARROW_WINDOW = {
  width: 443,
  height: 900,
} as const;

/**
 Focus labels in Tab order on the player page in the wide layout.
 */
export const UNFOLDED_ORDER: readonly string[] = [
  'open-pill',
  'folder-list',
  'track-list',
  'seek',
  'previous',
  'play',
  'next',
  'mode-group',
  'search',
  'settings',
];

/**
 Focus labels in Tab order on the player page in the folded layout.
 */
export const FOLDED_ORDER: readonly string[] = [
  'folder-trigger',
  'open-pill',
  'track-list',
  'seek',
  'previous',
  'play',
  'next',
  'mode-group',
  'search',
  'settings',
];

/**
 Pointer targets in the wide window, in pixels.
 */
export const WIDE_POINTS = {
  /**
   A folder row in the folder list.
   */
  folderRow: {
    x: 100,
    y: 118,
  },
  /**
   The Search button at the top right, where its badge also sits.
   */
  searchButton: {
    x: 911,
    y: 28,
  },
  /**
   A track row, for the context menu.
   */
  trackRow: {
    x: 700,
    y: 80,
  },
} as const;

/**
 Pointer targets of the click-through checks, in pixels of the wide window (1000 by 800, the track pane from x 500).
 */
export const WIDE_CLICK_POINTS = {
  /**
   Points over the track rows of the right pane, which the Search page covers while it is open.
   */
  underSearch: [
    {
      x: 700,
      y: 300,
    },
    {
      x: 800,
      y: 230,
    },
    {
      x: 900,
      y: 150,
    },
  ],
  /**
   A point on the body of a single-line toast at the bottom of the right pane, left of its text: the card starts 12 px
   inside the pane and ends 96 px above the window bottom, 48 px tall. A track row sits under it.
   */
  toastBody: {
    x: 530,
    y: 680,
  },
} as const;

/**
 Vertical distance between the click point of one track row and the next, in pixels: a row is about 72 px tall with its
 supporting line, and the second row spans the 70 px to 140 px below the first row's top edge.
 */
export const TRACK_ROW_STEP_PX = 90;

/**
 The click point of the second track row, in pixels: one row below the first.
 */
const SECOND_ROW_POINT_Y: number = WIDE_POINTS.trackRow
  .y
  + TRACK_ROW_STEP_PX;

/**
 The vertical position of every click-through point that lands on a track row, in pixels.
 */
const CLICK_THROUGH_POINT_Y: readonly number[] = [
  SECOND_ROW_POINT_Y,
  ...WIDE_CLICK_POINTS.underSearch
    .map(function pointY(point,): number {
      return point.y;
    },),
  WIDE_CLICK_POINTS.toastBody
    .y,
];

/**
 How many track rows the click-through points reach, derived from the lowest of them: the first row's click point is
 `WIDE_POINTS.trackRow.y`, and each row below it sits `TRACK_ROW_STEP_PX` further down, so the lowest point needs the row
 it lands in and every row above it.
 */
export const CLICK_THROUGH_TRACK_ROWS: number = Math.ceil(
  (Math.max(...CLICK_THROUGH_POINT_Y,)
    - WIDE_POINTS.trackRow
      .y) / TRACK_ROW_STEP_PX,
) + 1;

/**
 Pointer targets of the click-through check in the narrow window (443 by 900), in pixels.
 */
export const NARROW_CLICK_POINTS = {
  /**
   Points that, on the player page, press the pause button, a mode segment and a track row, all hidden by Search.
   */
  underSearch: [
    {
      x: 221,
      y: 804,
    },
    {
      x: 66,
      y: 859,
    },
    {
      x: 200,
      y: 300,
    },
  ],
  /**
   The mode segment among them, which changes the playback mode on the player page.
   */
  modeSegment: {
    x: 66,
    y: 859,
  },
} as const;

/**
 The accents and color schemes the screenshot checks cover.
 */
export const VISUAL_VARIANTS = [
  {
    scheme: 'dark',
    accent: '3B82F6',
  },
  {
    scheme: 'dark',
    accent: '22C55E',
  },
  {
    scheme: 'light',
    accent: '3B82F6',
  },
  {
    scheme: 'light',
    accent: '22C55E',
  },
] as const;
