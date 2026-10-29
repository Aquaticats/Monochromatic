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
  name: 'built wg-allowedips CLI',
  children: [
    //region Successful output

    it({
      name: 'writes only the minimized comma-separated AllowedIPs value',
      fn: async () => {
        await using directory = await makeTempDir();
        /**
         Allowed input fixture path.
         */
        const allowedPath = `${directory.path}/allowed.txt`;
        /**
         Disallowed input fixture path.
         */
        const disallowedPath = `${directory.path}/disallowed.txt`;
        await Promise.all([
          writeFile(allowedPath, '10.0.0.0/8\n2001:db8::/126\n',),
          writeFile(
            disallowedPath,
            '10.0.0.0/9\n127.0.0.0/8\n::1/128\n2001:db8::/127\n',
          ),
        ],);
        /**
         Successful built command result.
         */
        const result = await runCli({
          args: [
            '--allowed',
            allowedPath,
            '--disallowed',
            disallowedPath,
          ],
        },);
        expect(result.exitCode,).toBe(0,);
        expect(result.stdout,).toBe('10.128.0.0/9, 2001:db8::2/127\n',);
        expect(result.stdout,).not.toContain('AllowedIPs =',);
        expect(result.stderr,).toBe('',);
      },
    },),

    it({
      name: 'warns when disallowed input does not cover every loopback range',
      fn: async () => {
        await using directory = await makeTempDir();
        /**
         Allowed input fixture path.
         */
        const allowedPath = `${directory.path}/allowed.txt`;
        /**
         Partially loopback-covering disallowed input fixture path.
         */
        const disallowedPath = `${directory.path}/disallowed.txt`;
        await Promise.all([
          writeFile(allowedPath, '192.0.2.1\n',),
          writeFile(disallowedPath, '127.0.0.0/9\n::1/128\n',),
        ],);
        /**
         Successful built command result with loopback coverage warning.
         */
        const result = await runCli({
          args: [
            '--allowed',
            allowedPath,
            '--disallowed',
            disallowedPath,
          ],
        },);
        expect(result.exitCode,).toBe(0,);
        expect(result.stdout,).toBe('192.0.2.1/32\n',);
        expect(
          result.stderr.split(
            'disallowed IPs do not cover all loopback ranges; uncovered: 127.128.0.0/9',
          ).length - 1,
        ).toBe(1,);
      },
    },),

    it({
      name: 'warns once for each domain whose lookup returns ENOTFOUND',
      fn: async () => {
        await using directory = await makeTempDir();
        /**
         Allowed input containing one route and independently missing domains.
         */
        const allowedPath = `${directory.path}/allowed.txt`;
        /**
         Empty disallowed input fixture path.
         */
        const disallowedPath = `${directory.path}/disallowed.txt`;
        await Promise.all([
          writeFile(allowedPath, '192.0.2.1\nfirst.invalid\n',),
          writeFile(disallowedPath, 'second.invalid\n127.0.0.0/8\n::1/128\n',),
        ],);
        /**
         Successful built command result with warning-only DNS failures.
         */
        const result = await runCli({
          args: [
            '--allowed',
            allowedPath,
            '--disallowed',
            disallowedPath,
          ],
        },);
        expect(result.exitCode,).toBe(0,);
        expect(result.stdout,).toBe('192.0.2.1/32\n',);
        expect(
          result.stderr.split('DNS lookup returned ENOTFOUND for first.invalid; skipping domain',).length - 1,
        ).toBe(1,);
        expect(
          result.stderr.split('DNS lookup returned ENOTFOUND for second.invalid; skipping domain',).length - 1,
        ).toBe(1,);
      },
    },),

    it({
      name: 'writes no stdout for complete subtraction',
      fn: async () => {
        await using directory = await makeTempDir();
        /**
         Allowed input fixture path.
         */
        const allowedPath = `${directory.path}/allowed.txt`;
        /**
         Disallowed input fixture path.
         */
        const disallowedPath = `${directory.path}/disallowed.txt`;
        await Promise.all([
          writeFile(allowedPath, '0.0.0.0/0\n::/0\n',),
          writeFile(disallowedPath, '0.0.0.0/0\n::/0\n',),
        ],);
        /**
         Empty-result built command response.
         */
        const result = await runCli({
          args: [
            '--allowed',
            allowedPath,
            '--disallowed',
            disallowedPath,
          ],
        },);
        expect(result.exitCode,).toBe(0,);
        expect(result.stdout,).toBe('',);
        expect(result.stderr,).toBe('',);
      },
    },),

    //endregion Successful output

    //region Command contract failures

    it({
      name: 'fails when allowed option is missing',
      fn: async () => {
        /**
         Missing-allowed process result.
         */
        const result = await runCli({
          args: [
            '--disallowed',
            'disallowed.txt',
          ],
        },);
        expect(result.exitCode,).not.toBe(0,);
        expect(result.stderr,).toContain('Missing required option: --allowed',);
        expect(result.stdout,).toBe('',);
      },
    },),

    it({
      name: 'fails when disallowed option is missing',
      fn: async () => {
        /**
         Missing-disallowed process result.
         */
        const result = await runCli({
          args: [
            '--allowed',
            'allowed.txt',
          ],
        },);
        expect(result.exitCode,).not.toBe(0,);
        expect(result.stderr,).toContain('Missing required option: --disallowed',);
        expect(result.stdout,).toBe('',);
      },
    },),

    it({
      name: 'lets unknown-option failure propagate',
      fn: async () => {
        /**
         Strict parser process result.
         */
        const result = await runCli({ args: ['--other',], },);
        expect(result.exitCode,).not.toBe(0,);
        expect(result.stderr,).toContain('Unknown option',);
        expect(result.stdout,).toBe('',);
      },
    },),

    it({
      name: 'rejects positional input',
      fn: async () => {
        /**
         No-positionals parser result.
         */
        const result = await runCli({ args: ['allowed.txt',], },);
        expect(result.exitCode,).not.toBe(0,);
        expect(result.stderr,).toContain('Unexpected argument',);
        expect(result.stdout,).toBe('',);
      },
    },),

    it({
      name: 'lets unreadable-file failure propagate',
      fn: async () => {
        await using directory = await makeTempDir();
        /**
         Existing allowed input path.
         */
        const allowedPath = `${directory.path}/allowed.txt`;
        /**
         Missing disallowed input path.
         */
        const missingPath = `${directory.path}/missing.txt`;
        await writeFile(allowedPath, '192.0.2.1\n',);
        /**
         Missing-file process result.
         */
        const result = await runCli({
          args: [
            '--allowed',
            allowedPath,
            '--disallowed',
            missingPath,
          ],
        },);
        expect(result.exitCode,).not.toBe(0,);
        expect(result.stderr,).toContain('ENOENT',);
        expect(result.stderr,).toContain(missingPath,);
        expect(result.stdout,).toBe('',);
      },
    },),

    //endregion Command contract failures
  ],
},);
