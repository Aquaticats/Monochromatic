/**
 Guards against a module root logger that names no entry or slice (ledger
 A11), by reading the source: every module's root is a context root, so a
 line it writes inside an entry names that entry and slice.

 A root written as a plain `tagged` call reads nothing from the context, and
 its lines under an entry are the unattributed 3279 of 5943 that
 TianqiChen66616.log carried. Test files and test fixtures log outside any
 entry and are not read.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { readPackageSource, } from './family-source.test-fixture.ts';
import type { SourceText, } from './source-scan.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

/**
 How a context-less root begins, at module scope.
 */
const PLAIN_ROOT = ' = tagged({ tag: \'';

/**
 How a context root begins, at module scope.
 */
const CONTEXT_ROOT = ' = contextRoot({ tag: \'';

/**
 Module-scope declarations in one file's text that build a root of one kind.

 @param text - file contents

 @param shape - how the root's initializer begins

 @returns Lines declaring such a root

 @example
 ```ts
 rootsIn({ text: "const l = tagged({ tag: 'cat', },);", shape: PLAIN_ROOT, },); // one line
 ```
 */
function rootsIn(
  {
    text,
    shape,
  }: {
    readonly text: string;
    readonly shape: string;
  },
): readonly string[] {
  return text
    .split('\n',)
    .filter(function declaresRoot(line,): boolean {
      return line.startsWith('const ',) && line.includes(shape,);
    },);
}

/**
 Source files the scan reads: every `.ts` under the `src` of every family package but tests and fixtures.

 @returns Files, each located by its package directory and path under `src`

 @example
 ```ts
 const files = await sourceFiles();
 ```
 */
async function sourceFiles(): Promise<readonly SourceText[]> {
  return (await readPackageSource())
    .filter(function isSource({ path, isTest, },): boolean {
      return (!isTest) && (!path.endsWith('.d.ts',));
    },);
}

await describe({
  name: 'module root loggers',
  children: [
    it({
      name: 'SEES a plain root in a line of text, the positive control for the "BUILDS every module root as a '
        + 'context root" scan',
      fn: async () => {
        expect(rootsIn({
          text: 'import { tagged, } from \'x\';\nconst l = tagged({ tag: \'whiskers\', },);\n',
          shape: PLAIN_ROOT,
        },).length,).toBe(1,);
      },
    },),
    it({
      name: 'BUILDS every module root as a context root, so its lines name the entry and slice (ledger A11)',
      fn: async () => {
        /**
         Every root declaration per source file.
         */
        const scanned = (await sourceFiles()).map(function scanOne({ path: name, text, },): {
          readonly name: string;
          readonly plain: readonly string[];
          readonly context: readonly string[];
        } {
          return {
            name,
            plain: rootsIn({
              text,
              shape: PLAIN_ROOT,
            },),
            context: rootsIn({
              text,
              shape: CONTEXT_ROOT,
            },),
          };
        },);

        // The scan read real roots, so an empty `hasContextRoot` list is not a
        // scan that read nothing.
        expect(scanned.filter(function hasContextRoot({ context, },): boolean {
          return context.length > 0;
        },).length,).toBeGreaterThan(0,);

        expectNoFindings({
          findings: scanned
            .filter(function hasPlainRoot({ plain, },): boolean {
              return plain.length > 0;
            },)
            .map(function named({ name, },): string {
              return name;
            },),
        },);
      },
    },),
  ],
},);
