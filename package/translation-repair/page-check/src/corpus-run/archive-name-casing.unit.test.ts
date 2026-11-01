/**
 Guards class one hundred thirty-six (hulicaijia20, 2026-09-25): the archive
 writes a place's name title case every time it names it mid-sentence, in
 the body and again in a picture caption, and the bench wrote the same name
 with its later words in lower case on another line of the page, so the
 page carried one place under two casings. The archive's title-case form is
 restored where a shipped slice writes the name's first word as the archive
 does and a later word in another case. Cat-themed invention throughout; no
 corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import { restoreArchiveNameCasing, } from '../../dist/final/node/index.mjs';
import { pair, } from '@monochromatic-dev/translation-repair-slice-reading/ts/corpus-run/archive-slice-pair.test-fixture.ts';

/**
 Archive slices: a street named mid-sentence twice, a phrase that only ever
 opens a sentence, a phrase the archive also writes in lower case, and a
 heading.
 */
const SLICES: readonly ChunkPair[] = [
  pair({
    sliceIndex: 0,
    target: 'We napped on the Maowu Fish Street[^1] at noon. Sunny Days were long.',
  },),
  pair({
    sliceIndex: 1,
    target: '<p style="text-align: center;">(The two of us near Maowu Fish Street)</p>',
  },),
  pair({
    sliceIndex: 2,
    target: 'Sunny Days never end. We liked the Yarn Hall and the yarn hall liked us, near Yarn Hall again.',
  },),
  pair({
    sliceIndex: 3,
    target: '## Along Maowu Fish Street',
  },),
];

/**
 Replacement texts of a pass's result, in order.

 @param rows - replacement rows

 @returns Their texts

 @example
 ```ts
 textsOf({ rows: [], },); // []
 ```
 */
function textsOf(
  { rows, }: { readonly rows: readonly { readonly replacementText: string; }[]; },
): readonly string[] {
  return rows.map(function textOf(row,): string {
    return row.replacementText;
  },);
}

