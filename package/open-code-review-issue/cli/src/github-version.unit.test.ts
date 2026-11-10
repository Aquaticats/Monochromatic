import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  GitHubCliVersionError,
  parseGitHubCliVersion,
} from '../dist/final/node/index.mjs';

/**
 Asserts that version output is refused as a version error.
 
 @param stdout - Captured `gh --version` output.
 */
function expectRefused(stdout: string,): void {
  /**
   Captured version validation failure.
   */
  let caught: unknown;
  try {
    parseGitHubCliVersion({ stdout, },);
  }
  catch (error: unknown) {
    caught = error;
  }
  expect(caught,).toBeInstanceOf(GitHubCliVersionError,);
}

await describe({
  name: parseGitHubCliVersion.name,
  children: [
    it({
      name: 'accepts the audited minimum and later major versions',
      fn: async () => {
        expect(parseGitHubCliVersion({
          stdout: 'gh version 2.97.0 (2026-07-31)\nhttps://github.com/cli/cli/releases/tag/v2.97.0',
        },),).toStrictEqual({
          major: 2,
          minor: 97,
          patch: 0,
          text: '2.97.0',
        },);
        expect(parseGitHubCliVersion({ stdout: 'gh version 3.0.0', },).text,).toBe('3.0.0',);
      },
    },),
    it({
      name: 'rejects older and unparseable versions',
      fn: async () => {
        /**
         Version outputs outside supported contract.
         */
        const values = [
          'gh version 2.96.9',
          'unexpected output',
        ];
        for (const stdout of values) {
          expectRefused(stdout,);
        }
      },
    },),
    it({
      name: 'rejects a fullwidth digit in a version component',
      fn: async () => {
        expectRefused('gh version \uFF12.97.0',);
      },
    },),
    it({
      name: 'rejects a supplementary plane digit look-alike in a version component',
      fn: async () => {
        expectRefused('gh version 2.97.\u{1D7D8}',);
      },
    },),
    it({
      name: 'rejects an Arabic-Indic digit in a version component',
      fn: async () => {
        expectRefused('gh version 2.\u0669\u0667.0',);
      },
    },),
  ],
},);
