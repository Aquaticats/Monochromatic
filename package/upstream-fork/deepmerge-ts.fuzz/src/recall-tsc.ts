/**
 Compiler jobs for the type side of the historical-recall run
 (`./recall-types.ts`): resolve a release's declarations, write a tsconfig
 that maps `deepmerge-ts` to them, run the workspace compiler under a time
 limit, and key its diagnostics by file, position, and code.

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  mkdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import {
  join,
  relative,
  resolve,
} from 'node:path';

import {
  fieldOf,
  pathOf,
} from './recall-json.ts';

/**
 This package's root.
 */
export const PACKAGE_ROOT: string = resolve(
  import.meta.dirname,
  '..',
);

/**
 Writable output directory, inside the package so `node_modules` lookups from
 generated tsconfigs reach the package's dependencies.
 */
export const OUT_DIR: string = join(
  PACKAGE_ROOT,
  'dist',
  'recall',
  'types',
);

/**
 Per-job compiler limit; a release that makes the check hang is recorded as a
 timeout, which is itself the finding for type-checker cost bugs.
 */
const TSC_TIMEOUT_MS = 900_000;

/**
 Marker between a diagnostic's location and its code in `--pretty false` output.
 */
const ERROR_MARKER = ': error TS';

/**
 `lastIndexOf` result when a line has no `(line,col)` part.
 */
const NOT_FOUND = -1;

/**
 Start of a diagnostic without a location, such as a configuration error.
 */
const GLOBAL_ERROR = 'error TS';

/**
 Error for a job that cannot produce trustworthy evidence.
 */
export class RecallTypesError extends Error {
  /**
   @param message - What failed.
   */
  constructor(message: string,) {
    super(message,);
    this.name = 'RecallTypesError';
  }
}

/**
 Extracted npm releases, from the environment.

 @returns Directory holding `<version>/package`.

 @throws {@link RecallTypesError} When `RECALL_NPM_DIR` is unset.

 @example
 ```ts
 npmDir(); // '/home/user/temp/agent/deepmerge-ts-recall-npm'
 ```
 */
function npmDir(): string {
  /**
   Configured directory.
   */
  const dir = process.env
    .RECALL_NPM_DIR;
  if ((dir === undefined) || (dir === ''))
    throw new RecallTypesError('RECALL_NPM_DIR is not set',);
  return dir;
}

/**
 Declaration file a bundler-resolution consumer loads for `import`: the
 `exports` types condition, else the TypeScript 4.1+ `typesVersions` entry,
 else `types`.

 @param version - npm release.

 @returns Absolute declaration path.

 @throws {@link RecallTypesError} When the manifest names no declarations.

 @example
 ```ts
 await typesEntry('7.1.6'); // '.../7.1.6/package/dist/node/index.d.mts'
 ```
 */
export async function typesEntry(version: string,): Promise<string> {
  /**
   Package directory.
   */
  const dir = join(
    npmDir(),
    version,
    'package',
  );
  /**
   Parsed manifest.
   */
  const manifest: unknown = JSON.parse(await readFile(
    join(
      dir,
      'package.json',
    ),
    'utf8',
  ),);
  /**
   Candidates in resolution order.
   */
  const candidates: readonly unknown[] = [
    pathOf({
      path: [
        'exports',
        'types',
        'import',
      ],
      value: manifest,
    },),
    pathOf({
      path: [
        'typesVersions',
        '>=4.1',
        '*',
      ],
      value: manifest,
    },),
    fieldOf({
      key: 'types',
      value: manifest,
    },),
  ].map(function firstOf(candidate,): unknown {
    if (!Array.isArray(candidate,))
      return candidate;
    /**
     Candidate list, typed so reading its first entry stays `unknown`.
     */
    const list: readonly unknown[] = candidate;
    return list[0];
  },);
  /**
   First string candidate.
   */
  const entry = candidates.find(function isString(candidate,) {
    return (typeof candidate) === 'string';
  },);
  if ((typeof entry) !== 'string')
    throw new RecallTypesError(`deepmerge-ts ${version} names no declarations`,);
  return join(
    dir,
    entry,
  );
}

/**
 Compiler outcome of one job.
 */
export type TscOutcome = {
  readonly job: string;
  readonly entry: string;
  readonly ms: number;
  readonly timedOut: boolean;
  readonly diagnostics: readonly string[];
};

/**
 Diagnostic keys (`file(line,col): TSnnnn`, file relative to the package)
 from `--pretty false` output; continuation lines are dropped because the
 location and code identify the failing assertion.

 @param output - Compiler output.

 @param dir - Directory the compiler ran in, for relative paths it prints.

 @returns Keys in output order.

 @example
 ```ts
 diagnosticKeys({ dir, output: 'src/a.ts(1,2): error TS2322: x', }); // ['src/a.ts(1,2): TS2322']
 ```
 */
