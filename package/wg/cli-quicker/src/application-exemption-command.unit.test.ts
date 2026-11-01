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
  delimiter,
  dirname,
  join,
} from 'node:path';
import { pathToFileURL, } from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  resolveApplicationExemptionCommand,
  type ApplicationExemptionCommand,
} from '../dist/final/node/application-exemption-command.mjs';

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
  readonly resolveApplicationExemptionCommand: () => Promise<ApplicationExemptionCommand>;
};

/**
 Snapshots resolver discovery variables
 so workspace and path discovery run in isolation,
 restoring the exact prior values on dispose.
 */
function isolateResolverEnvironment(): AsyncDisposable {
  const { PATH: path, } = process.env;
  process.env.PATH = '/usr/bin:/bin';
  return {
    /**
     Restores the discovery variables captured before mutation.
     */
    async [Symbol.asyncDispose](): Promise<void> {
      if (path === undefined)
        delete process.env.PATH;
      else
        process.env.PATH = path;
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
      name: 'ignores an executable named in the environment, so the variable is not a switch for the process watch',
      fn: async () => {
        /**
         Original setting restored after test.
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
         Executable the environment points at.
         */
        const command = await writeCompanionExecutable({ path: join(
          directory,
          'custom-exempt',
        ), },);
        process.env.WG_QUICKER_EXEMPT_COMMAND = command;
        expect(await resolveApplicationExemptionCommand(),).not.toEqual({ found: true, path: command, },);
      },
    },),
    it({
      name: 'does not fail on an unavailable executable named in the environment',
      fn: async () => {
        /**
         Original setting restored after test.
         */
        const original = process.env.WG_QUICKER_EXEMPT_COMMAND;
        await using cleanup = {
          /**
           Restores the setting.
           */
          async [Symbol.asyncDispose](): Promise<void> {
            if (original === undefined)
              delete process.env.WG_QUICKER_EXEMPT_COMMAND;
            else
              process.env.WG_QUICKER_EXEMPT_COMMAND = original;
          },
        };
        process.env.WG_QUICKER_EXEMPT_COMMAND = '/does-not-exist/wg-quicker-exempt';
        expect(
          JSON.stringify(await resolveApplicationExemptionCommand(),),
        ).not.toContain('/does-not-exist/',);
      },
    },),
    it({
      name: 'finds repository sibling release outside secure path',
      fn: async () => {
        /**
         Original command and search paths restored after test.
         */
        const { PATH: originalPath, } = process.env;
        await using cleanup = {
          /**
           Restores environment after workspace resolution.
           */
          async [Symbol.asyncDispose](): Promise<void> {
            if (originalPath === undefined)
              delete process.env.PATH;
            else
              process.env.PATH = originalPath;
          },
        };
        process.env.PATH = '/usr/bin:/bin';
        /**
         Workspace release companion resolved from test file package layout.
         */
        const command = await resolveApplicationExemptionCommand();
        expect(command.found && command.path.endsWith(
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
        expect(await resolver.resolveApplicationExemptionCommand(),).toEqual({ found: true, path: companion, },);
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
        expect(await resolver.resolveApplicationExemptionCommand(),).toEqual({ found: true, path: releaseCompanion, },);
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
        expect(await resolver.resolveApplicationExemptionCommand(),).toEqual({ found: true, path: companion, },);
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
        expect(await resolver.resolveApplicationExemptionCommand(),).toEqual({ found: true, path: companion, },);
      },
    },),
    it({
      name: 'never searches a caller-set path, whatever variable carries it, so the caller cannot choose the program run as root',
      fn: async () => {
        await using cleanup = isolateResolverEnvironment();
        /**
         Original variable restored after test.
         */
        const original = process.env.WG_QUICKER_CALLER_PATH;
        /**
         Disposable directory outside any workspace layout.
         */
        const root = await mkdtemp(join(scratchRoot(), 'wg-quicker-companion-caller-',),);
        await using workspaceCleanup = {
          /**
           Restores the variable and removes the disposable directory tree.
           */
          async [Symbol.asyncDispose](): Promise<void> {
            if (original === undefined)
              delete process.env.WG_QUICKER_CALLER_PATH;
            else
              process.env.WG_QUICKER_CALLER_PATH = original;
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
         Companion a caller could plant in a directory it controls.
         */
        const planted = await writeCompanionExecutable({
          path: join(
            root,
            'bin',
            'wg-quicker-exempt',
          ),
        },);
        process.env.WG_QUICKER_CALLER_PATH = join(root, 'bin',);
        process.env.PATH = '/usr/bin:/bin';
        const resolver = await loadResolverCopy({ modulePath, },);
        /**
         Outcome with the planted companion named by the old variable.
         */
        const outcome = await resolver.resolveApplicationExemptionCommand();
        expect(outcome.found,).toBe(false,);
        expect(JSON.stringify(outcome,),).not.toContain(planted,);
      },
    },),
    it({
      name: 'skips relative and empty path entries, which mean the current directory and are not root-controlled',
      fn: async () => {
        await using cleanup = isolateResolverEnvironment();
        /**
         Directory the process starts in,
         restored afterwards.
         */
        const startDirectory = process.cwd();
        /**
         Disposable directory that becomes the current directory.
         */
        const root = await mkdtemp(join(scratchRoot(), 'wg-quicker-companion-relative-',),);
        await using workspaceCleanup = {
          /**
           Restores the directory and removes the disposable tree.
           */
          async [Symbol.asyncDispose](): Promise<void> {
            process.chdir(startDirectory,);
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
        await writeCompanionExecutable({
          path: join(
            root,
            'bin',
            'wg-quicker-exempt',
          ),
        },);
        await writeCompanionExecutable({
          path: join(
            root,
            'wg-quicker-exempt',
          ),
        },);
        process.chdir(root,);
        process.env.PATH = `bin${delimiter}${delimiter}.`;
        const resolver = await loadResolverCopy({ modulePath, },);
        expect(await resolver.resolveApplicationExemptionCommand(),).toEqual({ found: false, searched: [], },);
      },
    },),
    it({
      name: 'reports every searched location, and does not throw, when every companion candidate is missing',
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
        process.env.PATH = `${join(root, 'bin',)}${delimiter}${join(root, 'sbin',)}`;
        const resolver = await loadResolverCopy({ modulePath, },);
        /**
         Outcome of the exhausted discovery chain.
         */
        const outcome = await resolver.resolveApplicationExemptionCommand();
        expect(outcome,).toEqual({
          found: false,
          searched: [join(root, 'bin', 'wg-quicker-exempt',), join(root, 'sbin', 'wg-quicker-exempt',),],
        },);
      },
    },),
  ],
},);
