/**
 Tests that a built command of the run package that never reads the corpus pin runs past an unreadable
 `TRANSLATION_REPAIR_CORPUS_COMMIT`, as built: the variable is no refusal for a command that does not read the corpus.

 THE DEFECT (ledger B302, B381): `run-config.ts` resolved the pin in a module constant, so every command that loads
 it, whether it reads the corpus or not, exited 1 while it loaded, under Node's echo of the bundle's source line,
 before `reportingRefusals` ran. The commands that do read the pin are held to a stated refusal in the packages that
 own them, through the suite and the unreadable setting of `corpus-pin-entries.test-fixture.ts` of the adjudication
 package.

 Every child runs through `runBuiltCommand`, which removes every variable whose name ends in `_API_KEY` and every
 variable whose name starts `TRANSLATION_REPAIR_` before the case adds the scratch locations it needs and the
 unreadable commit.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runBuiltCommand, } from '@monochromatic-dev/translation-repair-adjudication/ts/child-environment.test-fixture.ts';
import { UNREADABLE_COMMIT, } from '@monochromatic-dev/translation-repair-adjudication/ts/corpus-run/corpus-pin-entries.test-fixture.ts';
import { RUN_ENTRY, } from './built-entry.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'commands that never read the corpus pin, as built',
      children: [
        it({
          name: 'RUNS PAST AN UNREADABLE CORPUS COMMIT in ledger-report, which never reads the corpus, and reports '
            + 'its empty runs directory',
          fn: async () => {
            await using scratch = await scratchDir({ prefix: 'corpus-pin-entries-', },);

            /**
             What the command wrote over an empty runs directory with the commit
             setting unreadable.
             */
            const run = await runBuiltCommand({
              command: 'ledger-report',
              packageEntry: RUN_ENTRY,
              args: [],
              env: {
                TRANSLATION_REPAIR_RUNS_DIR: scratch.path,
                TRANSLATION_REPAIR_CORPUS_COMMIT: UNREADABLE_COMMIT,
              },
            },);

            expect(run,).toEqual({
              code: 1,
              stdout: [
                `ledger-report: 0 contests under ${scratch.path}`,
                'NOTHING RECORDED. This run wrote no ledger, which is not the same as a run whose models '
                  + 'wrote nothing: every run started before candidate-ledger.ts landed has none, and so does '
                  + 'any run launched without TRANSLATION_REPAIR_RUNS_DIR set.',
                '',
              ].join('\n',),
              stderr: '',
            },);
          },
        },),
      ],
    },),
  ],
},);
