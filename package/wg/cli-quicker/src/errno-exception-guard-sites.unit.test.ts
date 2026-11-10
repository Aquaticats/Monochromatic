/**
 The package holds one errno-exception guard, and every caller imports it.
 
 A hand-written copy per module is what let two spellings spread: a guard
 testing only `Error.isError` and a guard testing only `'code' in error` treat
 the same thrown value differently, so the same failure is read as a missing
 file in one module and rethrown in another. `errors.ts` owns the guard. This
 suite fails when a source module declares its own, or reaches a guard it does
 not import from that module.
 */
import {
  readdirSync,
  readFileSync,
} from 'node:fs';
import { basename, join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

//region Source scanning

/**
 Directory holding the modules under test; this suite sits beside them.
 */
const sourceDirectory = import.meta.dirname;

/**
 Suffix marking a test file, which no bundle includes and which may name the
 guard in expectations without being a caller.
 */
const testFileSuffix = '.test.ts';

/**
 Name of the one guard this package may declare.
 */
const guardName = 'isErrnoException';

/**
 Text that opens a guard declaration.
 */
const declarationPrefix = `function ${guardName}`;

/**
 Text that opens an exported guard declaration.
 */
const exportedDeclarationPrefix = `export ${declarationPrefix}`;

/**
 Module that owns the guard.
 */
const ownerModuleName = 'errors.ts';

/**
 Every import of the owner module, with the imported names inside the braces.
 */
const ownerImportPattern = /import\s*\{[^{}]*\}\s*from\s*'\.\/errors\.ts'/gsu;

/**
 Source modules of this package, test files excluded.
 
 @returns Absolute paths of the modules a bundle can include.
 */
function sourcePaths(): string[] {
  return readdirSync(sourceDirectory,)
    .filter(function isSourceModule(entryName,) {
      return entryName.endsWith('.ts',) && (!entryName.endsWith(testFileSuffix,));
    },)
    .map(function toAbsolutePath(entryName,) {
      return join(
        sourceDirectory,
        entryName,
      );
    },);
}

/**
 Text of one module.
 
 @param filePath - Absolute path of the module to read.
 
 @returns Source text of the module.
 */
function sourceTextOf(filePath: string,): string {
  return readFileSync(
    filePath,
    'utf8',
  );
}

/**
 Modules that declare a guard of their own.
 
 @returns File names of every module holding a declaration.
 */
function guardDeclarationModules(): string[] {
  return sourcePaths()
    .filter(function declaresGuard(filePath,) {
      return sourceTextOf(filePath,).includes(declarationPrefix,);
    },)
    .map(function toFileName(filePath,) {
      return basename(filePath,);
    },);
}

/**
 Modules that use the guard without importing it from the owner module.
 
 @param filePath - Absolute path of the module to inspect.
 
 @returns Whether the module names the guard without an owner import.
 */
function usesGuardWithoutOwnerImport(filePath: string,): boolean {
  /**
   Source text of the module.
   */
  const sourceText = sourceTextOf(filePath,);
  if (!sourceText.includes(guardName,))
    return false;
  return ![...sourceText.matchAll(ownerImportPattern,)]
    .some(function importsGuard(importMatch,) {
      return importMatch[0].includes(guardName,);
    },);
}

//endregion Source scanning

await describe({
  name: 'errno exception guard sites',
  children: [
    it({
      name: 'declares the guard once, in the module that owns the error types',
      fn: async () => {
        expect(guardDeclarationModules(),).toEqual([ownerModuleName,],);
      },
    },),
    it({
      name: 'exports the one declaration, so every caller reaches the same body',
      fn: async () => {
        expect(
          sourceTextOf(join(sourceDirectory, ownerModuleName,),).includes(exportedDeclarationPrefix,),
        ).toBe(true,);
      },
    },),
    it({
      name: 'imports the guard from its owner wherever a module uses it',
      fn: async () => {
        /**
         Modules naming the guard without importing it from `errors.ts`.
         */
        const offenders = sourcePaths()
          .filter(function isNotOwner(filePath,) {
            return basename(filePath,) !== ownerModuleName;
          },)
          .filter(usesGuardWithoutOwnerImport,)
          .map(function toFileName(filePath,) {
            return basename(filePath,);
          },);
        expect(offenders,).toEqual([],);
      },
    },),
  ],
},);
