/**
 Tests for the three ways a directory id may stand as a page's visible name
 although the source names the person otherwise.

 WHY. On 2026-09-07 one page was refused for naming the person by the
 pinyin of the name, which is the directory id. The owner's answer was the pinyin check
 and the alias exemption, read on both front matters, since there has to be
 an English rendering of the name in the front matter. Each clause has a case
 that passes and the refusal has a case that stays.

 Cat-themed invention throughout; no corpus content appears here. The pinyin
 cases need real characters, and use common ones.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { splitFrontMatter, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  directoryIdNameStands,
  namesDirectoryId,
} from '../../dist/final/node/index.mjs';

/**
 Parses a front matter fixture, throwing where it does not parse so a broken
 fixture fails its case rather than passing an undefined through.

 @param text - page text starting with front matter

 @returns Parsed block

 @example
 ```ts
 const block = blockOf({ text: '---\nname: 猫童\n---\n', },);
 ```
 */
function blockOf({ text, }: { readonly text: string; },) {
  /**
   Parsed front matter, if the fixture carries one.
   */
  const block = splitFrontMatter({ text, },).frontMatter;
  if (block === undefined)
    throw new Error('fixture did not parse',);
  return block;
}

/**
 Page whose visible name is the id and whose alias is in the source script
 only, so nothing but the id itself can let it stand.
 */
const ID_ONLY_PAGE = '---\nname: maotong\ninfo:\n  alias: 猫童\n---\n\nBody.\n';

