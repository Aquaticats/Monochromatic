/**
 User-local spawn-pi CLI symlink setup.
 
 @module
 */

import { execFile, } from 'node:child_process';
import {
  chmod,
  mkdir,
  symlink,
  unlink,
} from 'node:fs/promises';
import {
  basename,
  dirname,
  join,
  resolve,
} from 'node:path';
import { promisify, } from 'node:util';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

//region Module helpers

/**
 Module logger tagged for spawn-pi CLI setup.
 */
const l = tagged({ tag: 'pi-spawn:setup-cli', },);

/* oxlint-disable typescript/strict-void-return -- node:util.promisify intentionally accepts execFile even though execFile also returns a ChildProcess handle; this wrapper only consumes the promise result. */
/**
 Promise-returning `execFile` used for inert command probing.
 */
const execFileAsync = promisify(execFile,);
/* oxlint-enable typescript/strict-void-return */

//endregion Module helpers

//region Sentinels

/**
 Sentinel returned when no CLI setup warning needs to be shown.
 
 @example
 ```typescript
 if (warning === NO_CLI_SETUP_WARNING) return;
 ```
 */
const NO_CLI_SETUP_WARNING: unique symbol = Symbol('spawn-pi/cli-setup-ok',);

//endregion Sentinels

//region Path derivation

/**
 Resolves ancestor directory by walking a fixed number of parent links.
 
 @param startPath - path whose ancestors are inspected.
 
 @param levels - number of `dirname` steps.
 
 @returns ancestor directory path.
 
 @example
 ```typescript
 ancestorDir({ startPath: '/a/b/c', levels: 2 }); // '/a'
 ```
 */
function ancestorDir(
  {
    startPath,
    levels,
  }: {
    readonly startPath: string;
    readonly levels: number;
  },
): string {
  for (let currentPath = startPath, remainingLevels = levels; remainingLevels >= 0;) {
    if (remainingLevels === 0)
      return currentPath;
    remainingLevels -= 1;
    currentPath = dirname(currentPath,);
  }

  return startPath;
}

/**
 Detects tsdown Node build output path shape.
 
 @param extensionPath - extension module path.
 
 @returns whether path appears under `dist/final/node`.
 
 @example
 ```typescript
 isBuiltExtensionPath('/pkg/dist/final/node/index.mjs');
 ```
 */
function isBuiltExtensionPath(extensionPath: string,): boolean {
  /**
   Directory containing extension module.
   */
  const moduleDir = dirname(extensionPath,);
  /**
   Directory one level above module directory.
   */
  const finalDir = dirname(moduleDir,);
  /**
   Directory two levels above module directory.
   */
  const distDir = dirname(finalDir,);

  return (basename(moduleDir,) === 'node')
    && (basename(finalDir,) === 'final')
    && (basename(distDir,) === 'dist');
}

/**
 Resolves package root from source or built extension path.
 
 @param extensionPath - current extension entry path.
 
 @returns package root path.
 
 @example
 ```typescript
 packageRootFromExtensionPath('/pkg/src/index.ts'); // '/pkg'
 ```
 */
function packageRootFromExtensionPath(extensionPath: string,): string {
  if (isBuiltExtensionPath(extensionPath,)) {
    return resolve(
      ancestorDir(
        {
          startPath: dirname(extensionPath,),
          levels: 3,
        },
      ),
    );
  }

  return resolve(
    dirname(extensionPath,),
    '..',
  );
}

/**
 Resolves source CLI path shipped with package.
 
 @param extensionPath - current extension entry path.
 
 @returns CLI entry path matching extension mode.
 
 @example
 ```typescript
 cliPathFromExtensionPath('/pkg/src/index.ts'); // '/pkg/src/cli.ts'
 cliPathFromExtensionPath('/pkg/dist/final/node/index.mjs'); // '/pkg/dist/final/node/cli.mjs'
 ```
 */
function cliPathFromExtensionPath(extensionPath: string,): string {
  if (isBuiltExtensionPath(extensionPath,)) {
    return join(
      dirname(extensionPath,),
      'cli.mjs',
    );
  }

  return join(
    packageRootFromExtensionPath(extensionPath,),
    'src',
    'cli.ts',
  );
}

