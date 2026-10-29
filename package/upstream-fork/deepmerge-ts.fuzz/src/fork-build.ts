/**
 In-container build of a deepmerge-ts checkout with source maps, run by the
 `fork:build` mise task (Q9 in `doc/handover/deepmerge-ts-hardening.md`).

 Upstream's `rollup.config.ts` disables source maps and its TypeScript plugin
 emits none, so a plain build cannot map coverage back to `src/*.ts`. This
 driver copies the read-only checkout mounted at `/checkout` to a scratch
 directory, enables source maps in that copy only, installs with the
 checkout's own lockfile and package manager version, builds, and copies
 `index.mjs` and its map to `/out`. The checkout itself is never written.

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  copyFile,
  cp,
  readFile,
  writeFile,
} from 'node:fs/promises';

/**
 Read-only checkout mount inside the container.
 */
const CHECKOUT = '/checkout';

/**
 Scratch copy the build may modify.
 */
const WORK = '/work';

/**
 Writable output mount inside the container.
 */
const OUT = '/out';

/**
 Error for a build step that exited unsuccessfully.
 */
export class ForkBuildError extends Error {
  /**
   @param message - Failed step and its exit.
   */
  constructor(message: string,) {
    super(message,);
    this.name = 'ForkBuildError';
  }
}

/**
 Run one command in the scratch copy, streaming its output.

 @param command - Executable.
 
 @param args - Arguments.

 @throws {@link ForkBuildError} When the command exits unsuccessfully.

 @example
 ```ts
 await run({ command: 'npx', args: ['--version',], });
 ```
 */
async function run({
  command,
  args,
}: {
  readonly command: string;
  readonly args: readonly string[]
},): Promise<void> {
  /**
   Child running the step.
   */
  const child = spawn(
    command,
    args,
    {
      cwd: WORK,
      stdio: 'inherit',
    },
  );
  /**
   Exit code of the step.
   */
  const closed: readonly unknown[] = await once(
    child,
    'close',
  );
  /**
   Exit code, not a number when a signal ended the step.
   */
  const [code,] = closed;
  if (code !== 0)
    throw new ForkBuildError(`${command} ${args.join(' ',)} exited with ${String(code,)}`,);
}

/**
 Replace one exact snippet, failing loudly when upstream's config changed.

 @param text - Source text.
 
 @param from - Snippet expected exactly once.
 
 @param to - Replacement.

 @returns Edited text.

 @throws {@link ForkBuildError} When the snippet is missing.

 @example
 ```ts
 replaceOnce({ text: 'a', from: 'a', to: 'b', }); // 'b'
 ```
 */
export function replaceOnce({
  text,
  from,
  to,
}: {
  readonly text: string;
  readonly from: string;
  readonly to: string
},): string {
  if (!text.includes(from,))
    throw new ForkBuildError(`rollup.config.ts no longer contains ${JSON.stringify(from,)}; update src/fork-build.ts`,);
  return text.replace(
    from,
    to,
  );
}

/**
 Build the checkout with source maps and export the ESM entry and its map.

 @throws {@link ForkBuildError} When a step fails or the config drifted.

 @example
 ```ts
 await buildFork();
 ```
 */
export async function buildFork(): Promise<void> {
  await cp(
    CHECKOUT,
    WORK,
    {
      recursive: true,
      filter: function skipBuildState(source,) {
      // Build state and Git metadata (which may hold sockets) are neither needed nor copyable.
        return [
          '/node_modules',
          '/.pnpm-store',
          '/.git',
        ].every(function absent(segment,) {
          return !source.includes(segment,);
        },) && (!source.startsWith(`${CHECKOUT}/dist`,));
      },
    },
  );
  /**
   Upstream config with both source map switches turned on.
   */
  const config = replaceOnce({
    text: replaceOnce({
      text: await readFile(
        `${WORK}/rollup.config.ts`,
        'utf8',
      ),
      from: 'format: "esm",\n      sourcemap: false,',
      to: 'format: "esm",\n      sourcemap: true,',
    },),
    from: 'outDir: "dist",',
    to: 'outDir: "dist",\n      sourceMap: true,',
  },);
  await writeFile(
    `${WORK}/rollup.config.ts`,
    config,
  );
  /**
   Package manager pinned by the checkout, e.g. `pnpm@11.24.0`.
   */
  const manifest: unknown = JSON.parse(await readFile(
    `${WORK}/package.json`,
    'utf8',
  ),);
  /**
   Raw `packageManager` field, narrowed before use.
   */
  const packageManagerField: unknown = ((typeof manifest) === 'object') && (manifest !== null)
    ? Reflect.get(
      manifest,
      'packageManager',
    )
    : '';
  if (((typeof packageManagerField) !== 'string') || (!packageManagerField
    .startsWith('pnpm@',)))
    throw new ForkBuildError(`checkout packageManager ${String(packageManagerField,)} is not pnpm`,);
  /**
   Narrowed package manager spec.
   */
  const packageManager = packageManagerField;
  await run({
    command: 'npx',
    args: [
      '--yes',
      packageManager,
      'install',
      '--frozen-lockfile',
      '--ignore-scripts',
    ],
  },);
  await run({
    command: 'npx',
    args: [
      '--yes',
      packageManager,
      'exec',
      'rollup',
      '-c',
      'rollup.config.ts',
      '--configPlugin',
      '@rollup/plugin-typescript',
      '--configImportAttributesKey',
      'with',
    ],
  },);
  await copyFile(
    `${WORK}/dist/index.mjs`,
    `${OUT}/index.mjs`,
  );
  await copyFile(
    `${WORK}/dist/index.mjs.map`,
    `${OUT}/index.mjs.map`,
  );
}

if (import.meta.main)
  await buildFork();
