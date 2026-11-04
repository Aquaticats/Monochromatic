import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  baselineSkipArguments,
  examinePatternOf,
  parseFileScopes,
  parseTimeoutAndExamine,
  passesOf,
} from './campaign-options.ts';
import {
  defaultBuildTimeoutSeconds,
  maximumBuildTimeoutSeconds,
} from './campaign-files.ts';
import { buildTimeoutFlag } from './constants.ts';
import { CargoMutantsToolingError } from './error.ts';

/**
 Timeout rules of the native wrapper, whose messages the pinned outcomes record.
 */
const nativeTimeouts = {
  defaultTimeoutSeconds: 300,
  maximumTimeoutSeconds: 3_600,
};

/**
 Scope rules of the native wrapper.
 */
const nativeScopes = {
  prefix: 'src/native/',
  suffix: '.rs',
};

/**
 Run a parser that must fail and return the message.

 @param run - the call under test
 @returns the message of the thrown error, which must be a tooling error
 */
function failureOf(run: () => unknown): string {
  try {
    run();
  } catch (error) {
    expect(error).toBeInstanceOf(CargoMutantsToolingError);
    return (error as Error).message;
  }
  return '';
}

await describe({
  name: '',
  children: [
    describe({
      name: parseTimeoutAndExamine.name,
      children: [
        it({
          name: 'defaults the timeout and leaves other options for the scope parser',
          fn: async () => {
            expect(parseTimeoutAndExamine({
              options: ['--file', 'src/native/a.rs'],
              rules: nativeTimeouts,
            })).toEqual({
              timeoutSeconds: nativeTimeouts.defaultTimeoutSeconds,
              buildTimeoutSeconds: defaultBuildTimeoutSeconds,
              examine: [],
              scopeOptions: ['--file', 'src/native/a.rs'],
            });
          },
        }),
        it({
          name: 'lets the last timeout win and collects every examine pattern',
          fn: async () => {
            expect(parseTimeoutAndExamine({
              options: ['--timeout', '10', '--examine', 'a|b', '--timeout', '60', '--examine', 'c::d'],
              rules: nativeTimeouts,
            })).toEqual({
              timeoutSeconds: 60,
              buildTimeoutSeconds: defaultBuildTimeoutSeconds,
              examine: ['a|b', 'c::d'],
              scopeOptions: [],
            });
          },
        }),
        it({
          name: 'reads --passes as 1 or 2 when the product accepts it, the last one winning, and defaults to 1',
          fn: async () => {
            /**
             Rules of a product that offers two-pass campaigns.
             */
            const rules = {
              ...nativeTimeouts,
              acceptPasses: true,
            };
            expect(parseTimeoutAndExamine({
              options: ['--file', 'src/native/a.rs'],
              rules,
            }).passes).toBe(1);
            expect(parseTimeoutAndExamine({
              options: ['--passes', '2', '--file', 'src/native/a.rs'],
              rules,
            })).toMatchObject({
              passes: 2,
              scopeOptions: ['--file', 'src/native/a.rs'],
            });
            expect(parseTimeoutAndExamine({
              options: ['--passes', '2', '--passes', '1'],
              rules,
            }).passes).toBe(1);
          },
        }),
        it({
          name: 'leaves --passes to the scope parser, which rejects it, when the product does not offer two-pass campaigns',
          fn: async () => {
            /**
             The parse for a product without the option.
             */
            const parsed = parseTimeoutAndExamine({
              options: ['--passes', '2'],
              rules: nativeTimeouts,
            });
            expect(parsed.scopeOptions).toEqual(['--passes', '2']);
            expect(parsed.passes).toBeUndefined();
          },
        }),
        ...[
          [['--passes', '3'], '--passes must be 1 or 2, got 3.'],
          [['--passes', '02'], '--passes must be 1 or 2, got 02.'],
          [['--passes', 'two'], '--passes must be 1 or 2, got two.'],
          [['--passes'], '--passes must be 1 or 2, got undefined.'],
        ].map(function badPasses([options, message]) {
          return it({
            name: `rejects ${JSON.stringify(options)} with its message`,
            fn: async () => {
              expect(failureOf(() => parseTimeoutAndExamine({
                options: options as readonly string[],
                rules: {
                  ...nativeTimeouts,
                  acceptPasses: true,
                },
              }))).toBe(message);
              expect(failureOf(() => passesOf((options as readonly string[])[1]))).toBe(message);
            },
          });
        }),
        ...[
          [['--timeout', '0'], '--timeout must be a whole number of seconds from 1 to 3600, got 0.'],
          [['--timeout', '3601'], '--timeout must be a whole number of seconds from 1 to 3600, got 3601.'],
          [['--timeout', '1.5'], '--timeout must be a whole number of seconds from 1 to 3600, got 1.5.'],
          [['--timeout', '007'], '--timeout must be a whole number of seconds from 1 to 3600, got 007.'],
          [['--timeout'], '--timeout must be a whole number of seconds from 1 to 3600, got undefined.'],
          [['--examine', 'a;b'], '--examine must be a regular expression of plain characters, got a;b.'],
          [['--examine'], '--examine must be a regular expression of plain characters, got undefined.'],
        ].map(function rejection([options, message]) {
          return it({
            name: `rejects ${JSON.stringify(options)} with its message`,
            fn: async () => {
              expect(failureOf(() => parseTimeoutAndExamine({
                options: options as readonly string[],
                rules: nativeTimeouts,
              }))).toBe(message);
            },
          });
        }),
        it({
          name: 'defaults the build timeout to the shared default and lets the last build timeout win',
          fn: async () => {
            const first = 10;
            const last = 20;
            expect(parseTimeoutAndExamine({ options: [], rules: nativeTimeouts }).buildTimeoutSeconds)
              .toBe(defaultBuildTimeoutSeconds);
            expect(parseTimeoutAndExamine({
              options: [buildTimeoutFlag, String(first), buildTimeoutFlag, String(last)],
              rules: nativeTimeouts,
            }).buildTimeoutSeconds).toBe(last);
          },
        }),
        it({
          name: 'takes the build timeout default and maximum from the product when it names them',
          fn: async () => {
            const rules = {
              ...nativeTimeouts,
              defaultBuildTimeoutSeconds: defaultBuildTimeoutSeconds + 1,
              maximumBuildTimeoutSeconds: defaultBuildTimeoutSeconds + 2,
            };
            expect(parseTimeoutAndExamine({ options: [], rules }).buildTimeoutSeconds)
              .toBe(rules.defaultBuildTimeoutSeconds);
            expect(failureOf(() => parseTimeoutAndExamine({
              options: [buildTimeoutFlag, String(rules.maximumBuildTimeoutSeconds + 1)],
              rules,
            }))).toBe(`${buildTimeoutFlag} must be a whole number of seconds from 1 to ${rules.maximumBuildTimeoutSeconds}, got ${rules.maximumBuildTimeoutSeconds + 1}.`);
          },
        }),
        ...[
          '0',
          String(maximumBuildTimeoutSeconds + 1),
          '1.5',
          '007',
          undefined,
        ].map(function buildRejection(value) {
          return it({
            name: `rejects a build timeout of ${String(value)} naming the flag and the shared maximum`,
            fn: async () => {
              expect(failureOf(() => parseTimeoutAndExamine({
                options: value === undefined ? [buildTimeoutFlag] : [buildTimeoutFlag, value],
                rules: nativeTimeouts,
              }))).toBe(`${buildTimeoutFlag} must be a whole number of seconds from 1 to ${maximumBuildTimeoutSeconds}, got ${String(value)}.`);
            },
          });
        }),
        it({
          name: 'uses the maximum the product names in its message',
          fn: async () => {
            expect(failureOf(() => parseTimeoutAndExamine({
              options: ['--timeout', '90'],
              rules: {
                defaultTimeoutSeconds: 10,
                maximumTimeoutSeconds: 60,
              },
            }))).toBe('--timeout must be a whole number of seconds from 1 to 60, got 90.');
          },
        }),
      ],
    }),
    describe({
      name: examinePatternOf.name,
      children: [
        it({
          name: 'returns a pattern of plain characters unchanged',
          fn: async () => {
            const pattern = String.raw`replace a|b::c (d) \* -x`;
            expect(examinePatternOf(pattern)).toBe(pattern);
          },
        }),
        ...['a;b', '$(id)', 'a`b`', "a'b", 'a\nb', 'a>b'].map(function badPattern(pattern) {
          return it({
            name: `rejects ${JSON.stringify(pattern)} naming the flag and the value`,
            fn: async () => {
              expect(failureOf(() => examinePatternOf(pattern)))
                .toBe(`--examine must be a regular expression of plain characters, got ${pattern}.`);
            },
          });
        }),
        it({
          name: 'rejects an absent value',
          fn: async () => {
            expect(failureOf(() => examinePatternOf())).toContain('got undefined');
          },
        }),
      ],
    }),
    describe({
      name: parseFileScopes.name,
      children: [
        it({
          name: 'accepts repeated file globs in order',
          fn: async () => {
            expect(parseFileScopes({
              options: ['--file', 'src/native/a.rs', '--file', 'src/native/b_*.rs'],
              rules: nativeScopes,
            })).toEqual(['src/native/a.rs', 'src/native/b_*.rs']);
          },
        }),
        it({
          name: 'accepts a file in a subdirectory and a glob over subdirectories',
          fn: async () => {
            expect(parseFileScopes({
              options: ['--file', 'src/native/command/command_add.rs', '--file', 'src/native/*/b_*.rs'],
              rules: nativeScopes,
            })).toEqual(['src/native/command/command_add.rs', 'src/native/*/b_*.rs']);
          },
        }),
        it({
          name: 'accepts no scopes',
          fn: async () => {
            expect(parseFileScopes({
              options: [],
              rules: nativeScopes,
            })).toEqual([]);
          },
        }),
        ...[
          [['--file', 'src/other/x.rs'], '--file must be a src/native/*.rs glob, got src/other/x.rs.'],
          [['--file', 'src/native/a b.rs'], '--file must be a src/native/*.rs glob, got src/native/a b.rs.'],
          [['--file', 'src/native/.rs'], '--file must be a src/native/*.rs glob, got src/native/.rs.'],
          [['--file', 'src/native/a.txt'], '--file must be a src/native/*.rs glob, got src/native/a.txt.'],
          [['--file', 'src/native/../x.rs'], '--file must be a src/native/*.rs glob, got src/native/../x.rs.'],
          [['--file', 'src/native/./x.rs'], '--file must be a src/native/*.rs glob, got src/native/./x.rs.'],
          [['--file', 'src/native//x.rs'], '--file must be a src/native/*.rs glob, got src/native//x.rs.'],
          [['--file', 'src/native/a/.rs'], '--file must be a src/native/*.rs glob, got src/native/a/.rs.'],
          [['--file', 'src/native/a b/x.rs'], '--file must be a src/native/*.rs glob, got src/native/a b/x.rs.'],
          [['--file'], 'Only repeated --file <glob> options are accepted.'],
          [['--bogus', '1'], 'Only repeated --file <glob> options are accepted.'],
        ].map(function rejection([options, message]) {
          return it({
            name: `rejects ${JSON.stringify(options)} with its message`,
            fn: async () => {
              expect(failureOf(() => parseFileScopes({
                options: options as readonly string[],
                rules: nativeScopes,
              }))).toBe(message);
            },
          });
        }),
        it({
          name: 'names the product directory and extension in its message',
          fn: async () => {
            expect(failureOf(() => parseFileScopes({
              options: ['--file', 'lib/x.txt'],
              rules: {
                prefix: 'src/',
                suffix: '.rs',
              },
            }))).toBe('--file must be a src/*.rs glob, got lib/x.txt.');
          },
        }),
      ],
    }),
    describe({
      name: baselineSkipArguments.name,
      children: [
        it({
          name: 'gives no arguments when the variable is unset or empty',
          fn: async () => {
            expect(baselineSkipArguments({ variable: 'V' })).toEqual([]);
            expect(baselineSkipArguments({
              variable: 'V',
              value: '',
            })).toEqual([]);
          },
        }),
        it({
          name: 'gives the nextest skip arguments for a test path',
          fn: async () => {
            expect(baselineSkipArguments({
              variable: 'V',
              value: 'module::tests::name',
            })).toEqual(['--', '--', '--skip', 'module::tests::name']);
          },
        }),
        ...['Bad', 'one', 'a::B', 'a::', '::a', 'a::1b', 'a b::c'].map(function badPath(value) {
          return it({
            name: `rejects ${JSON.stringify(value)} naming the variable`,
            fn: async () => {
              expect(failureOf(() => baselineSkipArguments({
                variable: 'NATIVE_MUTATION_BASELINE_SKIP',
                value,
              }))).toBe(`NATIVE_MUTATION_BASELINE_SKIP must be a test path, got ${value}.`);
            },
          });
        }),
      ],
    }),
  ],
});
