/**
 The errno-exception guard has one body, and every caller reaches it through
 `errors.ts`.
 
 A hand-written copy per module is what let two spellings spread: a guard
 testing only `Error.isError` and a guard testing only `'code' in error` treat
 the same thrown value differently, so the same failure is read as a missing
 file in one module and rethrown in another. That guard now lives in
 `@monochromatic-dev/type-guard-core`, so this package declares none of its
 own; `errors.ts` re-exports the shared one and every caller imports it from
 there. This suite fails when a source module declares a guard, or reaches a
 guard without importing it from `errors.ts`.
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
 Name of the guard this package reaches through `errors.ts`.
 */
const guardName = 'isErrnoException';

/**
 Import specifier of the package that owns the guard body.
 */
const sharedGuardSpecifier = '@monochromatic-dev/type-guard-core/ts';

/**
 Text that opens a guard declaration.
 */
const declarationPrefix = `function ${guardName}`;

/**
 Module that re-exports the guard to this package's callers.
 */
const ownerModuleName = 'errors.ts';

/**
 Text that closes an import of the owner module, named as every import of it
 spells the specifier.
 */
const ownerImportEnd = `} from './${ownerModuleName}'`;

/**
 Text that opens an import statement.
 */
const importPrefix = 'import';

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
 Text of one import statement, from after its keyword to its semicolon.
 
 @param statementText - Source text following an `import` keyword.
 
 @returns The statement text, without what follows its semicolon.
 */
function importStatementOf(statementText: string,): string {
  /**
   Position of the semicolon that ends this statement, absent when the
   statement runs to the end of the file.
   */
  const statementEnd = statementText.indexOf(';',);
  return statementEnd === (-1)
    ? statementText
    : statementText.slice(
      0,
      statementEnd,
    );
}

/**
 Tells whether a module imports the guard from the owner module.
 
 An import of the owner module spells its specifier, and the statement holding
 that specifier names every imported binding, so one statement names both.

 @param sourceText - Source text of the module.

 @returns Whether any owner import names the guard.
 */
function importsGuardFromOwner(sourceText: string,): boolean {
  return sourceText
    .split(importPrefix,)
    .slice(1,)
    .map(importStatementOf,)
    .some(function namesOwnerAndGuard(statement,) {
      return statement.includes(ownerImportEnd,) && statement.includes(guardName,);
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
  return !importsGuardFromOwner(sourceText,);
}

//endregion Source scanning

await describe({
  name: 'errno exception guard sites',
  children: [
    it({
      name: 'declares no guard of its own, because the shared package owns the body',
      fn: async () => {
        expect(guardDeclarationModules(),).toEqual([],);
      },
    },),
    it({
      name: 're-exports the shared guard from the error module callers import',
      fn: async () => {
        /**
         Source text of the module callers import the guard from.
         */
        const ownerText = sourceTextOf(join(sourceDirectory, ownerModuleName,),);
        expect(ownerText.includes(sharedGuardSpecifier,),).toBe(true,);
        expect(ownerText.includes(`export { ${guardName}, } from`,),).toBe(true,);
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
