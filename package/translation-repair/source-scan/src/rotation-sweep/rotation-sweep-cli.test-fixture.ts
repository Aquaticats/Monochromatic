import { argv, } from 'node:process';

import { rotationSweepMain, } from './rotation-sweep.test-fixture.ts';

//region Rotation sweep command line
// THE ENTRY OF THE `test:rotation-sweep` TASK (ledger B701). The task runs this file
// with the words after its `--`. The report is written by `rotationSweepMain`, which
// returns the exit code: 0 for a clean sweep and the stated-refusal code for one that
// found cases to fix, written as a refusal with no stack. An error out of the sweep
// itself is a fault, and escapes here with its stack.
//
// THE EXIT CODE IS ASSIGNED AT THIS FILE'S TOP, which the scan of process globals
// (`global-writes-sequenced.unit.test.ts`) allows for this file alone, by name: it is the
// process's own entry, imported by nothing, with no case beside it. `process.exit` is
// refused by the linter, and a thrown error is the stack trace this file exists to avoid.

process.exitCode = await rotationSweepMain({ args: argv.slice(2,), },);

//endregion Rotation sweep command line
