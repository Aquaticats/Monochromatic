/**
 Final merged package index.
 Combines auto-generated Repology data with hand-maintained overrides.
 
 Import this file to get the complete package list for {@link registerPackages}.
 
 @example
 ```ts
 import { packages } from './data/packages.ts';
 import { registerPackages } from './package/ensure-package.ts';
 registerPackages(packages);
 ```
 */

import type { PackageEntry, } from '../package/types.ts';
import { mergeOverrides, } from '../package/merge.ts';
import { generated, } from './packages.generated.ts';
import { overrides, } from './packages.overrides.ts';

/**
 Complete package index combining generated Repology data with hand-maintained overrides,
 produced by {@link mergeOverrides}.
 */
export const packages: readonly PackageEntry[] = mergeOverrides({
  generated,
  overrideEntries: overrides,
},);
