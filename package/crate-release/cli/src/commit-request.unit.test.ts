import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  buildCommitRequest,
  objectNameLength,
  parseCommitResponse,
} from './commit-request.ts';
import { CrateReleaseError } from './error.ts';

/**
 A full object name.
 */
const head = 'a'.repeat(objectNameLength);

/**
 Two files, one with bytes that are not UTF-8 text.
 */
const files = [
  { path: 'package/a/Cargo.toml', contents: new TextEncoder().encode('[package]\nversion = "0.1.1"\n') },
  { path: 'package/a/Cargo.lock', contents: new Uint8Array([0, 255, 128, 10]) },
];

/**
 Input of a valid request, which the refusal tests change one field of.
 */
const valid: Parameters<typeof buildCommitRequest>[0] = {
  repo: 'o/r',
  branch: 'main',
  expectedHeadOid: head,
  headline: 'h',
  files,
};

/**
 Build a request that must throw and return what it threw.

 @param input - the request input
 @returns the thrown value
 */
function thrownBy(input: Parameters<typeof buildCommitRequest>[0]): unknown {
  try {
    buildCommitRequest(input);
  }
  catch (error) {
    return error;
  }
  return undefined;
}

await describe({
  name: '',
  children: [
    describe({
      name: buildCommitRequest.name,
      children: [
        it({
          name: 'asks for one commit on the branch, on top of the expected head, with the whole contents of each file',
          fn: async () => {
            const request = JSON.parse(buildCommitRequest({
              repo: 'Aquaticats/Monochromatic',
              branch: 'main',
              expectedHeadOid: head,
              headline: 'chore(a): release 0.1.1: n',
              files,
            })) as { query: string; variables: { input: Record<string, unknown> } };
            expect(request.query).toContain('createCommitOnBranch(input: $input)');
            expect(request.variables.input).toEqual({
              branch: { repositoryNameWithOwner: 'Aquaticats/Monochromatic', branchName: 'main' },
              message: { headline: 'chore(a): release 0.1.1: n' },
              expectedHeadOid: head,
              fileChanges: {
                additions: [
                  { path: 'package/a/Cargo.toml', contents: Buffer.from('[package]\nversion = "0.1.1"\n').toString('base64') },
                  { path: 'package/a/Cargo.lock', contents: 'AP+ACg==' },
                ],
              },
            });
          },
        }),
        it({
          name: 'keeps a headline with quotes, dollars and a hyphen start as data in the JSON, never in the query text',
          fn: async () => {
            const headline = `-x "q" $HOME \`id\` $(id); && it's`;
            const text = buildCommitRequest({ repo: 'o/r', branch: 'main', expectedHeadOid: head, headline, files });
            const request = JSON.parse(text) as { query: string; variables: { input: { message: { headline: string } } } };
            expect(request.variables.input.message.headline).toBe(headline);
            expect(request.query).not.toContain('$HOME');
            expect(request.query).not.toContain('-x');
          },
        }),
        it({
          name: 'refuses a repository that is not owner/name',
          fn: async () => {
            const caught = thrownBy({ ...valid, repo: 'o' });
            expect(caught).toBeInstanceOf(CrateReleaseError);
            expect((caught as Error).message).toContain('--repo');
          },
        }),
        it({
          name: 'refuses a branch that could be read as an option or a path escape',
          fn: async () => {
            for (const branch of ['', '-x', '/main', 'a..b', 'a b', 'main/']) {
              const caught = thrownBy({ ...valid, branch });
              expect(caught).toBeInstanceOf(CrateReleaseError);
              expect((caught as Error).message).toContain('--branch');
            }
          },
        }),
        it({
          name: 'refuses an expected head that is not a full lower-case object name, because GitHub needs the whole name',
          fn: async () => {
            for (const expectedHeadOid of ['', 'abcdef0', 'A'.repeat(objectNameLength), 'g'.repeat(objectNameLength), `${head}0`]) {
              const caught = thrownBy({ ...valid, expectedHeadOid });
              expect(caught).toBeInstanceOf(CrateReleaseError);
              expect((caught as Error).message).toContain(`full ${String(objectNameLength)}-digit`);
            }
          },
        }),
        it({
          name: 'refuses a headline that is empty or has a line break, and a commit without files',
          fn: async () => {
            expect(thrownBy({ ...valid, headline: ' ' })).toBeInstanceOf(CrateReleaseError);
            expect(thrownBy({ ...valid, headline: 'a\nb' })).toBeInstanceOf(CrateReleaseError);
            expect(thrownBy({ ...valid, files: [] })).toBeInstanceOf(CrateReleaseError);
          },
        }),
      ],
    }),
    describe({
      name: parseCommitResponse.name,
      children: [
        it({
          name: 'reads the new commit',
          fn: async () => {
            const body = '{"data":{"createCommitOnBranch":{"commit":{"oid":"abc","url":"https://github.com/o/r/commit/abc"}}}}';
            expect(parseCommitResponse(body)).toEqual({
              found: true,
              value: { oid: 'abc', url: 'https://github.com/o/r/commit/abc' },
            });
          },
        }),
        it({
          name: 'reports every error message GitHub gave, as the branch moving makes it give one',
          fn: async () => {
            const body = JSON.stringify({
              data: { createCommitOnBranch: null },
              errors: [
                { message: 'Expected branch to point to "aaa" but it did not. Pull and try again.' },
                { type: 'FORBIDDEN' },
              ],
            });
            const result = parseCommitResponse(body);
            expect(result.found).toBe(false);
            const reason = result.found ? '' : result.reason;
            expect(reason).toContain('Expected branch to point to "aaa" but it did not');
            expect(reason).toContain('FORBIDDEN');
          },
        }),
        ...[
          '',
          'not json',
          '[]',
          'null',
          '{}',
          '{"data":null}',
          '{"data":{"createCommitOnBranch":{"commit":{"oid":5,"url":"u"}}}}',
          '{"data":{"createCommitOnBranch":{"commit":{"oid":"a"}}}}',
        ].map(function malformedCase(body) {
          return it({
            name: `reports no commit for ${JSON.stringify(body)}`,
            fn: async () => {
              expect(parseCommitResponse(body).found).toBe(false);
            },
          });
        }),
      ],
    }),
  ],
});
