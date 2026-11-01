/**
 Self-test entry point for `@monochromatic-dev/const-core`.
 
 Enumerates every per-category unit-test module beside this entry, so a new
 category test is executed by the package's `mise run test` task without
 editing this file.
 
 @module
 */

import { readdirSync, } from 'node:fs';

/**
 This entry's own file name, excluded so its dynamic imports do not recurse.
 */
const SELF_FILE = 'self.unit.test.ts';

/**
 Sibling unit-test modules, sorted so suites run in the same order every run.
 */
const SIBLING_TEST_FILES = readdirSync(import.meta.dirname,)
  .filter(function isSiblingTest(file,) {
    return file.endsWith('.unit.test.ts',) && (file !== SELF_FILE);
  },)
  .toSorted();

// Sequential on purpose: each module awaits its describe() as it is imported,
// so awaiting one import before the next keeps the suites in a single ordered
// stream instead of interleaving them.
/* oxlint-disable no-await-in-loop */
for (const file of SIBLING_TEST_FILES) {
  await import(new URL(file, import.meta.url,).href,);
}
/* oxlint-enable no-await-in-loop */
