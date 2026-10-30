import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { CrateReleaseError } from './error.ts';
import {
  parseCommitScope,
  parseCrateName,
  parseRepositoryPath,
  parseSha,
  parseSystemdValue,
  parseTaskName,
  parseTemplate,
  parseUnitName,
  parseUserAgent,
  parseWholeNumber,
} from './validate.ts';

/**
 Run a parser that must throw and return what it threw.

 @param action - parser call under test
 @returns the thrown value
 */
function thrownBy(action: () => unknown): unknown {
  try {
    action();
  }
  catch (error) {
    return error;
  }
  return undefined;
}

/**
 Build the accept and reject cases of one text parser.

 @param parse - parser under test; it receives the option name and the value
 @param accepted - values it must return unchanged
 @param rejected - values it must refuse with a `CrateReleaseError` naming the option
 @returns one `it` per value
 */
function textCases({
  parse,
  accepted,
  rejected,
}: {
  readonly parse: (input: { readonly option: string; readonly value: string }) => string;
  readonly accepted: readonly string[];
  readonly rejected: readonly string[];
}) {
  return [
    ...accepted.map(function acceptCase(value) {
      return it({
        name: `accepts ${JSON.stringify(value)}`,
        fn: async () => {
          expect(parse({ option: '--x', value })).toBe(value);
        },
      });
    }),
    ...rejected.map(function rejectCase(value) {
      return it({
        name: `rejects ${JSON.stringify(value)} and names the option`,
        fn: async () => {
          const caught = thrownBy(() => parse({ option: '--x', value }));
          expect(caught).toBeInstanceOf(CrateReleaseError);
          expect((caught as Error).message).toContain('--x');
        },
      });
    }),
  ];
}

await describe({
  name: '',
  children: [
    describe({
      name: parseCrateName.name,
      children: textCases({
        parse: parseCrateName,
        accepted: ['a', 'monochromatic-git-policy-cli', 'forbidden_strings2'],
        rejected: ['', '1abc', '-a', 'a b', 'a/b', 'a.b', 'a\nb', 'a"b', "a'b", 'a$b', 'a;b', '../a', 'é'],
      }),
    }),
    describe({
      name: parseRepositoryPath.name,
      children: textCases({
        parse: parseRepositoryPath,
        accepted: ['package/a/b', 'Cargo.toml', 'package/a b/c', 'package/x.fuzz/Cargo.toml'],
        rejected: ['', '/abs/path', '../outside', 'a/../../b', 'a/./b', './a', 'a//b', 'a/', String.raw`a\b`, 'a\0b', '..', '.'],
      }),
    }),
    describe({
      name: parseCommitScope.name,
      children: textCases({
        parse: parseCommitScope,
        accepted: ['pkg', 'git-policy/cli', 'a.b_c-d'],
        rejected: ['', 'has space', 'a(b', 'a)b', 'a:b', 'a\nb', '../x', '/abs', "a'b", 'a..b'],
      }),
    }),
    describe({
      name: parseSystemdValue.name,
      children: textCases({
        parse: parseSystemdValue,
        accepted: ['12G', '500%', 'infinity', '1.5G'],
        rejected: ['', '12 G', '12G;x', '$(x)', '-1', '12G\n', "1'2"],
      }),
    }),
    describe({
      name: parseUnitName.name,
      children: textCases({
        parse: parseUnitName,
        accepted: ['cli-git-publish-dry', 'a_b-1'],
        rejected: ['', '-a', 'a b', 'a.service', 'a/b', 'a\nb', 'a@b'],
      }),
    }),
    describe({
      name: parseTaskName.name,
      children: textCases({
        parse: parseTaskName,
        accepted: ['//package/git-policy/cli.fuzz:lock:members', 'lint', 'a:b'],
        rejected: ['', '-x', '--help', 'a b', 'a\nb', 'a\0b', 'a;b', 'a$b', "a'b"],
      }),
    }),
    describe({
      name: parseUserAgent.name,
      children: textCases({
        parse: parseUserAgent,
        accepted: ['crate-release-check', 'a b/1.0 (x)'],
        rejected: ['', 'a\nb', 'a\rb', 'a\0b', 'é', '\t'],
      }),
    }),
    describe({
      name: parseSha.name,
      children: [
        ...textCases({
          parse: parseSha,
          accepted: ['abcdef0', '0123456789abcdef0123456789abcdef01234567'],
          rejected: [
            '',
            'abcdef',
            '0123456789abcdef0123456789abcdef012345678',
            'abcdefg',
            'abc def0',
            '")) | .databaseId, ("',
            'abcdef0"',
            '../abcdef0',
            'abcdef0\n',
          ],
        }),
        it({
          name: 'lowercases an upper-case sha so it compares with the lower-case sha the API returns',
          fn: async () => {
            expect(parseSha({ option: '<sha>', value: 'ABCDEF0' })).toBe('abcdef0');
          },
        }),
      ],
    }),
    describe({
      name: parseWholeNumber.name,
      children: [
        it({
          name: 'returns the number of a positive whole number',
          fn: async () => {
            expect(parseWholeNumber({ option: '--attempts', value: '45' })).toBe(45);
          },
        }),
        ...['', '0', '-1', '1.5', '01', '1e3', ' 1', '1 ', 'abc', '99999999999'].map(function rejectCase(value) {
          return it({
            name: `rejects ${JSON.stringify(value)}`,
            fn: async () => {
              expect(thrownBy(() => parseWholeNumber({ option: '--attempts', value }))).toBeInstanceOf(CrateReleaseError);
            },
          });
        }),
      ],
    }),
    describe({
      name: parseTemplate.name,
      children: [
        it({
          name: 'accepts a template that holds every required placeholder',
          fn: async () => {
            expect(parseTemplate({
              option: '--tag-template',
              value: '{crate}-v{version}',
              required: ['{version}'],
            })).toBe('{crate}-v{version}');
          },
        }),
        it({
          name: 'rejects a template without the version placeholder, which would name every release alike',
          fn: async () => {
            const caught = thrownBy(() => parseTemplate({
              option: '--tag-template',
              value: 'release',
              required: ['{version}'],
            }));
            expect(caught).toBeInstanceOf(CrateReleaseError);
            expect((caught as Error).message).toContain('{version}');
          },
        }),
        ...['', 'a\0{version}', '{version}\n'].map(function rejectCase(value) {
          return it({
            name: `rejects ${JSON.stringify(value)}`,
            fn: async () => {
              expect(thrownBy(() => parseTemplate({ option: '--t', value, required: ['{version}'] }))).toBeInstanceOf(CrateReleaseError);
            },
          });
        }),
      ],
    }),
  ],
});
