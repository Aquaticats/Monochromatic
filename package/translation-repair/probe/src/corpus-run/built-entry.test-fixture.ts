/**
 Where the built commands of this package sit, for the as-built suites that start them.

 TEST SUPPORT, NOT PACKAGE SOURCE. No child is started here. A case hands the entry to `runBuiltCommand` of
 `child-environment.test-fixture.ts`, which finds the command's built file beside it; the shared fixture cannot
 resolve this package by name itself, since a package resolves only what its own `package.json` lists.

 @module
 */

/**
 URL of the built index of this package, as `import.meta.resolve` gives it for the package name, which every built
 command of the package sits beside.
 */
export const PROBE_ENTRY: string = import.meta.resolve('@monochromatic-dev/translation-repair-probe',);
