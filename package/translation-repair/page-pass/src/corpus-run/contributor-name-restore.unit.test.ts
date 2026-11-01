/**
 Guards class sixty-seven (2026-09-19): a contributor's name in a section
 heading was translated word for word while the archive renders the same
 person's signature as the handle the human translator knew, and one lane
 even rewrote the archive's handle in the signature into a transliteration. A name the original signs with is a person; the page renders it
 one way everywhere, the archive's way when the archive carries the
 signature, else the page's own signature rendering. Cat-themed invention
 throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { restoreContributorNames, } from '../../dist/final/node/index.mjs';
import { pair, } from './title-reference.test-fixture.ts';

/**
 A section the archive carries: the heading names the signer, the archive
 knows the signer's handle.
 */
const CARRIED = pair({
  sliceIndex: 0,
  source: '### 其三：猫猫\n\n它睡了。\n\n> <p style="text-align: end;">——猫猫, 2024 年 12 月 17 日</p>',
  target: '### Maomao\n\nIt sleeps.\n\n> <p style="text-align: end;">——Maomao, December 17, 2024</p>',
});

/**
 A section the archive never translated: the page's own signature rendering
 is the name.
 */
const UNCARRIED = pair({
  sliceIndex: 1,
  source: '### 其十：锦猫\n\n它醒了。\n\n<p style="text-align: end;">——锦猫, 2025 年 2 月 10 日</p>',
  target: '',
});

/**
 A signer the original never heads a section with, signed twice, whom the
 archive never rendered (class eighty-three).
 */
const SIGNED_TWICE = pair({
  sliceIndex: 2,
  source: '它唱了。\n\n<p style="text-align: end;">—— 云猫【白日梦】《七彩梦境》</p>\n\n它又唱了。\n\n<p style="text-align: end;">—— 云猫【白日梦】《午后猫语》</p>',
  target: '',
});

/**
 Two sections in one slice: the first heading names the signer, the second
 names no one.
 */
const TWO_SECTIONS = pair({
  sliceIndex: 3,
  source: '### 其三：猫猫\n\n它睡了。\n\n> <p style="text-align: end;">——猫猫, 2024 年 12 月 17 日</p>\n\n'
    + '### 其四：小猫\n\n它跑了。',
  target: '### Maomao\n\nIt sleeps.\n\n> <p style="text-align: end;">——Maomao, December 17, 2024</p>\n\n'
    + '### Kitten\n\nIt runs.',
});

/**
 A section whose archive signs with a handle that holds a colon.
 */
const COLON_HANDLE = pair({
  sliceIndex: 4,
  source: '### 其三：猫猫\n\n它睡了。\n\n> <p style="text-align: end;">——猫猫, 2024 年 12 月 17 日</p>',
  target: '### Mao: Cat\n\nIt sleeps.\n\n> <p style="text-align: end;">——Mao: Cat, December 17, 2024</p>',
});

/**
 Finding the pass writes when it restores the archive's handle into the
 heading `### Three: Cat Cat` of slice 0.
 */
const HEADING_RESTORED = 'contributor-name-restored (slice 0: "### Three: Cat Cat" to "### Three: Maomao" in a heading; '
  + 'the archive\'s signature rendering)';

/**
 Finding the pass writes when it restores the archive's handle into the
 signature of slice 0 that the page signed `Cat Cat`.
 */
const SIGNATURE_RESTORED =
  'contributor-name-restored (slice 0: "> <p style="text-align: end;">—Cat Cat, December 17, 2024</p>" to '
    + '"> <p style="text-align: end;">—Maomao, December 17, 2024</p>" in a signature; '
    + 'the archive\'s signature rendering)';

