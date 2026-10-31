import {
  chmod,
  copyFile,
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  dirname,
  join,
} from 'node:path';
import { pathToFileURL, } from 'node:url';

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

/**
 Absolute built resolver artifact copied into disposable layouts.
 */
const RESOLVER_ARTIFACT = join(
  import.meta.dirname,
  '..',
  'dist',
  'final',
  'node',
  'application-exemption-command.mjs',
);

/**
 Resolver module shape loaded from a disposable built-artifact copy.
 */
type ResolverModule = {
  readonly resolveApplicationExemptionCommand: () => Promise<string>;
};

/**
 Snapshots resolver discovery variables and clears the explicit command so
 workspace and path discovery run,
 restoring the exact prior values on dispose.
 */
function isolateResolverEnvironment(): AsyncDisposable {
  const {
    PATH: path,
    WG_QUICKER_CALLER_PATH: callerPath,
    WG_QUICKER_EXEMPT_COMMAND: command,
  } = process.env;
  delete process.env.WG_QUICKER_EXEMPT_COMMAND;
  process.env.PATH = '/usr/bin:/bin';
  process.env.WG_QUICKER_CALLER_PATH = '';
  return {
    /**
     Restores the three discovery variables captured before mutation.
     */
    async [Symbol.asyncDispose](): Promise<void> {
      if (path === undefined)
        delete process.env.PATH;
      else
        process.env.PATH = path;
      if (callerPath === undefined)
        delete process.env.WG_QUICKER_CALLER_PATH;
      else
        process.env.WG_QUICKER_CALLER_PATH = callerPath;
      if (command === undefined)
        delete process.env.WG_QUICKER_EXEMPT_COMMAND;
      else
        process.env.WG_QUICKER_EXEMPT_COMMAND = command;
    },
  };
}

/**
 Copies the built resolver into a disposable directory so its module
 directory selects the source,
 bundle,
 or non-workspace layout branch under test.
 */
async function installResolverCopy(
  { directory, }: { readonly directory: string; },
): Promise<string> {
  await mkdir(
    directory,
    {
      recursive: true,
    },
  );
  /**
   Copy path for the resolver module inside the disposable layout.
   */
  const modulePath = join(directory, 'application-exemption-command.mjs',);
  await copyFile(
    RESOLVER_ARTIFACT,
    modulePath,
  );
  return modulePath;
}

/**
 Loads a resolver copy so each test resolves through its own module instance.
 */
async function loadResolverCopy(
  { modulePath, }: { readonly modulePath: string; },
): Promise<ResolverModule> {
  return await import(pathToFileURL(modulePath,).href,) as ResolverModule;
}

/**
 Writes a disposable companion executable and returns its path for assertions.
 */
