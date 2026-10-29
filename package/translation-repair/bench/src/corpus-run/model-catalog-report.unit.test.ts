/**
 Tests for the catalog report an operator reads before changing the roster.

 WHAT THESE PIN is which list each section is rendered from. The report has
 three sections built from three arrays whose members are the same shape, so
 rendering the aliases where the unlisted models belong produces a report that
 reads perfectly and tells an operator to seat an alias. Its own comparison
 says what that costs: one model would vote twice on a panel, and a single
 opinion would read as two independent confirmations.

 SEPARATE FROM `model-catalog-compare.unit.test.ts` on purpose. `await
 describe` throws, so a failing suite aborts its whole file, and a `GFP` round
 over this report must not depend on the suites above it having passed.

 Fixtures are cat-themed invention in the shape of provider ids. No corpus
 content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { formatCatalogReport, } from '../../dist/final/node/index.mjs';

/**
 Catalog id the provider stopped serving.
 */
const DEPARTED = 'hf:someone/Departed-1';

/**
 Id the provider serves that the catalog does not list, on a model of its own.
 */
const NEWCOMER = 'syn:newcomer:text';

/**
 Model behind it, which nothing seated shares.
 */
const NEWCOMER_MODEL = 'someone/Newcomer-1';

/**
 Id the provider serves that resolves onto a model already seated.
 */
const SECOND_NAME = 'syn:large:text';

/**
 Model it resolves onto, which the catalog already lists under another id.
 */
const SEATED_MODEL = 'zai-org/GLM-5.2';

await describe({
  name: formatCatalogReport.name,
  children: [
    it({
      name:
        'RENDERS EACH SECTION FROM ITS OWN LIST, which is the whole risk here: three sections carry '
          + 'the same shape of member, so a report built from the wrong array renders perfectly and '
          + 'tells an operator to seat an alias as though it were an independent voice',
      fn: async () => {
        expect(formatCatalogReport({
          comparison: {
            missing: [DEPARTED,],
            unlisted: [{
              id: NEWCOMER,
              huggingFaceId: NEWCOMER_MODEL,
            },],
            aliases: [{
              id: SECOND_NAME,
              huggingFaceId: SEATED_MODEL,
            },],
          },
        },),).toBe([
          'MISSING from the provider but still in the catalog: 1',
          `  ${DEPARTED}  <- every call on this loses a voice to a 404`,
          'UNLISTED distinct models the provider serves: 1',
          `  ${NEWCOMER}  (${NEWCOMER_MODEL})`,
          'ALIASES onto models already seated: 1',
          `  ${SECOND_NAME} -> ${SEATED_MODEL}`,
        ].join('\n',),);
      },
    },),
    it({
      name:
        'KEEPS ALL THREE HEADINGS when every list is empty, since a clean catalog is a RESULT and a '
          + 'report that printed nothing would read as a run that never checked',
      fn: async () => {
        expect(formatCatalogReport({
          comparison: {
            missing: [],
            unlisted: [],
            aliases: [],
          },
        },),).toBe([
          'MISSING from the provider but still in the catalog: 0',
          'UNLISTED distinct models the provider serves: 0',
          'ALIASES onto models already seated: 0',
        ].join('\n',),);
      },
    },),
  ],
},);
