import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  RepositorySelectionError,
  parseRepositoryUrl,
  selectRepository,
  type BoundedProcessRunner,
} from '../dist/final/node/index.mjs';

/**
 Reports the error a repository selector is refused with.
 
 @param value - Candidate `--repo` value.
 
 @returns The thrown value, or `undefined` when the selector is accepted.
 */
function refusalOf(value: string,): unknown {
  try {
    parseRepositoryUrl(value,);
    return undefined;
  }
  catch (error: unknown) {
    return error;
  }
}

await describe({
  name: parseRepositoryUrl.name,
  children: [
    it({
      name: 'accepts only canonical GitHub HTTPS repository URL',
      fn: async () => {
        expect(parseRepositoryUrl('https://github.com/Aquaticat/issues-api',),).toStrictEqual({
          owner: 'Aquaticat',
          name: 'issues-api',
          url: 'https://github.com/Aquaticat/issues-api',
        },);
      },
    },),
    it({
      name: 'rejects shorthand suffixes credentials and non-GitHub hosts',
      fn: async () => {
        /**
         Non-canonical repository selectors.
         */
        const values = [
          'Aquaticat/issues-api',
          'https://github.com/Aquaticat/issues-api/',
          'https://github.com/Aquaticat/issues-api.git',
          'https://github.com/Aquaticat/issues-api?tab=issues',
          'https://user@github.com/Aquaticat/issues-api',
          'https://example.com/Aquaticat/issues-api',
        ];
        for (const value of values) {
          expect(refusalOf(value,),).toBeInstanceOf(RepositorySelectionError,);
        }
      },
    },),
    it({
      name: 'rejects a non-ASCII owner character',
      fn: async () => {
        expect(refusalOf('https://github.com/Aquaticat\u00E9/issues-api',),)
          .toBeInstanceOf(RepositorySelectionError,);
      },
    },),
    it({
      name: 'rejects a supplementary plane owner character',
      fn: async () => {
        expect(refusalOf('https://github.com/Aquaticat\u{1F600}/issues-api',),)
          .toBeInstanceOf(RepositorySelectionError,);
      },
    },),
    it({
      name: 'rejects a supplementary plane repository name character',
      fn: async () => {
        expect(refusalOf('https://github.com/Aquaticat/issues\u{1F600}-api',),)
          .toBeInstanceOf(RepositorySelectionError,);
      },
    },),
    it({
      name: 'rejects a lone surrogate in the repository name',
      fn: async () => {
        expect(refusalOf('https://github.com/Aquaticat/issues\uD800-api',),)
          .toBeInstanceOf(RepositorySelectionError,);
      },
    },),
    it({
      name: 'infers origin only from exact Git worktree root',
      fn: async () => {
        /**
         Git commands observed by fake process boundary.
         */
        const commands: string[][] = [];
        /**
         Fake Git process responses for root and origin.
         */
        const runProcess: BoundedProcessRunner = async ({ arguments: commandArguments, },) => {
          commands.push([...commandArguments,],);
          return commandArguments.includes('--show-toplevel',)
            ? { stdout: '/repo', stderr: '', durationMs: 1, }
            : {
              stdout: 'git@github.com:Aquaticat/issues-api.git',
              stderr: '',
              durationMs: 1,
            };
        };

        expect(await selectRepository({ cwd: '/repo', runProcess, },),).toStrictEqual({
          owner: 'Aquaticat',
          name: 'issues-api',
          url: 'https://github.com/Aquaticat/issues-api',
        },);
        expect(commands,).toStrictEqual([
          ['rev-parse', '--show-toplevel',],
          ['remote', 'get-url', 'origin',],
        ],);

        /**
         Captured subdirectory inference failure.
         */
        let caught: unknown;
        try {
          await selectRepository({ cwd: '/repo/src', runProcess, },);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(RepositorySelectionError,);
      },
    },),
  ],
},);
