import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { maximumBuildTimeoutSeconds } from './campaign-files.ts';
import { parseTimeoutAndExamine } from './campaign-options.ts';
import { defaultUnitTasksMax } from './constants.ts';
import { CargoMutantsToolingError } from './error.ts';
import {
  type Environment,
  parseLauncherOptions,
} from './options.ts';

/**
 Home directory the cases use; nothing is created there.
 */
const home = '/h';

/**
 Package directory the cases use; its grandparent's parent is the default working tree.
 */
const packageDirectory = '/repo/package/mutation-test/cli-cargo-mutants';

/**
 Arguments every valid launch needs.
 */
const required = ['--task', '//p:t', '--tag-env', 'IMG_TAG', '--image-repository', 'localhost/product-test'];

/**
 Parse a command line with the fixed home and package directory.

 @param argv - arguments after the script path
 @param environment - process environment
 @returns the parsed options
 */
function parse({
  argv,
  environment = {},
}: {
  readonly argv: readonly string[];
  readonly environment?: Environment;
}): ReturnType<typeof parseLauncherOptions> {
  return parseLauncherOptions({ argv, environment, home, packageDirectory });
}

/**
 Parse a command line that must fail and return the failure.

 @param argv - arguments after the script path
 @returns the thrown value
 */
function failure(argv: readonly string[]): unknown {
  try {
    parse({ argv });
  } catch (error) {
    return error;
  }
  return undefined;
}

