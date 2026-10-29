/**
 Stats section markup for the wc text-stats tool: six paired stat tiles.
 
 Chars, lines, words, sentences, and paragraphs each pair a headline
 count with a "longest" sub-stat; bytes stands alone. The pairing halves
 the tile count and demotes the maxima to secondary lines instead of
 giving them equal billing with the counts. Every headline and sub-stat
 carries a `title` attribute spelling out exactly how it's counted.
 
 No element carries an `id`: the client script matches {@link STAT_TILES}
 to rendered tiles positionally (`.tile-value` and `.tile-sub-value`
 always render in {@link STAT_TILES} order, one-to-one with headline
 counts and, for the subset with a {@link StatTile.sub}, sub-stats).
 */
import { hHtml as h, } from '@monochromatic-dev/hyperscript-core/ts';

import type { TextStats, } from './stat/index.ts';

/**
 Sub-stat of a tile: the "longest" line under the headline count.
 */
export type StatSub = Readonly<{
  /**
   {@link TextStats} field this sub-stat displays.
   */
  key: keyof TextStats;
  /**
   Unit word rendered after the value, disambiguating what "longest"
   measures (max sentence length is in words, max paragraph length in
   sentences).
   */
  unit: string;
  /**
   `title` attribute spelling out exactly how this sub-stat is counted.
   */
  title: string;
}>;

/**
 One stat tile: a display label, the {@link TextStats} field of its
 headline count, and optionally a "longest" sub-stat.
 */
export type StatTile = Readonly<{
  /**
   Display label shown above the headline count.
   */
  label: string;
  /**
   {@link TextStats} field the headline count displays.
   */
  key: keyof TextStats;
  /**
   `title` attribute spelling out exactly how the headline count is counted.
   */
  title: string;
  /**
   Optional "longest" sub-stat rendered under the headline count.
   */
  sub?: StatSub;
}>;

/**
 Every stat tile, in display order. Exported so the client script can
 reuse the same {@link TextStats} field mapping instead of duplicating
 it, matching rendered tiles to fields positionally.
 */
export const STAT_TILES: readonly StatTile[] = [
  {
    label: 'Bytes',
    key: 'bytes',
    title: 'UTF-8 encoded byte length of the input.',
  },
  {
    label: 'Chars',
    key: 'chars',
    title: 'Grapheme cluster count of the input.',
    sub: {
      key: 'maxCharLength',
      unit: 'bytes',
      title: 'UTF-8 encoded byte length of the widest grapheme cluster.',
    },
  },
  {
    label: 'Words',
    key: 'words',
    title: 'Word count, using Unicode word segmentation.',
    sub: {
      key: 'maxWordLength',
      unit: 'chars',
      title: 'Grapheme cluster length of the longest word.',
    },
  },
  {
    label: 'Lines',
    key: 'lines',
    title: 'Line count, excluding blank lines.',
    sub: {
      key: 'maxLineLength',
      unit: 'chars',
      title: 'Grapheme cluster length of the longest non-blank line.',
    },
  },
  {
    label: 'Sentences',
    key: 'sentences',
    title: 'Sentence count, using Unicode sentence segmentation.',
    sub: {
      key: 'maxSentenceLength',
      unit: 'words',
      title: 'Word count of the longest sentence.',
    },
  },
  {
    label: 'Paragraphs',
    key: 'paragraphs',
    title: 'Paragraph count, where paragraphs are separated by one or more blank lines.',
    sub: {
      key: 'maxParagraphLength',
      unit: 'sentences',
      title: 'Sentence count of the longest paragraph.',
    },
  },
];

/**
 Renders one {@link STAT_TILES} entry as a `<div class="tile">` wrapping
 a `<dt>` label and one or two `<dd>`s (headline count, optional
 "longest" sub-stat), each value starting at `0` until the client
 script computes real stats. Both the headline and the sub-stat carry a
 `title` attribute spelling out exactly how that metric is counted.
 Neither carries an `id`; the client script locates them positionally.
 
 @param tile - tile definition to render
 
 @returns HTML string for one tile
 */
function renderTile(tile: StatTile,): string {
  /**
   Optional sub-stat destructured once so member access stays flat.
   */
  const { sub, } = tile;

  /**
   `<dt>` label plus headline-count `<dd>` present on every tile.
   */
  const children = [
    h(
      {
        tag: 'dt',
        class: 'tile-label',
        text: tile.label,
      },
    ),
    h(
      {
        tag: 'dd',
        class: 'tile-value',
        attrs: { title: tile.title, },
        text: '0',
      },
    ),
  ];

  if (sub !== undefined) {
    children.push(
      h(
        {
          tag: 'dd',
          class: 'tile-sub',
          attrs: { title: sub.title, },
          children: [
            'longest ',
            h(
              {
                // Wraps the number and its unit so the only breakable
                // space is the one before this span (between "longest"
                // and the amount); without it, "longest 23" and "chars"
                // land on separate lines whenever a tile is too narrow
                // for the full phrase, since a plain text-node space
                // between the value and the unit is just as breakable
                // as the one before it.
                tag: 'span',
                class: 'tile-sub-amount',
                children: [
                  h(
                    {
                      tag: 'span',
                      class: 'tile-sub-value',
                      text: '0',
                    },
                  ),
                  ` ${sub.unit}`,
                ],
              },
            ),
          ],
        },
      ),
    );
  }

  return h(
    {
      tag: 'div',
      class: 'tile',
      children,
    },
  );
}

/**
 Renders the Stats section: a heading and the {@link STAT_TILES}
 definition list laid out as flex-wrapped tiles, via {@link renderTile}.
 
 @returns HTML string for the Stats section
 
 @example
 ```ts
 const statsSection = renderStatsSection();
 ```
 */
export function renderStatsSection(): string {
  return h(
    {
      tag: 'section',
      class: 'stats-section',
      children: [
        h(
          {
            tag: 'h2',
            text: 'Stats',
          },
        ),
        h(
          {
            tag: 'dl',
            attrs: { class: 'tiles', },
            html: STAT_TILES
              .map(function renderOneTile(tile,): string {
                return renderTile(tile,);
              },)
              .join('',),
          },
        ),
      ],
    },
  );
}
