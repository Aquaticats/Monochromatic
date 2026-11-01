import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  buildBranchHeadArguments,
  buildCompareArguments,
  buildTreeIdsRequest,
  movedForward,
  parseBranchComparison,
  parseBranchHead,
  parseChangedTreePaths,
} from './branch-api.ts';
import {
  compareFileLimit,
  comparisonStatuses,
} from './constants.ts';
import { CrateReleaseError } from './error.ts';

/**
 Two full object names.
 */
const base = 'a'.repeat(40);
const head = 'b'.repeat(40);

/**
 Run a call that must throw and return what it threw.

 @param action - call under test
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
 Build the answer of the compare endpoint.

 @param status - the status the endpoint reports
 @param files - the listed files
 @returns the JSON text
 */
function comparisonBody({
  status,
  files,
}: {
  readonly status: string;
  readonly files: readonly Readonly<Record<string, unknown>>[];
}): string {
  return JSON.stringify({ status, files });
}

/**
 Build a list of distinct listed files.

 @param count - how many files
 @returns the files
 */
function listedFiles(count: number): readonly { readonly filename: string }[] {
  return Array.from({ length: count }, function fileAt(_value, index) {
    return { filename: `docs/file-${String(index)}.md` };
  });
}

await describe({
  name: '',
  children: [
    describe({
      name: buildBranchHeadArguments.name,
      children: [
        it({
          name: 'reads the reference of the branch under heads',
          fn: async () => {
            expect(buildBranchHeadArguments({ repo: 'o/r', branch: 'release/0.6' })).toEqual([
              'api',
              'repos/o/r/git/ref/heads/release/0.6',
            ]);
          },
        }),
        it({
          name: 'refuses a repository or a branch that could change the endpoint',
          fn: async () => {
            expect(thrownBy(() => buildBranchHeadArguments({ repo: 'o', branch: 'main' }))).toBeInstanceOf(CrateReleaseError);
            expect(thrownBy(() => buildBranchHeadArguments({ repo: 'o/r', branch: '../x' }))).toBeInstanceOf(CrateReleaseError);
            expect(thrownBy(() => buildBranchHeadArguments({ repo: 'o/r', branch: '-main' }))).toBeInstanceOf(CrateReleaseError);
          },
        }),
      ],
    }),
    describe({
      name: parseBranchHead.name,
      children: [
        it({
          name: 'reads the commit the reference points to',
          fn: async () => {
            expect(
              parseBranchHead(JSON.stringify({ object: { type: 'commit', sha: head } })),
            ).toEqual({
              found: true,
              value: head,
            });
          },
        }),
        ...[
          ['text that is not JSON', 'not json'],
          ['an answer without an object', '{}'],
          ['a reference to a tag object', JSON.stringify({ object: { type: 'tag', sha: head } })],
          ['a name that is not a full object name', JSON.stringify({ object: { type: 'commit', sha: 'abc' } })],
          ['a name that is not text', JSON.stringify({ object: { type: 'commit', sha: 5 } })],
        ].map(function missingCase([label = '', body = '']) {
          return it({
            name: `reports no head for ${label}`,
            fn: async () => {
              expect(parseBranchHead(body).found).toBe(false);
            },
          });
        }),
      ],
    }),
    describe({
      name: buildCompareArguments.name,
      children: [
        it({
          name: 'compares the base with the head by their full names',
          fn: async () => {
            expect(buildCompareArguments({ repo: 'o/r', base, head })).toEqual([
              'api',
              `repos/o/r/compare/${base}...${head}`,
            ]);
          },
        }),
        it({
          name: 'refuses a commit that is not a full object name, so no text can change the endpoint',
          fn: async () => {
            expect(thrownBy(() => buildCompareArguments({ repo: 'o/r', base: 'main', head }))).toBeInstanceOf(CrateReleaseError);
            expect(thrownBy(() => buildCompareArguments({ repo: 'o/r', base, head: `${head}/../x` }))).toBeInstanceOf(CrateReleaseError);
            expect(thrownBy(() => buildCompareArguments({ repo: 'x', base, head }))).toBeInstanceOf(CrateReleaseError);
          },
        }),
      ],
    }),
    describe({
      name: parseBranchComparison.name,
      children: [
        it({
          name: 'reads the status and the changed paths',
          fn: async () => {
            const result = parseBranchComparison(comparisonBody({
              status: comparisonStatuses.ahead,
              files: [{ filename: 'a/x' }, { filename: 'b/y' }],
            }));
            expect(result).toEqual({
              found: true,
              value: { status: comparisonStatuses.ahead, changedPaths: ['a/x', 'b/y'], truncated: false },
            });
          },
        }),
        it({
          name: 'lists the old path of a renamed file beside its new one',
          fn: async () => {
            const result = parseBranchComparison(comparisonBody({
              status: comparisonStatuses.ahead,
              files: [{ filename: 'new/x', previous_filename: 'old/x' }],
            }));
            expect(result.found ? result.value.changedPaths : []).toEqual(['new/x', 'old/x']);
          },
        }),
        it({
          name: 'marks a list as cut when it is as long as the endpoint ever gives, and not one file earlier',
          fn: async () => {
            const cut = parseBranchComparison(comparisonBody({
              status: comparisonStatuses.ahead,
              files: listedFiles(compareFileLimit),
            }));
            const whole = parseBranchComparison(comparisonBody({
              status: comparisonStatuses.ahead,
              files: listedFiles(compareFileLimit - 1),
            }));
            expect(cut.found ? cut.value.truncated : undefined).toBe(true);
            expect(whole.found ? whole.value.truncated : undefined).toBe(false);
          },
        }),
        ...[
          ['text that is not JSON', 'not json'],
          ['an answer that is not an object', '[]'],
          ['an answer without a status', JSON.stringify({ files: [] })],
          ['an answer without files', JSON.stringify({ status: comparisonStatuses.ahead })],
          ['a file without a name', JSON.stringify({ status: comparisonStatuses.ahead, files: [{ sha: 'x' }] })],
        ].map(function missingCase([label = '', body = '']) {
          return it({
            name: `reports no comparison for ${label}`,
            fn: async () => {
              expect(parseBranchComparison(body).found).toBe(false);
            },
          });
        }),
      ],
    }),
    describe({
      name: movedForward.name,
      children: [
        ...Object.values(comparisonStatuses).map(function forwardCase(status) {
          return it({
            name: `accepts a head that is ${status}`,
            fn: async () => {
              expect(movedForward({ status, changedPaths: [], truncated: false })).toBe(true);
            },
          });
        }),
        ...['behind', 'diverged', ''].map(function otherCase(status) {
          return it({
            name: `refuses a head that is ${JSON.stringify(status)}, whose changes a comparison would not describe`,
            fn: async () => {
              expect(movedForward({ status, changedPaths: [], truncated: false })).toBe(false);
            },
          });
        }),
      ],
    }),
    describe({
      name: buildTreeIdsRequest.name,
      children: [
        it({
          name: 'asks for the object name of each path at both commits, with every path in a variable',
          fn: async () => {
            const paths = ['package/a', 'package/x/Cargo.toml'];
            const request = JSON.parse(buildTreeIdsRequest({ repo: 'Aquaticats/Monochromatic', base, head, paths })) as {
              query: string;
              variables: Record<string, string>;
            };
            expect(request.variables).toEqual({
              owner: 'Aquaticats',
              name: 'Monochromatic',
              b0: `${base}:${paths[0]}`,
              h0: `${head}:${paths[0]}`,
              b1: `${base}:${paths[1]}`,
              h1: `${head}:${paths[1]}`,
            });
            for (const alias of ['b0', 'h0', 'b1', 'h1']) {
              expect(request.query).toContain(`$${alias}: String!`);
              expect(request.query).toContain(`${alias}: object(expression: $${alias})`);
            }
          },
        }),
        it({
          name: 'keeps a path with quotes, braces and a newline out of the query text',
          fn: async () => {
            const hostile = 'a"b}{c\nd) { evil }';
            const request = JSON.parse(buildTreeIdsRequest({ repo: 'o/r', base, head, paths: [hostile] })) as {
              query: string;
              variables: Record<string, string>;
            };
            expect(request.query).not.toContain('evil');
            expect(request.variables.b0).toBe(`${base}:${hostile}`);
          },
        }),
        it({
          name: 'refuses no path, a malformed repository and a commit that is not a full object name',
          fn: async () => {
            expect(thrownBy(() => buildTreeIdsRequest({ repo: 'o/r', base, head, paths: [] }))).toBeInstanceOf(CrateReleaseError);
            expect(thrownBy(() => buildTreeIdsRequest({ repo: 'o', base, head, paths: ['a'] }))).toBeInstanceOf(CrateReleaseError);
            expect(thrownBy(() => buildTreeIdsRequest({ repo: 'o/r', base: 'main', head, paths: ['a'] }))).toBeInstanceOf(CrateReleaseError);
            expect(thrownBy(() => buildTreeIdsRequest({ repo: 'o/r', base, head: 'x', paths: ['a'] }))).toBeInstanceOf(CrateReleaseError);
          },
        }),
      ],
    }),
    describe({
      name: parseChangedTreePaths.name,
      children: [
        it({
          name: 'picks the paths whose object names differ, in the order given',
          fn: async () => {
            const body = JSON.stringify({
              data: {
                repository: {
                  b0: { oid: 'same' },
                  h0: { oid: 'same' },
                  b1: { oid: 'one' },
                  h1: { oid: 'two' },
                  b2: { oid: 'three' },
                  h2: { oid: 'four' },
                },
              },
            });
            expect(parseChangedTreePaths({ body, paths: ['p0', 'p1', 'p2'] })).toEqual({ found: true, value: ['p1', 'p2'] });
          },
        }),
        it({
          name: 'treats a path missing at both commits as unchanged and a path missing at one as changed',
          fn: async () => {
            const body = JSON.stringify({
              data: {
                repository: {
                  b0: null,
                  h0: null,
                  b1: { oid: 'one' },
                  h1: null,
                  b2: null,
                  h2: { oid: 'two' },
                },
              },
            });
            expect(parseChangedTreePaths({ body, paths: ['p0', 'p1', 'p2'] })).toEqual({ found: true, value: ['p1', 'p2'] });
          },
        }),
        it({
          name: 'reports the errors GitHub gave instead of guessing',
          fn: async () => {
            const result = parseChangedTreePaths({
              body: JSON.stringify({ errors: [{ message: 'Resource not accessible' }] }),
              paths: ['p0'],
            });
            expect(result.found).toBe(false);
            expect(result.found ? '' : result.reason).toContain('Resource not accessible');
          },
        }),
        ...[
          ['text that is not JSON', 'not json'],
          ['an answer without a repository', '{"data":{}}'],
          ['an answer that lacks the head name of a path', JSON.stringify({ data: { repository: { b0: { oid: 'x' } } } })],
          ['a name that is not text', JSON.stringify({ data: { repository: { b0: { oid: 5 }, h0: { oid: 'x' } } } })],
        ].map(function missingCase([label = '', body = '']) {
          return it({
            name: `reports a miss for ${label}`,
            fn: async () => {
              expect(parseChangedTreePaths({ body, paths: ['p0'] }).found).toBe(false);
            },
          });
        }),
      ],
    }),
  ],
});
