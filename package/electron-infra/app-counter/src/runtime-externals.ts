/**
 Externals shared by this package's two rolldown bundles.
 
 Electron supplies its own runtime and Node's builtins when it loads the app or
 one of the tools, so bundling either would ship a second runtime or fail to
 resolve at load time. Each config states only which packages the host supplies
 and takes the matching predicate from here.
 
 @module
 */

import type { ExternalOptionFunction, } from 'rolldown';

//region Predicate construction

/**
 Prefix every Node builtin specifier carries.
 */
const NODE_BUILTIN_PREFIX = 'node:';

/**
 Whether a specifier names a package or one of its subpaths.
 
 @param moduleSpecifier - specifier rolldown is resolving
 
 @param packageName - package root the host supplies
 
 @returns whether the specifier belongs to that package
 
 @example
 ```ts
 namesPackageOrSubpath({ moduleSpecifier: 'electron/main', packageName: 'electron', }); // true
 ```
 */
function namesPackageOrSubpath({
  moduleSpecifier,
  packageName,
}: {
  readonly moduleSpecifier: string;
  readonly packageName: string;
},): boolean {
  if (moduleSpecifier === packageName)
    return true;
  return moduleSpecifier.startsWith(`${packageName}/`);
}

/**
 Builds the rolldown external predicate for host-supplied packages.
 
 @param packageNames - package roots the host supplies at run time
 
 @returns predicate keeping those packages and Node builtins out of a bundle
 
 @example
 ```ts
 const external = runtimeExternals({ packageNames: ['electron',], });
 external('electron/main', undefined, false); // true
 ```
 */
export function runtimeExternals(
  { packageNames, }: { readonly packageNames: readonly string[]; },
): ExternalOptionFunction {
  return function isHostSuppliedModule(moduleSpecifier: string,): boolean {
    if (moduleSpecifier.startsWith(NODE_BUILTIN_PREFIX,))
      return true;
    return packageNames
      .some(function isHostPackage(packageName,) {
        return namesPackageOrSubpath({
          moduleSpecifier,
          packageName,
        },);
      },);
  };
}

//endregion Predicate construction