await describe({
  name: 'restoreArchiveNameCasing (class one hundred thirty-six)',
  children: [
    it({
      name: 'RESTORES the archive\'s title-case form where the bench lowered a later word',
      fn: async () => {
        /**
         Pass over a page that wrote the street with a lower-case "fish street".
         */
        const restored = restoreArchiveNameCasing({
          slices: SLICES,
          replacements: [
            {
              sliceIndex: 0,
              replacementText: 'We napped on the Maowu fish street[^1] around noon.',
            },
          ],
        },);
        expect([
          textsOf({ rows: restored.replacements, },),
          restored.findings,
        ],).toEqual([
          ['We napped on the Maowu Fish Street[^1] around noon.',],
          ['archive-name-casing-restored (slice 0: "Maowu fish street" to "Maowu Fish Street")',],
        ],);
      },
    },),
    it({
      name: 'LEAVES a phrase the archive only writes at a sentence start, or also writes in lower case',
      fn: async () => {
        /**
         Pass over text writing both phrases in lower case mid-sentence.
         */
        const restored = restoreArchiveNameCasing({
          slices: SLICES,
          replacements: [
            {
              sliceIndex: 2,
              replacementText: 'The Sunny days never end, and the Yarn hall liked us.',
            },
          ],
        },);
        expect([
          textsOf({ rows: restored.replacements, },),
          restored.restored,
          restored.findings,
        ],).toEqual([
          ['The Sunny days never end, and the Yarn hall liked us.',],
          [],
          [],
        ],);
      },
    },),
    it({
      name: 'LEAVES a heading, a link destination, an attribute, and a first word written otherwise',
      fn: async () => {
        /**
         Pass over lines where the name stands outside prose or opens in lower case.
         */
        const restored = restoreArchiveNameCasing({
          slices: SLICES,
          replacements: [
            {
              sliceIndex: 3,
              replacementText: '## Along Maowu fish street',
            },
            {
              sliceIndex: 1,
              replacementText: '<Photo alt="Maowu fish street" /> [map](https://cats.example/Maowu fish street) maowu fish street',
            },
          ],
        },);
        expect([
          textsOf({ rows: restored.replacements, },),
          restored.findings,
        ],).toEqual([
          [
            '## Along Maowu fish street',
            '<Photo alt="Maowu fish street" /> [map](https://cats.example/Maowu fish street) maowu fish street',
          ],
          [],
        ],);
      },
    },),
    it({
      name: 'READS the run that opens the whole page as a sentence start rather than mid-sentence, so only its '
        + 'later mid-sentence uses count toward the archive\'s form',
      fn: async () => {
        /** The name opens the page itself, so that use alone cannot count toward the two mid-sentence uses. */
        const restored = restoreArchiveNameCasing({
          slices: [
            pair({
              sliceIndex: 0,
              target: 'Mittens Paws dozed by the window, and Mittens Paws dozed again, purring softly.',
            },),
            pair({ sliceIndex: 1, target: 'Later, Mittens Paws napped once more, by the mat.', },),
            pair({ sliceIndex: 2, target: 'A quiet afternoon followed.', },),
          ],
          replacements: [
            { sliceIndex: 2, replacementText: 'We found Mittens paws near the gate.', },
          ],
        },);
        expect([
          textsOf({ rows: restored.replacements, },),
          restored.findings,
        ],).toEqual([
          ['We found Mittens Paws near the gate.',],
          ['archive-name-casing-restored (slice 2: "Mittens paws" to "Mittens Paws")',],
        ],);
      },
    },),
    it({
      name: 'READS an accented word before a name as mid-sentence, and an accented letter touching a phrase as another word (ledger B18)',
      fn: async () => {
        // With ASCII letters only, the name after `café` read as opening a
        // sentence, so neither use counted, and `hallō` or `ōmaowu` read as the
        // phrase in lower case, which vetoed the name.
        const restored = restoreArchiveNameCasing({
          slices: [
            pair({
              sliceIndex: 0,
              target: 'We napped at the café Maowu Yarn Hall[^1] at noon.',
            },),
            pair({
              sliceIndex: 1,
              target: 'The kittens met at the café Maowu Yarn Hall again.',
            },),
            pair({
              sliceIndex: 2,
              target: 'The maowu yarn hallō and the ōmaowu yarn hall are other words.',
            },),
          ],
          replacements: [
            {
              sliceIndex: 0,
              replacementText: 'We napped at the café Maowu yarn hall[^1] around noon.',
            },
          ],
        },);
        expect(textsOf({ rows: restored.replacements, },),).toEqual(['We napped at the café Maowu Yarn Hall[^1] around noon.',],);
      },
    },),
    it({
      name: 'KEEPS the other-casing veto and RESTORES a row on a page holding a letter whose lower case is longer '
        + 'than itself: İ (U+0130) lowers to two UTF-16 units, and one anywhere made the occurrence scan read the '
        + 'whole text past, so the archive\'s lower-case "yarn hall" vetoed nothing and a row naming İpek was '
        + 'never restored',
      fn: async () => {
        /**
         The page's slices with one more naming a neighbour İpek; the archive
         still writes "yarn hall" in lower case once, which must veto that name.
         */
        const dotted = [
          ...SLICES,
          pair({
            sliceIndex: 4,
            target: 'Our neighbour İpek visited.',
          },),
        ];
        const restored = restoreArchiveNameCasing({
          slices: dotted,
          replacements: [
            {
              sliceIndex: 2,
              replacementText: 'The kittens liked the Yarn hall too.',
            },
            {
              sliceIndex: 0,
              replacementText: 'We napped on the Maowu fish street[^1] with İpek.',
            },
          ],
        },);
        expect([
          textsOf({ rows: restored.replacements, },),
          restored.findings,
        ],).toEqual([
          [
            'The kittens liked the Yarn hall too.',
            'We napped on the Maowu Fish Street[^1] with İpek.',
          ],
          ['archive-name-casing-restored (slice 0: "Maowu fish street" to "Maowu Fish Street")',],
        ],);
      },
    },),
  ],
},);
