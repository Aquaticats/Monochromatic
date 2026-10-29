/**
 Tests the verdict on whether a built command may run in a process, given that
 process's environment and the command's name.

 The environment is handed in as an argument, so every case builds its own with
 invented variables and none reads the environment of the process running the
 suite, which holds real keys in some worktrees and none in others. The as-built
 cases are in `corpus-run/task-runner-guard-built.unit.test.ts`.

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
  STARTED_BY_VARIABLE,
  taskRunnerVerdict,
} from '../dist/final/node/index.mjs';

/**
 The whole text the refusal carries for a process holding one key variable,
 with nothing else found in its environment.
 */
const ONE_KEY_REFUSAL = 'will not run in a process that holds provider keys '
  + '(1 variable whose name ends in _API_KEY has a value) unless that process names this command as the one it '
  + 'means to start. '
  + 'The command\'s task does that: mise run //package/translation-repair/run:spend-report '
  + '(the task builds the package first). '
  + 'To start the built file without building, as when a pass is running and nothing may rebuild, '
  + 'set TRANSLATION_REPAIR_STARTED_BY=spend-report for that one start. '
  + 'A process that holds no provider key needs neither.';

/**
 The whole refusal for a process holding one key whose marker the refusal
 does not repeat, since it has not the shape of a command name.
 */