//endregion Path derivation

//region CLI setup

/**
 Detects whether `spawn-pi` is already discoverable on PATH.
 
 @param env - environment values used for command lookup.
 
 @returns whether `which spawn-pi` succeeds.
 
 @mutates env - `execFileAsync` may inspect or retain environment storage while launching `which`.
 
 @example
 ```typescript
 if (await cliIsOnPath()) return;
 ```
 */
async function cliIsOnPath(env: NodeJS.ProcessEnv = process.env,): Promise<boolean> {
  try {
    await execFileAsync(
      'which',
      ['spawn-pi',],
      { env: { ...env, }, },
    );
    return true;
  }
  catch (error: unknown) {
    // Non-zero `which` exit means the command is not yet discoverable.
    tagged({
      tag: cliIsOnPath.name,
      l,
    },)
      .debug(`spawn-pi not found on PATH: ${String(error,)}`,);
    return false;
  }
}

/**
 Symlinks `spawn-pi` into user-local bin when no CLI is on PATH.
 
 @param extensionPath - current extension entry path.
 
 @param env - environment values used for `HOME` and `PATH`.
 
 @returns warning text, or {@link NO_CLI_SETUP_WARNING} when setup is complete.
 
 @mutates env - `cliIsOnPath` can pass environment storage across native process launch.
 
 @example
 ```typescript
 await autoSetupCli({ extensionPath: '/pkg/dist/final/node/index.mjs' });
 ```
 */
async function autoSetupCli(
  {
    extensionPath,
    env = process.env,
  }: {
    readonly extensionPath: string;
    env?: NodeJS.ProcessEnv;
  },
): Promise<string | typeof NO_CLI_SETUP_WARNING> {
  if (await cliIsOnPath(env,))
    return NO_CLI_SETUP_WARNING;

  /**
   CLI path that symlink should execute with Node shebang.
   */
  const cliSource = cliPathFromExtensionPath(extensionPath,);
  /**
   User-local bin directory.
   */
  const localBin = join(
    env.HOME
      ?? '/tmp',
    '.local',
    'bin',
  );
  /**
   Destination symlink path for command discovery.
   */
  const symlinkPath = join(
    localBin,
    'spawn-pi',
  );

  try {
    await mkdir(
      localBin,
      { recursive: true, },
    );

    /**
     Unix executable permission bits.
     */
    const EXECUTABLE_PERMISSION = 0o755;
    await chmod(
      cliSource,
      EXECUTABLE_PERMISSION,
    );

    try {
      await unlink(symlinkPath,);
    }
    catch (error: unknown) {
      // Missing stale symlink is acceptable; nothing to remove before re-linking.
      tagged({
        tag: autoSetupCli.name,
        l,
      },)
        .debug(`No stale symlink to remove at ${symlinkPath}: ${String(error,)}`,);
    }

    await symlink(
      cliSource,
      symlinkPath,
    );

    /**
     PATH entries available to current process.
     */
    const pathDirs = (env.PATH ?? '').split(':',);
    return pathDirs.includes(localBin,)
      ? NO_CLI_SETUP_WARNING
      : [
        '[spawn-pi] Symlinked spawn-pi to ~/.local/bin/spawn-pi,',
        'but ~/.local/bin is not on PATH. Add it to your shell profile:',
        '  export PATH="$HOME/.local/bin:$PATH"',
      ].join('\n',);
  }
  catch (error: unknown) {
    // Symlink creation failed; surface a manual-setup warning instead.
    tagged({
      tag: autoSetupCli.name,
      l,
    },)
      .debug(`Could not auto-setup spawn-pi CLI: ${String(error,)}`,);
    return [
      '[spawn-pi] Could not auto-setup spawn-pi CLI.',
      `Symlink target: ${cliSource}`,
      `Symlink path: ${symlinkPath}`,
      'Create the symlink manually or add the package bin directory to PATH.',
    ].join('\n',);
  }
}

//endregion CLI setup

export {
  autoSetupCli,
  cliIsOnPath,
  cliPathFromExtensionPath,
  isBuiltExtensionPath,
  NO_CLI_SETUP_WARNING,
  packageRootFromExtensionPath,
};
