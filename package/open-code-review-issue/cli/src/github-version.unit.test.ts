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
 Reports the error version output is refused with.
 
 @param stdout - Captured `gh --version` output.
 
 @returns The thrown value, or `undefined` when the output is accepted.
 */
function refusalOf(stdout: string,): unknown {
  try {
    parseGitHubCliVersion({ stdout, },);
    return undefined;
  }
  catch (error: unknown) {
    return error;
  }
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
          expect(refusalOf(stdout,),).toBeInstanceOf(GitHubCliVersionError,);
        }
      },
    },),
    it({
      name: 'rejects a fullwidth digit in a version component',
      fn: async () => {
        expect(refusalOf('gh version \uFF12.97.0',),).toBeInstanceOf(GitHubCliVersionError,);
      },
    },),
    it({
      name: 'rejects a supplementary plane digit look-alike in a version component',
      fn: async () => {
        /**
         Refusal raised for the component holding the look-alike digit.
         */
        const refusal = refusalOf('gh version 2.97.\u{1D7D8}',);
        expect(refusal,).toBeInstanceOf(GitHubCliVersionError,);
        /**
         No astral look-alike has a numeric value, so the version floor refuses
         it too; the grammar must be what refuses it here.
         */
        expect(String(refusal,),).toContain('cannot parse',);
      },
    },),
    it({
      name: 'rejects an Arabic-Indic digit in a version component',
      fn: async () => {
        expect(refusalOf('gh version 2.\u0669\u0667.0',),)
          .toBeInstanceOf(GitHubCliVersionError,);
      },
    },),
  ],
},);