function diagnosticKeys(
  {
    output,
    dir,
  }: {
    readonly output: string;
    readonly dir: string;
  },
): readonly string[] {
  return output.split('\n',)
    .filter(function isError(line,) {
      return line.includes(ERROR_MARKER,) || line.startsWith(GLOBAL_ERROR,);
    },)
    .map(function keyOf(line,) {
      if (!line.includes(ERROR_MARKER,))
        return `<global>: TS${String(line.slice(GLOBAL_ERROR.length,)
          .split(':',)[0],)}`;
      /**
       Location part, `file(line,col)`.
       */
      const location = line.slice(
        0,
        line.indexOf(ERROR_MARKER,),
      );
      /**
       Diagnostic code digits after the marker.
       */
      const [code,] = line.slice(line.indexOf(ERROR_MARKER,) + ERROR_MARKER.length,)
        .split(':',);
      /**
       Start of `(line,col)`.
       */
      const paren = location.lastIndexOf('(',);
      /**
       File as printed.
       */
      const file = (paren === NOT_FOUND) ? location : location.slice(
        0,
        paren,
      );
      return `${relative(
        PACKAGE_ROOT,
        resolve(
          dir,
          file,
        ),
      )}${(paren === NOT_FOUND) ? '' : location.slice(paren,)}: TS${String(code,)}`;
    },);
}

/**
 Write a tsconfig mapping `deepmerge-ts` to `entry` and type-check it.

 @param job - Job name, also its directory under `jobs/`.

 @param entry - Declaration file for `deepmerge-ts`.

 @param include - Files and globs to check.

 @returns Compiler outcome, also written to `results/<job>.json`.

 @example
 ```ts
 await checkJob({ entry, include: [file,], job: 'control-x-buggy', });
 ```
 */
export async function checkJob(
  {
    job,
    entry,
    include,
  }: {
    readonly job: string;
    readonly entry: string;
    readonly include: readonly string[];
  },
): Promise<TscOutcome> {
  /**
   Job directory.
   */
  const dir = join(
    OUT_DIR,
    'jobs',
    job,
  );
  await mkdir(
    dir,
    { recursive: true, },
  );
  await writeFile(
    join(
      dir,
      'tsconfig.json',
    ),
    `${JSON.stringify(
      {
        compilerOptions: {
          composite: false,
          declaration: false,
          incremental: false,
          isolatedDeclarations: false,
          noEmit: true,
          paths: { 'deepmerge-ts': [entry,], },
          // Jobs include files outside their own directory; TypeScript 7
          // otherwise rejects them against the default rootDir (TS6059).
          rootDir: '/',
        },
        exclude: [join(
          PACKAGE_ROOT,
          'src',
          '**',
          '*.local.*',
        ),],
        extends: join(
          PACKAGE_ROOT,
          '..',
          '..',
          'config',
          'typescript',
          'tsconfig.dom.json',
        ),
        include,
      },
      undefined,
      2,
    )}\n`,
  );
  /**
   Start time.
   */
  const started = performance.now();
  /**
   Workspace compiler, the one `lint:types` uses.
   */
  const child = spawn(
    join(
      PACKAGE_ROOT,
      '..',
      '..',
      '..',
      'node_modules',
      '.bin',
      'tsc',
    ),
    [
      '--project',
      join(
        dir,
        'tsconfig.json',
      ),
      '--pretty',
      'false',
    ],
    {
      cwd: dir,
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
    },
  );
  /**
   Output chunks.
   */
  const chunks: Buffer[] = [];
  child.stdout
    .on(
    'data',
    function collect(chunk: Buffer,) {
      chunks.push(chunk,);
    },
    );
  child.stderr
    .on(
    'data',
    function collect(chunk: Buffer,) {
      chunks.push(chunk,);
    },
    );
  /**
   Whether the limit killed the compiler.
   */
  const timedOut = { value: false, };
  /**
   Kill timer.
   */
  const timer = setTimeout(
    function killHung() {
      timedOut.value = true;
      child.kill('SIGKILL',);
    },
    TSC_TIMEOUT_MS,
  );
  await once(
    child,
    'close',
  );
  clearTimeout(timer,);
  /**
   Outcome.
   */
  const outcome: TscOutcome = {
    diagnostics: diagnosticKeys({
      dir,
      output: Buffer.concat(chunks,)
        .toString('utf8',),
    },),
    entry,
    job,
    ms: Math.round(performance.now() - started,),
    timedOut: timedOut.value,
  };
  await mkdir(
    join(
      OUT_DIR,
      'results',
    ),
    { recursive: true, },
  );
  await writeFile(
    join(
      OUT_DIR,
      'results',
      `${job}.json`,
    ),
    `${JSON.stringify(
      outcome,
      undefined,
      2,
    )}\n`,
  );
  console.log(`${job}\t${String(outcome.diagnostics
    .length,)} diagnostic(s)\t${String(outcome.ms,)} ms${outcome.timedOut ? '\ttimed out' : ''}`,);
  return outcome;
}
