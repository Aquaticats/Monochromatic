import {
  dirname,
  resolve,
} from 'node:path';
import { fileURLToPath, } from 'node:url';

//region Built package directory
// WHERE A BUILT PACKAGE SITS, READ OFF THE PLACE ITS NAME RESOLVES TO.
//
// TEST SUPPORT, NOT PACKAGE SOURCE. A fixture shared by tests of several packages cannot
// join `dist/final/node` to its own directory, since that names the fixture's package and not
// the one whose build the child needs, and it cannot resolve a package by name either, since
// a package resolves only what its own `package.json` lists. So the test that knows the
// package resolves its name (`import.meta.resolve('@monochromatic-dev/translation-repair-<name>')`)
// and hands the resulting URL down, and the fixture reads the directories off it.

/**
 Directory the built files of a package sit in: the one holding the file the package's `"."`
 export names.

 @param packageEntry - URL of the package's built index, as `import.meta.resolve` gives it for the package name

 @returns Absolute path of `dist/final/node` in that package

 @example
 ```ts
 const builtDirectory = builtDirectoryOf({ packageEntry: import.meta.resolve('@monochromatic-dev/translation-repair-bench'), },);
 ```
 */
export function builtDirectoryOf({ packageEntry, }: { readonly packageEntry: string; },): string {
  return dirname(fileURLToPath(packageEntry,),);
}

/**
 Directory of a package, where its built commands run unless a case says otherwise.

 @param packageEntry - URL of the package's built index, as `import.meta.resolve` gives it for the package name

 @returns Absolute path of the package directory

 @example
 ```ts
 const root = packageRootOf({ packageEntry: import.meta.resolve('@monochromatic-dev/translation-repair-bench'), },);
 ```
 */
export function packageRootOf({ packageEntry, }: { readonly packageEntry: string; },): string {
  return resolve(
    builtDirectoryOf({ packageEntry, },),
    '..',
    '..',
    '..',
  );
}

//endregion Built package directory
