/**
 Tests for the last source-coverage guard before a corpus page is published.

 A live `Toka_ls` run recorded one source-only factual paragraph as unfilled,
 then published a page omitting its death date, time, cause, age and source
 link. These fixtures keep the corpus wording out while pinning that exact
 semantic failure: known missing source content cannot become a settled page.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { unfilledPageFindings, } from '../../dist/final/node/index.mjs';

await describe({
  name: unfilledPageFindings.name,
  children: [
    it({
      name: 'RECORDS each unfilled passage as one the page carries as its source text under a note (the owner, '
        + '2026-10-23, "Source text + note"), naming the slice and the '
        + 'reason (the no-loop design of 2026-09-01; XIEPT2 was dropped after 35 minutes on 2026-09-02 '
        + 'by `UnfilledPageError` over one passage two judge rounds could not back), and records nothing '
        + 'for a complete page',
      fn: async () => {
        expect(unfilledPageFindings({
          unfilled: [
            {
              sliceIndex: 15,
              reason: 'no-candidate-backed',
              findings: [],
            },
            {
              sliceIndex: 13,
              reason: 'not-corroborated',
              findings: ['a stage finding',],
            },
          ],
        },),).toEqual([
          'source-passage-unfilled (slice 15, no-candidate-backed): the page carries the source text of this '
            + 'passage under a note that it has not been translated yet, unless the source recorded for the slice '
            + 'renders as nothing or the artifact records none, or the page had to ship without the original (defect '
            + 'page-source-unparseable), in which case the page carries nothing for it',
          'source-passage-unfilled (slice 13, not-corroborated): the page carries the source text of this '
            + 'passage under a note that it has not been translated yet, unless the source recorded for the slice '
            + 'renders as nothing or the artifact records none, or the page had to ship without the original (defect '
            + 'page-source-unparseable), in which case the page carries nothing for it',
        ],);
        expect(unfilledPageFindings({ unfilled: [], },),).toEqual([],);
      },
    },),
  ],
},);
