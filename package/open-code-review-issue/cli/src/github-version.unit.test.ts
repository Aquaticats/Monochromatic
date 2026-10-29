import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  GitHubCliVersionError,
  parseGitHubCliVersion,
} from '../dist/final/node/index.mjs';

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
      },
    },),
  ],
},);
