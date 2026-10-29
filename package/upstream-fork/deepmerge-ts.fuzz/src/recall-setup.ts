/**
 Host-side inputs for the historical-recall run
 (`doc/audit/deepmerge-ts-recall-2026-09-24.md`), written under
 `~/temp/agent` so nothing lands in this repo:

 - `deepmerge-ts-recall-npm/<version>/package`: every stable npm release,
   plus `deps/is-plain-object/package` for the 1.x releases that need it.
 - `deepmerge-ts-recall-trees/<name>/`: upstream `src`, `tests`, and configs
   at v8.0.2 and at each `parent` row's buggy and fixed commits.

 Takes the path of a full-history upstream clone whose push URL is disabled.
 Light work (tarball downloads and `git archive`), so it runs on the host.

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  access,
  mkdir,
  readdir,
  rm,
} from 'node:fs/promises';
import { homedir, } from 'node:os';
import { join, } from 'node:path';

import { runStep, } from './mutation-container.ts';
import { RUNTIME_BUGS, } from './recall-ledger-runtime.ts';

/**
 Scratch root for agent work outside the repo.
 */
const SCRATCH = join(
  homedir(),
  'temp',
  'agent',
);

/**
 Error for a setup step whose output does not have the expected shape.
 */
export class RecallSetupError extends Error {
  /**
   @param message - What was missing.
   */
  constructor(message: string,) {
    super(message,);
    this.name = 'RecallSetupError';
  }
}

/**
 Run one command and collect its standard output.

 @param command - Executable.

 @param args - Arguments.

 @returns Standard output as UTF-8.

 @throws {@link RecallSetupError} When the command exits unsuccessfully.

 @example
 ```ts
 const json = await capture({ args: ['view', 'deepmerge-ts', 'versions', '--json',], command: 'npm', });
 ```
 */
async function capture(
  {
    command,
    args,
  }: {
    readonly command: string;
    readonly args: readonly string[];
  },
): Promise<string> {
  /**
   Child whose output is collected.
   */
  const child = spawn(
    command,
    args,
    { stdio: [
      'ignore',
      'pipe',
      'inherit',
    ], },
  );
  /**
   Output chunks in arrival order.
   */
  const chunks: Buffer[] = [];
  child.stdout
    .on(
    'data',
    function collect(chunk: Buffer,) {
      chunks.push(chunk,);
    },
    );
  /**
   Exit code and signal once the command closed.
   */
  const closed: readonly unknown[] = await once(
    child,
    'close',
  );
  /**
   Exit code, `null` when a signal ended the command.
   */
  const [code,] = closed;
  if (code !== 0)
    throw new RecallSetupError(`${command} ${args.join(' ',)} exited with ${String(code,)}`,);
  return Buffer.concat(chunks,)
    .toString('utf8',);
}

/**
 Whether a path exists.

 @param path - Path to probe.

 @returns True when it can be accessed.

 @throws Anything other than a missing-path error.

 @example
 ```ts
 await pathExists('/tmp',); // true
 ```
 */
async function pathExists(path: string,): Promise<boolean> {
  try {
    await access(path,);
    return true;
  } catch (error) {
    if ((Error.isError(error,)) && (Reflect.get(
      error,
      'code',
    ) === 'ENOENT'))
      return false;
    throw error;
  }
}

/**
 Download and extract one npm package version into `dir`.

 @param spec - npm spec, such as `deepmerge-ts@7.1.6`.

 @param dir - Destination; the tarball extracts to `dir/package`.

 @throws {@link RecallSetupError} When `npm pack` writes no tarball.

 @example
 ```ts
 await fetchPackage({ dir: '/tmp/x', spec: 'deepmerge-ts@8.0.2', });
 ```
 */
