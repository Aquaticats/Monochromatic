/**
 In-container packaging probes for the historical-recall run
 (`doc/audit/deepmerge-ts-recall-2026-09-24.md`), in the image built by
 `../container/mutation.Containerfile` (Node plus TypeScript 6.0.2 at
 `/upstream`), with this repo read-only and `dist/recall/packaging` writable
 at their host paths and extracted npm releases read-only at
 `RECALL_NPM_DIR/<version>/package`.

 Each release is installed into its own consumer directory and loaded six
 ways: the sidecar's way (TypeScript 7 `bundler` resolution, Node ESM named
 import) and four it never uses (TypeScript 6 `node10`, TypeScript 7
 `nodenext` from ESM and from CommonJS, Node `require`). A surface row is
 visible to the sidecar only if a sidecar-way probe separates its buggy
 release from its fixed one.

 @module
 */

import {
  cp,
  mkdir,
  rm,
  writeFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import { runProbe, } from './recall-probe-run.ts';

/**
 Characters of a failure's output kept when no line names an error.
 */
const FAILURE_EXCERPT = 200;


/**
 This package's root.
 */
const PACKAGE_ROOT = resolve(
  import.meta.dirname,
  '..',
);

/**
 Writable output directory inside the package.
 */
const OUT_DIR = join(
  PACKAGE_ROOT,
  'dist',
  'recall',
  'packaging',
);

/**
 Consumer directories, outside this repo so module resolution cannot walk up
 into the package's own `node_modules/deepmerge-ts`.
 */
const CONSUMER_ROOT = '/tmp/recall-consumers';

/**
 Workspace TypeScript 7 compiler.
 */
const TSC_7 = join(
  PACKAGE_ROOT,
  '..',
  '..',
  '..',
  'node_modules',
  '.bin',
  'tsc',
);

/**
 The image's TypeScript 6.0.2 compiler, the last major with `node10`.
 */
const TSC_6 = '/upstream/node_modules/.bin/tsc';

/**
 Consumer module every type probe compiles.
 */
const CONSUMER = [
  'import { deepmerge } from "deepmerge-ts";',
  'const merged = deepmerge({ a: 1 }, { b: "x" });',
  'const check: { a: number; b: string } = merged;',
  'export { check };',
  '',
].join('\n',);

/**
 One probe: a name, the command, and its arguments relative to a consumer directory.
 */
type Probe = {
  readonly name: string;
  readonly sidecarWay: boolean;
  readonly command: string;
  readonly args: (dir: string,) => readonly string[];
};

/**
 Compiler options per type probe, beside `strict` and `noEmit`.
 */
const TYPE_PROBES: Readonly<Record<string, Readonly<Record<string, string>>>> = {
  'tsc6-node10': {
    // TypeScript 6 deprecates node10 and errors (TS5107) without this.
    ignoreDeprecations: '6.0',
    module: 'commonjs',
    moduleResolution: 'node10',
  },
  'tsc7-bundler': {
    module: 'preserve',
    moduleResolution: 'bundler',
  },
  'tsc7-nodenext-cjs': {
    module: 'nodenext',
    moduleResolution: 'nodenext',
  },
  'tsc7-nodenext-esm': {
    module: 'nodenext',
    moduleResolution: 'nodenext',
  },
};

/**
 Every probe in report order.
 */
const PROBES: readonly Probe[] = [
  {
    args: function bundlerArgs(dir,) {
      return [
        '--project',
        join(
          dir,
          'tsc7-bundler',
          'tsconfig.json',
        ),
      ];
    },
    command: TSC_7,
    name: 'tsc7-bundler',
    sidecarWay: true,
  },
  {
    args: function esmArgs() {
      return [
        '--input-type=module',
        '--eval',
        'import { deepmerge } from "deepmerge-ts"; if (typeof deepmerge !== "function") throw new Error("no deepmerge");',
      ];
    },
    command: 'node',
    name: 'node-esm-named',
    sidecarWay: true,
  },
  {
    args: function node10Args(dir,) {
      return [
        '--project',
        join(
          dir,
          'tsc6-node10',
          'tsconfig.json',
        ),
      ];
    },
    command: TSC_6,
    name: 'tsc6-node10',
    sidecarWay: false,
  },
  {
    args: function nodenextEsmArgs(dir,) {
      return [
        '--project',
        join(
          dir,
          'tsc7-nodenext-esm',
          'tsconfig.json',
        ),
      ];
    },
    command: TSC_7,
    name: 'tsc7-nodenext-esm',
    sidecarWay: false,
  },
  {
    args: function nodenextCjsArgs(dir,) {
      return [
        '--project',
        join(
          dir,
          'tsc7-nodenext-cjs',
          'tsconfig.json',
        ),
      ];
    },
    command: TSC_7,
    name: 'tsc7-nodenext-cjs',
    sidecarWay: false,
  },
  {
    args: function cjsArgs() {
      return [
        '--eval',
        'const { deepmerge } = require("deepmerge-ts"); if (typeof deepmerge !== "function") throw new Error("no deepmerge");',
      ];
    },
    command: 'node',
    name: 'node-cjs-require',
    sidecarWay: false,
  },
];

/**
 Install one release into a consumer directory with a project per type probe.

 @param version - npm release.

 @returns Consumer directory.

 @example
 ```ts
 const dir = await installConsumer('7.0.1');
 ```
 */
async function installConsumer(version: string,): Promise<string> {
  /**
   Consumer directory.
   */
  const dir = join(
    CONSUMER_ROOT,
    version,
  );
  await rm(
    dir,
    {
      force: true,
      recursive: true,
    },
  );
  await cp(
    join(
      process.env
        .RECALL_NPM_DIR
        ?? '',
      version,
      'package',
    ),
    join(
      dir,
      'node_modules',
      'deepmerge-ts',
    ),
    { recursive: true, },
  );
  await writeFile(
    join(
      dir,
      'package.json',
    ),
    '{ "private": true }\n',
  );
  // 1.x releases depend on is-plain-object; its tarball sits under deps/.
  await cp(
    join(
      process.env
        .RECALL_NPM_DIR
        ?? '',
      'deps',
      'is-plain-object',
      'package',
    ),
    join(
      dir,
      'node_modules',
      'is-plain-object',
    ),
    { recursive: true, },
  );
  await Promise.all(Object.entries(TYPE_PROBES,)
    .map(async function writeProject([name, options,],) {
      /**
       Probe project directory.
       */
      const project = join(
        dir,
        name,
      );
      /**
       Consumer file name; the CommonJS probe needs `.cts`.
       */
      const file = (name === 'tsc7-nodenext-cjs') ? 'consumer.cts' : ((name === 'tsc7-nodenext-esm') ? 'consumer.mts' : 'consumer.ts');
      await mkdir(
        project,
        { recursive: true, },
      );
      await writeFile(
        join(
          project,
          file,
        ),
        CONSUMER,
      );
      await writeFile(
        join(
          project,
          'tsconfig.json',
        ),
        `${JSON.stringify({
          compilerOptions: {
            ...options,
            noEmit: true,
            skipLibCheck: false,
            strict: true,
            types: [],
          },
          files: [file,],
        },)}\n`,
      );
    },),);
  return dir;
}

/**
 Run every probe against one release.

 @param version - npm release.

 @returns Probe name to `pass` or the first line of the failure.

 @example
 ```ts
 await probeRelease('7.0.1');
 ```
 */
async function probeRelease(version: string,): Promise<Readonly<Record<string, string>>> {
  /**
   Consumer directory.
   */
  const dir = await installConsumer(version,);
  /**
   Outcomes in probe order.
   */
  const outcomes = await Promise.all(PROBES.map(async function probe(entry,) {
    /**
     Probe exit and output.
     */
    const {
      code,
      output,
    } = await runProbe({
      args: entry.args(dir,),
      command: entry.command,
      cwd: dir,
    },);
    if (code === 0) {
      return [
        entry.name,
        'pass',
      ] as const;
    }
    /**
     First line naming an error, else the start of the output.
     */
    const reason = output.split('\n',)
      .find(function isError(line,) {
        return line.includes('error',) || line.includes('Error',);
      },)
      ?? output.slice(
        0,
        FAILURE_EXCERPT,
      );
    return [
      entry.name,
      `fail: ${reason.trim()}`,
    ] as const;
  },),);
  return Object.fromEntries(outcomes,);
}

if (import.meta.main) {
  /**
   Releases to probe.
   */
  const versions = process.argv
    .slice(2,);
  /**
   Outcomes per release, filled one release at a time inside the 2 GiB cap.
   */
  const results: Record<string, Readonly<Record<string, string>>> = {};
  for (const version of versions) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- releases run one at a time; each already runs its probes in parallel.
    results[version] = await probeRelease(version,);
  }
  await writeFile(
    join(
      OUT_DIR,
      'results.json',
    ),
    `${JSON.stringify(
      {
        probes: PROBES.map(function describe(entry,) {
          return {
            name: entry.name,
            sidecarWay: entry.sidecarWay,
          };
        },),
        results,
      },
      undefined,
      2,
    )}\n`,
  );
  console.log(JSON.stringify(
    results,
    undefined,
    2,
  ),);
}
