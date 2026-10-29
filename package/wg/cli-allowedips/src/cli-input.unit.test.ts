import { writeFile, } from 'node:fs/promises';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  makeTempDir,
  runCli,
} from './test-fixtures.ts';

await describe({
  name: 'built wg-allowedips input failures',
  children: [
    it({
      name: 'lets CIDR parser failure propagate before stdout',
      fn: async () => {
        await using directory = await makeTempDir();
        /**
         Invalid allowed input path.
         */
        const allowedPath = `${directory.path}/allowed.txt`;
        /**
         Empty disallowed input path.
         */
        const disallowedPath = `${directory.path}/disallowed.txt`;
        await Promise.all([
          writeFile(allowedPath, '192.0.2.1/24junk\n',),
          writeFile(disallowedPath, '',),
        ],);
        /**
         Parser-failure process result.
         */
        const result = await runCli({
          args: [
            '--allowed',
            allowedPath,
            '--disallowed',
            disallowedPath,
          ],
        },);
        expect(result.exitCode,).not.toBe(0,);
        expect(result.stderr,).toContain('not a CIDR or IP',);
        expect(result.stdout,).toBe('',);
      },
    },),

    it({
      name: 'fails before stdout when allowed input is empty',
      fn: async () => {
        await using directory = await makeTempDir();
        /**
         Empty allowed input path.
         */
        const allowedPath = `${directory.path}/allowed.txt`;
        /**
         Empty disallowed input path.
         */
        const disallowedPath = `${directory.path}/disallowed.txt`;
        await Promise.all([
          writeFile(allowedPath, '# no addresses\n',),
          writeFile(disallowedPath, '',),
        ],);
        /**
         Empty-allowed process result.
         */
        const result = await runCli({
          args: [
            '--allowed',
            allowedPath,
            '--disallowed',
            disallowedPath,
          ],
        },);
        expect(result.exitCode,).not.toBe(0,);
        expect(result.stderr,).toContain('Allowed input must contain at least one address',);
        expect(result.stdout,).toBe('',);
      },
    },),
  ],
},);
