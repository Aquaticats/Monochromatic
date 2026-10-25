/**
 Tests for where a page goes. What each slice contributes to it is `shippable-replacements.unit.test.ts`.

 APART FROM `publish-fixed.unit.test.ts` DELIBERATELY, and the reason is a
 property of the runner rather than of the subject. A file is abandoned once
 any describe in it fails, so cheap cases sharing a file with expensive ones
 hide them exactly when something has broken: while the replacement builder
 was in that file, a break in it left every case that writes a page unrun and
 unreported, which reads as a narrow failure rather than a wide one.

 NOTHING HERE TOUCHES A DISK. The subject is a total function of an entry id,
 so a case that needed a directory would be measuring the writer.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { fixedPagePath, } from '../../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: fixedPagePath.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'NAMES THE CORPUS\'S OWN PATH for an entry, `people/<id>/page.en.md`, because the owner asked '
              + 'for the corpus directory structure replicated rather than for a flat pile named by id: a '
              + 'tree shaped this way can be diffed against the one it mirrors with nothing to translate first',
          fn: async () => {
            expect(fixedPagePath({
              publishDir: '/tmp/run/fixed',
              entryId: 'BookshopCat',
            },),).toBe('/tmp/run/fixed/people/BookshopCat/page.en.md',);
          },
        },),
      ],
    },),
  ],
},);
