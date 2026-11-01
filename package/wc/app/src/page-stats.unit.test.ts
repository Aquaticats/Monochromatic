/**
 Tests for the stat-tile markup and mappings.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  renderStatsSection,
  STAT_TILES,
} from './page-stats.ts';

await describe({
  name: renderStatsSection.name,
  children: [
    it({
      name: 'renders a heading and a tiles definition list',
      fn: async function rendersHeadingAndTiles(): Promise<void> {
        /**
         Full markup produced for the Stats section.
         */
        const html = renderStatsSection();

        expect(html,).toContain('<h2>Stats</h2>',);
        expect(html,).toContain('<dl class="tiles">',);
      },
    },),
    it({
      name: 'renders every STAT_TILES entry as a zero-valued labeled tile',
      fn: async function rendersEveryTile(): Promise<void> {
        /**
         Full markup produced for the Stats section.
         */
        const html = renderStatsSection();

        for (
          const {
            label,
            title,
          } of STAT_TILES
        ) {
          expect(html,).toContain(`<dt class="tile-label">${label}</dt>`,);
          expect(html,).toContain(
            `<dd class="tile-value" title="${title}">0</dd>`,
          );
        }
      },
    },),
    it({
      name: 'renders a longest sub-line with unit and title for every paired tile',
      fn: async function rendersSubLines(): Promise<void> {
        /**
         Full markup produced for the Stats section.
         */
        const html = renderStatsSection();

        for (const tile of STAT_TILES) {
          if (tile.sub !== undefined) {
            expect(html,).toContain(
              `<dd class="tile-sub" title="${tile.sub.title}">longest `
                + `<span class="tile-sub-amount">`
                + `<span class="tile-sub-value">0</span> ${tile.sub.unit}`
                + `</span></dd>`,
            );
          }
        }
      },
    },),
    it({
      name: 'pairs chars/lines/words/sentences/paragraphs and leaves bytes bare',
      fn: async function pairsExpectedTiles(): Promise<void> {
        /**
         Labels of tiles carrying a "longest" sub-stat.
         */
        const paired = STAT_TILES
          .filter(function hasSub(tile,): boolean {
            return tile.sub !== undefined;
          },)
          .map(function toLabel(tile,): string {
            return tile.label;
          },)
          .toSorted();

        expect(paired,).toEqual(
          [
            'Chars',
            'Lines',
            'Paragraphs',
            'Sentences',
            'Words',
          ],
        );
      },
    },),
    it({
      name: 'orders the Stats tiles as Bytes, Chars, Words, Lines, Sentences, Paragraphs',
      fn: async function ordersStatTiles(): Promise<void> {
        expect(
          STAT_TILES.map(function toLabel(tile,): string {
            return tile.label;
          },),
        ).toEqual(
          [
            'Bytes',
            'Chars',
            'Words',
            'Lines',
            'Sentences',
            'Paragraphs',
          ],
        );
      },
    },),
    it({
      name: 'never emits an id attribute',
      fn: async function omitsIds(): Promise<void> {
        /**
         Full markup produced for the Stats section.
         */
        const html = renderStatsSection();

        expect(html,).not
          .toContain(' id="',);
      },
    },),
  ],
},);
