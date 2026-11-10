/**
 The monorepo's pnpm, as the host runs it.
 
 Every fixture installs with the pnpm this repository runs instead of a version
 hardcoded in the matrix, which would drift from mise's floating
 `pnpm = "latest"`. Two facts come back: the `pnpm@<version>` spec that pins the
 fixture's `packageManager`, and the store directory the repository's
 `node_modules` entries link into, which each container has to mount.
 
 Under mise the `pnpm` on `PATH` is this repository's `pnpm-shadow`, which
 prints pnpm's own output and then a notice naming itself, in the shape `hub`
 and `mpm-shadow` use. That notice is deliberate
 (`package/pnpm-shadow/cli/src/shadow_notice.rs`), so a version read as the
 first line carrying a version is right under either printing order, where the
 whole trimmed stdout is a spec no `packageManager` field can hold.
 */

import {
  isAbsolute,
} from 'node:path';

import spawn from 'nano-spawn';

import {
  HOST_REPO_ROOT,
} from './container-paths.ts';

//region Version resolution

/**
 Reads the version from `pnpm --version` output, ignoring the notice a
 wrapper prints after it.
 
 pnpm writes its own version first, and this repository's wrapper appends its
 notice after a successful run
 (`package/pnpm-shadow/cli/src/shadow_notice.rs`), so the first line is pnpm's
 version. The notice names itself and the pnpm it shadows, so any first line
 holding whitespace is not a version and is refused instead of being handed to
 corepack as a spec.
 
 @param output - everything `pnpm --version` wrote to stdout
 
 @returns `pnpm@<version>` spec for the fixture's `packageManager` field and corepack
 
 @throws Error when the first line is empty or is not a single token
 
 @example
 ```ts
 pnpmSpecFromVersionOutput('12.18.0\npnpm-shadow 0.1.3, shadowing pnpm at /opt/pnpm\n') // 'pnpm\@12.18.0'
 ```
 */
export function pnpmSpecFromVersionOutput(output: string,): string {
  /**
   First line of the output, which pnpm writes its version on.
   */
  const [firstLine,] = output.split('\n',);
  /**
   That line without surrounding whitespace, which a shell or a wrapper may add.
   */
  const version = (firstLine ?? '').trim();
  if ((version.length === 0) || version.includes(' ',)) {
    throw new Error(
      `could not resolve the monorepo pnpm version via \`pnpm --version\`; its output was ${JSON.stringify(output,)}`,
    );
  }
  return `pnpm@${version}`;
}

/**
 Resolves the spec the host's pnpm runs, through the wrapper on `PATH`.
 
 @returns `pnpm@<version>` spec for the fixture's `packageManager` field and corepack
 
 @throws Error when `pnpm --version` reports no version
 
 @example
 ```ts
 await resolvePnpmSpec() // 'pnpm\@12.18.0'
 ```
 */
export async function resolvePnpmSpec(): Promise<string> {
  /**
   `pnpm --version` result, read from the repository root so the workspace's
   config applies.
   */
  const result = await spawn(
    'pnpm',
    ['--version',],
    { cwd: HOST_REPO_ROOT, },
  );
  return pnpmSpecFromVersionOutput(result.stdout,);
}

//endregion Version resolution

//region Store resolution

/**
 Resolves the store the repository's `node_modules` entries link into, as the
 host's pnpm reports it.
 
 @returns absolute store directory on the host
 
 @throws Error when `pnpm store path` names no absolute directory
 
 @example
 ```ts
 await resolveStoreDir() // '/home/user/.local/share/pnpm/store/v11'
 ```
 */
export async function resolveStoreDir(): Promise<string> {
  /**
   `pnpm store path` result, read from the repository root so the workspace's
   `storeDir` applies.
   */
  const result = await spawn(
    'pnpm',
    [
      'store',
      'path',
    ],
    { cwd: HOST_REPO_ROOT, },
  );
  /**
   Absolute paths the output names, in order; a wrapper that prints its notice
   on this request too would leave the store as the one absolute entry.
   */
  const absolutePaths = result.stdout
    .split('\n',)
    .map(function trimLine(line,): string {
      return line.trim();
    },)
    .filter(function keepAbsolute(line,): boolean {
      return isAbsolute(line,);
    },);
  /**
   Absolute store path the output names first, or undefined when it names none.
   */
  const [storeDir,] = absolutePaths;
  if (storeDir === undefined) {
    throw new Error(
      `could not resolve the monorepo pnpm store via \`pnpm store path\`; its output was ${JSON.stringify(result.stdout,)}`,
    );
  }
  return storeDir;
}

//endregion Store resolution
