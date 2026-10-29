import {
  readFile,
  stat,
} from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  runGitHubApi,
  type BoundedProcessRunner,
} from '../dist/final/node/index.mjs';

await describe({
  name: runGitHubApi.name,
  children: [
    it({
      name: 'passes JSON through a private named input file',
      fn: async () => {
        /**
         Request body observed by fake process boundary.
         */
        let observedBody = '';
        /**
         Request file mode observed while file exists.
         */
        let observedMode = 0;
        /**
         Captured process arguments.
         */
        let observedArguments: readonly string[] = [];
        /**
         Fake process boundary that inspects private request file.
         */
        const runProcess: BoundedProcessRunner = async ({ arguments: commandArguments, },) => {
          observedArguments = commandArguments;
          /**
           Input flag position in exact GitHub CLI argument vector.
           */
          const inputIndex = commandArguments.indexOf('--input',);
          /**
           Named request file following input flag.
           */
          const inputPath = commandArguments[inputIndex + 1];
          if (inputPath === undefined) {
            throw new Error('missing request input file',);
          }
          observedBody = await readFile(inputPath, 'utf8',);
          observedMode = (await stat(inputPath,)).mode & 0o777;
          return {
            stdout: 'HTTP/2.0 201 Created\nContent-Type: application/json\n\n{"number":7}',
            stderr: '',
            durationMs: 1,
          };
        };

        /**
         Parsed fake GitHub response.
         */
        const response = await runGitHubApi({
          request: {
            method: 'POST',
            endpoint: 'repos/Aquaticat/issues-api/issues',
            body: {
              title: 'Finding',
              body: 'Body',
            },
          },
          cwd: process.cwd(),
          runProcess,
        },);

        expect(response.status,).toBe(201,);
        expect(response.body,).toStrictEqual({ number: 7, },);
        expect(observedBody,).toBe('{"title":"Finding","body":"Body"}',);
        expect(observedMode,).toBe(0o600,);
        expect(observedArguments.slice(0, 7,),).toStrictEqual([
          'api',
          '--include',
          '--method',
          'POST',
          'repos/Aquaticat/issues-api/issues',
          '--input',
          observedArguments[6],
        ],);
      },
    },),
  ],
},);
