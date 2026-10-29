/**
 Scratch toolchain shared by the surface-audit container scripts
 (`./surface-type-matrix.ts`, `./surface-type-cost.ts`): the pinned install
 (run this file with a scratch directory to write its `package.json`),
 TypeScript command lines, and a child-process runner that captures output.

 The install lives in a writable scratch directory (the requested tasks use
 `dist/surface/scratch`), never in this repo's workspace, so eleven
 TypeScript versions never touch the lockfile. Findings and controls:
 `doc/audit/deepmerge-ts-surface-2026-09-24.md`.

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

/**
 TypeScript releases compared, as npm aliases. The oldest is the floor upstream's
 changelog states (5.0.0: "remove typing support for typescript<4.7"); 7.0.2 is
 the native compiler upstream's TypeScript 7 support targets. Pinned so reruns
 compare like with like.
 */
export const TYPESCRIPT_RELEASES = {
  ts47: '4.7.4',
  ts48: '4.8.4',
  ts49: '4.9.5',
  ts50: '5.0.4',
  ts52: '5.2.2',
  ts54: '5.4.5',
  ts56: '5.6.3',
  ts58: '5.8.3',
  ts59: '5.9.3',
  ts60: '6.0.2',
  ts70: '7.0.2',
} as const;

/**
 Alias of one installed TypeScript release.
 */
export type TypeScriptAlias = keyof typeof TYPESCRIPT_RELEASES;

/**
 Aliases whose compilers predate `moduleResolution: bundler` (added in 5.0)
 and the ES2023 lib.
 */
const LEGACY_ALIASES: ReadonlySet<TypeScriptAlias> = new Set([
  'ts47',
  'ts48',
  'ts49',
],);

/**
 Aliases that refuse command-line files while a `tsconfig.json` exists in an
 ancestor directory (TS5112) unless told `--ignoreConfig`, which older
 releases reject as unknown.
 */
const IGNORE_CONFIG_ALIASES: ReadonlySet<TypeScriptAlias> = new Set([
  'ts60',
  'ts70',
],);

/**
 Other scratch dependencies, pinned: the release under audit, the
 `expectTypeOf` implementation this repo's harness re-exports, the Node types
 upstream builds against, and the bundler used for per-file transpilation.
 */
const OTHER_DEPENDENCIES = {
  '@types/node': '22.20.1',
  'deepmerge-ts': '8.0.2',
  esbuild: '0.28.2',
  'expect-type': '1.4.0',
} as const;

/**
 Whether a string names an installed TypeScript alias.

 @param value - Candidate from the command line.

 @returns True for a key of {@link TYPESCRIPT_RELEASES}.

 @example
 ```ts
 isTypeScriptAlias('ts60',); // true
 ```
 */
export function isTypeScriptAlias(value: string,): value is TypeScriptAlias {
  return Object.hasOwn(
    TYPESCRIPT_RELEASES,
    value,
  );
}

/**
 Write the scratch `package.json`; `npm install` in that directory then
 installs every pinned tool.

 @param scratch - Writable scratch directory.

 @example
 ```ts
 await writeScratchManifest('/scratch',);
 ```
 */
export async function writeScratchManifest(scratch: string,): Promise<void> {
  /**
   Every dependency, TypeScript releases as npm aliases.
   */
  const dependencies = {
    ...OTHER_DEPENDENCIES,
    ...Object.fromEntries(Object.entries(TYPESCRIPT_RELEASES,)
      .map(function asAlias([alias, version,],) {
        return [
          alias,
          `npm:typescript@${version}`,
        ] as const;
      },),),
  };
  await writeFile(
    join(
      scratch,
      'package.json',
    ),
    `${JSON.stringify(
      {
        dependencies,
        name: 'deepmerge-ts-surface-scratch',
        private: true,
        type: 'module',
      },
      undefined,
      2,
    )}\n`,
  );
}

/**
 Command-line prefix shared by every surface `tsc` run: no emit, library
 declarations checked, and the newest resolution and lib each release has.

 @param alias - Release to run.

 @param scratch - Scratch directory holding the install.

 @returns Arguments for `node`, starting with the release's `tsc` script.

 @example
 ```ts
 tscPrefix({ alias: 'ts60', scratch: '/scratch', });
 ```
 */