async function fetchPackage(
  {
    spec,
    dir,
  }: {
    readonly spec: string;
    readonly dir: string;
  },
): Promise<void> {
  if (await pathExists(join(
    dir,
    'package',
    'package.json',
  ),))
    return;
  await mkdir(
    dir,
    { recursive: true, },
  );
  await runStep({
    args: [
      'pack',
      spec,
      '--pack-destination',
      dir,
      '--silent',
    ],
    command: 'npm',
    cwd: dir,
  },);
  /**
   Downloaded tarball name.
   */
  const tarball = (await readdir(dir,))
    .find(function isTarball(name,) {
      return name.endsWith('.tgz',);
    },);
  if (tarball === undefined)
    throw new RecallSetupError(`npm pack ${spec} wrote no tarball`,);
  await runStep({
    args: [
      '--extract',
      '--gzip',
      '--file',
      join(
        dir,
        tarball,
      ),
      '--directory',
      dir,
    ],
    command: 'tar',
    cwd: dir,
  },);
}

/**
 Extract upstream files at one revision into a fresh tree directory.

 @param clone - Full-history upstream clone.

 @param name - Tree name under `deepmerge-ts-recall-trees`.

 @param revision - Git revision to archive.

 @example
 ```ts
 await extractTree({ clone, name: 'v8.0.2', revision: 'v8.0.2', });
 ```
 */
async function extractTree(
  {
    clone,
    name,
    revision,
  }: {
    readonly clone: string;
    readonly name: string;
    readonly revision: string;
  },
): Promise<void> {
  /**
   Tree directory.
   */
  const dir = join(
    SCRATCH,
    'deepmerge-ts-recall-trees',
    name,
  );
  /**
   Archive beside the tree, removed with it on the next run.
   */
  const archive = `${dir}.tar`;
  await rm(
    dir,
    {
      force: true,
      recursive: true,
    },
  );
  await mkdir(
    dir,
    { recursive: true, },
  );
  await runStep({
    args: [
      '-C',
      clone,
      'archive',
      '--format=tar',
      `--output=${archive}`,
      revision,
      'src',
      'tests',
      'tsconfig.json',
      'vitest.config.ts',
    ],
    command: 'git',
    cwd: dir,
  },);
  await runStep({
    args: [
      '--extract',
      '--file',
      archive,
      '--directory',
      dir,
    ],
    command: 'tar',
    cwd: dir,
  },);
  await rm(
    archive,
    { force: true, },
  );
  console.log(`${name} <- ${revision}`,);
}

if (import.meta.main) {
  /**
   Full-history upstream clone.
   */
  const [clone,] = process.argv
    .slice(2,);
  if ((clone === undefined) || (clone === ''))
    throw new RecallSetupError('usage: recall-setup.ts <full-history deepmerge-ts clone>',);
  /**
   Release directory.
   */
  const npmRoot = join(
    SCRATCH,
    'deepmerge-ts-recall-npm',
  );
  /**
   Every published version.
   */
  const versions: unknown = JSON.parse(await capture({
    args: [
      'view',
      'deepmerge-ts',
      'versions',
      '--json',
    ],
    command: 'npm',
  },),);
  if (!Array.isArray(versions,))
    throw new RecallSetupError('npm view returned no version list',);
  /**
   Stable releases only; prereleases never shipped a fix.
   */
  const stable = versions.map(String,)
    .filter(function isStable(entry,) {
      return !entry.includes('-',);
    },);
  // One download at a time keeps npm's cache writes and the registry load sequential.
  for (const version of stable) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- sequential by design, see the comment above.
    await fetchPackage({
      dir: join(
        npmRoot,
        version,
      ),
      spec: `deepmerge-ts@${version}`,
    },);
  }
  await fetchPackage({
    dir: join(
      npmRoot,
      'deps',
      'is-plain-object',
    ),
    spec: 'is-plain-object@5',
  },);
  /**
   Tree name to revision: v8.0.2 plus both sides of every `parent` row.
   */
  const trees = new Map<string, string>([
    [
      'v8.0.2',
      'v8.0.2',
    ],
    ...RUNTIME_BUGS.flatMap(function sides(bug,): readonly (readonly [
      string,
      string
    ])[] {
      return (bug.source
        .kind
        === 'parent')
        ? [
          [
            bug.source
              .buggyTree,
            `${bug.commit}^`,
          ],
          [
            bug.source
              .fixedTree,
            bug.commit,
          ],
        ]
        : [];
    },),
  ],);
  await Promise.all([...trees,].map(async function extract([name, revision,],) {
    await extractTree({
      clone,
      name,
      revision,
    },);
  },),);
}
