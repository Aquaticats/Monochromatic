import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  chmod,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

/**
 Built CLI artifact exercised at process privilege boundary.
 */
const CLI_BUNDLE_PATH = new URL(
  '../dist/final/node/index.mjs',
  import.meta.url,
).pathname;

await describe({
  name: 'privilege launch',
  children: [
    it({
      name: 'relaunches exact runtime and script through sudo before reading config',
      fn: async () => {
        /**
         Disposable directory containing fake sudo command and invocation record.
         */
        const directory = await mkdtemp(join(
          tmpdir(),
          'wg-quicker-privilege-',
        ),);
        await using cleanup = {
          /**
           Removes disposable fake command and record.
           */
          async [Symbol.asyncDispose](): Promise<void> {
            await rm(
              directory,
              {
                force: true,
                recursive: true,
              },
            );
          },
        };
        /**
         Path at which fake sudo records exact argument vector.
         */
        const recordPath = join(
          directory,
          'invocation.json',
        );
        /**
         Executable fake sudo found before system command in child PATH.
         */
        const sudoPath = join(
          directory,
          'sudo',
        );
        await writeFile(
          sudoPath,
          `#!/usr/bin/env node\nconst { readFileSync, writeFileSync } = require('node:fs');\nconst args = process.argv.slice(2);\nconst context = JSON.parse(readFileSync(args[4], 'utf8'));\nwriteFileSync(process.env.WG_QUICKER_TEST_SUDO_RECORD, JSON.stringify({ args, context }));\n`,
        );
        await chmod(
          sudoPath,
          0o700,
        );
        /**
         Non-root CLI process expected to delegate before opening absent config.
         */
        const child = spawn(
          process.execPath,
          [
            CLI_BUNDLE_PATH,
            'up',
            '/does-not-exist/restricted.conf',
          ],
          {
            env: {
              ...process.env,
              HOME: '/caller/home',
              IPINFO_TOKEN: 'secret-token',
              PATH: `${directory}:${process.env.PATH ?? ''}`,
              WG_ALLOWEDIPS_CACHE_DIRECTORY: '/caller/cache/allowedips',
              WG_QUICKER_EXEMPT_COMMAND: '/caller/bin/wg-quicker-exempt',
              WG_QUICKER_EXEMPT_UID: '2000',
              WG_QUICKER_OPENSNITCH_DAEMON_CONFIG: '/caller/opensnitch/default-config.json',
              WG_QUICKER_OPENSNITCH_SYSTEM_FIREWALL_CONFIG: '/caller/opensnitch/system-fw.json',
              WG_QUICKER_RUNTIME_DIRECTORY: '/caller/run/wg-quicker',
              WG_QUICKER_TEST_SUDO_RECORD: recordPath,
              XDG_CACHE_HOME: '/caller/cache',
            },
            stdio: 'ignore',
          },
        );
        await once(
          child,
          'close',
        );
        expect(child.exitCode,).toBe(0,);
        /**
         Parsed exact sudo invocation recorded by fake command.
         */
        const invocation = JSON.parse(await readFile(
          recordPath,
          'utf8',
        ),) as {
          readonly args: readonly string[];
          readonly context: unknown;
        };
        expect([
          ...invocation.args.slice(0, 4,),
          '<private-context-path>',
          ...invocation.args.slice(5,),
        ],).toEqual([
          '--',
          process.execPath,
          CLI_BUNDLE_PATH,
          '--wg-quicker-privilege-context',
          '<private-context-path>',
          'up',
          '/does-not-exist/restricted.conf',
        ],);
        expect(invocation.context,).toEqual({
          environment: {
            HOME: '/caller/home',
            IPINFO_TOKEN: 'secret-token',
            WG_ALLOWEDIPS_CACHE_DIRECTORY: '/caller/cache/allowedips',
            WG_QUICKER_CALLER_PATH: `${directory}:${process.env.PATH ?? ''}`,
            WG_QUICKER_EXEMPT_COMMAND: '/caller/bin/wg-quicker-exempt',
            WG_QUICKER_EXEMPT_UID: '2000',
            WG_QUICKER_OPENSNITCH_DAEMON_CONFIG: '/caller/opensnitch/default-config.json',
            WG_QUICKER_OPENSNITCH_SYSTEM_FIREWALL_CONFIG: '/caller/opensnitch/system-fw.json',
            WG_QUICKER_RUNTIME_DIRECTORY: '/caller/run/wg-quicker',
            XDG_CACHE_HOME: '/caller/cache',
          },
          uid: process.getuid?.(),
          version: 1,
        },);
      },
    },),
  ],
},);
