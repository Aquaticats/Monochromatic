import {
  chmod,
  mkdtempDisposable,
  writeFile,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { runBoundedProcess, } from '../dist/final/node/index.mjs';

await describe({
  name: runBoundedProcess.name,
  children: [
    it({
      name: 'captures output and supplies no inherited standard input',
      fn: async () => {
        /**
         Disposable executable fixture directory.
         */
        await using directory = await mkdtempDisposable(join(tmpdir(), 'ocr-gh-process-',),);
        /**
         Executable fixture path.
         */
        const executable = join(directory.path, 'fake-gh',);
        await writeFile(
          executable,
          [
            '#!/usr/bin/env node',
            "import { readFileSync } from 'node:fs';",
            "const stdin = readFileSync(0, 'utf8');",
            "process.stdout.write(JSON.stringify({ stdin, args: process.argv.slice(2) }));",
          ].join('\n',),
          'utf8',
        );
        await chmod(executable, 0o700,);

        /**
         Captured bounded child result.
         */
        const result = await runBoundedProcess({
          file: executable,
          arguments: ['api', '--include',],
          cwd: directory.path,
        },);

        expect(JSON.parse(result.stdout,),).toStrictEqual({
          stdin: '',
          args: ['api', '--include',],
        },);
        expect(result.stderr,).toBe('',);
      },
    },),
  ],
},);