await describe({
  name: 'a contributor name is rendered one way across headings and signatures (class sixty-seven)',
  children: [
    it({
      name: 'RESTORES the archive\'s handle into a heading translated word for word and into a respelt signature',
      fn: async () => {
        const restored = restoreContributorNames({
          slices: [CARRIED,],
          replacements: [{
            sliceIndex: 0,
            replacementText: '### Three: Cat Cat\n\nIt sleeps.\n\n> <p style="text-align: end;">—Mao Mao, December 17, 2024</p>',
          },],
        },);
        expect(restored.replacements[0]?.replacementText,)
          .toBe('### Three: Maomao\n\nIt sleeps.\n\n> <p style="text-align: end;">—Maomao, December 17, 2024</p>',);
        expect(restored.restored.length,).toBe(1,);
        expect(restored.findings.length,).toBe(2,);
      },
    },),
    it({
      name: 'RESTORES a handle that holds a colon into its heading and its signature once, and LEAVES a heading '
        + 'and a signature that already carry it, since the heading\'s prefix ends at its first colon',
      fn: async () => {
        /**
         Pass over a page that translated the handle word for word in both
         places.
         */
        const respelt = restoreContributorNames({
          slices: [COLON_HANDLE,],
          replacements: [{
            sliceIndex: 4,
            replacementText:
              '### Three: Cat Cat\n\nIt sleeps.\n\n> <p style="text-align: end;">—Cat Cat, December 17, 2024</p>',
          },],
        },);
        /**
         Pass over a page that already writes the handle in both places.
         */
        const carried = restoreContributorNames({
          slices: [COLON_HANDLE,],
          replacements: [{
            sliceIndex: 4,
            replacementText:
              '### Three: Mao: Cat\n\nIt sleeps.\n\n> <p style="text-align: end;">—Mao: Cat, December 17, 2024</p>',
          },],
        },);
        expect([
          respelt.replacements[0]
            ?.replacementText,
          respelt.findings.length,
          carried.restored,
          carried.findings,
        ],).toEqual([
          '### Three: Mao: Cat\n\nIt sleeps.\n\n> <p style="text-align: end;">—Mao: Cat, December 17, 2024</p>',
          2,
          [],
          [],
        ],);
      },
    },),
    it({
      name: 'USES the page\'s own signature rendering where the archive never carried the section',
      fn: async () => {
        const restored = restoreContributorNames({
          slices: [UNCARRIED,],
          replacements: [{
            sliceIndex: 1,
            replacementText: '### Ten: Brocade Cat\n\nIt wakes.\n\n<p style="text-align: end;">—Jinmao, February 10, 2025</p>',
          },],
        },);
        expect(restored.replacements[0]?.replacementText,)
          .toBe('### Ten: Jinmao\n\nIt wakes.\n\n<p style="text-align: end;">—Jinmao, February 10, 2025</p>',);
      },
    },),
    it({
      name: 'LEAVES a page whose names already agree untouched',
      fn: async () => {
        const restored = restoreContributorNames({
          slices: [CARRIED,],
          replacements: [{
            sliceIndex: 0,
            replacementText: '### Three: Maomao\n\nIt dozes.\n\n> <p style="text-align: end;">——Maomao, December 17, 2024</p>',
          },],
        },);
        expect(restored.restored,).toEqual([],);
        expect(restored.findings,).toEqual([],);
      },
    },),
    it({
      name: 'ROMANISES a handle the archive never rendered when the page leaves it in Han: one capitalised '
        + 'word of pinyin in the heading and the signature (class eighty-three, 2026-09-22: one handle '
        + 'shipped in Han in a heading and signature where earlier runs wrote two different pinyin spellings). '
        + 'The pinyin only: the literal meaning the owner\'s rule adds belongs to the gloss pass, which has none '
        + 'to place when no writer glossed the handle (ledger A17)',
      fn: async () => {
        const restored = restoreContributorNames({
          slices: [UNCARRIED,],
          replacements: [{
            sliceIndex: 1,
            replacementText: '### Ten: 锦猫\n\nIt wakes.\n\n<p style="text-align: end;">— 锦猫, February 10, 2025</p>',
          },],
        },);
        expect(restored.replacements[0]?.replacementText,)
          .toBe('### Ten: Jinmao\n\nIt wakes.\n\n<p style="text-align: end;">— Jinmao, February 10, 2025</p>',);
        expect(restored.findings.length,).toBe(2,);
        expect(restored.findings.every(function namesPinyin(finding,): boolean {
          return finding.includes('the pinyin reading of the original\'s handle',);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'LEAVES a heading carrying the rendering with its literal meaning in parentheses, and takes the '
        + 'rendering without the parenthetical as the page\'s authority (class eighty-three)',
      fn: async () => {
        const restored = restoreContributorNames({
          slices: [UNCARRIED,],
          replacements: [{
            sliceIndex: 1,
            replacementText: '### Ten: Jinmao (Brocade Cat)\n\nIt wakes.\n\n<p style="text-align: end;">—Jinmao, February 10, 2025</p>',
          },],
        },);
        expect(restored.restored,).toEqual([],);
        expect(restored.findings,).toEqual([],);
      },
    },),
    it({
      name: 'RENDERS a signer no heading names one way across the page: the second signature takes the first '
        + 'signature\'s rendering, and a signature left in Han takes the pinyin (class eighty-three: one '
        + 'signer shipped in pinyin on one song attribution and in Han on the next)',
      fn: async () => {
        const restored = restoreContributorNames({
          slices: [SIGNED_TWICE,],
          replacements: [{
            sliceIndex: 2,
            replacementText: 'It sang.\n\n<p style="text-align: end;">—— Yun Mao【Daydream】“Rainbow Dreamland”</p>\n\nIt sang again.\n\n<p style="text-align: end;">—— 云猫【Daydream】《Wu Hou Mao Yu》</p>',
          },],
        },);
        expect(restored.replacements[0]?.replacementText,)
          .toBe('It sang.\n\n<p style="text-align: end;">—— Yun Mao【Daydream】“Rainbow Dreamland”</p>\n\nIt sang again.\n\n<p style="text-align: end;">—— Yun Mao【Daydream】《Wu Hou Mao Yu》</p>',);
        expect(restored.findings.length,).toBe(1,);
      },
    },),
    it({
      name: 'LEAVES THE HEADINGS OF A SLICE WHOSE PAGE DROPPED ONE OF THEM AS THE PAGE HAS THEM: pairing the page\'s '
        + 'headings with the original\'s titles by position once wrote the signer\'s name over the next '
        + 'section\'s title (ledger B84)',
      fn: async () => {
        /**
         What the page writes for the two sections, the first heading dropped.
         */
        const dropped = {
          sliceIndex: 3,
          replacementText: 'It sleeps.\n\n> <p style="text-align: end;">——Maomao, December 17, 2024</p>\n\n'
            + '### Four: Kitten\n\nIt runs.',
        };
        const restored = restoreContributorNames({
          slices: [TWO_SECTIONS,],
          replacements: [dropped,],
        },);
        expect(restored,).toEqual({
          replacements: [dropped,],
          restored: [],
          findings: [],
        },);
      },
    },),

    it({
      name: 'WRITES the rendering alone into a heading whose original is the contributor\'s name and no more, '
        + 'dropping the prefix and colon the page put before the name',
      fn: async () => {
        /**
         The one row the page writes, as the pass leaves it.
         */
        const restoredRow = {
          sliceIndex: 0,
          replacementText: '### Maomao\n\nIt sleeps.\n\n> <p style="text-align: end;">—Maomao, December 17, 2024</p>',
        };
        expect(restoreContributorNames({
          slices: [pair({
            sliceIndex: 0,
            source: '### 猫猫\n\n它睡了。\n\n> <p style="text-align: end;">——猫猫, 2024 年 12 月 17 日</p>',
            target: '### Maomao\n\nIt sleeps.\n\n> <p style="text-align: end;">—Maomao, December 17, 2024</p>',
          },),],
          replacements: [{
            sliceIndex: 0,
            replacementText: '### Our friend: Cat Cat\n\nIt sleeps.\n\n> <p style="text-align: end;">—Cat Cat, December 17, 2024</p>',
          },],
        },),).toEqual({
          replacements: [restoredRow,],
          restored: [restoredRow,],
          findings: [
            'contributor-name-restored (slice 0: "### Our friend: Cat Cat" to "### Maomao" in a heading; '
              + 'the archive\'s signature rendering)',
            SIGNATURE_RESTORED,
          ],
        },);
      },
    },),

    it({
      name: 'LEAVES a heading whose original names no contributor as the page has it, beside a heading of the '
        + 'same slice it restores',
      fn: async () => {
        /**
         The one row the page writes, its first heading restored and its
         second as the page wrote it.
         */
        const restoredRow = {
          sliceIndex: 0,
          replacementText: '### Three: Maomao\n\nIt sleeps.\n\n### Little Cat\n\nIt runs.\n\n> <p style="text-align: end;">—Maomao, December 17, 2024</p>',
        };
        expect(restoreContributorNames({
          slices: [pair({
            sliceIndex: 0,
            source: '### 其三：猫猫\n\n它睡了。\n\n### 小猫\n\n它跑了。\n\n> <p style="text-align: end;">——猫猫, 2024 年 12 月 17 日</p>',
            target: '### Maomao\n\nIt sleeps.\n\n### Kitten\n\nIt runs.\n\n> <p style="text-align: end;">—Maomao, December 17, 2024</p>',
          },),],
          replacements: [{
            sliceIndex: 0,
            replacementText: '### Three: Cat Cat\n\nIt sleeps.\n\n### Little Cat\n\nIt runs.\n\n> <p style="text-align: end;">—Maomao, December 17, 2024</p>',
          },],
        },),).toEqual({
          replacements: [restoredRow,],
          restored: [restoredRow,],
          findings: [HEADING_RESTORED,],
        },);
      },
    },),

    it({
      name: 'LEAVES a slice no lane replaced as the archive has it, though its heading names the signer word '
        + 'for word, and restores the replaced slice beside it',
      fn: async () => {
        /**
         A section only the archive carries, headed word for word and signed
         with the handle.
         */
        const archiveOnly = pair({
          sliceIndex: 1,
          source: '### 其四：猫猫\n\n它醒了。\n\n> <p style="text-align: end;">——猫猫, 2024 年 12 月 18 日</p>',
          target: '### Four: Cat Cat\n\nIt wakes.\n\n> <p style="text-align: end;">——Maomao, December 18, 2024</p>',
        },);
        /**
         The one row the page writes, restored.
         */
        const restoredRow = {
          sliceIndex: 0,
          replacementText: '### Three: Maomao\n\nIt sleeps.\n\n> <p style="text-align: end;">—Maomao, December 17, 2024</p>',
        };
        expect(restoreContributorNames({
          slices: [
            CARRIED,
            archiveOnly,
          ],
          replacements: [{
            sliceIndex: 0,
            replacementText: '### Three: Cat Cat\n\nIt sleeps.\n\n> <p style="text-align: end;">—Cat Cat, December 17, 2024</p>',
          },],
        },),).toEqual({
          replacements: [restoredRow,],
          restored: [restoredRow,],
          findings: [
            HEADING_RESTORED,
            SIGNATURE_RESTORED,
          ],
        },);
      },
    },),

    it({
      name: 'WRITES THE PINYIN OVER A PAGE SIGNATURE THAT SHOWS A READER NOTHING, and into the heading naming the '
        + 'signer: a signature of a zero-width space or a Hangul filler is no rendering the page could repeat, and '
        + 'taken for one it wrote the same nothing over the heading\'s name',
      fn: async () => {
        expect([
          '\u{200B}',
          '\u{3164}',
        ].map(function restoredOver(name,): ReturnType<typeof restoreContributorNames> {
          return restoreContributorNames({
            slices: [UNCARRIED,],
            replacements: [{
              sliceIndex: 1,
              replacementText: `### Ten: Brocade Cat\n\nIt wakes.\n\n<p style="text-align: end;">—${name}, February 10, 2025</p>`,
            },],
          },);
        },),).toEqual([
          '\u{200B}',
          '\u{3164}',
        ].map(function pinyinWritten(name,): ReturnType<typeof restoreContributorNames> {
          /**
           The one row the page writes, its heading and signature restored.
           */
          const restoredRow = {
            sliceIndex: 1,
            replacementText: '### Ten: Jinmao\n\nIt wakes.\n\n<p style="text-align: end;">—Jinmao, February 10, 2025</p>',
          };
          return {
            replacements: [restoredRow,],
            restored: [restoredRow,],
            findings: [
              'contributor-name-restored (slice 1: "### Ten: Brocade Cat" to "### Ten: Jinmao" in a heading; '
                + 'the pinyin reading of the original\'s handle)',
              `contributor-name-restored (slice 1: "<p style="text-align: end;">—${name}, February 10, 2025</p>" to `
                + '"<p style="text-align: end;">—Jinmao, February 10, 2025</p>" in a signature; '
                + 'the pinyin reading of the original\'s handle)',
            ],
          };
        },),);
      },
    },),
  ],
},);
