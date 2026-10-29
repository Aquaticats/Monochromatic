import {
  chmod,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { resolveApplicationExemptionCommand, } from '../dist/final/node/application-exemption-command.mjs';

/**
 Sentinel proving resolver threw before assertion.
 */
const COMMAND_DID_NOT_FAIL: unique symbol = Symbol('companion resolver did not fail',);

await describe({
  name: resolveApplicationExemptionCommand.name,
  concurrency: 1,
  children: [
    it({
      name: 'uses exact configured executable',
      fn: async () => {
        /**
         Original explicit companion setting restored after test.
         */
        const original = process.env.WG_QUICKER_EXEMPT_COMMAND;
        /**
         Disposable executable directory.
         */
        const directory = await mkdtemp(join(
          scratchRoot(),
          'wg-quicker-exempt-command-',
        ),);
        await using cleanup = {
          /**
           Restores environment and removes executable.
           */
          async [Symbol.asyncDispose](): Promise<void> {
            if (original === undefined)
              delete process.env.WG_QUICKER_EXEMPT_COMMAND;
            else
              process.env.WG_QUICKER_EXEMPT_COMMAND = original;
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
         Explicit executable fixture.
         */
        const command = join(
          directory,
          'custom-exempt',
        );
        await writeFile(
          command,
          '#!/usr/bin/env node\n',
        );
        await chmod(
          command,
          0o700,
        );
        process.env.WG_QUICKER_EXEMPT_COMMAND = command;
        expect(await resolveApplicationExemptionCommand(),).toBe(command,);
      },
    },),
    it({
      name: 'rejects unavailable configured executable',
      fn: async () => {
        /**
         Original explicit companion setting restored after test.
         */
        const original = process.env.WG_QUICKER_EXEMPT_COMMAND;
        await using cleanup = {
          /**
           Restores explicit companion setting.
           */
          async [Symbol.asyncDispose](): Promise<void> {
            if (original === undefined)
              delete process.env.WG_QUICKER_EXEMPT_COMMAND;
            else
              process.env.WG_QUICKER_EXEMPT_COMMAND = original;
          },
        };
        process.env.WG_QUICKER_EXEMPT_COMMAND = '/does-not-exist/wg-quicker-exempt';
        /**
         Domain error captured from exact-path validation.
         */
        let caught: unknown = COMMAND_DID_NOT_FAIL;
        try {
          await resolveApplicationExemptionCommand();
        }
        catch (error) {
          caught = error;
        }
        expect(String(caught,),).toContain('Configured wg-quicker-exempt executable is unavailable',);
      },
    },),
    it({
      name: 'resolves configured bare name through captured caller path',
      fn: async () => {
        /**
         Original command and caller path restored after test.
         */
        const {
          WG_QUICKER_CALLER_PATH: originalCallerPath,
          WG_QUICKER_EXEMPT_COMMAND: originalCommand,
        } = process.env;
        /**
         Disposable caller path directory.
         */
        const directory = await mkdtemp(join(
          scratchRoot(),
          'wg-quicker-caller-path-',
        ),);
        await using cleanup = {
          /**
           Restores environment and removes executable.
           */
          async [Symbol.asyncDispose](): Promise<void> {
            if (originalCallerPath === undefined)
              delete process.env.WG_QUICKER_CALLER_PATH;
            else
              process.env.WG_QUICKER_CALLER_PATH = originalCallerPath;
            if (originalCommand === undefined)
              delete process.env.WG_QUICKER_EXEMPT_COMMAND;
            else
              process.env.WG_QUICKER_EXEMPT_COMMAND = originalCommand;
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
         Bare-name executable fixture.
         */
        const command = join(
          directory,
          'caller-exempt',
        );
        await writeFile(
          command,
          '#!/usr/bin/env node\n',
        );
        await chmod(
          command,
          0o700,
        );
        process.env.WG_QUICKER_CALLER_PATH = directory;
        process.env.WG_QUICKER_EXEMPT_COMMAND = 'caller-exempt';
        expect(await resolveApplicationExemptionCommand(),).toBe(command,);
      },
    },),
    it({
      name: 'finds repository sibling release outside secure path',
      fn: async () => {
        /**
         Original command and search paths restored after test.
         */
        const {
          PATH: originalPath,
          WG_QUICKER_CALLER_PATH: originalCallerPath,
          WG_QUICKER_EXEMPT_COMMAND: originalCommand,
        } = process.env;
        await using cleanup = {
          /**
           Restores environment after workspace resolution.
           */
          async [Symbol.asyncDispose](): Promise<void> {
            if (originalPath === undefined)
              delete process.env.PATH;
            else
              process.env.PATH = originalPath;
            if (originalCallerPath === undefined)
              delete process.env.WG_QUICKER_CALLER_PATH;
            else
              process.env.WG_QUICKER_CALLER_PATH = originalCallerPath;
            if (originalCommand === undefined)
              delete process.env.WG_QUICKER_EXEMPT_COMMAND;
            else
              process.env.WG_QUICKER_EXEMPT_COMMAND = originalCommand;
          },
        };
        delete process.env.WG_QUICKER_EXEMPT_COMMAND;
        process.env.PATH = '/usr/bin:/bin';
        process.env.WG_QUICKER_CALLER_PATH = '';
        /**
         Workspace release companion resolved from test file package layout.
         */
        const command = await resolveApplicationExemptionCommand();
        expect(command.endsWith(
          '/package/wg/cli-quicker-exempt/target/release/wg-quicker-exempt',
        ),).toBe(true,);
      },
    },),
  ],
},);
