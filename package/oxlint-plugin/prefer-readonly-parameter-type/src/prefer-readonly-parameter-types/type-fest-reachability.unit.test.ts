import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  fixtureSourceRoot,
} from '@monochromatic-dev/oxlint-plugin-test-support/ts';

import { typeFestResolvesFrom, } from '../../dist/final/node/index.mjs';

/**
 Fixture source root, in a package that does declare the dependency.
 */
const FIXTURES = fixtureSourceRoot({
  fixturePackageName: 'oxlint-no-restricted-syntax',
},);

await describe({
  name: typeFestResolvesFrom.name,
  children: [
    it({
      name: 'answers from the installed tree rather than from a manifest',
      fn: async () => {
        /* The question the removed import gate was answering by accident. Requiring the
         * file to already import `ReadonlyDeep` guaranteed the emitted name would resolve,
         * and dropping that test alone traded one broken emission for another: measured on
         * a workspace package that does not depend on `type-fest`, the inline form produced
         * `TS2307: Cannot find module 'type-fest'` where the named form had produced
         * `TS2552`. Eighteen of a hundred and fifty packages here declare it, so the
         * unreachable case is the common one rather than the corner. */
        expect(typeFestResolvesFrom({
          fileName: `${FIXTURES}/readonly-structural-store-invalid.ts`,
        },),).toBe(true,);
        /* This package is the negative case, and not a contrived one: the rule that emits
         * the projection lives here and does not depend on `type-fest`, so it can never
         * suggest it for its own sources. */
        expect(typeFestResolvesFrom({
          fileName: import.meta.filename,
        },),).toBe(false,);
        /* Outside the repository entirely, so no ancestor installed anything. */
        expect(typeFestResolvesFrom({
          fileName: '/nonexistent-root-for-reachability/src/probe.ts',
        },),).toBe(false,);
      },
    },),
  ],
},);
