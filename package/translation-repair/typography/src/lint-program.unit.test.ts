/**
 Tests for starting the linter program and reading how it ended.

 THE RUNS ARE REAL, of small stand-ins written into a directory each case owns
 (`lint-stand-in.test-fixture.ts`): a stopped-by-signal run, a non-zero exit
 and a missing program are what Node reports, not objects built to look like
 its reports. The one hand-built result is the one Node does not let a case
 provoke, a run that names neither an error, a signal nor a status.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  LintBinaryMissingError,
  LintBinaryRunError,
  PINNED_LINT_BINARY_VERSION,
  refuseFailedRun,
  startProgram,
} from '../dist/final/node/index.mjs';
import {
  caughtFrom,
  standIn,
  startsOf,
} from './lint-stand-in.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: startProgram.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'HANDS THE PROGRAM ITS ARGUMENTS AND ITS STANDARD INPUT and returns what it wrote, the '
            + 'passage coming back byte for byte',
          fn: async () => {
            await using owned = await scratchDir({ prefix: 'lint-program-echo-', },);
            const { program, log, } = await standIn({
              dir: owned.path,
              version: PINNED_LINT_BINARY_VERSION,
              behaviour: 'echo',
            },);

            const run = startProgram({
              binary: program,
              args: ['--fix',],
              input: 'Quoted "cat", back\\slash, \u{1F408}.\n',
              cwd: owned.path,
            },);

            expect({
              status: run.status,
              stdout: run.stdout,
              starts: await startsOf({ log, },),
            },).toEqual({
              status: 0,
              stdout: 'Quoted "cat", back\\slash, \u{1F408}.\n',
              starts: [['--fix',],],
            },);
          },
        },),

        it({
          name: 'REPORTS A PROGRAM THAT IS NOT THERE as an ENOENT error on the run, not as a throw',
          fn: async () => {
            await using owned = await scratchDir({ prefix: 'lint-program-absent-', },);
            const run = startProgram({
              binary: 'purr-lint-nobody-installed',
              args: ['--version',],
              cwd: owned.path,
            },);

            expect(run.error,).toHaveProperty(
              'code',
              'ENOENT',
            );
          },
        },),
      ],
    },),

    describe({
      name: refuseFailedRun.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'RETURNS FOR A CLEAN EXIT and RAISES THE MISSING-PROGRAM ERROR for an ENOENT run',
          fn: async () => {
            await using owned = await scratchDir({ prefix: 'lint-program-clean-', },);
            const { program, } = await standIn({
              dir: owned.path,
              version: PINNED_LINT_BINARY_VERSION,
              behaviour: 'echo',
            },);
            const clean = startProgram({
              binary: program,
              args: ['--version',],
              cwd: owned.path,
            },);
            const absent = startProgram({
              binary: 'purr-lint-nobody-installed',
              args: ['--version',],
              cwd: owned.path,
            },);

            expect(refuseFailedRun({
              binary: program,
              characters: 0,
              run: clean,
            },),).toBeUndefined();
            expect(caughtFrom({
              body: () =>
                refuseFailedRun({
                  binary: 'purr-lint-nobody-installed',
                  characters: 0,
                  run: absent,
                },),
            },),).toBeInstanceOf(LintBinaryMissingError,);
          },
        },),

        it({
          name: 'RAISES A RUN ERROR FOR A NON-ZERO STATUS, A SIGNAL AND ANOTHER SYSTEM CODE, each stated by its own '
            + 'kind and none by the program\'s words',
          fn: async () => {
            await using owned = await scratchDir({ prefix: 'lint-program-failures-', },);
            const failing = await standIn({
              dir: owned.path,
              version: PINNED_LINT_BINARY_VERSION,
              behaviour: 'fail-two',
            },);
            const failed = startProgram({
              binary: failing.program,
              args: ['--fix',],
              input: 'The tabby hid the toebean.',
              cwd: owned.path,
            },);
            // A directory is a path Node finds but cannot run: EACCES.
            const unrunnable = startProgram({
              binary: owned.path,
              args: ['--fix',],
              cwd: owned.path,
            },);

            await using killedDir = await scratchDir({ prefix: 'lint-program-killed-', },);
            const killing = await standIn({
              dir: killedDir.path,
              version: PINNED_LINT_BINARY_VERSION,
              behaviour: 'kill-itself',
            },);
            const killed = startProgram({
              binary: killing.program,
              args: ['--fix',],
              input: 'The tabby hid the toebean.',
              cwd: killedDir.path,
            },);

            expect({
              status: (caughtFrom({
                body: () =>
                  refuseFailedRun({
                    binary: failing.program,
                    characters: 26,
                    run: failed,
                  },),
              },) as Error).message.includes('exited with status 2 and wrote 26 bytes',),
              signal: (caughtFrom({
                body: () =>
                  refuseFailedRun({
                    binary: killing.program,
                    characters: 26,
                    run: killed,
                  },),
              },) as Error).message.includes('was stopped by signal SIGKILL',),
              code: caughtFrom({
                body: () =>
                  refuseFailedRun({
                    binary: owned.path,
                    characters: 26,
                    run: unrunnable,
                  },),
              },) instanceof LintBinaryRunError,
            },).toEqual({
              status: true,
              signal: true,
              code: true,
            },);
          },
        },),

        it({
          name: 'REFUSES A RUN THAT NAMES NOTHING, since a finished run always names an error, a signal or a status',
          fn: async () => {
            const thrown = caughtFrom({
              body: () =>
                refuseFailedRun({
                  binary: 'purr-lint',
                  characters: 0,
                  run: {
                    pid: 0,
                    output: [],
                    stdout: '',
                    stderr: '',
                    status: null,
                    signal: null,
                  },
                },),
            },);

            expect((thrown as Error).message,).toContain('unreachable:',);
          },
        },),
      ],
    },),
  ],
},);