await describe({
  name: '',
  children: [
    describe({
      name: parseLauncherOptions.name,
      children: [
        //region Defaults

        it({
          name: 'applies every documented default',
          fn: async () => {
            const options = parse({ argv: [...required, 'w5', 'a.rs', 'b.rs'], environment: { PATH: '/bin' } });
            expect(options).toMatchObject({
              label: 'w5',
              files: ['a.rs', 'b.rs'],
              task: '//p:t',
              tagEnvironmentVariable: 'IMG_TAG',
              tagTemplate: '{label}{index}',
              extraEnvironment: [],
              scopePrefix: '',
              scopeFlag: '--file',
              root: '/repo',
              logDirectory: '/h/temp/agent',
              temporaryDirectory: '/h/temp/agent/tmp',
              unitPrefix: 'mutpar',
              logPrefix: 'mutpar',
              gateSlotPath: '/repo/package/mutation-test/cli-cargo-mutants/bin/gate-slot.ts',
              gateSlotArguments: [],
              unitMemoryMax: '12G',
              unitCpuQuota: '500%',
              unitTasksMax: defaultUnitTasksMax,
              timeoutSeconds: 900,
              attempts: 3,
              retryMarker: 'FAILED   Unmutated baseline',
              home: '/h',
              path: '/bin',
            });
            expect(options.memory).toBeUndefined();
            expect(options.cpus).toBeUndefined();
          },
        }),
        it({
          name: 'reads the image repository, and has no source tag unless one is given',
          fn: async () => {
            const options = parse({ argv: [...required, 'l', 'f.rs'] });
            expect(options.imageRepository).toBe('localhost/product-test');
            expect(options.fromTag).toBeUndefined();
          },
        }),
        it({
          name: 'reads the source tag',
          fn: async () => {
            expect(parse({ argv: [...required, '--from-tag', 'development', 'l', 'f.rs'] }).fromTag).toBe('development');
          },
        }),
        it({
          name: 'has no build timeout and no pattern unless they are given',
          fn: async () => {
            const options = parse({ argv: [...required, 'l', 'f.rs'] });
            expect(options.buildTimeoutSeconds).toBeUndefined();
            expect(options.examine).toEqual([]);
          },
        }),

        //endregion Defaults

        //region Overrides

        it({
          name: 'reads every override',
          fn: async () => {
            const options = parse({
              argv: [
                ...required,
                '--tag-template', 'x{index}{label}',
                '--setenv', 'A=1',
                '--setenv', 'B=two words',
                '--scope-prefix', 'src/',
                '--scope-flag=--only',
                '--root', '/w',
                '--log-dir', '/logs',
                '--tmp-dir', '/scratch',
                '--unit-prefix', 'up',
                '--log-prefix', 'lp',
                '--gate-slot', '/g.ts',
                '--slot-count', '4',
                '--lock-dir', '/locks',
                '--unit-memory-max', '8G',
                '--unit-cpu-quota', '300%',
                '--unit-tasks-max', '96',
                '--timeout', '60',
                '--attempts', '2',
                '--retry-marker', 'again',
                '--memory', '4g',
                '--cpus', '3',
                'lab', 'f.rs',
              ],
            });
            expect(options).toMatchObject({
              tagTemplate: 'x{index}{label}',
              extraEnvironment: ['A=1', 'B=two words'],
              scopePrefix: 'src/',
              scopeFlag: '--only',
              root: '/w',
              logDirectory: '/logs',
              temporaryDirectory: '/scratch',
              unitPrefix: 'up',
              logPrefix: 'lp',
              gateSlotPath: '/g.ts',
              gateSlotArguments: ['--slot-count', '4', '--lock-dir', '/locks'],
              unitMemoryMax: '8G',
              unitCpuQuota: '300%',
              unitTasksMax: 96,
              timeoutSeconds: 60,
              attempts: 2,
              retryMarker: 'again',
              memory: '4g',
              cpus: '3',
            });
          },
        }),
        it({
          name: 'keeps the container limits and the unit caps apart',
          fn: async () => {
            const options = parse({ argv: [...required, '--memory', '2g', '--cpus', '2', 'l', 'f.rs'] });
            expect(options.memory).toBe('2g');
            expect(options.unitMemoryMax).toBe('12G');
            expect(options.cpus).toBe('2');
            expect(options.unitCpuQuota).toBe('500%');
          },
        }),

        it({
          name: 'reads the build timeout as a number and every repeated pattern in order',
          fn: async () => {
            const buildTimeout = 600;
            const patterns = ['read_regular_file', 'a|b (c)'];
            const options = parse({
              argv: [
                ...required,
                '--build-timeout', String(buildTimeout),
                ...patterns.flatMap((pattern) => ['--examine', pattern]),
                'l', 'f.rs',
              ],
            });
            expect(options.buildTimeoutSeconds).toBe(buildTimeout);
            expect(options.examine).toEqual(patterns);
          },
        }),
        it({
          name: 'reads the passes as 1 or 2 and has none unless they are given',
          fn: async () => {
            expect(parse({ argv: [...required, 'l', 'f.rs'] }).passes).toBeUndefined();
            expect(parse({ argv: [...required, '--passes', '2', 'l', 'f.rs'] }).passes).toBe(2);
            expect(parse({ argv: [...required, '--passes', '1', 'l', 'f.rs'] }).passes).toBe(1);
          },
        }),
        it({
          name: 'rejects passes other than 1 or 2 with the message the scoped task gives',
          fn: async () => {
            expect(() => parse({ argv: [...required, '--passes', '3', 'l', 'f.rs'] })).toThrow('--passes must be 1 or 2, got 3.');
          },
        }),
        it({
          name: 'accepts a build timeout above the shared maximum, because the scoped task enforces its own maximum',
          fn: async () => {
            const options = parse({
              argv: [...required, '--build-timeout', String(maximumBuildTimeoutSeconds + 1), 'l', 'f.rs'],
            });
            expect(options.buildTimeoutSeconds).toBe(maximumBuildTimeoutSeconds + 1);
          },
        }),
        it({
          name: 'rejects a pattern with the message the scoped task gives for the same pattern',
          fn: async () => {
            const pattern = 'a;b';
            let taskMessage = '';
            try {
              parseTimeoutAndExamine({
                options: ['--examine', pattern],
                rules: { defaultTimeoutSeconds: 1, maximumTimeoutSeconds: 1 },
              });
            } catch (error) {
              taskMessage = (error as Error).message;
            }
            expect(taskMessage).not.toBe('');
            expect((failure([...required, '--examine', pattern, 'l', 'f.rs']) as Error).message).toContain(taskMessage);
          },
        }),

        //endregion Overrides

        //region Environment fallbacks

        it({
          name: 'takes the working tree from MUT_ROOT when --root is absent',
          fn: async () => {
            const options = parse({ argv: [...required, 'l', 'f.rs'], environment: { MUT_ROOT: '/from-env' } });
            expect(options.root).toBe('/from-env');
          },
        }),
        it({
          name: 'prefers --root over MUT_ROOT',
          fn: async () => {
            const options = parse({ argv: [...required, '--root', '/flag', 'l', 'f.rs'], environment: { MUT_ROOT: '/from-env' } });
            expect(options.root).toBe('/flag');
          },
        }),
        it({
          name: 'forwards the slot environment to the limiter, because the units do not inherit it',
          fn: async () => {
            const options = parse({
              argv: [...required, 'l', 'f.rs'],
              environment: { GATE_SLOT_COUNT: '3', GATE_SLOT_DIRECTORY: '/env-locks' },
            });
            expect(options.gateSlotArguments).toEqual(['--slot-count', '3', '--lock-dir', '/env-locks']);
          },
        }),

        //endregion Environment fallbacks

        //region Rejections

        ...[
          ['a missing label', [...required]],
          ['a label without files', [...required, 'l']],
          ['a missing task', ['--tag-env', 'X', '--image-repository', 'r', 'l', 'f.rs']],
          ['a missing tag variable', ['--task', 't', '--image-repository', 'r', 'l', 'f.rs']],
          ['a missing image repository', ['--task', 't', '--tag-env', 'X', 'l', 'f.rs']],
          ['an image repository that starts with a hyphen', [...required, '--image-repository=-r', 'l', 'f.rs']],
          ['an image repository with a space', [...required, '--image-repository', 'a b', 'l', 'f.rs']],
          ['an image repository with a shell metacharacter', [...required, '--image-repository', 'a;b', 'l', 'f.rs']],
          ['an image repository with an at sign', [...required, '--image-repository', 'a@sha256', 'l', 'f.rs']],
          ['an empty source tag', [...required, '--from-tag', '', 'l', 'f.rs']],
          ['a source tag that starts with a hyphen', [...required, '--from-tag=-x', 'l', 'f.rs']],
          ['a source tag with a slash', [...required, '--from-tag', 'a/b', 'l', 'f.rs']],
          ['a source tag with a colon', [...required, '--from-tag', 'a:b', 'l', 'f.rs']],
          ['a label with a slash', [...required, 'a/b', 'f.rs']],
          ['a label with a space', [...required, 'a b', 'f.rs']],
          ['a zero timeout', [...required, '--timeout', '0', 'l', 'f.rs']],
          ['a fractional timeout', [...required, '--timeout', '1.5', 'l', 'f.rs']],
          ['a zero build timeout', [...required, '--build-timeout', '0', 'l', 'f.rs']],
          ['a fractional build timeout', [...required, '--build-timeout', '1.5', 'l', 'f.rs']],
          ['a pattern with a command separator', [...required, '--examine', 'a;b', 'l', 'f.rs']],
          ['a pattern with a dollar sign', [...required, '--examine', '$(id)', 'l', 'f.rs']],
          ['a zero attempt count', [...required, '--attempts', '0', 'l', 'f.rs']],
          ['a non-numeric slot count', [...required, '--slot-count', 'many', 'l', 'f.rs']],
          ['a unit memory cap with a shell metacharacter', [...required, '--unit-memory-max', '1G;x', 'l', 'f.rs']],
          ['a unit CPU quota with a space', [...required, '--unit-cpu-quota', '5 00%', 'l', 'f.rs']],
          ['a zero unit task limit', [...required, '--unit-tasks-max', '0', 'l', 'f.rs']],
          ['a fractional unit task limit', [...required, '--unit-tasks-max', '1.5', 'l', 'f.rs']],
          ['an unlimited unit task limit', [...required, '--unit-tasks-max', 'infinity', 'l', 'f.rs']],
          ['an empty container memory limit', [...required, '--memory', '', 'l', 'f.rs']],
          ['an environment entry without a name', [...required, '--setenv', '=x', 'l', 'f.rs']],
          ['an environment entry with a bad name', [...required, '--setenv', 'A-B=1', 'l', 'f.rs']],
          ['an empty retry marker', [...required, '--retry-marker', '', 'l', 'f.rs']],
          ['a unit prefix with a dot', [...required, '--unit-prefix', 'a.b', 'l', 'f.rs']],
          ['an empty file name', [...required, 'l', '']],
        ].map(function rejection([name, argv]) {
          return it({
            name: `rejects ${String(name)}`,
            fn: async () => {
              expect(failure(argv as readonly string[])).toBeInstanceOf(CargoMutantsToolingError);
            },
          });
        }),
        ...['--build-timeout', '--examine'].map(function valueless(flag) {
          return it({
            name: `rejects ${flag} without its value with the parser error`,
            fn: async () => {
              expect(failure([...required, 'l', 'f.rs', flag])).toBeInstanceOf(TypeError);
            },
          });
        }),
        it({
          name: 'rejects an unknown option with the parser error',
          fn: async () => {
            expect(failure([...required, '--bogus', 'x', 'l', 'f.rs'])).toBeInstanceOf(TypeError);
          },
        }),
        it({
          name: 'appends the usage text to an option error',
          fn: async () => {
            expect((failure([...required]) as Error).message).toContain('Usage: mutation-parallel.ts');
          },
        }),

        //endregion Rejections
      ],
    }),
  ],
});