export function tscPrefix(
  {
    alias,
    scratch,
  }: {
    readonly alias: TypeScriptAlias;
    readonly scratch: string;
  },
): readonly string[] {
  /**
   Whether this release needs the pre-5.0 fallbacks.
   */
  const legacy = LEGACY_ALIASES.has(alias,);
  return [
    '--max-old-space-size=1536',
    '--stack-size=4000',
    join(
      scratch,
      'node_modules',
      alias,
      'bin',
      'tsc',
    ),
    '--noEmit',
    '--pretty',
    'false',
    '--skipLibCheck',
    'false',
    '--target',
    'es2022',
    '--lib',
    legacy ? 'es2022,dom' : 'esnext,dom',
    '--types',
    'node',
    '--module',
    'esnext',
    '--moduleResolution',
    legacy ? 'node' : 'bundler',
    ...(IGNORE_CONFIG_ALIASES.has(alias,) ? ['--ignoreConfig',] : []),
  ];
}

/**
 Outcome of one child process: whether it exited 0, its exit code or ending
 signal as text, and its combined output.
 */
export type ChildOutcome = {
  readonly passed: boolean;
  readonly exit: string;
  readonly output: string;
};

/**
 Run a command to completion, capturing stdout and stderr together, and
 kill it with SIGKILL after `timeoutMs`.

 @param command - Executable.

 @param args - Arguments.

 @param cwd - Working directory.

 @param timeoutMs - Limit before the child is killed.

 @returns Exit status and combined output.

 @example
 ```ts
 const outcome = await runChild({ args: ['--version',], command: 'node', cwd: '.', timeoutMs: 10_000, });
 ```
 */
export async function runChild(
  {
    command,
    args,
    cwd,
    timeoutMs,
  }: {
    readonly command: string;
    readonly args: readonly string[];
    readonly cwd: string;
    readonly timeoutMs: number;
  },
): Promise<ChildOutcome> {
  /**
   Child being run.
   */
  const child = spawn(
    command,
    args,
    { cwd, },
  );
  /**
   Output chunks in arrival order.
   */
  const chunks: string[] = [];
  child.stdout
    .on(
    'data',
    function collect(chunk: Buffer,) {
      chunks.push(chunk.toString(),);
    },
    );
  child.stderr
    .on(
    'data',
    function collect(chunk: Buffer,) {
      chunks.push(chunk.toString(),);
    },
    );
  /**
   Timer that ends a hung child.
   */
  const timer = setTimeout(
    function kill() {
      child.kill('SIGKILL',);
    },
    timeoutMs,
  );
  /**
   Exit code and signal once the child closed.
   */
  const closed: readonly unknown[] = await once(
    child,
    'close',
  );
  clearTimeout(timer,);
  /**
   Exit code and signal.
   */
  const [code, signal,] = closed;
  return {
    exit: ((typeof signal) === 'string') ? signal : String(code,),
    output: chunks.join('',),
    passed: code === 0,
  };
}

/**
 TypeScript diagnostic codes in `tsc --pretty false` output, grouped by file.

 @param output - Compiler output.

 @returns `file code` keys with their counts, sorted.

 @example
 ```ts
 diagnosticCounts('a.ts(1,1): error TS2322: x',); // [['a.ts TS2322', 1]]
 ```
 */
export function diagnosticCounts(output: string,): readonly (readonly [
  string,
  number,
])[] {
  /**
   Count per file and code.
   */
  const counts = new Map<string, number>();
  for (const line of output.split('\n',)) {
    /**
     Offset of the diagnostic code; option and config errors start the line with no file.
     */
    const codeAt = line.startsWith('error TS',) ? 0 : line.indexOf(' error TS',);
    if (codeAt !== (-1)) {
      /**
       File path before the position suffix.
       */
      const file = (codeAt === 0) ? '(options)' : line.slice(
        0,
        line.indexOf('(',),
      );
      /**
       Code such as `TS2322`.
       */
      const code = line.slice(
        line.indexOf(
          'TS',
          codeAt,
        ),
        line.indexOf(
          ':',
          codeAt,
        ),
      );
      counts.set(
        `${file} ${code}`,
        (counts.get(`${file} ${code}`,) ?? 0) + 1,
      );
    }
  }
  return [...counts,].toSorted(function byKey(
    [left,],
    [right,],
  ) {
    return left.localeCompare(right,);
  },);
}

if (import.meta.main) {
  /**
   Scratch directory to write the manifest into.
   */
  const [scratch,] = process.argv
    .slice(2,);
  if (scratch === undefined)
    throw new Error('usage: surface-toolchain.ts <scratch>; then run npm install in <scratch>',);
  await writeScratchManifest(scratch,);
  console.log(`wrote ${join(
    scratch,
    'package.json',
  )}; run npm install there`,);
}