async function writeCompanionExecutable(
  { path, }: { readonly path: string; },
): Promise<string> {
  await mkdir(
    dirname(path,),
    {
      recursive: true,
    },
  );
  await writeFile(
    path,
    '#!/usr/bin/env node\n',
  );
  await chmod(
    path,
    0o700,
  );
  return path;
}

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
    it({
      name: 'finds repository sibling release from source module layout',
      fn: async () => {
        await using cleanup = isolateResolverEnvironment();
        /**
         Disposable workspace tree matching `package/wg` repository layout.
         */
        const root = await mkdtemp(join(scratchRoot(), 'wg-quicker-companion-source-',),);
        await using workspaceCleanup = {
          /**
           Removes the disposable workspace tree.
           */
          async [Symbol.asyncDispose](): Promise<void> {
            await rm(
              root,
              {
                force: true,
                recursive: true,
              },
            );
          },
        };
        /**
         Source-layout resolver copy beside its disposable package root.
         */
        const modulePath = await installResolverCopy({
          directory: join(
            root,
            'wg',
            'cli-quicker',
            'src',
          ),
        },);
        /**
         Release companion beside the disposable package root.
         */
        const companion = await writeCompanionExecutable({
          path: join(
            root,
            'wg',
            'cli-quicker-exempt',
            'target',
            'release',
            'wg-quicker-exempt',
          ),
        },);
        const resolver = await loadResolverCopy({ modulePath, },);
        expect(await resolver.resolveApplicationExemptionCommand(),).toBe(companion,);
      },
    },),
    it({
      name: 'prefers repository sibling release build over debug build',
      fn: async () => {
        await using cleanup = isolateResolverEnvironment();
        /**
         Disposable workspace tree matching `package/wg` repository layout.
         */
        const root = await mkdtemp(join(scratchRoot(), 'wg-quicker-companion-both-',),);
        await using workspaceCleanup = {
          /**
           Removes the disposable workspace tree.
           */
          async [Symbol.asyncDispose](): Promise<void> {
            await rm(
              root,
              {
                force: true,
                recursive: true,
              },
            );
          },
        };
        /**
         Bundle-layout resolver copy beside its disposable package root.
         */
        const modulePath = await installResolverCopy({
          directory: join(
            root,
            'wg',
            'cli-quicker',
            'dist',
            'final',
            'node',
          ),
        },);
        /**
         Release companion ahead of the debug companion in probe order.
         */
        const releaseCompanion = await writeCompanionExecutable({
          path: join(
            root,
            'wg',
            'cli-quicker-exempt',
            'target',
            'release',
            'wg-quicker-exempt',
          ),
        },);
        await writeCompanionExecutable({
          path: join(
            root,
            'wg',
            'cli-quicker-exempt',
            'target',
            'debug',
            'wg-quicker-exempt',
          ),
        },);
        const resolver = await loadResolverCopy({ modulePath, },);
        expect(await resolver.resolveApplicationExemptionCommand(),).toBe(releaseCompanion,);
      },
    },),
    it({
      name: 'uses repository sibling debug build when release build is absent',
      fn: async () => {
        await using cleanup = isolateResolverEnvironment();
        /**
         Disposable workspace tree matching `package/wg` repository layout.
         */
        const root = await mkdtemp(join(scratchRoot(), 'wg-quicker-companion-debug-',),);
        await using workspaceCleanup = {
          /**
           Removes the disposable workspace tree.
           */
          async [Symbol.asyncDispose](): Promise<void> {
            await rm(
              root,
              {
                force: true,
                recursive: true,
              },
            );
          },
        };
        /**
         Bundle-layout resolver copy beside its disposable package root.
         */
        const modulePath = await installResolverCopy({
          directory: join(
            root,
            'wg',
            'cli-quicker',
            'dist',
            'final',
            'node',
          ),
        },);
        /**
         Debug companion standing in for development before the first release build.
         */
        const companion = await writeCompanionExecutable({
          path: join(
            root,
            'wg',
            'cli-quicker-exempt',
            'target',
            'debug',
            'wg-quicker-exempt',
          ),
        },);
        const resolver = await loadResolverCopy({ modulePath, },);
        expect(await resolver.resolveApplicationExemptionCommand(),).toBe(companion,);
      },
    },),
    it({
      name: 'falls back to installed companion on privileged path',
      fn: async () => {
        await using cleanup = isolateResolverEnvironment();
        /**
         Disposable directory outside any workspace layout.
         */
        const root = await mkdtemp(join(scratchRoot(), 'wg-quicker-companion-installed-',),);
        await using workspaceCleanup = {
          /**
           Removes the disposable directory tree.
           */
          async [Symbol.asyncDispose](): Promise<void> {
            await rm(
              root,
              {
                force: true,
                recursive: true,
              },
            );
          },
        };
        /**
         Resolver copy whose module directory matches no workspace layout.
         */
        const modulePath = await installResolverCopy({
          directory: join(
            root,
            'isolated',
            'node',
          ),
        },);
        /**
         Installed companion reachable only through the privileged path.
         */
        const companion = await writeCompanionExecutable({
          path: join(
            root,
            'bin',
            'wg-quicker-exempt',
          ),
        },);
        process.env.PATH = join(root, 'bin',);
        const resolver = await loadResolverCopy({ modulePath, },);
        expect(await resolver.resolveApplicationExemptionCommand(),).toBe(companion,);
      },
    },),
    it({
      name: 'falls back to caller installation in captured caller path',
      fn: async () => {
        await using cleanup = isolateResolverEnvironment();
        /**
         Disposable directory outside any workspace layout.
         */
        const root = await mkdtemp(join(scratchRoot(), 'wg-quicker-companion-caller-',),);
        await using workspaceCleanup = {
          /**
           Removes the disposable directory tree.
           */
          async [Symbol.asyncDispose](): Promise<void> {
            await rm(
              root,
              {
                force: true,
                recursive: true,
              },
            );
          },
        };
        /**
         Resolver copy whose module directory matches no workspace layout.
         */
        const modulePath = await installResolverCopy({
          directory: join(
            root,
            'isolated',
            'node',
          ),
        },);
        /**
         Caller-installed companion reachable only through the captured caller path.
         */
        const companion = await writeCompanionExecutable({
          path: join(
            root,
            'bin',
            'wg-quicker-exempt',
          ),
        },);
        process.env.WG_QUICKER_CALLER_PATH = join(root, 'bin',);
        const resolver = await loadResolverCopy({ modulePath, },);
        expect(await resolver.resolveApplicationExemptionCommand(),).toBe(companion,);
      },
    },),
    it({
      name: 'reports build guidance when every companion candidate is missing',
      fn: async () => {
        await using cleanup = isolateResolverEnvironment();
        /**
         Disposable directory outside any workspace layout.
         */
        const root = await mkdtemp(join(scratchRoot(), 'wg-quicker-companion-missing-',),);
        await using workspaceCleanup = {
          /**
           Removes the disposable directory tree.
           */
          async [Symbol.asyncDispose](): Promise<void> {
            await rm(
              root,
              {
                force: true,
                recursive: true,
              },
            );
          },
        };
        /**
         Resolver copy whose module directory matches no workspace layout.
         */
        const modulePath = await installResolverCopy({
          directory: join(
            root,
            'isolated',
            'node',
          ),
        },);
        process.env.PATH = '';
        const resolver = await loadResolverCopy({ modulePath, },);
        /**
         Domain error captured from the exhausted discovery chain.
         */
        let caught: unknown = COMMAND_DID_NOT_FAIL;
        try {
          await resolver.resolveApplicationExemptionCommand();
        }
        catch (error) {
          caught = error;
        }
        expect(String(caught,),).toContain('wg-quicker-exempt is unavailable',);
        expect(String(caught,),).toContain('Build it with',);
      },
    },),
  ],
},);
