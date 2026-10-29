/**
 Tests how a line reads past its Markdown quote markers (audit area six): the
 one definition the line structure guard and the bilingual pair bound share.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  carriesContent,
  pastQuoteMarkers,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'quote lines',
  children: [
    it({
      name: 'A LINE CARRIES CONTENT past its quote markers and whitespace, and a bare marker carries none',
      fn: async () => {
        expect(['> 猫醒了。', 'Cats nap.', '>', '> > ', '   ', '',].map(function carries(line,): boolean {
          return carriesContent({ line, },);
        },),).toEqual([true, true, false, false, false, false,],);
      },
    },),
    it({
      name: 'READS A LINE FROM ITS FIRST CONTENT CHARACTER, keeping what follows as written',
      fn: async () => {
        expect(['> > cat  naps ', 'Cats nap.', '>', '',].map(function past(line,): string {
          return pastQuoteMarkers({ line, },);
        },),).toEqual(['cat  naps ', 'Cats nap.', '', '',],);
      },
    },),
  ],
},);
