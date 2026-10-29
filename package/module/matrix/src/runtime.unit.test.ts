/**
 Tests for runtime installation and execution command generation.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  runtimeExecCommand,
  runtimeInstallCommand,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'runtime',
  children: [
    //region runtimeInstallCommand

    describe({
      name: runtimeInstallCommand.name,
      children: [
        it({
          name: 'bun install uses bun.sh',
          fn: async () => {
            const cmd = runtimeInstallCommand('bun',);
            expect(cmd,).toContain('bun.sh/install',);
            expect(cmd,).toContain('curl',);
          },
        },),

        it({
          name: 'deno install uses deno.land',
          fn: async () => {
            const cmd = runtimeInstallCommand('deno',);
            expect(cmd,).toContain('deno.land/install.sh',);
            expect(cmd,).toContain('curl',);
          },
        },),

        it({
          name: 'node install reads the locked artifact from toolchain.lock.json and checks its SHA-256',
          fn: async () => {
            const cmd = runtimeInstallCommand('node',);

            expect(cmd,).toContain('cd /workspace',);
            expect(cmd,).toContain('toolchain.lock.json',);
            expect(cmd,).toContain('.tools.node.artifacts[$p].url',);
            expect(cmd,).toContain('.tools.node.artifacts[$p].sha256',);
            expect(cmd,).toContain('sha256sum --check',);
            expect(cmd,).toContain('--strip-components=1',);
            // Order: read the lock, fetch, verify, then extract.
            expect(cmd.indexOf('toolchain.lock.json',),)
              .toBeLessThan(cmd.indexOf('curl --fail',),);
            expect(cmd.indexOf('curl --fail',),)
              .toBeLessThan(cmd.indexOf('sha256sum --check',),);
            expect(cmd.indexOf('sha256sum --check',),)
              .toBeLessThan(cmd.indexOf('tar --extract',),);
            // Nothing resolves a version ad hoc.
            expect(cmd.includes('mise install',),)
              .toBe(false,);
          },
        },),

        it({
          name: 'node install extracts into $HOME/node before any exec path uses it',
          fn: async () => {
            const cmd = runtimeInstallCommand('node',);

            expect(cmd.indexOf('mkdir -p "$HOME/node"',),)
              .toBeLessThan(cmd.indexOf('tar --extract',),);
          },
        },),
      ],
    },),

    //endregion runtimeInstallCommand

    //region runtimeExecCommand

    describe({
      name: runtimeExecCommand.name,
      children: [
        it({
          name: 'bun exec uses $HOME/.bun/bin/bun run',
          fn: async () => {
            const cmd = runtimeExecCommand({
              runtime: 'bun',
              filePath: '/workspace/test.ts',
            },);
            expect(cmd,).toBe('$HOME/.bun/bin/bun run /workspace/test.ts',);
          },
        },),

        it({
          name: 'deno exec uses $HOME/.deno/bin/deno run --allow-all',
          fn: async () => {
            const cmd = runtimeExecCommand({
              runtime: 'deno',
              filePath: '/workspace/test.ts',
            },);
            expect(cmd,).toBe('$HOME/.deno/bin/deno run --allow-all /workspace/test.ts',);
          },
        },),

        it({
          name: 'preserves absolute file paths',
          fn: async () => {
            const cmd = runtimeExecCommand({
              runtime: 'bun',
              filePath: '/workspace/packages/foo/src/bar.ts',
            },);
            expect(cmd,).toContain('/workspace/packages/foo/src/bar.ts',);
          },
        },),

        it({
          name: 'node exec runs the file through the locked node binary',
          fn: async () => {
            const cmd = runtimeExecCommand({
              runtime: 'node',
              filePath: '/workspace/test.ts',
            },);

            expect(cmd,).toBe('$HOME/node/bin/node /workspace/test.ts',);
          },
        },),
      ],
    },),
    //endregion runtimeExecCommand
  ],
},);
