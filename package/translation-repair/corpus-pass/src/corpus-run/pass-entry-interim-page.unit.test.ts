/**
 Tests for the ladder under every stop of an entry (B1220 to B1222), driven through `shipPageOfStop` itself: the
 cases of `pass-entry-every-stop.unit.test.ts` drive it through the shipper that also writes the record and the line.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { readFile, } from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { fixedPagePath, } from '@monochromatic-dev/translation-repair-page-check';
import {
  shipPageOfStop,
  startEntryProgress,
} from '../../dist/final/node/index.mjs';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 Original with two sections.
 */
const SOURCE_DOC = '## 第一节\n\n猫猫在窗台上睡觉。\n\n## 第二节\n\n猫猫有自己的碗。\n';

/**
 Archive English of the same shape.
 */
const TARGET_DOC = '## Section one\n\nThe cat sleeps on the sill.\n\n## Section two\n\nThe cat has a bowl.\n';

/**
 A page whose front matter is no YAML, which the deterministic preparation refuses.
 */
const BROKEN_FRONT_MATTER = '---\n: [\n---\nThe cat sleeps.\n';

await describe({
  name: shipPageOfStop.name,
  children: [
    it({
      name: 'CUTS THE ARCHIVE BY THE DETERMINISTIC PREPARATION when the entry holds no slicing, and counts every slice '
        + 'as the archive\'s',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-interim-page-', },);
        const shipped = await shipPageOfStop({
          progress: startEntryProgress(),
          sourceText: SOURCE_DOC,
          targetText: TARGET_DOC,
          entryId: 'CatEntry1',
          publishDir: root.path,
          l: capturingLogger({ messages: [], },),
        },);
        expect({
          page: await readFile(
            fixedPagePath({
              publishDir: root.path,
              entryId: 'CatEntry1',
            },),
            'utf8',
          ),
          settled: shipped.settledSlices,
          source: shipped.sourceSlices,
        },).toEqual({
          page: TARGET_DOC,
          settled: 0,
          source: 0,
        },);
        expect(shipped.archiveSlices,).toBeGreaterThan(0,);
      },
    },),
    it({
      name: 'WRITES THE ARCHIVE\'S TEXT AS IT STANDS, counting no slice, when the two texts cannot be cut either',
      fn: async () => {
        await using root = await scratchDir({ prefix: 'pass-entry-interim-page-', },);
        const shipped = await shipPageOfStop({
          progress: startEntryProgress(),
          sourceText: BROKEN_FRONT_MATTER,
          targetText: BROKEN_FRONT_MATTER,
          entryId: 'CatEntry1',
          publishDir: root.path,
          l: capturingLogger({ messages: [], },),
        },);
        expect([shipped.pageText, shipped.settledSlices, shipped.archiveSlices, shipped.sourceSlices,],).toEqual([
          BROKEN_FRONT_MATTER,
          0,
          0,
          0,
        ],);
      },
    },),
  ],
},);