await describe({
  name: directoryIdNameStands.name,
  children: [
    it({
      name: 'STANDS where the id is the pinyin of the source name, letters compared without case or '
        + 'tone (one entry, 2026-09-07)',
      fn: async () => {
        expect(directoryIdNameStands({
          entryId: 'maotong',
          source: blockOf({ text: '---\nname: 猫童\ninfo:\n  alias: 猫童\n---\n', },),
          page: blockOf({ text: ID_ONLY_PAGE, },),
          archives: [],
        },),).toBe(true,);
        expect(directoryIdNameStands({
          entryId: 'MaoTong',
          source: blockOf({ text: '---\nname: 猫童\ninfo:\n  alias: 猫童\n---\n', },),
          page: blockOf({ text: ID_ONLY_PAGE, },),
          archives: [],
        },),).toBe(true,);
      },
    },),

    it({
      name: 'STANDS on a heteronym the common reading would miss, since every reading of every '
        + 'character is allowed',
      fn: async () => {
        // 单 reads dan, shan and chan; the common reading is dan.
        expect(directoryIdNameStands({
          entryId: 'shanmao',
          source: blockOf({ text: '---\nname: 单猫\ninfo:\n  alias: 单猫\n---\n', },),
          page: blockOf({ text: '---\nname: shanmao\ninfo:\n  alias: 单猫\n---\n', },),
          archives: [],
        },),).toBe(true,);
      },
    },),

    it({
      name: 'STANDS where the source itself carries the id among its aliases, so the handle is the '
        + 'person\'s own',
      fn: async () => {
        expect(directoryIdNameStands({
          entryId: 'SnowPuff',
          source: blockOf({ text: '---\nname: 雪团\ninfo:\n  alias: 雪团, SnowPuff\n---\n', },),
          page: blockOf({ text: '---\nname: SnowPuff\ninfo:\n  alias: 雪团\n---\n', },),
          archives: [],
        },),).toBe(true,);
      },
    },),

    it({
      name: 'STANDS where the page or the archive carries a Latin-script alias other than the id, '
        + 'the English rendering the front matter has to have',
      fn: async () => {
        expect(directoryIdNameStands({
          entryId: 'Whiskers',
          source: blockOf({ text: '---\nname: 猫猫\ninfo:\n  alias: 猫咪\n---\n', },),
          page: blockOf({ text: '---\nname: Whiskers\ninfo:\n  alias: 猫咪, Maomao\n---\n', },),
          archives: [],
        },),).toBe(true,);
        expect(directoryIdNameStands({
          entryId: 'Whiskers',
          source: blockOf({ text: '---\nname: 猫猫\ninfo:\n  alias: 猫咪\n---\n', },),
          page: blockOf({ text: '---\nname: Whiskers\ninfo:\n  alias: 猫咪\n---\n', },),
          archives: [blockOf({ text: '---\nname: Whiskers\ninfo:\n  alias: Little Whiskers (cat)\n---\n', },),],
        },),).toBe(true,);
      },
    },),

    it({
      name: 'FALLS where the id spells nothing of the name, the source does not own it, and no '
        + 'alias on either side is a Latin rendering other than the id itself',
      fn: async () => {
        expect(directoryIdNameStands({
          entryId: 'Whiskers',
          source: blockOf({ text: '---\nname: 猫猫\ninfo:\n  alias: 猫咪\n---\n', },),
          page: blockOf({ text: '---\nname: Whiskers\ninfo:\n  alias: 猫咪, whiskers\n---\n', },),
          archives: [blockOf({ text: '---\nname: Whiskers\ninfo:\n  alias: WHISKERS\n---\n', },),],
        },),).toBe(false,);
      },
    },),

    it({
      name: 'FALLS on a partial reading, since the letters must spell the whole name and nothing '
        + 'more',
      fn: async () => {
        expect(directoryIdNameStands({
          entryId: 'mao',
          source: blockOf({ text: '---\nname: 猫童\ninfo:\n  alias: 猫童\n---\n', },),
          page: blockOf({ text: '---\nname: mao\ninfo:\n  alias: 猫童\n---\n', },),
          archives: [],
        },),).toBe(false,);
        expect(directoryIdNameStands({
          entryId: 'maotongcat',
          source: blockOf({ text: '---\nname: 猫童\ninfo:\n  alias: 猫童\n---\n', },),
          page: blockOf({ text: '---\nname: maotongcat\ninfo:\n  alias: 猫童\n---\n', },),
          archives: [],
        },),).toBe(false,);
      },
    },),

    it({
      name: 'READS NO name and no alias out of front matter holding no data, or holding a list, neither '
        + 'being a record',
      fn: async () => {
        // No data parses to null, which only the record check keeps the name
        // and alias reads from destructuring.
        expect(directoryIdNameStands({
          entryId: 'maotong',
          source: blockOf({ text: '---\n---\n', },),
          page: blockOf({ text: ID_ONLY_PAGE, },),
          archives: [],
        },),).toBe(false,);
        expect(directoryIdNameStands({
          entryId: 'maotong',
          source: blockOf({ text: '---\n- 猫童\n- MaoTong\n---\n', },),
          page: blockOf({ text: ID_ONLY_PAGE, },),
          archives: [],
        },),).toBe(false,);
      },
    },),

    it({
      name: 'READS NO name where the name is no string, since a number names nobody',
      fn: async () => {
        expect(directoryIdNameStands({
          entryId: 'maotong',
          source: blockOf({ text: '---\nname: 42\n---\n', },),
          page: blockOf({ text: ID_ONLY_PAGE, },),
          archives: [],
        },),).toBe(false,);
      },
    },),

    it({
      name: 'READS NO alias where the info block holds nothing or holds a number, neither being a record',
      fn: async () => {
        // An empty info block parses to null, which only the record check
        // keeps the alias read from destructuring.
        expect(directoryIdNameStands({
          entryId: 'maotong',
          source: blockOf({ text: '---\nname: 咪咪\ninfo:\n---\n', },),
          page: blockOf({ text: ID_ONLY_PAGE, },),
          archives: [],
        },),).toBe(false,);
        expect(directoryIdNameStands({
          entryId: 'maotong',
          source: blockOf({ text: '---\nname: 咪咪\ninfo: 7\n---\n', },),
          page: blockOf({ text: ID_ONLY_PAGE, },),
          archives: [],
        },),).toBe(false,);
      },
    },),

    it({
      name: 'STANDS where the alias is a list whose entries name the id, and FALLS where the alias is '
        + 'neither text nor list, though written out it would spell the id',
      fn: async () => {
        expect(directoryIdNameStands({
          entryId: 'maotong',
          source: blockOf({ text: '---\nname: 咪咪\ninfo:\n  alias:\n    - maotong\n---\n', },),
          page: blockOf({ text: ID_ONLY_PAGE, },),
          archives: [],
        },),).toBe(true,);
        // A YAML boolean under the alias, and an id spelled as that boolean
        // prints: a reader that wrote the value out would find the id in it.
        expect(directoryIdNameStands({
          entryId: 'true',
          source: blockOf({ text: '---\nname: 咪咪\ninfo:\n  alias: true\n---\n', },),
          page: blockOf({ text: ID_ONLY_PAGE, },),
          archives: [],
        },),).toBe(false,);
      },
    },),

    it({
      name: 'FALLS where the id holds no Latin letter and no alias is the id itself, since an alias with '
        + 'no Latin letter spells nothing of it, and STANDS where an alias is that id character for '
        + 'character (ledger B131)',
      fn: async () => {
        /**
         Source naming the person in Han, with a Han alias.
         */
        const source = blockOf({ text: '---\nname: 猫童\ninfo:\n  alias: 猫童\n---\n', },);

        /**
         Page that offers no English rendering to stand on.
         */
        const page = blockOf({ text: '---\nname: nothing\n---\n', },);
        expect([
          '，。',
          '2024',
          '猫咪',
        ].map(function stands(entryId,): boolean {
          return directoryIdNameStands({
            entryId,
            source,
            page,
            archives: [],
          },);
        },),).toEqual([
          false,
          false,
          false,
        ],);
        expect(directoryIdNameStands({
          entryId: '猫童',
          source,
          page,
          archives: [],
        },),).toBe(true,);
        // A source with no name: the pinyin walk has no character to refuse
        // an id of no letters at, so only its own guard does.
        expect(directoryIdNameStands({
          entryId: '，。',
          source: blockOf({ text: '---\ninfo:\n  alias: Whiskers\n---\n', },),
          page,
          archives: [],
        },),).toBe(false,);
      },
    },),

    it({
      name: 'NAMES the id when the visible name is the id itself, and not when it is another handle',
      fn: async () => {
        expect(namesDirectoryId({
          metadata: blockOf({ text: '---\nname: maotong\n---\n', },),
          entryId: 'maotong',
        },),).toBe(true,);
        expect(namesDirectoryId({
          metadata: blockOf({ text: '---\nname: 狗狗\n---\n', },),
          entryId: 'maotong',
        },),).toBe(false,);
      },
    },),
  ],
},);
