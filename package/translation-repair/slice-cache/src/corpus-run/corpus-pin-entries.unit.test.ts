/**
 Tests that every built command of the slice-cache package that reads the corpus pin reads it inside its refusal
 boundary, run as built: an unreadable `TRANSLATION_REPAIR_CORPUS_COMMIT` is a stated refusal at exit 6 naming the
 variable, with nothing on stdout.

 THE DEFECT (ledger B302, B381): `run-config.ts` resolved the pin in a module constant, so every command that loads
 it, whether it reads the corpus or not, exited 1 while it loaded, under Node's echo of the bundle's source line,
 before `reportingRefusals` ran. The suite and the unreadable setting are shared with the other packages that own
 such commands, in `corpus-pin-entries.test-fixture.ts` of the adjudication package.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { describe, } from '@monochromatic-dev/test-harness-core/ts';

import {
  type PinReader,
  pinReadersSuite,
} from '@monochromatic-dev/translation-repair-adjudication/ts/corpus-run/corpus-pin-entries.test-fixture.ts';
import { SLICE_CACHE_ENTRY, } from './built-entry.test-fixture.ts';

/**
 Each built command of this package that reads the corpus pin, with arguments its command line accepts, so the run
 reaches the pin rather than a usage refusal.
 */
const PIN_READERS: readonly PinReader[] = [
  {
    command: 'slice-census',
    packageEntry: SLICE_CACHE_ENTRY,
    args: [],
  },
];

await describe({
  name: '',
  concurrency: 1,
  children: [pinReadersSuite({ readers: PIN_READERS, },),],
},);
