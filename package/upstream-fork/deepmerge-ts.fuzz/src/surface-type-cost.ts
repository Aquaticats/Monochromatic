/**
 Type-checker cost of deepmerge-ts result types: generates one scenario file
 per shape family and size, type-checks each alone with
 `tsc --extendedDiagnostics`, and prints instantiations, types, check time,
 memory, and any diagnostic codes.

 Runs inside a capped container with the scratch install from
 `./surface-toolchain.ts`:

 ```sh
 node src/surface-type-cost.ts <scratch> [alias] [family-prefix] [repeat]
 ```

 Instantiation and type counts are deterministic; check time and memory vary
 run to run (measure the band with `repeat` on one case before comparing).
 Findings: `doc/audit/deepmerge-ts-surface-2026-09-24.md`.

 @module
 */

import {
  mkdir,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  isTypeScriptAlias,
  runChild,
  tscPrefix,
} from './surface-toolchain.ts';
import {
  FAMILIES,
  range,
  seriesOf,
} from './surface-type-cost-family.ts';

/**
 Per-case limit; the largest case measured takes under 3 seconds.
 */
const CASE_TIMEOUT_MS = 180_000;


/**
 Value of one `--extendedDiagnostics` line.

 @param output - Compiler output.

 @param name - Metric label, such as `Instantiations`.

 @returns The value text, or `-` when the line is absent (the checker crashed or stopped early).

 @example
 ```ts
 metric({ name: 'Types', output: 'Types: 12\n', },); // '12'
 ```
 */
function metric({
  output,
  name,
}: {
  readonly output: string;
  readonly name: string
},): string {
  /**
   Line carrying the metric.
   */
  const line = output.split('\n',)
    .find(function labelled(candidate,) {
      return candidate.startsWith(`${name}:`,);
    },);
  return (line === undefined) ? '-' : line.slice(name.length + 1,)
    .trim();
}

/**
 Write every scenario of the families matching `prefix`, then check each.

 @param scratch - Scratch directory holding the install.

 @param alias - TypeScript release.

 @param prefix - Family name prefix; empty runs every family.

 @param repeat - Runs per case.

 @example
 ```ts
 await runCost({ alias: 'ts60', prefix: 'wide', repeat: 1, scratch: '/scratch', },);
 ```
 */
async function runCost(
  {
    scratch,
    alias,
    prefix,
    repeat,
  }: {
    readonly scratch: string;
    readonly alias: Parameters<typeof tscPrefix>[0]['alias'];
    readonly prefix: string;
    readonly repeat: number;
  },
): Promise<void> {
  /**
   Directory for generated scenarios.
   */
  const cases = join(
    scratch,
    'cost',
  );
  await rm(
    cases,
    {
      force: true,
      recursive: true,
    },
  );
  await mkdir(
    cases,
    { recursive: true, },
  );
  for (const [family, {
    series,
    body,
  },] of Object.entries(FAMILIES,)
    .filter(function wanted([name,],) {
      return name.startsWith(prefix,);
    },)) {
    for (const size of seriesOf(series,)) {
      /**
       Scenario file, relative to the scratch directory.
       */
      const file = join(
        'cost',
        `${family}-${String(size,)}.ts`,
      );
      // oxlint-disable-next-line eslint/no-await-in-loop -- one scenario at a time so each check gets the whole container.
      await writeFile(
        join(
          scratch,
          file,
        ),
        `import { deepmerge, deepmergeInto } from "deepmerge-ts";\n${body(size,)}\nexport {};\n`,
      );
      for (const _run of range(repeat,)) {
        /* oxlint-disable eslint/no-await-in-loop -- sequential for comparable timings. */
        /**
         Compiler run for this case.
         */
        const outcome = await runChild({
          args: [
            ...tscPrefix({
              alias,
              scratch,
            },),
            '--strict',
            '--skipLibCheck',
            '--extendedDiagnostics',
            file,
          ],
          command: 'node',
          cwd: scratch,
          timeoutMs: CASE_TIMEOUT_MS,
        },);
        /* oxlint-enable eslint/no-await-in-loop */
        /**
         Diagnostic codes in the output.
         */
        const codes = [...new Set(outcome.output
          .split('\n',)
          .filter(function isError(line,) {
            return line.includes('error TS',);
          },)
          .map(function codeOf(line,) {
            return line.slice(
              line.indexOf(
                'TS',
                line.indexOf('error',),
              ),
              line.indexOf(
                ':',
                line.indexOf('error',),
              ),
            );
          },),),];
        console.log([
          `${family}-${String(size,)}`,
          `inst=${metric({
            name: 'Instantiations',
            output: outcome.output,
          },)}`,
          `types=${metric({
            name: 'Types',
            output: outcome.output,
          },)}`,
          `check=${metric({
            name: 'Check time',
            output: outcome.output,
          },)}`,
          `mem=${metric({
            name: 'Memory used',
            output: outcome.output,
          },)}`,
          `exit=${outcome.exit}`,
          (codes.length > 0) ? `errors=${codes.join(',',)}` : '',
        ].join(' ',),);
      }
    }
  }
}

if (import.meta.main) {
  /**
   Scratch directory, release, family prefix, and repeat count.
   */
  const [scratch, alias = 'ts60', prefix = '', repeatText = '1',] = process.argv
    .slice(2,);
  /**
   Runs per case.
   */
  const repeat = Number(repeatText,);
  if ((scratch === undefined) || (!isTypeScriptAlias(alias,))
    || (!Number.isInteger(repeat,))
    || (repeat < 1))
    throw new Error('usage: surface-type-cost.ts <scratch> [alias] [family-prefix] [repeat]',);
  await runCost({
    alias,
    prefix,
    repeat,
    scratch,
  },);
}
