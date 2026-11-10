/**
 Tests that the package manifest declares every module the source imports
 ({@link https://github.com/Aquaticats/Monochromatic/issues/1663}).

 The workspace links packages with `nodeLinker: isolated` and `hoist: false`,
 so a module a package never declares resolves only by walking up to the
 repository root's `node_modules`. That holds while the root manifest happens to
 declare it, and breaks for anyone consuming the package standalone.

 @module
 */

import {
  readFile,
  readdir,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { fileURLToPath, } from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

//region Manifest

/**
 Manifest fields that declare a module this package may import.
 */
const DEPENDENCY_FIELDS = [
  'dependencies',
  'devDependencies',
  'peerDependencies',
  'optionalDependencies',
] as const;

/**
 Source directory whose import specifiers the scan collects.
 */
const SOURCE_DIRECTORY = fileURLToPath(new URL('.', import.meta.url,),);

/**
 This package's manifest, one level above the source directory.
 */
const MANIFEST_PATH = fileURLToPath(
  new URL('../package.json', import.meta.url,),
);

/**
 Decide whether a parsed JSON value is an object with named members.

 @param value - Value decoded from the manifest

 @returns Whether value can be read by member name
 */
function isRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object') && (value !== null);
}

/**
 Collect every module name the manifest declares in any dependency field.

 @returns Module names the package promises to provide

 @throws when the manifest file does not decode to a JSON object
 */
async function declaredModuleNames(): Promise<ReadonlySet<string>> {
  const manifest: unknown = JSON.parse(
    await readFile(MANIFEST_PATH, 'utf8',),
  );
  if (!isRecord(manifest,))
    throw new Error(`${MANIFEST_PATH} does not decode to an object`,);

  const declared = new Set<string>();
  for (const field of DEPENDENCY_FIELDS) {
    const block = manifest[field];
    if (!isRecord(block,))
      continue;
    for (const moduleName of Object.keys(block,)) {
      declared.add(moduleName,);
    }
  }
  return declared;
}

//endregion

//region Source scan

/**
 Offset in a source line where the quoted import specifier starts.

 A static import and a re-export both put the specifier after ` from `; a
 side-effect import puts it straight after `import`. Lines that import nothing
 return undefined.

 @param line - One line of TypeScript source

 @returns Offset of the opening quote, or undefined when the line imports nothing
 */
function specifierClauseStart(line: string,): number | undefined {
  const fromMarker = ' from ';
  const fromIndex = line.indexOf(fromMarker,);
  if (fromIndex !== -1)
    return fromIndex + fromMarker.length;
  if (line.startsWith('import ',))
    return 'import '.length;
  return undefined;
}

/**
 Read the import specifier quoted on one line, if the line imports or re-exports.

 The scan reads lines rather than the whole file because a specifier never spans
 lines in this codebase.

 @param line - One line of TypeScript source

 @returns The quoted specifier, or undefined for a line that imports nothing
 */
function quotedImportSpecifier(line: string,): string | undefined {
  const clauseStart = specifierClauseStart(line,);
  if (clauseStart === undefined)
    return undefined;

  const openingIndex = line.indexOf('\'', clauseStart,);
  if (openingIndex === -1)
    return undefined;
  const closingIndex = line.indexOf('\'', openingIndex + 1,);
  if (closingIndex === -1)
    return undefined;
  return line.slice(openingIndex + 1, closingIndex,);
}

/**
 Reduce an import specifier to the module that provides it.

 Relative paths and `node:` builtins name no package, so they return undefined.
 A subpath import (`@scope/name/subpath`) belongs to the package before it.

 @param specifier - Import specifier as written in source

 @returns Package name, or undefined when the specifier is not a package
 */
function packageNameOf(specifier: string,): string | undefined {
  if (specifier.startsWith('.',))
    return undefined;
  if (specifier.startsWith('node:',))
    return undefined;

  const [first, second,] = specifier.split('/',);
  if (specifier.startsWith('@',) && (second !== undefined))
    return `${first}/${second}`;
  return first;
}

/**
 Collect every package name imported anywhere under the source directory.

 @returns Package names the source depends on
 */
async function importedPackageNames(): Promise<ReadonlySet<string>> {
  const imported = new Set<string>();
  const entries = await readdir(
    SOURCE_DIRECTORY,
    { withFileTypes: true, },
  );
  for (const entry of entries) {
    if (!entry.isFile() || !entry.name.endsWith('.ts',))
      continue;
    const source = await readFile(
      join(SOURCE_DIRECTORY, entry.name,),
      'utf8',
    );
    for (const line of source.split('\n',)) {
      const specifier = quotedImportSpecifier(line,);
      if (specifier === undefined)
        continue;
      const packageName = packageNameOf(specifier,);
      if (packageName !== undefined)
        imported.add(packageName,);
    }
  }
  return imported;
}

//endregion

await describe({
  name: '',
  children: [
    describe({
      name: 'package.json dependencies',
      children: [
        it({
          name: 'declare every package the source imports',
          fn: async () => {
            const declared = await declaredModuleNames();
            const imported = await importedPackageNames();

            // A scan that read nothing would report every import as declared.
            expect(imported.size,).toBeGreaterThan(0,);

            const undeclared = [...imported,]
              .filter(function isUndeclared(name,): boolean {
                return !declared.has(name,);
              },)
              .sort();

            // Joined so a failure names every undeclared package at once.
            expect(undeclared.join(', ',),).toBe('',);
          },
        },),
      ],
    },),
  ],
},);
