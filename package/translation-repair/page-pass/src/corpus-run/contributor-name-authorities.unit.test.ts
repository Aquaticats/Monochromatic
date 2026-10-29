/**
 Tests for the contributor-name authorities, one per original name.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { nameAuthorities, } from '../../dist/final/node/index.mjs';
import { pair, } from './title-reference.test-fixture.ts';

/**
 Authority of one name under an archive and a page that sign as given.

 @param archiveName - name the archive's signature carries

 @param pageName - name the page's signature carries

 @returns Authorities, as entries

 @example
 ```ts
 const entries = authorityOf({ archiveName: 'Brocade Cat', pageName: 'Brocade Cat', },);
 ```
 */
function authorityOf(
  {
    archiveName,
    pageName,
  }: {
    readonly archiveName: string;
    readonly pageName: string;
  },
): unknown {
  return [...nameAuthorities({
    slices: [pair({
      sliceIndex: 1,
      source: '它醒了。\n\n——锦猫, 2025 年 2 月 10 日',
      target: `It woke.\n\n——${archiveName}, February 10, 2025`,
    },),],
    pageText: new Map([[1, `It woke.\n\n——${pageName}, February 10, 2025`,],]),
  },),];
}

await describe({
  name: nameAuthorities.name,
  children: [
    it({
      name: 'READS NO page rendering where the page carries no signature at that position, leaving '
        + 'the handle to its pinyin reading',
      fn: async () => {
        /**
         Authorities of one named slice whose page has a section heading but
         no signature for the name at its position.
         */
        const authorities = nameAuthorities({
          slices: [pair({
            sliceIndex: 1,
            source: '### 其十：锦猫\n\n它醒了。\n\n<p style="text-align: end;">——锦猫, 2025 年 2 月 10 日</p>',
            target: '',
          },),],
          pageText: new Map([[1, '### Anon\n\nUnrelated.',],]),
        },);
        expect([...authorities,],).toEqual([[
          '锦猫',
          {
            rendering: 'Jinmao',
            origin: 'the pinyin reading of the original\'s handle',
          },
        ],],);
      },
    },),
    it({
      name: 'TAKES NO page rendering that shows a reader nothing, leaving the handle to its pinyin reading: a page '
        + 'signature whose name is a zero-width space, a Hangul filler, both among spaces, or either before a '
        + 'meaning in parentheses renders the signer as nothing a page could repeat',
      fn: async () => {
        /**
         Names a page signs with that show a reader nothing once any meaning
         in parentheses is set aside.
         */
        const names = [
          '\u{200B}',
          '\u{3164}',
          ' \u{200B}\u{3164} ',
          '\u{200B} (Brocade Cat)',
          '\u{3164} (Brocade Cat)',
        ];
        expect(names.map(function authoritiesUnder(name,): readonly (readonly [string, unknown,])[] {
          return [...nameAuthorities({
            slices: [pair({
              sliceIndex: 1,
              source: '它醒了。\n\n——锦猫, 2025 年 2 月 10 日',
              target: '',
            },),],
            pageText: new Map([[1, `It woke.\n\n——${name}, February 10, 2025`,],]),
          },),];
        },),).toEqual(names.map(function pinyinAlone(): readonly (readonly [string, unknown,])[] {
          return [[
            '锦猫',
            {
              rendering: 'Jinmao',
              origin: 'the pinyin reading of the original\'s handle',
            },
          ],];
        },),);
      },
    },),
    it({
      name: 'TAKES NO archive rendering that shows a reader nothing, leaving the signer to the page\'s rendering, and '
        + 'to the pinyin reading where the page shows nothing too',
      fn: async () => {
        expect(authorityOf({ archiveName: '\u{200B}', pageName: 'Brocade Cat', },),).toEqual([[
          '锦猫',
          {
            rendering: 'Brocade Cat',
            origin: 'the page\'s signature rendering',
          },
        ],],);
        expect(authorityOf({ archiveName: ' \u{3164}\u{200B} ', pageName: '\u{200B}', },),).toEqual([[
          '锦猫',
          {
            rendering: 'Jinmao',
            origin: 'the pinyin reading of the original\'s handle',
          },
        ],],);
      },
    },),
  ],
},);