const UNREPEATED_MARKER_REFUSAL = `${ONE_KEY_REFUSAL} ${STARTED_BY_VARIABLE} is set in this process, but not to the `
  + 'name of a command.';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: taskRunnerVerdict.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ALLOWS A PROCESS THAT HOLDS NO KEY, with no marker, and with a marker naming another command',
          fn: async () => {
            expect([
              {},
              { PLAIN_VARIABLE: 'nap', PATH: '/bin', },
              { TRANSLATION_REPAIR_STARTED_BY: 'purr-report', },
            ].map((env,) => taskRunnerVerdict({ env, command: 'spend-report', owner: 'run', },)),).toEqual([
              { allowed: true, },
              { allowed: true, },
              { allowed: true, },
            ],);
          },
        },),

        it({
          name: 'ALLOWS A PROCESS THAT HOLDS A KEY when the marker names the command itself, whoever set it: the '
            + 'command\'s task, the test fixture, or an operator starting the built file by hand',
          fn: async () => {
            expect(taskRunnerVerdict({
              env: { WHISKER_API_KEY: 'purr', TRANSLATION_REPAIR_STARTED_BY: 'spend-report', },
              command: 'spend-report',
              owner: 'run',
            },),).toEqual({ allowed: true, },);
          },
        },),

        it({
          name: 'REFUSES A PROCESS THAT HOLDS ONE KEY and no marker, in the whole text, which names no variable and '
            + 'no value',
          fn: async () => {
            expect(taskRunnerVerdict({
              env: { WHISKER_API_KEY: 'purr', },
              command: 'spend-report',
              owner: 'run',
            },),).toEqual({ allowed: false, says: ONE_KEY_REFUSAL, },);
          },
        },),

        it({
          name: 'NAMES THE TASK OF THE OWNER IT IS GIVEN, so a command another package owns is started by that '
            + 'package\'s task and not by a fixed one',
          fn: async () => {
            expect(taskRunnerVerdict({
              env: { WHISKER_API_KEY: 'purr', },
              command: 'spend-report',
              owner: 'tabby',
            },),).toEqual({
              allowed: false,
              says: ONE_KEY_REFUSAL.replace(
                '//package/translation-repair/run:spend-report',
                '//package/translation-repair/tabby:spend-report',
              ),
            },);
          },
        },),

        it({
          name: 'REFUSES A PROCESS THAT HOLDS SEVERAL KEYS and no marker, counting each variable once',
          fn: async () => {
            expect(taskRunnerVerdict({
              env: {
                WHISKER_API_KEY: 'purr',
                TRANSLATION_REPAIR_PAW_API_KEY: 'mew',
                CLAW_API_KEY: 'hiss',
                PLAIN_VARIABLE: 'nap',
              },
              command: 'cap-census',
              owner: 'run',
            },),).toEqual({
              allowed: false,
              says: 'will not run in a process that holds provider keys '
                + '(3 variables whose name ends in _API_KEY have a value) unless that process names this command as '
                + 'the one it means to start. '
                + 'The command\'s task does that: mise run //package/translation-repair/run:cap-census '
                + '(the task builds the package first). '
                + 'To start the built file without building, as when a pass is running and nothing may rebuild, '
                + 'set TRANSLATION_REPAIR_STARTED_BY=cap-census for that one start. '
                + 'A process that holds no provider key needs neither.',
            },);
          },
        },),

        it({
          name: 'REFUSES A PROCESS THAT HOLDS A KEY and carries the marker of another command, and names that command',
          fn: async () => {
            expect(taskRunnerVerdict({
              env: { WHISKER_API_KEY: 'purr', TRANSLATION_REPAIR_STARTED_BY: 'purr-report', },
              command: 'spend-report',
              owner: 'run',
            },),).toEqual({
              allowed: false,
              says: `${ONE_KEY_REFUSAL} ${STARTED_BY_VARIABLE} is set in this process to purr-report, `
                + 'which names another command.',
            },);
          },
        },),

        it({
          name: 'REFUSES A PROCESS THAT HOLDS A KEY and carries a marker that is no command name, without repeating it',
          fn: async () => {
            expect(taskRunnerVerdict({
              env: { WHISKER_API_KEY: 'purr', TRANSLATION_REPAIR_STARTED_BY: 'Purr Report; rm *', },
              command: 'spend-report',
              owner: 'run',
            },),).toEqual({
              allowed: false,
              says: UNREPEATED_MARKER_REFUSAL,
            },);
          },
        },),

        it({
          name: 'REPEATS A MARKER SHAPED AS A COMMAND NAME OF 64 CHARACTERS, and repeats none of 65, nor one led by a '
            + 'letter whose later character is no lower case letter, digit or hyphen',
          fn: async () => {
            /**
             A command-shaped name at the longest the refusal repeats.
             */
            const longest = `purr-${'a'.repeat(59,)}`;
            expect([
              longest,
              `${longest}a`,
              'purr_report',
            ].map((marker,) => taskRunnerVerdict({
              env: { WHISKER_API_KEY: 'purr', TRANSLATION_REPAIR_STARTED_BY: marker, },
              command: 'spend-report',
              owner: 'run',
            },)),).toEqual([
              {
                allowed: false,
                says: `${ONE_KEY_REFUSAL} ${STARTED_BY_VARIABLE} is set in this process to ${longest}, `
                  + 'which names another command.',
              },
              { allowed: false, says: UNREPEATED_MARKER_REFUSAL, },
              { allowed: false, says: UNREPEATED_MARKER_REFUSAL, },
            ],);
          },
        },),

        it({
          name: 'TREATS AN EMPTY MARKER AS NO MARKER when the process holds a key',
          fn: async () => {
            expect(taskRunnerVerdict({
              env: { WHISKER_API_KEY: 'purr', TRANSLATION_REPAIR_STARTED_BY: '', },
              command: 'spend-report',
              owner: 'run',
            },),).toEqual({ allowed: false, says: ONE_KEY_REFUSAL, },);
          },
        },),

        it({
          name: 'COUNTS NO KEY in a variable that is empty, blank or undefined, so a process with only those is allowed',
          fn: async () => {
            expect(taskRunnerVerdict({
              env: { WHISKER_API_KEY: '', CLAW_API_KEY: undefined, PAW_API_KEY: ' \t', },
              command: 'spend-report',
              owner: 'run',
            },),).toEqual({ allowed: true, },);
          },
        },),

        it({
          name: 'COUNTS ONLY THE KEYS THAT HAVE A VALUE when some key variables are empty',
          fn: async () => {
            expect(taskRunnerVerdict({
              env: { WHISKER_API_KEY: '', CLAW_API_KEY: 'hiss', PAW_API_KEY: undefined, },
              command: 'spend-report',
              owner: 'run',
            },),).toEqual({ allowed: false, says: ONE_KEY_REFUSAL, },);
          },
        },),

        it({
          name: 'COUNTS NO KEY in a name that holds _API_KEY anywhere but at its end',
          fn: async () => {
            expect(taskRunnerVerdict({
              env: {
                WHISKER_API_KEY_FILE: 'whisker.txt',
                API_KEY: 'purr',
                _API_KEY_WHISKER: 'mew',
                whisker_api_key: 'hiss',
              },
              command: 'spend-report',
              owner: 'run',
            },),).toEqual({ allowed: true, },);
          },
        },),
      ],
    },),
  ],
},);
